use crate::{EImpl, Embedding};
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use anyhow::Result;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use wasmtime::component::Resource;
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
    req: Box<dyn crate::http::IncomingRequest>,
    headers: FieldsResource,
    body: Option<Box<dyn crate::http::IncomingBody>>,
}

impl IncomingRequestResource {
    // only place this gets constructed is for a call to an incoming-handler export. we can wrap
    // over the instance.wasi_http_incoming_handler().call_handle(&mut store, ...) that constructs
    // this resource and the response-outparam.
    pub fn new(
        req: impl crate::http::IncomingRequest + 'static,
        headers: impl crate::http::Fields + 'static,
        body: impl crate::http::IncomingBody + 'static,
    ) -> Self {
        Self {
            req: Box::new(req),
            headers: FieldsResource::new(headers),
            body: Some(Box::new(body)),
        }
    }
}

impl<E: Embedding> types::HostIncomingRequest for EImpl<E> {
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
            Ok(Ok(table.push(body)?))
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
    resp: Box<dyn crate::http::OutgoingResponse>,
    headers: FieldsResource,
    body: Option<Box<dyn crate::http::OutgoingBody>>,
}
impl OutgoingResponseResource {
    pub fn new(
        resp: impl crate::http::OutgoingResponse + 'static,
        headers: impl crate::http::Fields + 'static,
        body: impl crate::http::OutgoingBody + 'static,
    ) -> Self {
        Self {
            resp: Box::new(resp),
            headers: FieldsResource::new(headers),
            body: Some(Box::new(body)),
        }
    }
}

