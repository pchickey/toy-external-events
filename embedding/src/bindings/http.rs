use crate::ctx::EmbeddingCtx;
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use anyhow::{anyhow, Result};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use wasmtime::component::{Resource, ResourceTable};
use wasmtime_wasi_io::{
    async_trait,
    poll::{subscribe, DynPollable, Pollable},
    streams::{DynInputStream, DynOutputStream},
    IoView,
};

use super::wasi::clocks::monotonic_clock;
use super::wasi::http::types;

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
    headers: FieldsResource,
    body: Option<crate::http::OutgoingBody>,
}
impl OutgoingResponseResource {
    pub fn new(
        resp: crate::http::OutgoingResponse,
        headers: FieldsResource,
        body: crate::http::OutgoingBody,
    ) -> Self {
        Self {
            resp,
            headers,
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
            crate::http::OutgoingResponse {},
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
        Ok(table.push(headers)?)
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
    headers: FieldsResource,
    body: Option<crate::http::OutgoingBody>,
}
impl OutgoingRequestResource {
    pub fn new(
        req: crate::http::OutgoingRequest,
        headers: FieldsResource,
        body: crate::http::OutgoingBody,
    ) -> Self {
        Self {
            req,
            headers,
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
            crate::http::OutgoingRequest {},
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
        Ok(table.push(headers)?)
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

pub struct FutureIncomingResponse {
    // Task may be pending completion
    task: Pin<Box<async_task::Task<Result<IncomingResponseResource, types::ErrorCode>>>>,
    // Result of task, if it completed while awaiting in a Pollable
    res: Option<Result<IncomingResponseResource, types::ErrorCode>>,
    // Indicates the completed task's result has been returned by get already
    gone: bool,
}
impl FutureIncomingResponse {
    // gets constructed in implementation of outgoing-handler.handle.
    pub fn new(task: async_task::Task<Result<IncomingResponseResource, types::ErrorCode>>) -> Self {
        Self {
            task: Box::pin(task),
            res: None,
            gone: false,
        }
    }
}

impl Future for FutureIncomingResponse {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        if self.gone {
            return Poll::Ready(());
        }
        if self.res.is_some() {
            return Poll::Ready(());
        }
        match self.task.as_mut().poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(res) => {
                self.res = Some(res);
                Poll::Ready(())
            }
        }
    }
}

#[async_trait]
impl Pollable for FutureIncomingResponse {
    async fn ready(&mut self) {
        self.await
    }
}

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
        fn res_into_table(
            table: &mut ResourceTable,
            res: Result<IncomingResponseResource, types::ErrorCode>,
        ) -> Result<Result<Resource<types::IncomingResponse>, types::ErrorCode>> {
            match res {
                Ok(resource) => Ok(Ok(table.push(resource)?)),
                Err(code) => Ok(Err(code)),
            }
        }

        let this = self.table().get_mut(&this)?;
        if let Some(res) = this.res.take() {
            this.gone = true;
            return Ok(Some(Ok(res_into_table(self.table(), res)?)));
        }
        if this.gone {
            return Ok(Some(Err(())));
        }
        // Poll checks for task completion. Doing so in this way will replace any existing waker
        // with a noop waker. This is ok because it will get a "real" waker when it is polled via a
        // wasi Pollable if there is actually progress to be made in wasi:io/poll waiting on it.
        // This operation should be very fast - in this crate's single threaded context, there are
        // some uncontended atomic swaps in there, but otherwise its just checking state and
        // returning the task's result if it is complete.
        match this
            .task
            .as_mut()
            .poll(&mut Context::from_waker(&crate::noop_waker::noop_waker()))
        {
            Poll::Pending => Ok(None),
            Poll::Ready(res) => {
                this.gone = true;
                return Ok(Some(Ok(res_into_table(self.table(), res)?)));
            }
        }
    }
    fn drop(&mut self, this: Resource<types::FutureIncomingResponse>) -> Result<()> {
        self.table().delete(this)?;
        Ok(())
    }
}

pub struct IncomingBodyResource(crate::http::IncomingBody);

impl types::HostIncomingBody for EmbeddingCtx {
    fn stream(
        &mut self,
        _: Resource<types::IncomingBody>,
    ) -> Result<Result<Resource<DynInputStream>, ()>> {
        todo!()
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

impl types::HostOutgoingBody for EmbeddingCtx {
    fn write(
        &mut self,
        _: Resource<types::OutgoingBody>,
    ) -> Result<Result<Resource<DynOutputStream>, ()>> {
        todo!()
    }
    fn finish(
        &mut self,
        _: Resource<types::OutgoingBody>,
        _: Option<Resource<types::Trailers>>,
    ) -> Result<Result<(), types::ErrorCode>> {
        todo!()
    }
    fn drop(&mut self, this: Resource<types::OutgoingBody>) -> Result<()> {
        self.table().delete(this)?;
        Ok(())
    }
}

#[derive(Clone)]
#[allow(dead_code)] // Temporary - while HostFields and Fields trait are just stubs
pub enum FieldsResource {
    Mut(Rc<crate::http::Fields>),
    Immut(Rc<crate::http::ImmutFields>),
}
impl FieldsResource {
    pub fn new(fields: crate::http::Fields) -> Self {
        Self::Mut(Rc::new(fields))
    }

    pub fn freeze(self) -> Result<Self> {
        match self {
            Self::Immut(rc) => Ok(Self::Immut(rc)),
            Self::Mut(rc) => {
                let fields = Rc::try_unwrap(rc).map_err(|rc| {
                    anyhow!(
                        "{} outstanding references to mut fields, should be impossible",
                        Rc::strong_count(&rc)
                    )
                })?;
                Ok(Self::Immut(Rc::new(fields.into_immut())))
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
            FieldsResource::Mut(fs) => Ok(fs.get(&key).into_iter().cloned().collect()),
            FieldsResource::Immut(fs) => Ok(fs.get(&key).into_iter().cloned().collect()),
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
    fn clone(&mut self, this: Resource<types::Fields>) -> Result<Resource<types::Fields>> {
        let entries = match self.table().get(&this)? {
            FieldsResource::Mut(fs) => fs.entries(),
            FieldsResource::Immut(fs) => fs.entries(),
        };
        self.from_list(entries)
            .map(|r| r.expect("from_list wont reject entries from another fields"))
    }
    fn drop(&mut self, this: Resource<types::Fields>) -> Result<()> {
        self.table().delete(this)?;
        Ok(())
    }
}

pub struct FutureTrailers;

impl types::HostFutureTrailers for EmbeddingCtx {
    fn subscribe(&mut self, _: Resource<types::FutureTrailers>) -> Result<Resource<DynPollable>> {
        todo!()
    }
    fn get(
        &mut self,
        _: Resource<types::FutureTrailers>,
    ) -> Result<Option<Result<Result<Option<Resource<types::Trailers>>, types::ErrorCode>, ()>>>
    {
        todo!()
    }
    fn drop(&mut self, this: Resource<types::FutureTrailers>) -> Result<()> {
        self.table().delete(this)?;
        Ok(())
    }
}

pub struct ResponseOutparamResource(crate::http::ResponseOutparam);

impl types::HostResponseOutparam for EmbeddingCtx {
    fn set(
        &mut self,
        _: Resource<types::ResponseOutparam>,
        _: Result<Resource<types::OutgoingResponse>, types::ErrorCode>,
    ) -> Result<()> {
        todo!()
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
        _: Resource<wasmtime_wasi_io::streams::Error>,
    ) -> Result<Option<types::ErrorCode>> {
        todo!()
    }
}
