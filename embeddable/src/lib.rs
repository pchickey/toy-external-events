#![no_std]
extern crate alloc;

use wasmtime::component::ResourceTable;
use wasmtime_wasi_io::{
    poll::Pollable,
    streams::{InputStream, OutputStream},
    IoImpl, IoView,
};

mod bindings;
pub mod http;

pub use bindings::{add_to_linker_async, Bindings, BindingsPre};

pub struct EImpl<T>(IoImpl<T>);
impl<T: IoView> IoView for EImpl<T> {
    fn table(&mut self) -> &mut ResourceTable {
        T::table(&mut self.0 .0)
    }
}
pub trait Embedding: wasmtime_wasi_io::IoView {
    fn monotonic_now(&self) -> u64;
    fn monotonic_timer(&self, deadline: u64) -> impl Pollable;
    fn stdin(&self) -> impl InputStream;
    fn stdout(&self) -> impl OutputStream;
    fn stderr(&self) -> impl OutputStream;

    // FIXME additions here along the lines of:
    // fn create_outgoing_request(&self) -> (impl OutgoingRequest, impl OutgoingBody)
    // fn create_outgoing_response(&self) -> (impl OutgoingResponse, impl OutgoingBody)
    // fn outbound_http(&self, outgoing request) -> mailbox<incoming response>
}

impl<T: Embedding> Embedding for EImpl<T> {
    fn monotonic_now(&self) -> u64 {
        T::monotonic_now(&self.0 .0)
    }
    fn monotonic_timer(&self, deadline: u64) -> impl Pollable {
        T::monotonic_timer(&self.0 .0, deadline)
    }
    fn stdin(&self) -> impl InputStream {
        T::stdin(&self.0 .0)
    }
    fn stdout(&self) -> impl OutputStream {
        T::stdout(&self.0 .0)
    }
    fn stderr(&self) -> impl OutputStream {
        T::stderr(&self.0 .0)
    }
}

impl<T: ?Sized + Embedding> Embedding for &mut T {
    fn monotonic_now(&self) -> u64 {
        T::monotonic_now(self)
    }
    fn monotonic_timer(&self, deadline: u64) -> impl Pollable {
        T::monotonic_timer(self, deadline)
    }
    fn stdin(&self) -> impl InputStream {
        T::stdin(self)
    }
    fn stdout(&self) -> impl OutputStream {
        T::stdout(self)
    }
    fn stderr(&self) -> impl OutputStream {
        T::stderr(self)
    }
}
