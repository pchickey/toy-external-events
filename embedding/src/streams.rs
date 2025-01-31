use crate::clock::Clock;

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::rc::Rc;
use alloc::string::String;
use bytes::Bytes;
use core::cell::RefCell;

use wasmtime_wasi_io::poll::Pollable;
use wasmtime_wasi_io::streams::{InputStream, OutputStream};

pub struct NeverReadable;
#[wasmtime_wasi_io::async_trait]
impl Pollable for NeverReadable {
    async fn ready(&mut self) {
        futures_lite::future::pending().await
    }
}
impl InputStream for NeverReadable {
    fn read(&mut self, _: usize) -> wasmtime_wasi_io::streams::StreamResult<Bytes> {
        unreachable!("never ready for reading")
    }
}

#[derive(Clone)]
pub struct TimestampedWrites {
    clock: Clock,
    log: Rc<RefCell<VecDeque<(u64, Bytes)>>>,
}
impl TimestampedWrites {
    pub fn new(clock: Clock) -> Self {
        Self {
            clock,
            log: Rc::new(RefCell::new(VecDeque::new())),
        }
    }
    pub fn report(&self, out: &mut impl core::fmt::Write) -> core::fmt::Result {
        for (time, line) in self.log.borrow_mut().iter() {
            write!(out, "{:08} {:?}\n", time, String::from_utf8_lossy(line))?;
        }
        Ok(())
    }
}
// SAFETY: only will consume this crate in single-threaded environment
unsafe impl Send for TimestampedWrites {}
unsafe impl Sync for TimestampedWrites {}

#[wasmtime_wasi_io::async_trait]
impl Pollable for TimestampedWrites {
    async fn ready(&mut self) {}
}
impl OutputStream for TimestampedWrites {
    fn check_write(&mut self) -> wasmtime_wasi_io::streams::StreamResult<usize> {
        Ok(usize::MAX)
    }
    fn write(&mut self, contents: Bytes) -> wasmtime_wasi_io::streams::StreamResult<()> {
        let time = self.clock.get();
        self.log.borrow_mut().push_back((time, contents));
        Ok(())
    }
    fn flush(&mut self) -> wasmtime_wasi_io::streams::StreamResult<()> {
        Ok(())
    }
}
