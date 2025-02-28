use crate::clock::{Clock, Deadline};
use crate::runtime::Executor;
use crate::streams::{NeverReadable, TimestampedWrites};
use alloc::string::String;
use wasmtime::component::ResourceTable;
use wasmtime_wasi_io::{
    poll::Pollable,
    streams::{InputStream, OutputStream},
    IoImpl, IoView,
};

pub struct EImpl<T>(IoImpl<T>);
impl<T: IoView> IoView for EImpl<T> {
    fn table(&mut self) -> &mut ResourceTable {
        T::table(&mut self.0 .0)
    }
}

pub struct EmbeddingCtx {
    table: ResourceTable,
    executor: Executor,
    clock: Clock,
    stdout: TimestampedWrites,
    stderr: TimestampedWrites,
}

impl EmbeddingCtx {
    pub fn new(executor: Executor, clock: Clock) -> Self {
        let stdout = TimestampedWrites::new(clock.clone());
        let stderr = TimestampedWrites::new(clock.clone());

        EmbeddingCtx {
            table: ResourceTable::new(),
            executor,
            clock,
            stdout,
            stderr,
        }
    }

    pub fn report(&self) -> String {
        use core::fmt::Write;
        let mut out = String::new();
        core::write!(&mut out, "stdout:\n").unwrap();
        self.stdout.report(&mut out).unwrap();
        core::write!(&mut out, "stderr:\n").unwrap();
        self.stderr.report(&mut out).unwrap();
        out
    }
    pub(crate) fn monotonic_now(&self) -> u64 {
        let now = self.clock.get();
        //println!("wasm told now is: {now}");
        now
    }
    pub(crate) fn monotonic_timer(&self, deadline: u64) -> impl Pollable {
        Deadline::new(self.executor.clone(), self.clock.clone(), deadline)
    }
    pub(crate) fn executor(&self) -> &Executor {
        &self.executor
    }
    pub(crate) fn stdin(&self) -> impl InputStream {
        NeverReadable
    }
    pub(crate) fn stdout(&self) -> impl OutputStream {
        self.stdout.clone()
    }
    pub(crate) fn stderr(&self) -> impl OutputStream {
        self.stderr.clone()
    }

    // FIXME additions here along the lines of:
    // fn create_fields(&self) -> impl Fields
    // fn create_outgoing_request(&self) -> (impl OutgoingRequest, impl OutgoingBody)
    // fn create_outgoing_response(&self) -> (impl OutgoingResponse, impl OutgoingBody)
    // fn outbound_http(&self, outgoing request) -> mailbox<incoming response>
}
impl wasmtime_wasi_io::IoView for EmbeddingCtx {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}
// SAFETY: only will consume this crate in single-threaded environment
unsafe impl Send for EmbeddingCtx {}
unsafe impl Sync for EmbeddingCtx {}
