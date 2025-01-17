use anyhow::{anyhow, Result};
use bytes::Bytes;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
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
    tick: u64,
    stdout: UnlimitedWrites,
    stderr: UnlimitedWrites,
}
impl Ctx {
    fn new() -> Self {
        Ctx {
            table: ResourceTable::new(),
            tick: 0,
            stdout: UnlimitedWrites::new(),
            stderr: UnlimitedWrites::new(),
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
        self.tick
    }
    fn monotonic_timer(&self, deadline: u64) -> impl Pollable {
        Deadline(deadline)
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

#[derive(Debug)]
struct Deadline(u64);
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
struct UnlimitedWrites(Arc<Mutex<VecDeque<Bytes>>>);
impl UnlimitedWrites {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(VecDeque::new())))
    }
}
#[wasmtime_wasi_io::async_trait]
impl Pollable for UnlimitedWrites {
    async fn ready(&mut self) {}
}
impl OutputStream for UnlimitedWrites {
    fn check_write(&mut self) -> wasmtime_wasi_io::streams::StreamResult<usize> {
        Ok(usize::MAX)
    }
    fn write(&mut self, contents: Bytes) -> wasmtime_wasi_io::streams::StreamResult<()> {
        self.0.lock().unwrap().push_back(contents);
        Ok(())
    }
    fn flush(&mut self) -> wasmtime_wasi_io::streams::StreamResult<()> {
        Ok(())
    }
}
