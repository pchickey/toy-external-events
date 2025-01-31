use crate::{Clock, Deadline, NeverReadable, TimestampedWrites};
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
    clock: Clock,
    stdout: TimestampedWrites,
    stderr: TimestampedWrites,
}

impl EmbeddingCtx {
    pub fn new(clock: Clock) -> Self {
        let stdout = TimestampedWrites::new(clock.clone());
        let stderr = TimestampedWrites::new(clock.clone());

        EmbeddingCtx {
            table: ResourceTable::new(),
            clock,
            stdout,
            stderr,
        }
    }

    pub fn report(&self, out: &mut impl core::fmt::Write) -> core::fmt::Result {
        core::write!(out, "stdout:\n")?;
        self.stdout.report(out)?;
        core::write!(out, "stderr:\n")?;
        self.stderr.report(out)
    }
    pub(crate) fn monotonic_now(&self) -> u64 {
        let now = self.clock.get();
        //println!("wasm told now is: {now}");
        now
    }
    pub(crate) fn monotonic_timer(&self, deadline: u64) -> impl Pollable {
        Deadline::new(self.clock.clone(), deadline)
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
