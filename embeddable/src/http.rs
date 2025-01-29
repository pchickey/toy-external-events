pub use crate::bindings::wasi::http::types::{Method, Scheme, StatusCode};
use alloc::string::String;

pub trait IncomingRequest: Send {
    fn method(&self) -> Method;
    fn path_with_query(&self) -> Option<String>;
    fn scheme(&self) -> Option<Scheme>;
    fn authority(&self) -> Option<String>;
}
pub trait IncomingResponse: Send {
    fn status_code(&self) -> StatusCode;
}
pub trait OutgoingResponse: Send {
    fn status_code(&self) -> StatusCode;
    fn set_status_code(&self, _: StatusCode) -> Result<(), ()>;
}

pub trait OutgoingRequest: Send {
    fn method(&self) -> Method;
    fn set_method(&self, _: Method) -> Result<(), ()>;

    fn path_with_query(&self) -> Option<String>;
    fn set_path_with_query(&self, _: Option<String>) -> Result<(), ()>;

    fn scheme(&self) -> Option<Scheme>;
    fn set_scheme(&self, _: Option<Scheme>) -> Result<(), ()>;

    fn authority(&self) -> Option<String>;
    fn set_authority(&self, _: Option<String>) -> Result<(), ()>;
}

// TODO: these have pretty straightforward set of getters/setters reflecting
// the resource
pub trait Fields: Send {}
pub trait RequestOptions: Send {}

// FIXME: idk if either of these are even traits. design just not fleshed out for these,
// need to keep thinking about it.
pub trait IncomingBody: Send {}
pub trait OutgoingBody: Send {}

// FIXME: not fleshed out either, idk
pub trait ResponseOutparam: Send {}
