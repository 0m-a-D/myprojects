pub mod executor;
pub mod keyboard;
pub mod simple_executor;
use alloc::boxed::Box;

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
#[allow(unused)]
pub struct Task {
    id: TaskId,
    future: Pin<Box<dyn Future<Output = ()>>>,
}
// this Task type is a wrapper around a Pinned, Heap-allocated, dynamically dispatched future with
// empty type as output.

// The future corresponding to a Task has an empty type '()' because tasks are executed only for
// their side effects. Tasks don't return anything.

// "dyn" keyword tells that we are storing a trait object in Box. And these methods on the future
// are dynamically dispatched which is necessary because each "async fn" has it's own type and we
// want to create multiple different tasks.

#[allow(unused)]
impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
        }
        // 'static is required here because future needs to be valid as long as the Task.
    }

    fn poll(&mut self, cx: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(cx)
    }
    // "as_mut" was used because "poll" expects to be called on Pin<&mut T>
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);
// this is required because in order to create an "executor" with proper support for notifications,
// we need a way to specify which task should be woken.
use core::sync::atomic::{AtomicU64, Ordering};
impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        // static is used so each task is assigned ID only once.
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}
