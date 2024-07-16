// using a linked list to keep track of freed memory regions in the for of nodes. Such allocators
// are also called POOL ALLOCATORS.
use super::align_up;
use core::mem;

struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>,
}
// "&'static mut" semantically means "owned object behind a pointer". [BASICALLY KIND OF BOX
// WITHOUT DESTRUCTOR WHICH FREES MEMORY ONCE OBJECT GOES OUT OF SCOPE.]
impl ListNode {
    const fn new(size: usize) -> Self {
        ListNode { size, next: None }
    }
    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }
    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

pub struct LinkedListAllocator {
    head: ListNode,
}
impl LinkedListAllocator {
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0),
        }
    }

    /// # Safety
    /// this function is unsafe because caller should give valid heap_start address and heap size.
    /// Also this function should only be called once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_regions(heap_start, heap_size);
    }

    /// # Safety
    /// adds given memory address in front of the list. Should only be called once.
    unsafe fn add_free_regions(&mut self, addr: usize, size: usize) {
        // check to see if the freed region can hold ListNode
        assert_eq!(addr, align_up(addr, mem::align_of::<ListNode>()));
        assert!(size >= mem::size_of::<ListNode>());

        // creating a new list node and appending to start of list
        let mut node = ListNode::new(size);
        node.next = self.head.next.take();
        let node_ptr = addr as *mut ListNode;
        node_ptr.write(node);
        self.head.next = Some(unsafe { &mut *node_ptr });
    }

    // this is the central operation of this allocator. To find the region with it's entry and remove
    // from list [returns a tuple]
    fn find_regions(
        &mut self,
        size: usize,
        align: usize,
    ) -> Option<(&'static mut ListNode, usize)> {
        // reference to current list node
        let mut current = &mut self.head;
        // look for large enough memory in linked list
        while let Some(ref mut region) = current.next {
            if let Ok(start_alloc) = Self::alloc_from_region(region, size, align) {
                // region suitable for allocation -> remove from list
                let next = region.next.take();
                let ret = Some((current.next.take().unwrap(), start_alloc));
                current.next = next;
                return ret;
            }
            // region not suitable -> continue with next region
            current = current.next.as_mut().unwrap();
        }
        // no suitable region found
        None
    }

    // this function tries to use given regions for allocation with given size and align
    fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> Result<usize, ()> {
        let start_alloc = align_up(region.start_addr(), align);
        let end_alloc = start_alloc.checked_add(size).ok_or(())?;

        if end_alloc > region.end_addr() {
            return Err(());
        }
        let excess_size = region.end_addr() - end_alloc;
        if excess_size > 0 && excess_size < mem::size_of::<ListNode>() {
            // rest of the region is too small to hold a ListNode. needed because allocation splits
            // the region into used and free part
            return Err(());
        }
        // returns region suitable for allocation
        Ok(start_alloc)
    }

    // adjusts the layout so that resulting allocated memory is also capable of storing a 'ListNode'
    // -> returns adjusted size and alignment
    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(mem::align_of::<ListNode>())
            .expect("adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(mem::size_of::<ListNode>());
        (size, layout.align())
    }
}
impl Default for LinkedListAllocator {
    fn default() -> Self {
        Self::new()
    }
}

// final step: implementing GlobAlloc trait on Locked<LinkedListAllocator>...reason: allows to
// mutate linked list even though alloc() and dealloc() takes "&self" using spinlock's interior
// mutability
use super::Locked;
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;
unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // perform layout adjustments
        let (size, align) = LinkedListAllocator::size_align(layout);
        let mut allocator = self.lock();

        if let Some((region, start_alloc)) = allocator.find_regions(size, align) {
            let end_alloc = start_alloc.checked_add(size).expect("overflow");
            let excess_size = region.end_addr() - end_alloc;

            if excess_size > 0 {
                allocator.add_free_regions(end_alloc, excess_size);
            }
            start_alloc as *mut u8
        } else {
            ptr::null_mut()
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // perform size adjustments
        let (size, _) = LinkedListAllocator::size_align(layout);
        self.lock().add_free_regions(ptr as usize, size);
    }
}
