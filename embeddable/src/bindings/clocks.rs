use crate::{EImpl, Embedding};
use anyhow::Result;
use wasmtime::component::Resource;
use wasmtime_wasi_io::{
    poll::{subscribe, DynPollable},
    IoView,
};

use super::wasi::clocks::monotonic_clock;

impl<E: Embedding> monotonic_clock::Host for EImpl<E> {
    fn now(&mut self) -> Result<monotonic_clock::Instant> {
        Ok(self.monotonic_now())
    }
    fn resolution(&mut self) -> Result<monotonic_clock::Duration> {
        Ok(1)
    }
    fn subscribe_duration(
        &mut self,
        duration: monotonic_clock::Duration,
    ) -> Result<Resource<DynPollable>> {
        self.subscribe_instant(self.monotonic_now() + duration)
    }
    fn subscribe_instant(
        &mut self,
        deadline: monotonic_clock::Instant,
    ) -> Result<Resource<DynPollable>> {
        let timer = self.monotonic_timer(deadline);
        let deadline = self.table().push(timer)?;
        Ok(subscribe(self.table(), deadline)?)
    }
}
