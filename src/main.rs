use anyhow::{anyhow, Result};
use bytes::Bytes;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use wasmtime_wasi_io::poll::Pollable;
use wasmtime_wasi_io::streams::{InputStream, OutputStream};

use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};

fn main() -> Result<()> {
    let mut args = std::env::args();
    let _current_exe = args.next();
    let wasm_path = args
        .next()
        .ok_or_else(|| anyhow!("missing required argument: wasm path"))?;

    let mut config = Config::new();
    config.async_support(true);
    let engine = Engine::new(&config)?;

    let component = Component::from_file(&engine, wasm_path)?;

    let mut linker: Linker<Ctx> = Linker::new(&engine);
    wasmtime_wasi_io::add_to_linker_async(&mut linker)?;
    toy_external_events::add_to_linker_async(&mut linker)?;
    let instance_pre = linker.instantiate_pre(&component)?;
    let command_pre = toy_external_events::CommandPre::new(instance_pre)?;

    async {
        let mut store = Store::new(&engine, Ctx::new());
        let instance = command_pre.instantiate_async(&mut store).await?;

        Ok::<_, anyhow::Error>(())
    };

    Ok(())
}

struct Ctx {
    table: ResourceTable,
    clock: Clock,
    stdout: TimestampedWrites,
    stderr: TimestampedWrites,
}
impl Ctx {
    fn new() -> Self {
        let clock = Clock::new();
        let stdout = TimestampedWrites::new(clock.clone());
        let stderr = TimestampedWrites::new(clock.clone());

        Ctx {
            table: ResourceTable::new(),
            clock,
            stdout,
            stderr,
        }
    }
}
impl wasmtime_wasi_io::IoView for Ctx {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl toy_external_events::Embedding for Ctx {
    fn monotonic_now(&self) -> u64 {
        self.clock.get()
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
struct Clock(Arc<Mutex<u64>>);
impl Clock {
    pub fn new() -> Self {
        Clock(Arc::new(Mutex::new(0)))
    }
    pub fn get(&self) -> u64 {
        *self.0.lock().unwrap()
    }
}

#[derive(Debug, Clone)]
struct Deadline {
    clock: Clock,
    deadline: u64,
}
impl Deadline {
    fn new(clock: Clock, deadline: u64) -> Self {
        Self { clock, deadline }
    }
}
impl Future for Deadline {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let now = self.clock.get();
        if now > self.deadline {
            todo!("register waker with executor!!!");
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}
#[wasmtime_wasi_io::async_trait]
impl Pollable for Deadline {
    async fn ready(&mut self) {
        todo!()
    }
}

struct NeverReadable;
#[wasmtime_wasi_io::async_trait]
impl Pollable for NeverReadable {
    async fn ready(&mut self) {
        futures::future::pending().await
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
    log: Arc<Mutex<VecDeque<(u64, Bytes)>>>,
}
impl TimestampedWrites {
    fn new(clock: Clock) -> Self {
        Self {
            clock,
            log: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}
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
        self.log.lock().unwrap().push_back((time, contents));
        Ok(())
    }
    fn flush(&mut self) -> wasmtime_wasi_io::streams::StreamResult<()> {
        Ok(())
    }
}

struct Executor {
    deadlines: Vec<(Deadline, std::task::Waker)>,
}
