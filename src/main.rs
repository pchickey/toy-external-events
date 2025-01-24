use anyhow::{anyhow, Result};
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};

use embedding::{block_on, Clock, Ctx};

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
    embeddable::add_to_linker_async(&mut linker)?;
    let instance_pre = linker.instantiate_pre(&component)?;
    let proxy_pre = embeddable::BindingsPre::new(instance_pre)?;

    let clock = Clock::new();
    let ctx = block_on(clock.clone(), async move {
        let mut store = Store::new(&engine, Ctx::new(clock));
        let instance = proxy_pre.instantiate_async(&mut store).await?;
        /*
            instance
                .wasi_http_incoming_handler()
                .call_handle(&mut store, todo!(), todo!())
                .await?;
        */
        instance
            .wasi_cli_run()
            .call_run(&mut store)
            .await?
            .map_err(|()| anyhow!("run returned error"))?;
        Ok(store.into_data())
    })?;

    ctx.report();

    Ok(())
}
