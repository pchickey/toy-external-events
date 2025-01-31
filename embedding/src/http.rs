pub use crate::bindings::wasi::http::types::{Method, Scheme, StatusCode};
use alloc::string::String;
use core::time::Duration;

pub struct IncomingRequest {}

impl IncomingRequest {
    pub fn method(&self) -> Method {
        todo!()
    }
    pub fn path_with_query(&self) -> Option<String> {
        todo!()
    }
    pub fn scheme(&self) -> Option<Scheme> {
        todo!()
    }
    pub fn authority(&self) -> Option<String> {
        todo!()
    }
}

pub struct IncomingResponse {}

impl IncomingResponse {
    pub fn status_code(&self) -> StatusCode {
        todo!()
    }
}

pub struct OutgoingResponse {}

impl OutgoingResponse {
    pub fn status_code(&self) -> StatusCode {
        todo!()
    }
    pub fn set_status_code(&self, _: StatusCode) -> Result<(), ()> {
        todo!()
    }
}

pub struct OutgoingRequest {}
impl OutgoingRequest {
    pub fn method(&self) -> Method {
        todo!()
    }
    pub fn set_method(&self, _: Method) -> Result<(), ()> {
        todo!()
    }

    pub fn path_with_query(&self) -> Option<String> {
        todo!()
    }
    pub fn set_path_with_query(&self, _: Option<String>) -> Result<(), ()> {
        todo!()
    }

    pub fn scheme(&self) -> Option<Scheme> {
        todo!()
    }
    pub fn set_scheme(&self, _: Option<Scheme>) -> Result<(), ()> {
        todo!()
    }

    pub fn authority(&self) -> Option<String> {
        todo!()
    }
    pub fn set_authority(&self, _: Option<String>) -> Result<(), ()> {
        todo!()
    }
}

pub struct Fields {}
impl Fields {
    pub fn into_immut(self) -> ImmutFields {
        ImmutFields {}
    }
}
pub struct ImmutFields {}

#[derive(Default)]
pub struct RequestOptions {
    pub connect_timeout: Option<Duration>,
    pub first_byte_timeout: Option<Duration>,
    pub between_bytes_timeout: Option<Duration>,
}

pub struct IncomingBody {}
pub struct OutgoingBody {}

pub struct ResponseOutparam {}
