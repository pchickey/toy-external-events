use crate::{EImpl, Embedding};
use alloc::string::String;
use alloc::vec::Vec;
use anyhow::Result;
use wasmtime::component::Resource;
use wasmtime_wasi_io::{
    poll::DynPollable,
    streams::{DynInputStream, DynOutputStream},
};

use super::wasi::clocks::monotonic_clock;
use super::wasi::http::types;

impl<E: Embedding> types::HostIncomingRequest for EImpl<E> {
    fn method(&mut self, _: Resource<types::IncomingRequest>) -> Result<types::Method> {
        todo!()
    }
    fn path_with_query(&mut self, _: Resource<types::IncomingRequest>) -> Result<Option<String>> {
        todo!()
    }
    fn scheme(&mut self, _: Resource<types::IncomingRequest>) -> Result<Option<types::Scheme>> {
        todo!()
    }
    fn authority(&mut self, _: Resource<types::IncomingRequest>) -> Result<Option<String>> {
        todo!()
    }
    fn headers(&mut self, _: Resource<types::IncomingRequest>) -> Result<Resource<types::Headers>> {
        todo!()
    }
    fn consume(
        &mut self,
        _: Resource<types::IncomingRequest>,
    ) -> Result<Result<Resource<types::IncomingBody>, ()>> {
        todo!()
    }
    fn drop(&mut self, _: Resource<types::IncomingRequest>) -> Result<()> {
        todo!()
    }
}
impl<E: Embedding> types::HostOutgoingResponse for EImpl<E> {
    fn new(&mut self, _: Resource<types::Headers>) -> Result<Resource<types::OutgoingResponse>> {
        todo!()
    }
    fn status_code(&mut self, _: Resource<types::OutgoingResponse>) -> Result<types::StatusCode> {
        todo!()
    }
    fn set_status_code(
        &mut self,
        _: Resource<types::OutgoingResponse>,
        _: types::StatusCode,
    ) -> Result<Result<(), ()>> {
        todo!()
    }
    fn headers(
        &mut self,
        _: Resource<types::OutgoingResponse>,
    ) -> Result<Resource<types::Headers>> {
        todo!()
    }
    fn body(
        &mut self,
        _: Resource<types::OutgoingResponse>,
    ) -> Result<Result<Resource<types::OutgoingBody>, ()>> {
        todo!()
    }
    fn drop(&mut self, _: Resource<types::OutgoingResponse>) -> Result<()> {
        todo!()
    }
}
impl<E: Embedding> types::HostOutgoingRequest for EImpl<E> {
    fn new(&mut self, _: Resource<types::Headers>) -> Result<Resource<types::OutgoingRequest>> {
        todo!()
    }
    fn body(
        &mut self,
        _: Resource<types::OutgoingRequest>,
    ) -> Result<Result<Resource<types::OutgoingBody>, ()>> {
        todo!()
    }
    fn method(&mut self, _: Resource<types::OutgoingRequest>) -> Result<types::Method> {
        todo!()
    }
    fn set_method(
        &mut self,
        _: Resource<types::OutgoingRequest>,
        _: types::Method,
    ) -> Result<Result<(), ()>> {
        todo!()
    }
    fn path_with_query(&mut self, _: Resource<types::OutgoingRequest>) -> Result<Option<String>> {
        todo!()
    }
    fn set_path_with_query(
        &mut self,
        _: Resource<types::OutgoingRequest>,
        _: Option<String>,
    ) -> Result<Result<(), ()>> {
        todo!()
    }
    fn scheme(&mut self, _: Resource<types::OutgoingRequest>) -> Result<Option<types::Scheme>> {
        todo!()
    }
    fn set_scheme(
        &mut self,
        _: Resource<types::OutgoingRequest>,
        _: Option<types::Scheme>,
    ) -> Result<Result<(), ()>> {
        todo!()
    }
    fn authority(&mut self, _: Resource<types::OutgoingRequest>) -> Result<Option<String>> {
        todo!()
    }
    fn set_authority(
        &mut self,
        _: Resource<types::OutgoingRequest>,
        _: Option<String>,
    ) -> Result<Result<(), ()>> {
        todo!()
    }
    fn headers(&mut self, _: Resource<types::OutgoingRequest>) -> Result<Resource<types::Headers>> {
        todo!()
    }
    fn drop(&mut self, _: Resource<types::OutgoingRequest>) -> Result<()> {
        todo!()
    }
}
impl<E: Embedding> types::HostIncomingResponse for EImpl<E> {
    fn status(&mut self, _: Resource<types::IncomingResponse>) -> Result<types::StatusCode> {
        todo!()
    }
    fn headers(
        &mut self,
        _: Resource<types::IncomingResponse>,
    ) -> Result<Resource<types::Headers>> {
        todo!()
    }
    fn consume(
        &mut self,
        _: Resource<types::IncomingResponse>,
    ) -> Result<Result<Resource<types::IncomingBody>, ()>> {
        todo!()
    }
    fn drop(&mut self, _: Resource<types::IncomingResponse>) -> Result<()> {
        todo!()
    }
}
impl<E: Embedding> types::HostFutureIncomingResponse for EImpl<E> {
    fn subscribe(
        &mut self,
        _: Resource<types::FutureIncomingResponse>,
    ) -> Result<Resource<DynPollable>> {
        todo!()
    }
    fn get(
        &mut self,
        _: Resource<types::FutureIncomingResponse>,
    ) -> Result<Option<Result<Result<Resource<types::IncomingResponse>, types::ErrorCode>, ()>>>
    {
        todo!()
    }
    fn drop(&mut self, _: Resource<types::FutureIncomingResponse>) -> Result<()> {
        todo!()
    }
}
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
    fn drop(&mut self, _: Resource<types::IncomingBody>) -> Result<()> {
        todo!()
    }
}
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
    fn drop(&mut self, _: Resource<types::OutgoingBody>) -> Result<()> {
        todo!()
    }
}
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
    fn drop(&mut self, _: Resource<types::Fields>) -> Result<()> {
        todo!()
    }
}
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
    fn drop(&mut self, _: Resource<types::FutureTrailers>) -> Result<()> {
        todo!()
    }
}
impl<E: Embedding> types::HostResponseOutparam for EImpl<E> {
    fn set(
        &mut self,
        _: Resource<types::ResponseOutparam>,
        _: Result<Resource<types::OutgoingResponse>, types::ErrorCode>,
    ) -> Result<()> {
        todo!()
    }
    fn drop(&mut self, _: Resource<types::ResponseOutparam>) -> Result<()> {
        todo!()
    }
}
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
    fn drop(&mut self, _: Resource<types::RequestOptions>) -> Result<()> {
        todo!()
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
