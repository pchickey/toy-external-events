pub use crate::bindings::wasi::http::types::{
    ErrorCode, FieldName, FieldValue, HeaderError, Method, Scheme, StatusCode,
};
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::{Cell, RefCell};
use core::time::Duration;

// Placeholder fields. This will contain pointers to some external resource
// and the methods will retrieve these values out of there.
#[derive(Debug)]
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
#[derive(Debug)]
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
#[derive(Debug)]
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
#[derive(Debug)]
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

// Minimum viable implementation
#[derive(Debug)]
pub struct Fields {
    pairs: RefCell<Vec<(String, String)>>,
}
impl Fields {
    pub fn new() -> Self {
        Fields {
            pairs: RefCell::new(Vec::new()),
        }
    }
    pub fn insert(&self, name: FieldName, value: FieldValue) -> Result<(), HeaderError> {
        // FIXME: need to reject any forbidden headers here (content-length etc)
        let name = name.to_lowercase();
        let value = String::from_utf8(value).map_err(|_| HeaderError::InvalidSyntax)?;
        self.pairs.borrow_mut().push((name, value));
        Ok(())
    }
    pub fn get(&self, name: &FieldName) -> Vec<FieldValue> {
        let name = name.to_lowercase();
        self.pairs
            .borrow()
            .iter()
            .filter_map(|(k, v)| {
                if *k == name {
                    Some(v.as_bytes().to_vec())
                } else {
                    None
                }
            })
            .collect()
    }
    pub fn delete(&self, name: &FieldName) {
        let name = name.to_lowercase();
        self.pairs.borrow_mut().retain(|(k, _)| *k != name);
    }
    pub fn entries(&self) -> Vec<(FieldName, FieldValue)> {
        self.pairs
            .borrow()
            .iter()
            .map(|(k, v)| (k.clone(), v.as_bytes().to_vec()))
            .collect()
    }
    pub fn into_immut(self) -> ImmutFields {
        ImmutFields {
            pairs: self.pairs.into_inner(),
        }
    }
}

// Minimum viable implementation
#[derive(Debug)]
pub struct ImmutFields {
    pairs: Vec<(String, String)>,
}
impl ImmutFields {
    pub fn get(&self, name: &FieldName) -> Vec<FieldValue> {
        let name = name.to_lowercase();
        self.pairs
            .iter()
            .filter_map(|(k, v)| {
                if *k == name {
                    Some(v.as_bytes().to_vec())
                } else {
                    None
                }
            })
            .collect()
    }
    pub fn entries(&self) -> Vec<(FieldName, FieldValue)> {
        self.pairs
            .iter()
            .map(|(k, v)| (k.clone(), v.as_bytes().to_vec()))
            .collect()
    }
}

#[derive(Default, Debug)]
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
#[derive(Clone)]
pub struct ResponseOutparam {
    mailbox: alloc::rc::Rc<
        core::cell::RefCell<Option<Result<(OutgoingResponse, ImmutFields), ErrorCode>>>,
    >,
}
// SAFETY: single threaded
unsafe impl Send for ResponseOutparam {}
unsafe impl Sync for ResponseOutparam {}
impl ResponseOutparam {
    pub fn new() -> Self {
        Self {
            mailbox: alloc::rc::Rc::new(core::cell::RefCell::new(None)),
        }
    }
    pub fn send_success(
        self,
        resp: OutgoingResponse,
        headers: ImmutFields,
        _body: Option<OutgoingBody>,
    ) {
        *self.mailbox.borrow_mut() = Some(Ok((resp, headers)));
    }
    pub fn send_error(self, err: ErrorCode) {
        *self.mailbox.borrow_mut() = Some(Err(err));
    }
    pub fn into_inner(self) -> anyhow::Result<(OutgoingResponse, ImmutFields)> {
        Ok(self
            .mailbox
            .borrow_mut()
            .take()
            .ok_or_else(|| anyhow::anyhow!("no response sent to outparam"))??)
    }
}
