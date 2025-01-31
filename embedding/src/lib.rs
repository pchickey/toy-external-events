#![no_std]
extern crate alloc;

mod bindings;
mod ctx;
mod http;
mod runtime;

use ctx::EmbeddingCtx;
use runtime::Executor;

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::rc::Rc;
use alloc::string::String;
use anyhow::Result;
use async_task::Task;
use bytes::Bytes;
use core::cell::{Cell, RefCell};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};

use wasmtime_wasi_io::poll::Pollable;
use wasmtime_wasi_io::streams::{InputStream, OutputStream};

use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};

pub struct Runtime {
    engine: Engine,
    linker: Linker<EmbeddingCtx>,
}

impl Runtime {
    pub fn new() -> Result<Self> {
        let mut config = Config::new();
        config.async_support(true);
        let engine = Engine::new(&config)?;
        let mut linker = Linker::new(&engine);
        wasmtime_wasi_io::add_to_linker_async(&mut linker)?;
        bindings::add_to_linker_async(&mut linker)?;
        Ok(Runtime { engine, linker })
    }

    pub fn load(&self, cwasm: &[u8]) -> Result<RunnableComponent> {
        let component = unsafe { Component::deserialize(&self.engine, cwasm)? };
        let instance_pre = self.linker.instantiate_pre(&component)?;
        let bindings_pre = bindings::BindingsPre::new(instance_pre)?;
        Ok(RunnableComponent {
            engine: self.engine.clone(),
            bindings_pre,
        })
    }
}

pub struct RunnableComponent {
    engine: Engine,
    bindings_pre: bindings::BindingsPre<EmbeddingCtx>,
}

impl RunnableComponent {
    pub fn create(&self) -> Result<RunningComponent> {
        let clock = Clock::new();
        let mut store = Store::new(&self.engine, EmbeddingCtx::new(clock.clone()));
        let bindings_pre = self.bindings_pre.clone();
        let fut = async move {
            let instance = bindings_pre.instantiate_async(&mut store).await?;
            instance
                .wasi_cli_run()
                .call_run(&mut store)
                .await?
                .map_err(|()| anyhow::anyhow!("cli run exited with error"))?;
            Ok::<_, anyhow::Error>(store.into_data())
        };
        let executor = Executor::new();
        let task = executor.spawn(fut);

        Ok(RunningComponent {
            clock,
            executor,
            output: Box::pin(task),
        })
    }
}

pub struct RunningComponent {
    clock: Clock,
    executor: Executor,
    output: Pin<Box<Task<Result<EmbeddingCtx>>>>,
}

impl RunningComponent {
    pub fn earliest_deadline(&self) -> Option<u64> {
        self.executor.earliest_deadline()
    }

    pub fn increment_clock(&self) {
        self.clock.set(self.clock.get() + 1);
        self.check_for_wake();
    }

    pub fn advance_clock(&self, to: u64) {
        self.clock.set(to);
        self.check_for_wake();
    }

    fn check_for_wake(&self) {
        for waker in self.executor.ready_deadlines(self.clock.get()) {
            waker.wake()
        }
    }

    pub fn step(&mut self) -> usize {
        self.executor.step()
    }

    pub fn check_complete(&mut self) -> Option<Result<String>> {
        match self
            .output
            .as_mut()
            .poll(&mut Context::from_waker(&noop_waker()))
        {
            Poll::Pending => None,
            Poll::Ready(Ok(ctx)) => {
                let mut out = String::new();
                if let Err(e) = ctx.report(&mut out) {
                    return Some(Err(e.into()));
                }
                Some(Ok(out))
            }
            Poll::Ready(Err(e)) => Some(Err(e)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Clock(Rc<Cell<u64>>);
impl Clock {
    pub fn new() -> Self {
        Clock(Rc::new(Cell::new(0)))
    }
    pub fn get(&self) -> u64 {
        self.0.get()
    }
    fn set(&self, to: u64) {
        //println!("clock advancing to {to}");
        self.0.set(to)
    }
}
// SAFETY: only will consume this crate in single-threaded environment
unsafe impl Send for Clock {}
unsafe impl Sync for Clock {}

#[derive(Debug, Clone)]
pub struct Deadline {
    clock: Clock,
    due: u64,
}
impl Deadline {
    fn new(clock: Clock, due: u64) -> Self {
        Self { clock, due }
    }
}
impl Future for Deadline {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let now = self.clock.get();
        if now < self.due {
            Executor::current().push_deadline(self.due, cx.waker().clone());
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}
// SAFETY: only will consume this crate in single-threaded environment
unsafe impl Send for Deadline {}
unsafe impl Sync for Deadline {}

#[wasmtime_wasi_io::async_trait]
impl Pollable for Deadline {
    async fn ready(&mut self) {
        self.clone().await
    }
}

struct NeverReadable;
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
struct TimestampedWrites {
    clock: Clock,
    log: Rc<RefCell<VecDeque<(u64, Bytes)>>>,
}
impl TimestampedWrites {
    fn new(clock: Clock) -> Self {
        Self {
            clock,
            log: Rc::new(RefCell::new(VecDeque::new())),
        }
    }
    fn report(&self, out: &mut impl core::fmt::Write) -> core::fmt::Result {
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

// Yanked from core::task::wake, which is unfortunately still unstable :/
fn noop_waker() -> Waker {
    use core::task::{RawWaker, RawWakerVTable};
    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        // Cloning just returns a new no-op raw waker
        |_| RAW,
        // `wake` does nothing
        |_| {},
        // `wake_by_ref` does nothing
        |_| {},
        // Dropping does nothing as we don't allocate anything
        |_| {},
    );
    const RAW: RawWaker = RawWaker::new(core::ptr::null(), &VTABLE);

    unsafe { Waker::from_raw(RAW) }
}
