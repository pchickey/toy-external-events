use crate::ctx::EmbeddingCtx;
use crate::job::{Job, Mailbox};
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use anyhow::{anyhow, Result};
use wasmtime::component::{Resource, ResourceTable};
use wasmtime_wasi_io::{
    async_trait,
    poll::{subscribe, DynPollable, Pollable},
    streams::{DynInputStream, DynOutputStream},
    IoView,
};

use super::wasi::clocks::monotonic_clock;
use super::wasi::http::{outgoing_handler, types};

/// bindgen with clause makes types::IncomingRequest an alias to this:
pub struct IncomingRequestResource {
    req: crate::http::IncomingRequest,
    headers: FieldsResource,
    body: Option<crate::http::IncomingBody>,
}

impl IncomingRequestResource {
    // only place this gets constructed is for a call to an incoming-handler export. we can wrap
    // over the instance.wasi_http_incoming_handler().call_handle(&mut store, ...) that constructs
    // this resource and the response-outparam.
    pub fn new(
        req: crate::http::IncomingRequest,
        headers: crate::http::Fields,
        body: crate::http::IncomingBody,
    ) -> Self {
        Self {
            req,
            headers: FieldsResource::new(headers),
            body: Some(body),
        }
    }
}

impl types::HostIncomingRequest for EmbeddingCtx {
    fn method(&mut self, this: Resource<types::IncomingRequest>) -> Result<types::Method> {
        Ok(self.table().get(&this)?.req.method())
    }
    fn path_with_query(
        &mut self,
        this: Resource<types::IncomingRequest>,
    ) -> Result<Option<String>> {
        Ok(self.table().get(&this)?.req.path_with_query())
    }
    fn scheme(&mut self, this: Resource<types::IncomingRequest>) -> Result<Option<types::Scheme>> {
        Ok(self.table().get(&this)?.req.scheme())
    }
    fn authority(&mut self, this: Resource<types::IncomingRequest>) -> Result<Option<String>> {
        Ok(self.table().get(&this)?.req.authority())
    }
    fn headers(
        &mut self,
        this: Resource<types::IncomingRequest>,
    ) -> Result<Resource<types::Headers>> {
        let table = self.table();
        let headers = table.get(&this)?.headers.clone();
        Ok(table.push(headers)?)
    }
    fn consume(
        &mut self,
        this: Resource<types::IncomingRequest>,
    ) -> Result<Result<Resource<types::IncomingBody>, ()>> {
        let table = self.table();
        // Inner result: only return the IncomingBody resource once. Subsequent returns error.
        if let Some(body) = table.get_mut(&this)?.body.take() {
            Ok(Ok(table.push(IncomingBodyResource(body))?))
        } else {
            Ok(Err(()))
        }
    }
    fn drop(&mut self, this: Resource<types::IncomingRequest>) -> Result<()> {
        self.table().delete(this)?;
        Ok(())
    }
}

pub struct OutgoingResponseResource {
    resp: crate::http::OutgoingResponse,
    headers: Rc<crate::http::ImmutFields>,
    body: Option<crate::http::OutgoingBody>,
}
// SAFETY: single-threaded embedding only
unsafe impl Send for OutgoingResponseResource {}
unsafe impl Sync for OutgoingResponseResource {}

impl OutgoingResponseResource {
    pub fn new(
        resp: crate::http::OutgoingResponse,
        headers: crate::http::ImmutFields,
        body: crate::http::OutgoingBody,
    ) -> Self {
        Self {
            resp,
            headers: Rc::new(headers),
            body: Some(body),
        }
    }
}

