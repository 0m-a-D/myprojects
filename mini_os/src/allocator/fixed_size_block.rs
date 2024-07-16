use core::{alloc::Layout, ptr};

struct ListNode {
    next: Option<&'static mut ListNode>,
}
// notice that unlike linked list allocator, we don't need to specify "SIZE" field because all
// sizes of the nodes(free regions) are same of that particular list.

const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];
// notice we don't add 4. Because each block must be capable to store a 64-bit POINTER to the next
// block when freed.

pub struct FixedSizeBlockAllocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap,
}
impl FixedSizeBlockAllocator {
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;
        Self {
            list_heads: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
        // we used EMPTY to tell rust compiler that we want to initialise list_heads with a
        // constant value. Using [None, BLOCK_SIZES.len()] won't work because then compiler would
        // want Option<&'static mut ListNode> to implement "Copy" trait which it doesn't.
        // IT'S A LIMITATION OF RUST COMPILER WHICH MIGHT BE RESOLVED IN FUTURE.
    }
    /// # Safety
    /// this function is unsafe because caller should make sure that starting heap address and heap
    /// size is valid. Also this function should only be called once
    pub unsafe fn init(&mut self, heap_start: /*usize*/ *mut u8, heap_size: usize) {
        self.fallback_allocator.init(heap_start, heap_size);
    }

    // creating a helper function that allocates using fallback allocator
    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }

    // this helper function returns appropriate index from BLOCK_SIZES array according to requested
    // allocation
}
impl Default for FixedSizeBlockAllocator {
    fn default() -> Self {
        Self::new()
    }
}

// this helper function returns appropriate index from BLOCK_SIZES array according to requested
// allocation
fn list_index(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}

use super::Locked;
use alloc::alloc::GlobalAlloc;
use core::{mem, ptr::NonNull};
unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();
        match list_index(&layout) {
            Some(index) => {
                match allocator.list_heads[index].take() {
                    Some(node) => {
                        allocator.list_heads[index] = node.next.take();
                        node as *mut ListNode as *mut u8
                    }
                    None => {
                        // no block exists in list => allocate new block
                        let block_size = BLOCK_SIZES[index];
                        // only works if all block sizes are powers of 2
                        let block_align = block_size;
                        let layout = Layout::from_size_align(block_size, block_align).unwrap();
                        allocator.fallback_alloc(layout)
                    }
                }
            }
            None => allocator.fallback_alloc(layout),
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();
        match list_index(&layout) {
            Some(index) => {
                let new_node = ListNode {
                    next: allocator.list_heads[index].take(),
                };
                // verify that block has size and align for storing the node
                assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
                assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);
                let new_node_ptr = ptr as *mut ListNode;
                new_node_ptr.write(new_node);
                allocator.list_heads[index] = Some(&mut *new_node_ptr);
            }
            None => {
                let ptr = NonNull::new(ptr).unwrap();
                allocator.fallback_allocator.deallocate(ptr, layout);
            }
        }
    }
}
