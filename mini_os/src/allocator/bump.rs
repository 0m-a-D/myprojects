pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    heap_next: usize,
    allocations: usize,
}
impl BumpAllocator {
    // declaring new as "const" because initialization expression of a static must be evaluable at compile time.
    pub const fn new() -> BumpAllocator {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            heap_next: 0,
            allocations: 0,
        }
    }
    /// # Safety
    /// caller should make sure address to heap start *heap_start* should be valid...
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.heap_next = heap_start;
    }
}
// implementing Default trait for BumpAllocator [no point of this...cargo clippy forced me to :(]
impl Default for BumpAllocator {
    fn default() -> Self {
        Self::new()
    }
}

// every allocator must implement the GlobalAlloc trait
use super::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.lock(); // get a mutable reference

        let start_alloc = align_up(bump.heap_next, layout.align());
        let end_alloc = match start_alloc.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };
        if end_alloc > bump.heap_end {
            ptr::null_mut() // end of memory
        } else {
            bump.heap_next = end_alloc;
            bump.allocations += 1;
            start_alloc as *mut u8
        }
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut bump = self.lock(); // get a mutable reference
        bump.allocations -= 1;
        if bump.allocations == 0 {
            bump.heap_next = bump.heap_start;
        }
    }
}