impl types::HostOutgoingResponse for EmbeddingCtx {
    fn new(
        &mut self,
        headers: Resource<types::Headers>,
    ) -> Result<Resource<types::OutgoingResponse>> {
        let headers = self.table().delete(headers)?.freeze()?;
        Ok(self.table().push(OutgoingResponseResource::new(
            crate::http::OutgoingResponse::new(),
            headers,
            crate::http::OutgoingBody {},
        ))?)
    }
    fn status_code(
        &mut self,
        this: Resource<types::OutgoingResponse>,
    ) -> Result<types::StatusCode> {
        Ok(self.table().get(&this)?.resp.status_code())
    }
    fn set_status_code(
        &mut self,
        this: Resource<types::OutgoingResponse>,
        code: types::StatusCode,
    ) -> Result<Result<(), ()>> {
        Ok(self.table().get(&this)?.resp.set_status_code(code))
    }
    fn headers(
        &mut self,
        this: Resource<types::OutgoingResponse>,
    ) -> Result<Resource<types::Headers>> {
        let table = self.table();
        let headers = table.get(&this)?.headers.clone();
        Ok(table.push(FieldsResource::Immut(headers))?)
    }
    fn body(
        &mut self,
        this: Resource<types::OutgoingResponse>,
    ) -> Result<Result<Resource<types::OutgoingBody>, ()>> {
        let table = self.table();
        // Inner result: only return the OutgoingBody resource once. Subsequent returns error.
        if let Some(body) = table.get_mut(&this)?.body.take() {
            Ok(Ok(table.push(OutgoingBodyResource(body))?))
        } else {
            Ok(Err(()))
        }
    }
    fn drop(&mut self, this: Resource<types::OutgoingResponse>) -> Result<()> {
        self.table().delete(this)?;
        Ok(())
    }
}

pub struct OutgoingRequestResource {
    req: crate::http::OutgoingRequest,
    headers: Rc<crate::http::ImmutFields>,
    body: Option<crate::http::OutgoingBody>,
}
// SAFETY: single-threaded embedding only
unsafe impl Send for OutgoingRequestResource {}
unsafe impl Sync for OutgoingRequestResource {}
impl OutgoingRequestResource {
    pub fn new(
        req: crate::http::OutgoingRequest,
        headers: crate::http::ImmutFields,
        body: crate::http::OutgoingBody,
    ) -> Self {
        Self {
            req,
            headers: Rc::new(headers),
            body: Some(body),
        }
    }
}

impl types::HostOutgoingRequest for EmbeddingCtx {
    fn new(
        &mut self,
        headers: Resource<types::Headers>,
    ) -> Result<Resource<types::OutgoingRequest>> {
        let headers = self.table().delete(headers)?.freeze()?;
        Ok(self.table().push(OutgoingRequestResource::new(
            crate::http::OutgoingRequest::new(),
            headers,
            crate::http::OutgoingBody {},
        ))?)
    }
    fn body(
        &mut self,
        this: Resource<types::OutgoingRequest>,
    ) -> Result<Result<Resource<types::OutgoingBody>, ()>> {
        let table = self.table();
        // Inner result: only return the OutgoingBody resource once. Subsequent returns error.
        if let Some(body) = table.get_mut(&this)?.body.take() {
            Ok(Ok(table.push(OutgoingBodyResource(body))?))
        } else {
            Ok(Err(()))
        }
    }
    fn method(&mut self, this: Resource<types::OutgoingRequest>) -> Result<types::Method> {
        Ok(self.table().get(&this)?.req.method())
    }
    fn set_method(
        &mut self,
        this: Resource<types::OutgoingRequest>,
        m: types::Method,
    ) -> Result<Result<(), ()>> {
        Ok(self.table().get(&this)?.req.set_method(m))
    }
    fn path_with_query(
        &mut self,
        this: Resource<types::OutgoingRequest>,
    ) -> Result<Option<String>> {
        Ok(self.table().get(&this)?.req.path_with_query())
    }
    fn set_path_with_query(
        &mut self,
        this: Resource<types::OutgoingRequest>,
        what: Option<String>,
    ) -> Result<Result<(), ()>> {
        Ok(self.table().get(&this)?.req.set_path_with_query(what))
    }
    fn scheme(&mut self, this: Resource<types::OutgoingRequest>) -> Result<Option<types::Scheme>> {
        Ok(self.table().get(&this)?.req.scheme())
    }
    fn set_scheme(
        &mut self,
        this: Resource<types::OutgoingRequest>,
        what: Option<types::Scheme>,
    ) -> Result<Result<(), ()>> {
        Ok(self.table().get(&this)?.req.set_scheme(what))
    }
    fn authority(&mut self, this: Resource<types::OutgoingRequest>) -> Result<Option<String>> {
        Ok(self.table().get(&this)?.req.authority())
    }
    fn set_authority(
        &mut self,
        this: Resource<types::OutgoingRequest>,
        what: Option<String>,
    ) -> Result<Result<(), ()>> {
        Ok(self.table().get(&this)?.req.set_authority(what))
    }
    fn headers(
        &mut self,
        this: Resource<types::OutgoingRequest>,
    ) -> Result<Resource<types::Headers>> {
        let table = self.table();
        let headers = table.get(&this)?.headers.clone();
        Ok(table.push(FieldsResource::Immut(headers))?)
    }
    fn drop(&mut self, this: Resource<types::OutgoingRequest>) -> Result<()> {
        self.table().delete(this)?;
        Ok(())
    }
}

