#![no_std]
extern crate alloc;

mod bindings;
mod clock;
mod ctx;
pub mod http;
pub mod job;
mod noop_waker;
mod runtime;
mod streams;

use clock::Clock;
use ctx::EmbeddingCtx;
use runtime::Executor;

use alloc::boxed::Box;
use alloc::string::String;
use anyhow::{Context as _, Result};
use async_task::Task;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};

pub struct Runtime {
    engine: Engine,
    linker: Linker<EmbeddingCtx>,
}

impl Runtime {
    pub fn new() -> Result<Self> {
        let mut config = Config::new();
        config.async_support(true);
        let engine = Engine::new(&config)?;
        let mut linker = Linker::new(&engine);
        wasmtime_wasi_io::add_to_linker_async(&mut linker)?;
        bindings::add_to_linker_async(&mut linker)?;
        Ok(Runtime { engine, linker })
    }

    pub fn load(&self, cwasm: &[u8]) -> Result<RunnableComponent> {
        let component = unsafe { Component::deserialize(&self.engine, cwasm)? };
        let instance_pre = self.linker.instantiate_pre(&component)?;
        let bindings_pre = bindings::BindingsPre::new(instance_pre)?;
        Ok(RunnableComponent {
            engine: self.engine.clone(),
            bindings_pre,
        })
    }
}

pub struct RunnableComponent {
    engine: Engine,
    bindings_pre: bindings::BindingsPre<EmbeddingCtx>,
}

impl RunnableComponent {
    pub fn create(
        &self,
        incoming: crate::http::IncomingRequest,
        headers: crate::http::Fields,
        body: crate::http::IncomingBody,
    ) -> Result<RunningComponent> {
        let executor = Executor::new();
        let clock = Clock::new();
        let mut store = Store::new(
            &self.engine,
            EmbeddingCtx::new(executor.clone(), clock.clone()),
        );
        let mailbox = crate::http::ResponseOutparam::new();
        let bindings_pre = self.bindings_pre.clone();
        let fut = async move {
            let instance = match bindings_pre.instantiate_async(&mut store).await {
                Ok(i) => i,
                Err(e) => return (store.into_data(), Err(e).context("instantiating")),
            };
            match instance
                .wasi_http_incoming_handler_handle(
                    &mut store,
                    incoming,
                    headers,
                    body,
                    mailbox.clone(),
                )
                .await
            {
                Ok(()) => {}
                Err(e) => return (store.into_data(), Err(e).context("running handler")),
            };

            (store.into_data(), mailbox.into_inner())
        };
        let task = executor.spawn(fut);

        Ok(RunningComponent {
            clock,
            executor,
            output: Box::pin(task),
        })
    }
}

pub struct RunningComponent {
    clock: Clock,
    executor: Executor,
    output: Pin<
        Box<
            Task<(
                EmbeddingCtx,
                Result<(crate::http::OutgoingResponse, crate::http::ImmutFields)>,
            )>,
        >,
    >,
}

impl RunningComponent {
    pub fn earliest_deadline(&self) -> Option<u64> {
        self.executor.earliest_deadline()
    }

    pub fn increment_clock(&self) {
        self.clock.set(self.clock.get() + 1);
        self.check_for_wake();
    }

    pub fn advance_clock(&self, to: u64) {
        self.clock.set(to);
        self.check_for_wake();
    }

    fn check_for_wake(&self) {
        for waker in self.executor.ready_deadlines(self.clock.get()) {
            waker.wake()
        }
    }

    pub fn step(&mut self) -> usize {
        self.executor.step()
    }

    pub fn check_complete(
        &mut self,
    ) -> Option<(
        String,
        Result<(crate::http::OutgoingResponse, crate::http::ImmutFields)>,
    )> {
        match self
            .output
            .as_mut()
            .poll(&mut Context::from_waker(&noop_waker::noop_waker()))
        {
            Poll::Pending => None,
            Poll::Ready((ctx, res)) => {
                let report = ctx.report();
                Some((report, res))
            }
        }
    }
}
