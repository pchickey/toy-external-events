mod cli;
mod clocks;
mod filesystem;
mod http;
mod random;

use crate::ctx::EmbeddingCtx;
use anyhow::Result;
use wasmtime::component::Linker;
use wasmtime::Store;

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
        "wasi:http/types/incoming-body": http::IncomingBodyResource,
        "wasi:http/types/outgoing-body": http::OutgoingBodyResource,
        "wasi:http/types/fields": http::FieldsResource,
        "wasi:http/types/future-trailers": http::FutureTrailers,
        "wasi:http/types/response-outparam": http::ResponseOutparamResource,
        "wasi:http/types/request-options": http::RequestOptionsResource,
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
    wasi::http::outgoing_handler::add_to_linker_get_host(linker, closure)?;
    Ok(())
}

impl Bindings {
    pub async fn wasi_http_incoming_handler_handle(
        &self,
        store: &mut Store<EmbeddingCtx>,
        incoming: crate::http::IncomingRequest,
        headers: crate::http::Fields,
        body: crate::http::IncomingBody,
        outgoing: crate::http::ResponseOutparam,
    ) -> Result<()> {
        use wasmtime_wasi_io::IoView;
        let incoming = http::IncomingRequestResource::new(incoming, headers, body);
        let outgoing = http::ResponseOutparamResource::new(outgoing);
        let incoming = store.data_mut().table().push(incoming)?;
        let outgoing = store.data_mut().table().push(outgoing)?;
        self.wasi_http_incoming_handler()
            .call_handle(store, incoming, outgoing)
            .await
    }
}