/// bindgen with clause makes types::IncomingResponse an alias to this:
pub struct IncomingResponseResource {
    resp: crate::http::IncomingResponse,
    headers: FieldsResource,
    body: Option<crate::http::IncomingBody>,
}

impl IncomingResponseResource {
    // this will get constructed in the implementation of
    // outgoing-handler.handle, which i guess is gonna be a method in
    // Embedding?
    pub fn new(
        resp: crate::http::IncomingResponse,
        headers: crate::http::Fields,
        body: crate::http::IncomingBody,
    ) -> Self {
        Self {
            resp,
            headers: FieldsResource::new(headers),
            body: Some(body),
        }
    }
}

impl types::HostIncomingResponse for EmbeddingCtx {
    fn status(&mut self, this: Resource<types::IncomingResponse>) -> Result<types::StatusCode> {
        Ok(self.table().get(&this)?.resp.status_code())
    }
    fn headers(
        &mut self,
        this: Resource<types::IncomingResponse>,
    ) -> Result<Resource<types::Headers>> {
        let table = self.table();
        let headers = table.get(&this)?.headers.clone();
        Ok(table.push(headers)?)
    }
    fn consume(
        &mut self,
        this: Resource<types::IncomingResponse>,
    ) -> Result<Result<Resource<types::IncomingBody>, ()>> {
        let table = self.table();
        // Inner result: only return the IncomingBody resource once. Subsequent returns error.
        if let Some(body) = table.get_mut(&this)?.body.take() {
            Ok(Ok(table.push(IncomingBodyResource(body))?))
        } else {
            Ok(Err(()))
        }
    }
    fn drop(&mut self, this: Resource<types::IncomingResponse>) -> Result<()> {
        self.table().delete(this)?;
        Ok(())
    }
}

pub type FutureIncomingResponse = Job<Result<IncomingResponseResource, types::ErrorCode>>;

impl types::HostFutureIncomingResponse for EmbeddingCtx {
    fn subscribe(
        &mut self,
        this: Resource<types::FutureIncomingResponse>,
    ) -> Result<Resource<DynPollable>> {
        subscribe(self.table(), this)
    }
    fn get(
        &mut self,
        this: Resource<types::FutureIncomingResponse>,
    ) -> Result<Option<Result<Result<Resource<types::IncomingResponse>, types::ErrorCode>, ()>>>
    {
        let this = self.table().get_mut(&this)?;
        match this.mailbox() {
            Mailbox::Pending => Ok(None),
            Mailbox::Done(Ok(resource)) => Ok(Some(Ok(Ok(self.table().push(resource)?)))),
            Mailbox::Done(Err(code)) => Ok(Some(Ok(Err(code)))),
            Mailbox::Gone => Ok(Some(Err(()))),
        }
    }
    fn drop(&mut self, this: Resource<types::FutureIncomingResponse>) -> Result<()> {
        self.table().delete(this)?;
        Ok(())
    }
}

pub struct IncomingBodyResource(crate::http::IncomingBody);

struct TrapOnRead;

impl wasmtime_wasi_io::streams::InputStream for TrapOnRead {
    fn read(&mut self, _: usize) -> Result<bytes::Bytes, wasmtime_wasi_io::streams::StreamError> {
        Err(wasmtime_wasi_io::streams::StreamError::trap("cant read!!!"))
    }
}
#[wasmtime_wasi_io::async_trait]
impl wasmtime_wasi_io::poll::Pollable for TrapOnRead {
    async fn ready(&mut self) {}
}

impl types::HostIncomingBody for EmbeddingCtx {
    fn stream(
        &mut self,
        this: Resource<types::IncomingBody>,
    ) -> Result<Result<Resource<DynInputStream>, ()>> {
        let _this = self.table().get(&this)?;
        let input_stream: wasmtime_wasi_io::streams::DynInputStream = Box::new(TrapOnRead);
        Ok(Ok(self.table().push(input_stream)?))
    }
    fn finish(
        &mut self,
        _: Resource<types::IncomingBody>,
    ) -> Result<Resource<types::FutureTrailers>> {
        todo!()
    }
    fn drop(&mut self, this: Resource<types::IncomingBody>) -> Result<()> {
        self.table().delete(this)?;
        Ok(())
    }
}

