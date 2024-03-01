use x86_64::{
    structures::paging::{OffsetPageTable, PageTable},
    VirtAddr,
};

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3; // for getting access to physical address of L4 page table

    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64(); // convert phys to u64 data type

    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe (can be called only once[cannot have multiple mut references])
}

// use x86_64::Pphysical_memory_offsethysAddr;

// pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
//     translate_addr_inner(addr, physical_memory_offset)
// }
// // defining a private function to limit the scope of unsafe function
// fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
//     use x86_64::registers::control::Cr3;
//     use x86_64::structures::paging::page_table::FrameError;
//
//     let (level_4_table_frame, _) = Cr3::read();
//     let table_index = [
//         addr.p4_index(),
//         addr.p3_index(),
//         addr.p2_index(),
//         addr.p1_index(),
//     ];
//
//     let mut frame = level_4_table_frame;
//
//     // traverse the multi-level
//     for &index in &table_index {
//         let virt = physical_memory_offset + frame.start_address().as_u64();
//         let table_ptr: *const PageTable = virt.as_ptr();
//         let table = unsafe { &*table_ptr };
//
//         // read the page table entry and update the frame
//         let entry = &table[index];
//         frame = match entry.frame() {
//             Ok(frame) => frame,
//             Err(FrameError::FrameNotPresent) => return None,
//             Err(FrameError::HugeFrame) => panic!("huge pages are not supported"),
//         };
//     }
//     // calculate the physical address by adding the page offset
//     Some(frame.start_address() + u64::from(addr.page_offset()))
// }

// initialize a new OffsetPageTable
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

use x86_64::{
    structures::paging::{FrameAllocator, Mapper, Page, PhysFrame, Size4KiB},
    PhysAddr,
};

// creating an example mapping for the given page to frame '0xb8000'...using this frame to test if
// frame allocator works when we write something to this frame and see output on qemu screen
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        // FIXME: this is not safe: we only do it for testing
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}

// a frame allocator that always returns None
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        None
    }
}

use bootloader::bootinfo::MemoryMap;
// a frame allocator that returns usable frames from the bootloader's memory map
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}
use bootloader::bootinfo::MemoryRegionType;
impl BootInfoFrameAllocator {
    // creates a FrameAllocator from the passed memory map
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }
    // returns an iterator over the usable frames specified in a memory map
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // get usable regions from memory map
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);

        // map each region to its address range
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());

        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // create PhysFrame types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}
unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
