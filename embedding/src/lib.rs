#![no_std]
extern crate alloc;

mod runtime;
use runtime::Executor;

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::rc::Rc;
use alloc::string::String;
use anyhow::Result;
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
    linker: Linker<Ctx>,
}

impl Runtime {
    pub fn new() -> Result<Self> {
        let mut config = Config::new();
        config.async_support(true);
        let engine = Engine::new(&config)?;
        let mut linker = Linker::new(&engine);
        wasmtime_wasi_io::add_to_linker_async(&mut linker)?;
        embeddable::add_to_linker_async(&mut linker)?;
        Ok(Runtime { engine, linker })
    }

    pub fn load(&self, cwasm: &[u8]) -> Result<RunnableComponent> {
        let component = unsafe { Component::deserialize(&self.engine, cwasm)? };
        let instance_pre = self.linker.instantiate_pre(&component)?;
        let bindings_pre = embeddable::BindingsPre::new(instance_pre)?;
        Ok(RunnableComponent {
            engine: self.engine.clone(),
            bindings_pre,
        })
    }
}

pub struct RunnableComponent {
    engine: Engine,
    bindings_pre: embeddable::BindingsPre<Ctx>,
}

impl RunnableComponent {
    pub fn create(&self) -> Result<RunningComponent> {
        let clock = Clock::new();
        let mut store = Store::new(&self.engine, Ctx::new(clock.clone()));
        let bindings_pre = self.bindings_pre.clone();
        let fut = async move {
            let instance = bindings_pre.instantiate_async(&mut store).await?;
            instance
                .wasi_cli_run()
                .call_run(&mut store)
                .await?
                .map_err(|()| anyhow::anyhow!(""))?;
            Ok(store.into_data())
        };
        let executor = Executor::new();

        let waker = noop_waker();

        Ok(RunningComponent {
            clock,
            executor,
            waker,
            fut: Some(Box::pin(fut)),
            output: None,
        })
    }
}

pub struct RunningComponent {
    clock: Clock,
    executor: Executor,
    waker: Waker,
    fut: Option<Pin<Box<dyn Future<Output = Result<Ctx>>>>>,
    output: Option<Result<Ctx>>,
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

    pub fn step(&mut self, poll_calls: usize) {
        self.executor.with(|| {
            let mut fut = self.fut.take().unwrap();

            let mut cx = Context::from_waker(&self.waker);

            for _ in 0..poll_calls {
                match fut.as_mut().poll(&mut cx) {
                    Poll::Pending => {}
                    Poll::Ready(res) => {
                        self.output = Some(res);
                        break;
                    }
                }
            }

            self.fut = Some(fut);
        })
    }
    pub fn check_complete(&mut self) -> Option<Result<String>> {
        match self.output.take() {
            None => None,
            Some(Ok(ctx)) => {
                let mut out = String::new();
                if let Err(e) = ctx.report(&mut out) {
                    return Some(Err(e.into()));
                }
                Some(Ok(out))
            }
            Some(Err(e)) => Some(Err(e)),
        }
    }
}

struct Ctx {
    table: ResourceTable,
    clock: Clock,
    stdout: TimestampedWrites,
    stderr: TimestampedWrites,
}
impl Ctx {
    pub fn new(clock: Clock) -> Self {
        let stdout = TimestampedWrites::new(clock.clone());
        let stderr = TimestampedWrites::new(clock.clone());

        Ctx {
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
}
impl wasmtime_wasi_io::IoView for Ctx {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl embeddable::Embedding for Ctx {
    fn monotonic_now(&self) -> u64 {
        let now = self.clock.get();
        //println!("wasm told now is: {now}");
        now
    }
    fn monotonic_timer(&self, deadline: u64) -> impl Pollable {
        Deadline::new(self.clock.clone(), deadline)
    }
    fn stdin(&self) -> impl InputStream {
        NeverReadable
    }
    fn stdout(&self) -> impl OutputStream {
        self.stdout.clone()
    }
    fn stderr(&self) -> impl OutputStream {
        self.stderr.clone()
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