pub struct OutgoingBodyResource(crate::http::OutgoingBody);

struct TrapOnWrite;

impl wasmtime_wasi_io::streams::OutputStream for TrapOnWrite {
    fn check_write(&mut self) -> Result<usize, wasmtime_wasi_io::streams::StreamError> {
        Ok(usize::MAX)
    }
    fn write(&mut self, bs: bytes::Bytes) -> Result<(), wasmtime_wasi_io::streams::StreamError> {
        Err(wasmtime_wasi_io::streams::StreamError::Trap(
            anyhow::anyhow!("cant write!!! {bs:?}"),
        ))
    }
    fn flush(&mut self) -> Result<(), wasmtime_wasi_io::streams::StreamError> {
        Err(wasmtime_wasi_io::streams::StreamError::trap(
            "cant write!!!",
        ))
    }
}
#[wasmtime_wasi_io::async_trait]
impl wasmtime_wasi_io::poll::Pollable for TrapOnWrite {
    async fn ready(&mut self) {}
}

impl types::HostOutgoingBody for EmbeddingCtx {
    fn write(
        &mut self,
        this: Resource<types::OutgoingBody>,
    ) -> Result<Result<Resource<DynOutputStream>, ()>> {
        let _this = self.table().get(&this)?;
        let output_stream: wasmtime_wasi_io::streams::DynOutputStream = Box::new(TrapOnWrite);
        Ok(Ok(self.table().push(output_stream)?))
    }
    fn finish(
        &mut self,
        this: Resource<types::OutgoingBody>,
        trailers: Option<Resource<types::Trailers>>,
    ) -> Result<Result<(), types::ErrorCode>> {
        self.table().delete(this)?;
        if let Some(trailers) = trailers {
            self.table().delete(trailers)?;
        }
        Ok(Ok(()))
    }
    fn drop(&mut self, this: Resource<types::OutgoingBody>) -> Result<()> {
        self.table().delete(this)?;
        Ok(())
    }
}

#[derive(Clone)]
pub enum FieldsResource {
    Mut(Rc<crate::http::Fields>),
    Immut(Rc<crate::http::ImmutFields>),
}
impl FieldsResource {
    pub fn new(fields: crate::http::Fields) -> Self {
        Self::Mut(Rc::new(fields))
    }

    pub fn freeze(self) -> Result<crate::http::ImmutFields> {
        match self {
            Self::Immut(_) => unreachable!(
                "I think we can't call freeze on an immut fields but tbh I'm not 100% sure"
            ),
            Self::Mut(rc) => {
                let fields = Rc::try_unwrap(rc).map_err(|rc| {
                    anyhow!(
                        "{} outstanding references to mut fields, should be impossible",
                        Rc::strong_count(&rc)
                    )
                })?;
                Ok(fields.into_immut())
            }
        }
    }
}
// SAFETY: single-threaded embedding only
unsafe impl Send for FieldsResource {}
unsafe impl Sync for FieldsResource {}

