pub use crate::bindings::wasi::http::types::{
    ErrorCode, FieldName, FieldValue, HeaderError, Method, Scheme, StatusCode,
};
use alloc::string::String;
use alloc::vec::Vec;
use core::time::Duration;

// Placeholder fields. This will contain pointers to some external resource
// and the methods will retrieve these values out of there.
pub struct IncomingRequest {
    pub method: Method,
    pub path_with_query: Option<String>,
    pub scheme: Option<Scheme>,
    pub authority: Option<String>,
}

impl IncomingRequest {
    pub fn method(&self) -> Method {
        self.method.clone()
    }
    pub fn path_with_query(&self) -> Option<String> {
        self.path_with_query.clone()
    }
    pub fn scheme(&self) -> Option<Scheme> {
        self.scheme.clone()
    }
    pub fn authority(&self) -> Option<String> {
        self.authority.clone()
    }
}

// Placeholder fields. This will contain pointers to some external resource
// and the methods will retrieve these values out of there.
pub struct IncomingResponse {
    pub status_code: StatusCode,
}

impl IncomingResponse {
    pub fn status_code(&self) -> StatusCode {
        self.status_code.clone()
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
    pub fn new() -> Self {
        Fields {}
    }
    pub fn insert(&self, _name: FieldName, _value: FieldValue) -> Result<(), HeaderError> {
        todo!()
    }
    pub fn get(&self, _name: &FieldName) -> Vec<&FieldValue> {
        todo!()
    }
    pub fn delete(&self, _name: &FieldName) {
        todo!()
    }
    pub fn entries(&self) -> Vec<(FieldName, FieldValue)> {
        todo!()
    }
    pub fn into_immut(self) -> ImmutFields {
        ImmutFields {}
    }
}
pub struct ImmutFields {}
impl ImmutFields {
    pub fn get(&self, _name: &FieldName) -> Vec<&FieldValue> {
        todo!()
    }
    pub fn entries(&self) -> Vec<(FieldName, FieldValue)> {
        todo!()
    }
}

#[derive(Default)]
pub struct RequestOptions {
    pub connect_timeout: Option<Duration>,
    pub first_byte_timeout: Option<Duration>,
    pub between_bytes_timeout: Option<Duration>,
}

pub struct IncomingBody {}
pub struct OutgoingBody {}

pub struct ResponseOutparam {}
impl ResponseOutparam {
    pub fn send_success(
        self,
        _resp: OutgoingResponse,
        _headers: Fields,
        _body: Option<OutgoingBody>,
    ) {
        todo!()
    }
    pub fn send_error(self, _err: ErrorCode) {
        todo!()
    }
}
