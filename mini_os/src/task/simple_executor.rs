use super::Task;
use alloc::collections::VecDeque;
use core::task::{Context, Poll};

pub struct SimpleExecutor {
    task_queue: VecDeque<Task>,
}
impl SimpleExecutor {
    pub fn new() -> Self {
        SimpleExecutor {
            task_queue: VecDeque::new(),
        }
    }
    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task);
    }
    pub fn run(&mut self) {
        while let Some(mut task) = self.task_queue.pop_front() {
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {} // task done
                Poll::Pending => self.task_queue.push_back(task),
            }
        }
    }
    // the above "run" method is not very efficient because we do not utilise the notifications of
    // the "Waker" type...
}
impl Default for SimpleExecutor {
    fn default() -> Self {
        Self::new()
    }
}

use core::task::{RawWaker, RawWakerVTable, Waker};

fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(core::ptr::null::<()>(), vtable)
}
fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}