impl types::HostFields for EmbeddingCtx {
    fn new(&mut self) -> Result<Resource<types::Fields>> {
        Ok(self
            .table()
            .push(FieldsResource::new(crate::http::Fields::new()))?)
    }
    fn from_list(
        &mut self,
        values: Vec<(types::FieldKey, types::FieldValue)>,
    ) -> Result<Result<Resource<types::Fields>, types::HeaderError>> {
        let this = crate::http::Fields::new();
        for (key, value) in values {
            if let Err(herr) = this.insert(key, value) {
                return Ok(Err(herr));
            }
        }
        Ok(Ok(self.table().push(FieldsResource::new(this))?))
    }
    fn get(
        &mut self,
        this: Resource<types::Fields>,
        key: types::FieldKey,
    ) -> Result<Vec<types::FieldValue>> {
        match self.table().get(&this)? {
            FieldsResource::Mut(fs) => Ok(fs.get(&key).into_iter().collect()),
            FieldsResource::Immut(fs) => Ok(fs.get(&key).into_iter().collect()),
        }
    }
    fn has(&mut self, this: Resource<types::Fields>, key: types::FieldKey) -> Result<bool> {
        match self.table().get(&this)? {
            FieldsResource::Mut(fs) => Ok(!fs.get(&key).is_empty()),
            FieldsResource::Immut(fs) => Ok(!fs.get(&key).is_empty()),
        }
    }
    fn set(
        &mut self,
        this: Resource<types::Fields>,
        key: types::FieldKey,
        values: Vec<types::FieldValue>,
    ) -> Result<Result<(), types::HeaderError>> {
        match self.table().get(&this)? {
            FieldsResource::Mut(fs) => {
                fs.delete(&key);
                for value in values {
                    if let Err(e) = fs.insert(key.clone(), value) {
                        return Ok(Err(e));
                    }
                }
                Ok(Ok(()))
            }
            FieldsResource::Immut(_) => Ok(Err(types::HeaderError::Immutable)),
        }
    }
    fn delete(
        &mut self,
        this: Resource<types::Fields>,
        key: types::FieldKey,
    ) -> Result<Result<(), types::HeaderError>> {
        match self.table().get(&this)? {
            FieldsResource::Mut(fs) => {
                fs.delete(&key);
                Ok(Ok(()))
            }
            FieldsResource::Immut(_) => Ok(Err(types::HeaderError::Immutable)),
        }
    }
    fn append(
        &mut self,
        this: Resource<types::Fields>,
        key: types::FieldKey,
        value: types::FieldValue,
    ) -> Result<Result<(), types::HeaderError>> {
        match self.table().get(&this)? {
            FieldsResource::Mut(fs) => {
                if let Err(e) = fs.insert(key, value) {
                    return Ok(Err(e));
                }
                Ok(Ok(()))
            }
            FieldsResource::Immut(_) => Ok(Err(types::HeaderError::Immutable)),
        }
    }
    fn entries(
        &mut self,
        this: Resource<types::Fields>,
    ) -> Result<Vec<(types::FieldKey, types::FieldValue)>> {
        match self.table().get(&this)? {
            FieldsResource::Mut(fs) => Ok(fs.entries()),
            FieldsResource::Immut(fs) => Ok(fs.entries()),
        }
    }
    // Very likely a more efficient implementation will exist, just a placeholder
    fn clone(&mut self, this: Resource<types::Fields>) -> Result<Resource<types::Fields>> {
        let entries = match self.table().get(&this)? {
            FieldsResource::Mut(fs) => fs.entries(),
            FieldsResource::Immut(fs) => fs.entries(),
        };
        self.from_list(entries)
            .map(|r| r.expect("Fields constructor wont reject entries from another Fields"))
    }
    fn drop(&mut self, this: Resource<types::Fields>) -> Result<()> {
        self.table().delete(this)?;
        Ok(())
    }
}

// Just stub this out completely until later.
pub struct FutureTrailers {
    gone: bool,
}
#[async_trait]
impl Pollable for FutureTrailers {
    async fn ready(&mut self) {}
}

impl types::HostFutureTrailers for EmbeddingCtx {
    fn subscribe(
        &mut self,
        this: Resource<types::FutureTrailers>,
    ) -> Result<Resource<DynPollable>> {
        subscribe(self.table(), this)
    }
    fn get(
        &mut self,
        this: Resource<types::FutureTrailers>,
    ) -> Result<Option<Result<Result<Option<Resource<types::Trailers>>, types::ErrorCode>, ()>>>
    {
        let this = self.table().get_mut(&this)?;
        if this.gone {
            Ok(Some(Err(())))
        } else {
            this.gone = true;
            Ok(Some(Ok(Ok(None))))
        }
    }
    fn drop(&mut self, this: Resource<types::FutureTrailers>) -> Result<()> {
        self.table().delete(this)?;
        Ok(())
    }
}

pub struct ResponseOutparamResource(crate::http::ResponseOutparam);
impl ResponseOutparamResource {
    pub fn new(inner: crate::http::ResponseOutparam) -> Self {
        Self(inner)
    }
}
impl types::HostResponseOutparam for EmbeddingCtx {
    fn set(
        &mut self,
        this: Resource<types::ResponseOutparam>,
        result: Result<Resource<types::OutgoingResponse>, types::ErrorCode>,
    ) -> Result<()> {
        let this = self.table().delete(this)?;
        match result {
            Ok(out_resp) => {
                let resp = self.table().delete(out_resp)?;
                let headers = Rc::try_unwrap(resp.headers).map_err(|rc| {
                    anyhow!(
                        "{} outstanding references to mut fields, should be impossible",
                        Rc::strong_count(&rc)
                    )
                })?;
                this.0.send_success(resp.resp, headers, resp.body);
            }
            Err(e) => {
                this.0.send_error(e);
            }
        }
        Ok(())
    }
    fn drop(&mut self, this: Resource<types::ResponseOutparam>) -> Result<()> {
        self.table().delete(this)?;
        Ok(())
    }
}

