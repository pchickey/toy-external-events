use anyhow::{anyhow, Result};
use wasmtime::component::Component;
use wasmtime::{Config, Engine};

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
    let cwasm = component.serialize()?;

    let runtime = embedding::Runtime::new()?;
    let runnable_component = runtime.load(&cwasm)?;
    let mut running_component = runnable_component.create()?;

    loop {
        running_component.step();
        if let Some(report) = running_component.check_complete() {
            let report = report?;
            println!("{report}");
            return Ok(());
        }

        if let Some(sleep_until) = running_component.earliest_deadline() {
            running_component.advance_clock(sleep_until);
        } else {
            running_component.increment_clock();
        }
    }
}
