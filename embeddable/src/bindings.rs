mod cli;
mod clocks;
mod filesystem;
mod http;
mod random;

use crate::{EImpl, Embedding};
use anyhow::Result;
use wasmtime::component::Linker;

wasmtime::component::bindgen!({
    world: "toy:embedding/bindings",
    async: { only_imports: [] },
    trappable_imports: true,
    with: {
        "wasi:io": wasmtime_wasi_io::bindings::wasi::io,
    }
});

pub fn add_to_linker_async<T: Embedding>(linker: &mut Linker<T>) -> Result<()> {
    let closure = type_annotate::<T, _>(|t| EImpl(wasmtime_wasi_io::IoImpl(t)));
    wasi::clocks::monotonic_clock::add_to_linker_get_host(linker, closure)?;
    wasi::cli::environment::add_to_linker_get_host(linker, closure)?;
    wasi::cli::exit::add_to_linker_get_host(linker, closure)?;
    wasi::cli::stdin::add_to_linker_get_host(linker, closure)?;
    wasi::cli::stdout::add_to_linker_get_host(linker, closure)?;
    wasi::cli::stderr::add_to_linker_get_host(linker, closure)?;
    wasi::filesystem::preopens::add_to_linker_get_host(linker, closure)?;
    wasi::filesystem::types::add_to_linker_get_host(linker, closure)?;
    wasi::random::random::add_to_linker_get_host(linker, closure)?;
    wasi::http::types::add_to_linker_get_host(linker, closure)?;
    Ok(())
}
fn type_annotate<T: Embedding, F>(val: F) -> F
where
    F: Fn(&mut T) -> EImpl<&mut T>,
{
    val
}
