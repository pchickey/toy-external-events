use anyhow::Result;
use wasmtime::component::{Linker, Resource};

wasmtime::component::bindgen!({
    world: "wasi:cli/command",
    async: { only_imports: [] },
    trappable_imports: true,
    with: {
        "wasi:io": wasmtime_wasi_io::bindings::wasi::io,
    }
});

pub fn add_to_linker_async<T: Embedding>(linker: &mut Linker<T>) -> Result<()> {
    let closure = type_annotate::<T, _>(|t| EImpl(wasmtime_wasi_io::IoImpl(t)));
    wasi::clocks::monotonic_clock::add_to_linker_get_host(linker, closure)?;
    Ok(())
}
fn type_annotate<T: Embedding, F>(val: F) -> F
where
    F: Fn(&mut T) -> EImpl<&mut T>,
{
    val
}

use wasmtime_wasi_io::IoView;

pub struct EImpl<T>(wasmtime_wasi_io::IoImpl<T>);
impl<T: IoView> IoView for EImpl<T> {
    fn table(&mut self) -> &mut wasmtime::component::ResourceTable {
        T::table(&mut self.0 .0)
    }
}
pub trait Embedding: wasmtime_wasi_io::IoView {
    fn monotonic_now(&self) -> u64;
}

impl<T: Embedding> Embedding for EImpl<T> {
    fn monotonic_now(&self) -> u64 {
        T::monotonic_now(&self.0 .0)
    }
}

impl<T: ?Sized + Embedding> Embedding for &mut T {
    fn monotonic_now(&self) -> u64 {
        T::monotonic_now(self)
    }
}

impl<E: Embedding> wasi::clocks::monotonic_clock::Host for EImpl<E> {
    fn now(&mut self) -> Result<wasi::clocks::monotonic_clock::Instant> {
        Ok(self.monotonic_now())
    }
    fn resolution(&mut self) -> Result<wasi::clocks::monotonic_clock::Duration> {
        Ok(1)
    }
    fn subscribe_duration(
        &mut self,
        duration: wasi::clocks::monotonic_clock::Duration,
    ) -> Result<Resource<wasmtime_wasi_io::poll::DynPollable>> {
        todo!()
    }
    fn subscribe_instant(
        &mut self,
        instant: wasi::clocks::monotonic_clock::Instant,
    ) -> Result<Resource<wasmtime_wasi_io::poll::DynPollable>> {
        let deadline = self.table().push(ClocksDeadline(instant))?;
        Ok(wasmtime_wasi_io::poll::subscribe(self.table(), deadline)?)
    }
}

#[derive(Debug, Clone, Copy)]
struct ClocksDeadline(u64);
#[wasmtime_wasi_io::async_trait]
impl wasmtime_wasi_io::poll::Pollable for ClocksDeadline {
    async fn ready(&mut self) {
        todo!()
    }
}
