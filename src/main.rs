use anyhow::{anyhow, Result};

use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};

fn main() -> Result<()> {
    let mut args = std::env::args();
    let _current_exe = args.next();
    let cwasm_path = args
        .next()
        .ok_or_else(|| anyhow!("missing required argument: cwasm path"))?;

    let mut config = Config::new();
    config.async_support(true);
    let engine = Engine::new(&config)?;

    let component = unsafe { Component::deserialize(&engine, cwasm_path)? };

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
    start: std::time::Instant,
}
impl Ctx {
    fn new() -> Self {
        Ctx {
            table: ResourceTable::new(),
            start: std::time::Instant::now(),
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
        std::time::Instant::now()
            .duration_since(self.start)
            .as_micros() as u64
    }
}
