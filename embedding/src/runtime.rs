use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::task::Waker;

struct ExecutorGlobal(RefCell<Option<Executor>>);
impl ExecutorGlobal {
    const fn new() -> Self {
        ExecutorGlobal(RefCell::new(None))
    }
}
// SAFETY: only will consume this crate in single-threaded environment
unsafe impl Send for ExecutorGlobal {}
unsafe impl Sync for ExecutorGlobal {}

static EXECUTOR: ExecutorGlobal = ExecutorGlobal::new();

pub struct Executor(Rc<RefCell<ExecutorInner>>);

impl Executor {
    pub fn new() -> Self {
        Executor(Rc::new(RefCell::new(ExecutorInner {
            deadlines: Vec::new(),
        })))
    }
    pub fn current() -> Self {
        Executor(
            EXECUTOR
                .0
                .borrow_mut()
                .as_ref()
                .expect("Executor::current must be called within a running executor")
                .0
                .clone(),
        )
    }

    pub(crate) fn with<R>(&self, f: impl FnOnce() -> R) -> R {
        if EXECUTOR.0.borrow_mut().is_some() {
            panic!("cannot block_on while executor is running!")
        }
        *EXECUTOR.0.borrow_mut() = Some(Executor(self.0.clone()));
        let r = f();
        let _ = EXECUTOR
            .0
            .borrow_mut()
            .take()
            .expect("executor vacated global while running");
        r
    }

    pub fn push_deadline(&self, deadline: u64, waker: Waker) {
        self.0.borrow_mut().deadlines.push((deadline, waker))
    }
    pub fn earliest_deadline(&self) -> Option<u64> {
        self.0.borrow().earliest_deadline()
    }
    pub fn ready_deadlines(&self, now: u64) -> Vec<Waker> {
        self.0.borrow_mut().ready_deadlines(now)
    }
}

struct ExecutorInner {
    deadlines: Vec<(u64, Waker)>,
}

impl ExecutorInner {
    fn earliest_deadline(&self) -> Option<u64> {
        self.deadlines.iter().map(|(d, _)| d).min().copied()
    }
    fn ready_deadlines(&mut self, now: u64) -> Vec<Waker> {
        let mut i = 0;
        let mut wakers = Vec::new();
        // This is basically https://doc.rust-lang.org/std/vec/struct.Vec.html#method.extract_if,
        // which is unstable
        while i < self.deadlines.len() {
            if let Some((deadline, _)) = self.deadlines.get(i) {
                if *deadline <= now {
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
