pub mod bump;
pub mod fixed_size_block;
pub mod linked_list;
// use bump::BumpAllocator;
use fixed_size_block::FixedSizeBlockAllocator;
// use linked_list::LinkedListAllocator;

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
// use linked_list_allocator::LockedHeap;

pub const HEAP_START: /*usize*/ *mut u8 = 0x_4444_4444_0000 as *mut u8; // any address can be chosen untill the address
pub const HEAP_SIZE: usize = 100 * 1024; // 100KiB...will increase in future when needed.

pub struct Dummy; // zero-sized type [not of any use: just to learn how GlobalAlloc trait works]
unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should never be called!");
    }
}

#[global_allocator]
// this attribute tells Rust Compiler what allocator instance to be used for global heap
// allocation (this attribute is only applicable to STATICS)...
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(FixedSizeBlockAllocator::new());

use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags as PTF, Size4KiB,
    },
    VirtAddr,
};

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        // -1 because we need last byte address of heap (inclusive)...
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PTF::PRESENT | PTF::WRITABLE;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
        // flush is necessary to get rid of old translations from TLB...
    }
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }
    Ok(())
}

// own wrapper type around spin::Mutex to get mutable references to allocators
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}
impl<A> Locked<A> {
    /// Creates a new [`Locked<A>`].
    // declaring new as "const" because initialization expression of a static must be evaluable at compile time.
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }
    /// Returns the lock of this [`Locked<A>`].
    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}
/*adding an align_up method to align address to alignment 'align' [not an efficient function]
fn align_up(addr: usize, align: usize) -> usize {
    let remainder = addr % align;
    if remainder == 0 {
        addr // address already aligned
    } else {
        addr - remainder + align
    }

}*/
// TODO: understand how the operators have been used here
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
