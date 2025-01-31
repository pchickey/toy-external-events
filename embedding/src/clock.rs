use alloc::boxed::Box;
use alloc::rc::Rc;
use core::cell::Cell;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::runtime::Executor;

use wasmtime_wasi_io::poll::Pollable;

#[derive(Debug, Clone)]
pub struct Clock(Rc<Cell<u64>>);
impl Clock {
    pub fn new() -> Self {
        Clock(Rc::new(Cell::new(0)))
    }
    pub fn get(&self) -> u64 {
        self.0.get()
    }
    pub fn set(&self, to: u64) {
        //println!("clock advancing to {to}");
        self.0.set(to)
    }
}
// SAFETY: only will consume this crate in single-threaded environment
unsafe impl Send for Clock {}
unsafe impl Sync for Clock {}

#[derive(Debug, Clone)]
pub struct Deadline {
    clock: Clock,
    due: u64,
}
impl Deadline {
    pub fn new(clock: Clock, due: u64) -> Self {
        Self { clock, due }
    }
}
impl Future for Deadline {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let now = self.clock.get();
        if now < self.due {
            Executor::current().push_deadline(self.due, cx.waker().clone());
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}
// SAFETY: only will consume this crate in single-threaded environment
unsafe impl Send for Deadline {}
unsafe impl Sync for Deadline {}

#[wasmtime_wasi_io::async_trait]
impl Pollable for Deadline {
    async fn ready(&mut self) {
        self.clone().await
    }
}