fn to_wasi_duration(d: core::time::Duration) -> monotonic_clock::Duration {
    d.as_nanos()
        .try_into()
        .unwrap_or(monotonic_clock::Duration::MAX)
}
fn from_wasi_duration(d: monotonic_clock::Duration) -> core::time::Duration {
    core::time::Duration::from_nanos(d)
}

pub struct RequestOptionsResource(crate::http::RequestOptions);

impl types::HostRequestOptions for EmbeddingCtx {
    fn new(&mut self) -> Result<Resource<types::RequestOptions>> {
        let opts = crate::http::RequestOptions::default();
        Ok(self.table().push(RequestOptionsResource(opts))?)
    }
    fn connect_timeout(
        &mut self,
        this: Resource<types::RequestOptions>,
    ) -> Result<Option<monotonic_clock::Duration>> {
        let this = self.table().get(&this)?;
        Ok(this.0.connect_timeout.map(to_wasi_duration))
    }
    fn set_connect_timeout(
        &mut self,
        this: Resource<types::RequestOptions>,
        val: Option<monotonic_clock::Duration>,
    ) -> Result<Result<(), ()>> {
        let this = self.table().get_mut(&this)?;
        this.0.connect_timeout = val.map(from_wasi_duration);
        Ok(Ok(()))
    }
    fn first_byte_timeout(
        &mut self,
        this: Resource<types::RequestOptions>,
    ) -> Result<Option<monotonic_clock::Duration>> {
        let this = self.table().get(&this)?;
        Ok(this.0.first_byte_timeout.map(to_wasi_duration))
    }
    fn set_first_byte_timeout(
        &mut self,
        this: Resource<types::RequestOptions>,
        val: Option<monotonic_clock::Duration>,
    ) -> Result<Result<(), ()>> {
        let this = self.table().get_mut(&this)?;
        this.0.first_byte_timeout = val.map(from_wasi_duration);
        Ok(Ok(()))
    }
    fn between_bytes_timeout(
        &mut self,
        this: Resource<types::RequestOptions>,
    ) -> Result<Option<monotonic_clock::Duration>> {
        let this = self.table().get(&this)?;
        Ok(this.0.between_bytes_timeout.map(to_wasi_duration))
    }
    fn set_between_bytes_timeout(
        &mut self,
        this: Resource<types::RequestOptions>,
        val: Option<monotonic_clock::Duration>,
    ) -> Result<Result<(), ()>> {
        let this = self.table().get_mut(&this)?;
        this.0.first_byte_timeout = val.map(from_wasi_duration);
        Ok(Ok(()))
    }
    fn drop(&mut self, this: Resource<types::RequestOptions>) -> Result<()> {
        self.table().delete(this)?;
        Ok(())
    }
}

impl types::Host for EmbeddingCtx {
    fn http_error_code(
        &mut self,
        this: Resource<wasmtime_wasi_io::streams::Error>,
    ) -> Result<Option<types::ErrorCode>> {
        let err = self.table().get(&this)?;
        Ok(err.downcast_ref::<types::ErrorCode>().cloned())
    }
}

impl outgoing_handler::Host for EmbeddingCtx {
    fn handle(
        &mut self,
        request: Resource<types::OutgoingRequest>,
        options: Option<Resource<types::RequestOptions>>,
    ) -> Result<Result<Resource<types::FutureIncomingResponse>, types::ErrorCode>> {
        let OutgoingRequestResource { req, headers, body } = self.table().delete(request)?;
        let headers = Rc::try_unwrap(headers).map_err(|rc| {
            anyhow!(
                "{} outstanding references to immut fields, should be impossible",
                Rc::strong_count(&rc)
            )
        })?;
        let options = options
            .map(|options| self.table().delete(options))
            .transpose()?
            .map(|o| o.0);
        let resp = FutureIncomingResponse::spawn(self.executor(), async move {
            let (resp, headers, body) = req.send(headers, body, options).await?;
            Ok(IncomingResponseResource::new(resp, headers, body))
        });
        Ok(Ok(self.table().push(resp)?))
    }
}
