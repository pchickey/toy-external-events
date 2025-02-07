use crate::runtime::Executor;
use alloc::boxed::Box;
use async_task::Task;
use core::future::{poll_fn, Future};
use core::pin::Pin;
use core::task::{Context, Poll};
use wasmtime_wasi_io::async_trait;
use wasmtime_wasi_io::poll::Pollable;

/// A Job is an async-task Task that integrates with the Pollable trait to
/// check for completion, and gives access to the Task's return value as a
/// mailbox.
///
/// When a Job is complete, the value yielded may be retrieved exactly once
/// using the `Job::mailbox` method.
///
/// When a Job is dropped, the spawned Task is canceled.
///
/// This mechanism can be used to implement the pseudo-futures frequently
/// exposed in WASI 0.2 interfaces.
pub struct Job<T> {
    task: Pin<Box<Task<T>>>,
    received: Option<T>,
    gone: bool,
}

// Safety: this crate is only used in a single-threaded context.
// Not auto traits because Job holds an Rc inside Executor
unsafe impl<T> Send for Job<T> {}
unsafe impl<T> Sync for Job<T> {}

/// This value indicates the state of a `Job`. It is returned
/// by `Job::mailbox`.
pub enum Mailbox<T> {
    /// The Job is still pending. The Job's Pollable is not yet ready.
    Pending,
    /// The Job has completed, yielding a value of `T`. The Job's Pollable is ready.
    Done(T),
    /// The Job has completed, and a prior call of `Job::mailbox` has
    /// retrieved the value. The Job's Pollable is ready.
    Gone,
}

impl<T> Job<T>
where
    T: Send + 'static,
{
    pub fn spawn(executor: &Executor, f: impl Future<Output = T> + Send + 'static) -> Self {
        let task = Box::pin(executor.spawn(f));
        Self {
            task,
            received: None,
            gone: false,
        }
    }
    fn poll(&mut self, cx: &mut Context) -> Poll<()> {
        if self.gone {
            return Poll::Ready(());
        }
        if self.received.is_some() {
            return Poll::Ready(());
        }
        match self.task.as_mut().poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(res) => {
                self.received = Some(res);
                Poll::Ready(())
            }
        }
    }
    pub fn mailbox(&mut self) -> Mailbox<T> {
        if self.gone {
            Mailbox::Gone
        } else if let Some(mail) = self.received.take() {
            Mailbox::Done(mail)
        } else {
            // Poll checks for task completion. Doing so in this way will replace any existing waker
            // with a noop waker. This is ok because it will get a "real" waker when it is polled via a
            // wasi Pollable if there is actually progress to be made in wasi:io/poll waiting on it.
            // This operation should be very fast - in this crate's single threaded context, there are
            // some uncontended atomic swaps in there, but otherwise its just checking state and
            // returning the task's result if it is complete.
            match self
                .task
                .as_mut()
                .poll(&mut Context::from_waker(&crate::noop_waker::noop_waker()))
            {
                Poll::Pending => Mailbox::Pending,
                Poll::Ready(res) => {
                    self.gone = true;
                    Mailbox::Done(res)
                }
            }
        }
    }
}

#[async_trait]
impl<T> Pollable for Job<T>
where
    T: Send + 'static,
{
    async fn ready(&mut self) {
        poll_fn(|cx| self.poll(cx)).await
    }
}
