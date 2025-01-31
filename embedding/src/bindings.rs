mod cli;
mod clocks;
mod filesystem;
mod http;
mod random;

use crate::ctx::EmbeddingCtx;
use anyhow::Result;
use wasmtime::component::Linker;

wasmtime::component::bindgen!({
    world: "toy:embedding/bindings",
    async: { only_imports: [] },
    trappable_imports: true,
    with: {
        "wasi:io": wasmtime_wasi_io::bindings::wasi::io,
        "wasi:http/types/incoming-request": http::IncomingRequestResource,
        "wasi:http/types/outgoing-response": http::OutgoingResponseResource,
        "wasi:http/types/outgoing-request": http::OutgoingRequestResource,
        "wasi:http/types/incoming-response": http::IncomingResponseResource,
        "wasi:http/types/future-incoming-response": http::FutureIncomingResponse,
        "wasi:http/types/incoming-body": http::DynIncomingBody,
        "wasi:http/types/outgoing-body": http::DynOutgoingBody,
        "wasi:http/types/fields": http::FieldsResource,
        "wasi:http/types/future-trailers": http::FutureTrailers,
        "wasi:http/types/response-outparam": http::DynResponseOutparam,
        "wasi:http/types/request-options": http::DynRequestOptions,
    }
});

pub fn add_to_linker_async(linker: &mut Linker<EmbeddingCtx>) -> Result<()> {
    fn type_annotate<F>(val: F) -> F
    where
        F: Fn(&mut EmbeddingCtx) -> &mut EmbeddingCtx,
    {
        val
    }

    let closure = type_annotate(|t| t);
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
    // FIXME: need wasi::http::outgoing_handler in here as well.
    Ok(())
}
