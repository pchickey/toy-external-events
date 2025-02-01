pub use crate::bindings::wasi::http::types::{
    ErrorCode, FieldName, FieldValue, HeaderError, Method, Scheme, StatusCode,
};
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::{Cell, RefCell};
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

// Placeholder fields. This will contain pointers to some external resource
// and the methods will set/get values out of there.
pub struct OutgoingResponse {
    pub status_code: Cell<StatusCode>,
}

impl OutgoingResponse {
    pub fn new() -> Self {
        OutgoingResponse {
            status_code: Cell::new(200),
        }
    }

    pub fn status_code(&self) -> StatusCode {
        self.status_code.get()
    }
    pub fn set_status_code(&self, code: StatusCode) -> Result<(), ()> {
        if code < 600 {
            self.status_code.set(code);
            Ok(())
        } else {
            Err(())
        }
    }
}

// Placeholder fields. This will contain pointers to some external resource
// and the methods will set/get values out of there.
pub struct OutgoingRequest {
    pub method: RefCell<Method>,
    pub path_with_query: RefCell<Option<String>>,
    pub scheme: RefCell<Option<Scheme>>,
    pub authority: RefCell<Option<String>>,
}
impl OutgoingRequest {
    pub fn new() -> Self {
        OutgoingRequest {
            method: RefCell::new(Method::Get),
            path_with_query: RefCell::new(None),
            scheme: RefCell::new(None),
            authority: RefCell::new(None),
        }
    }

    pub fn method(&self) -> Method {
        self.method.borrow().clone()
    }
    pub fn set_method(&self, meth: Method) -> Result<(), ()> {
        *self.method.borrow_mut() = meth;
        Ok(())
    }

    pub fn path_with_query(&self) -> Option<String> {
        self.path_with_query.borrow().clone()
    }
    pub fn set_path_with_query(&self, pwq: Option<String>) -> Result<(), ()> {
        *self.path_with_query.borrow_mut() = pwq;
        Ok(())
    }

    pub fn scheme(&self) -> Option<Scheme> {
        self.scheme.borrow().clone()
    }
    pub fn set_scheme(&self, scheme: Option<Scheme>) -> Result<(), ()> {
        *self.scheme.borrow_mut() = scheme;
        Ok(())
    }

    pub fn authority(&self) -> Option<String> {
        self.authority.borrow().clone()
    }
    pub fn set_authority(&self, auth: Option<String>) -> Result<(), ()> {
        *self.authority.borrow_mut() = auth;
        Ok(())
    }

    pub async fn send(
        self,
        _headers: ImmutFields,
        _body: Option<OutgoingBody>,
        _options: Option<RequestOptions>,
    ) -> Result<(IncomingResponse, Fields, IncomingBody), ErrorCode> {
        todo!()
    }
}

// Not doing placeholders here for the moment
pub struct Fields {}
impl Fields {
    pub fn new() -> Self {
        Fields {}
    }
    pub fn insert(&self, _name: FieldName, _value: FieldValue) -> Result<(), HeaderError> {
        todo!()
    }
    pub fn get(&self, _name: &FieldName) -> Vec<&FieldValue> {
        Vec::new()
    }
    pub fn delete(&self, _name: &FieldName) {
        todo!()
    }
    pub fn entries(&self) -> Vec<(FieldName, FieldValue)> {
        Vec::new()
    }
    pub fn into_immut(self) -> ImmutFields {
        ImmutFields {}
    }
}

// Not doing placeholders here for the moment
pub struct ImmutFields {}
impl ImmutFields {
    pub fn get(&self, _name: &FieldName) -> Vec<&FieldValue> {
        Vec::new()
    }
    pub fn entries(&self) -> Vec<(FieldName, FieldValue)> {
        Vec::new()
    }
}

#[derive(Default)]
pub struct RequestOptions {
    pub connect_timeout: Option<Duration>,
    pub first_byte_timeout: Option<Duration>,
    pub between_bytes_timeout: Option<Duration>,
}

// putting off figuring out bodies for later
pub struct IncomingBody {}
pub struct OutgoingBody {}

// This will contain some pointers that know where to write an outgoing response into the
// embedding???
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
