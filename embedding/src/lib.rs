use anyhow::Result;
use bytes::Bytes;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use wasmtime_wasi_io::poll::Pollable;
use wasmtime_wasi_io::streams::{InputStream, OutputStream};

use wasmtime::component::ResourceTable;

pub struct Ctx {
    table: ResourceTable,
    clock: Clock,
    stdout: TimestampedWrites,
    stderr: TimestampedWrites,
}
impl Ctx {
    pub fn new(clock: Clock) -> Self {
        let stdout = TimestampedWrites::new(clock.clone());
        let stderr = TimestampedWrites::new(clock.clone());

        Ctx {
            table: ResourceTable::new(),
            clock,
            stdout,
            stderr,
        }
    }

    pub fn report(&self) {
        println!("stdout:");
        self.stdout.report();
        println!("stderr:");
        self.stderr.report();
    }
}
impl wasmtime_wasi_io::IoView for Ctx {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl embeddable::Embedding for Ctx {
    fn monotonic_now(&self) -> u64 {
        let now = self.clock.get();
        println!("wasm told now is: {now}");
        now
    }
    fn monotonic_timer(&self, deadline: u64) -> impl Pollable {
        Deadline::new(self.clock.clone(), deadline)
    }
    fn stdin(&self) -> impl InputStream {
        NeverReadable
    }
    fn stdout(&self) -> impl OutputStream {
        self.stdout.clone()
    }
    fn stderr(&self) -> impl OutputStream {
        self.stderr.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Clock(Arc<AtomicU64>);
impl Clock {
    pub fn new() -> Self {
        Clock(Arc::new(AtomicU64::new(0)))
    }
    pub fn get(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }
    fn set(&self, to: u64) {
        println!("clock advancing to {to}");
        self.0.store(to, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone)]
pub struct Deadline {
    clock: Clock,
    due: u64,
}
impl Deadline {
    fn new(clock: Clock, due: u64) -> Self {
        Self { clock, due }
    }
}
impl Future for Deadline {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let now = self.clock.get();
        if now < self.due {
            Executor::current().push_deadline(self.clone(), cx.waker().clone());
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}
#[wasmtime_wasi_io::async_trait]
impl Pollable for Deadline {
    async fn ready(&mut self) {
        self.clone().await
    }
}

struct NeverReadable;
#[wasmtime_wasi_io::async_trait]
impl Pollable for NeverReadable {
    async fn ready(&mut self) {
        futures::future::pending().await
    }
}
impl InputStream for NeverReadable {
    fn read(&mut self, _: usize) -> wasmtime_wasi_io::streams::StreamResult<Bytes> {
        unreachable!("never ready for reading")
    }
}

#[derive(Clone)]
struct TimestampedWrites {
    clock: Clock,
    log: Arc<Mutex<VecDeque<(u64, Bytes)>>>,
}
impl TimestampedWrites {
    fn new(clock: Clock) -> Self {
        Self {
            clock,
            log: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
    fn report(&self) {
        for (time, line) in self.log.lock().unwrap().iter() {
            println!("{:08} {:?}", time, String::from_utf8_lossy(line));
        }
    }
}
#[wasmtime_wasi_io::async_trait]
impl Pollable for TimestampedWrites {
    async fn ready(&mut self) {}
}
impl OutputStream for TimestampedWrites {
    fn check_write(&mut self) -> wasmtime_wasi_io::streams::StreamResult<usize> {
        Ok(usize::MAX)
    }
    fn write(&mut self, contents: Bytes) -> wasmtime_wasi_io::streams::StreamResult<()> {
        let time = self.clock.get();
        self.log.lock().unwrap().push_back((time, contents));
        Ok(())
    }
    fn flush(&mut self) -> wasmtime_wasi_io::streams::StreamResult<()> {
        Ok(())
    }
}

static EXECUTOR: Mutex<Option<Executor>> = Mutex::new(None);

pub struct Executor(Arc<Mutex<ExecutorInner>>);

impl Executor {
    pub fn current() -> Self {
        Executor(
            EXECUTOR
                .lock()
                .unwrap()
                .as_ref()
                .expect("Executor::current must be called within a running executor")
                .0
                .clone(),
        )
    }
    pub fn push_deadline(&mut self, deadline: Deadline, waker: Waker) {
        self.0.lock().unwrap().deadlines.push((deadline, waker))
    }
}

pub fn block_on<R>(clock: Clock, f: impl Future<Output = Result<R>> + Send + 'static) -> Result<R> {
    if EXECUTOR.lock().unwrap().is_some() {
        panic!("cannot block_on while executor is running!")
    }
    let executor = Executor(Arc::new(Mutex::new(ExecutorInner {
        deadlines: Vec::new(),
    })));
    *EXECUTOR.lock().unwrap() = Some(Executor(executor.0.clone()));

    let waker = futures::task::noop_waker();
    let mut cx = Context::from_waker(&waker);

    let mut f = std::pin::pin!(f);
    let r = 'outer: loop {
        // Run some wasm:
        const POLLS_PER_CLOCK: usize = 200; // Arbitrary, tune this i guess?
        for _ in 0..POLLS_PER_CLOCK {
            match f.as_mut().poll(&mut cx) {
                Poll::Pending => {}
                Poll::Ready(r) => break 'outer r,
            }
        }

        // Wait for input from the outside world:
        if let Some(sleep_until) = executor.0.lock().unwrap().earliest_deadline() {
            clock.set(sleep_until);
        } else {
            clock.set(clock.get() + 1);
        }

        // any wakers become ready now.
        for waker in executor.0.lock().unwrap().ready_deadlines(clock.get()) {
            waker.wake()
        }
    };

    let _ = EXECUTOR
        .lock()
        .unwrap()
        .take()
        .expect("executor vacated global while running");
    r
}

struct ExecutorInner {
    deadlines: Vec<(Deadline, Waker)>,
}

impl ExecutorInner {
    fn earliest_deadline(&self) -> Option<u64> {
        self.deadlines.iter().map(|(d, _)| d.due).min()
    }
    fn ready_deadlines(&mut self, now: u64) -> Vec<Waker> {
        let mut i = 0;
        let mut wakers = Vec::new();
        // This is basically https://doc.rust-lang.org/std/vec/struct.Vec.html#method.extract_if,
        // which is unstable
        while i < self.deadlines.len() {
            if let Some((deadline, _)) = self.deadlines.get(i) {
                if deadline.due <= now {
                    let (_, waker) = self.deadlines.remove(i);
                    wakers.push(waker);
                } else {
                    i += 1;
                }
            } else {
                break;
            }
        }
        wakers
    }
}