impl<E: Embedding> types::HostOutgoingResponse for EImpl<E> {
    fn new(&mut self, _: Resource<types::Headers>) -> Result<Resource<types::OutgoingResponse>> {
        // FIXME: need some method in Embedding here that returns (impl OutgoingResponse, impl
        // OutgoingBody). then construct the OutgoingResponseResource and stick it into the table.
        todo!()
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
            Ok(Ok(table.push(body)?))
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
    req: Box<dyn crate::http::OutgoingRequest>,
    headers: FieldsResource,
    body: Option<Box<dyn crate::http::OutgoingBody>>,
}
impl OutgoingRequestResource {
    pub fn new(
        req: impl crate::http::OutgoingRequest + 'static,
        headers: impl crate::http::Fields + 'static,
        body: impl crate::http::OutgoingBody + 'static,
    ) -> Self {
        Self {
            req: Box::new(req),
            headers: FieldsResource::new(headers),
            body: Some(Box::new(body)),
        }
    }
}

impl<E: Embedding> types::HostOutgoingRequest for EImpl<E> {
    fn new(&mut self, _: Resource<types::Headers>) -> Result<Resource<types::OutgoingRequest>> {
        // FIXME: need some method in Embedding here that returns (impl OutgoingRequest, impl
        // OutgoingBody). then construct the OutgoingRequestResource and stick it into the table.
        todo!()
    }
    fn body(
        &mut self,
        this: Resource<types::OutgoingRequest>,
    ) -> Result<Result<Resource<types::OutgoingBody>, ()>> {
        let table = self.table();
        // Inner result: only return the OutgoingBody resource once. Subsequent returns error.
        if let Some(body) = table.get_mut(&this)?.body.take() {
            Ok(Ok(table.push(body)?))
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
    resp: Box<dyn crate::http::IncomingResponse>,
    headers: FieldsResource,
    body: Option<Box<dyn crate::http::IncomingBody>>,
}

impl IncomingResponseResource {
    // this will get constructed in the implementation of
    // outgoing-handler.handle, which i guess is gonna be a method in
    // Embedding?
    pub fn new(
        resp: impl crate::http::IncomingResponse + 'static,
        headers: impl crate::http::Fields + 'static,
        body: impl crate::http::IncomingBody + 'static,
    ) -> Self {
        Self {
            resp: Box::new(resp),
            headers: FieldsResource::new(headers),
            body: Some(Box::new(body)),
        }
    }
}

impl<E: Embedding> types::HostIncomingResponse for EImpl<E> {
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
            Ok(Ok(table.push(body)?))
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
    // WRONG - this isnt the future that delivers the Result<IncomingResponse,_>, its a mailbox,
    // delivered by the conclusion of some future thats running in a task. because we need this to
    // resolve without awaiting for it. subscribing to it is optional, get has to work while just
    // busy-looping in the worst case. so even though this mockup has a Pollable impl, its bogus.
    fut: Pin<
        Box<dyn Future<Output = Result<IncomingResponseResource, types::ErrorCode>> + Send + Sync>,
    >,
    res: Option<Result<IncomingResponseResource, types::ErrorCode>>,
    gone: bool,
}
impl FutureIncomingResponse {
    // gets constructed in implementation of outgoing-handler.handle.
    pub fn new(
        fut: impl Future<Output = Result<IncomingResponseResource, types::ErrorCode>>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        Self {
            fut: Box::pin(fut),
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
        let fut = core::pin::pin!(&mut self.fut);
        match fut.poll(cx) {
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

impl<E: Embedding> types::HostFutureIncomingResponse for EImpl<E> {
    fn subscribe(
        &mut self,
        this: Resource<types::FutureIncomingResponse>,
    ) -> Result<Resource<DynPollable>> {
        subscribe(self.table(), this)
    }
    fn get(
        &mut self,
        _: Resource<types::FutureIncomingResponse>,
    ) -> Result<Option<Result<Result<Resource<types::IncomingResponse>, types::ErrorCode>, ()>>>
    {
        todo!()
    }
    fn drop(&mut self, this: Resource<types::FutureIncomingResponse>) -> Result<()> {
        self.table().delete(this)?;
        Ok(())
    }
}

pub type DynIncomingBody = Box<dyn crate::http::IncomingBody>;

impl<E: Embedding> types::HostIncomingBody for EImpl<E> {
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

pub type DynOutgoingBody = Box<dyn crate::http::OutgoingBody>;

impl<E: Embedding> types::HostOutgoingBody for EImpl<E> {
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
pub struct FieldsResource(Rc<dyn crate::http::Fields>);
impl FieldsResource {
    pub fn new(fields: impl crate::http::Fields + 'static) -> Self {
        Self(Rc::new(fields))
    }
}
// SAFETY: single-threaded embedding only
unsafe impl Send for FieldsResource {}
unsafe impl Sync for FieldsResource {}

impl<E: Embedding> types::HostFields for EImpl<E> {
    fn new(&mut self) -> Result<Resource<types::Fields>> {
        todo!()
    }
    fn from_list(
        &mut self,
        _: Vec<(types::FieldKey, types::FieldValue)>,
    ) -> Result<Result<Resource<types::Fields>, types::HeaderError>> {
        todo!()
    }
    fn get(
        &mut self,
        _: Resource<types::Fields>,
        _: types::FieldKey,
    ) -> Result<Vec<types::FieldValue>> {
        todo!()
    }
    fn has(&mut self, _: Resource<types::Fields>, _: types::FieldKey) -> Result<bool> {
        todo!()
    }
    fn set(
        &mut self,
        _: Resource<types::Fields>,
        _: types::FieldKey,
        _: Vec<types::FieldValue>,
    ) -> Result<Result<(), types::HeaderError>> {
        todo!()
    }
    fn delete(
        &mut self,
        _: Resource<types::Fields>,
        _: types::FieldKey,
    ) -> Result<Result<(), types::HeaderError>> {
        todo!()
    }
    fn append(
        &mut self,
        _: Resource<types::Fields>,
        _: types::FieldKey,
        _: types::FieldValue,
    ) -> Result<Result<(), types::HeaderError>> {
        todo!()
    }
    fn entries(
        &mut self,
        _: Resource<types::Fields>,
    ) -> Result<Vec<(types::FieldKey, types::FieldValue)>> {
        todo!()
    }
    fn clone(&mut self, _: Resource<types::Fields>) -> Result<Resource<types::Fields>> {
        todo!()
    }
    fn drop(&mut self, this: Resource<types::Fields>) -> Result<()> {
        self.table().delete(this)?;
        Ok(())
    }
}

pub struct FutureTrailers;

impl<E: Embedding> types::HostFutureTrailers for EImpl<E> {
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

pub type DynResponseOutparam = Box<dyn crate::http::ResponseOutparam>;

impl<E: Embedding> types::HostResponseOutparam for EImpl<E> {
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

pub type DynRequestOptions = Box<dyn crate::http::RequestOptions>;

impl<E: Embedding> types::HostRequestOptions for EImpl<E> {
    fn new(&mut self) -> Result<Resource<types::RequestOptions>> {
        todo!()
    }
    fn connect_timeout(
        &mut self,
        _: Resource<types::RequestOptions>,
    ) -> Result<Option<monotonic_clock::Duration>> {
        todo!()
    }
    fn set_connect_timeout(
        &mut self,
        _: Resource<types::RequestOptions>,
        _: Option<monotonic_clock::Duration>,
    ) -> Result<Result<(), ()>> {
        todo!()
    }
    fn first_byte_timeout(
        &mut self,
        _: Resource<types::RequestOptions>,
    ) -> Result<Option<monotonic_clock::Duration>> {
        todo!()
    }
    fn set_first_byte_timeout(
        &mut self,
        _: Resource<types::RequestOptions>,
        _: Option<monotonic_clock::Duration>,
    ) -> Result<Result<(), ()>> {
        todo!()
    }
    fn between_bytes_timeout(
        &mut self,
        _: Resource<types::RequestOptions>,
    ) -> Result<Option<monotonic_clock::Duration>> {
        todo!()
    }
    fn set_between_bytes_timeout(
        &mut self,
        _: Resource<types::RequestOptions>,
        _: Option<monotonic_clock::Duration>,
    ) -> Result<Result<(), ()>> {
        todo!()
    }
    fn drop(&mut self, this: Resource<types::RequestOptions>) -> Result<()> {
        self.table().delete(this)?;
        Ok(())
    }
}

impl<E: Embedding> types::Host for EImpl<E> {
    fn http_error_code(
        &mut self,
        _: Resource<wasmtime_wasi_io::streams::Error>,
    ) -> Result<Option<types::ErrorCode>> {
        todo!()
    }
}
