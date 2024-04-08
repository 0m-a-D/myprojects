use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
// ArrayQueue::new() could not be used as it does heap allocation which is not possible yet
// see [https://github.com/rust-lang/const-eval/issues/20] for more details
// Instead we use OnceCell which performs one-time initialization of static values.

use crate::println;

/// called by keyboard interrupt handler. must not block or allocate
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if queue.push(scancode).is_err() {
            println!("WARNING: scancode queue full. Dropping keyboard input.");
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue uninitialised.");
    }
} // pub(crate) makes this function only "visible" to our "lib.rs" as this function should not be
  // callable from "main.rs" (no heap allocation possible else will cause "deadlock")

pub struct ScancodeStream {
    _private: (),
    // purpose of this field is to prevent construction of this struct from outside the module
}
impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ArrayQueue::new() should only be called once");
        ScancodeStream { _private: () }
    }
}
impl Default for ScancodeStream {
    fn default() -> Self {
        Self::new()
    }
}

// implementing "Stream" trait for our ScancodeStream to provide values of SCANCODE_QUEUE in an
// asynchronous way.
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use futures_util::stream::Stream;

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let queue = SCANCODE_QUEUE.try_get().expect("not initialised!");

        // fast path. just in case if scancode is ready, we can avoid the performance overhead of
        // registering a waker when the queue is not empty.
        if let Ok(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(cx.waker());
        match queue.pop() {
            Ok(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}

// WAKER SUPPORT
use futures_util::task::AtomicWaker;
static WAKER: AtomicWaker = AtomicWaker::new();
// since this type is based on atomic instructions, it can be safely stored and modified
// concurrently

// KEYBOARD TASK
use crate::print;
use futures_util::stream::StreamExt;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

pub async fn key_presses() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => print!("{}", character),
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }
    }
}
