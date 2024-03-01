#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(mini_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use mini_os::println;
// static HELLO: &[u8] = b"Hello World!";

/// This function is called on panic!!
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    mini_os::hlt_loop();
}
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    mini_os::test_panic_handler(info);
}

entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // use mini_os::memory::active_level_4_table;
    use mini_os::memory;
    use x86_64::structures::paging::Page;
    use x86_64::VirtAddr;

    println!("hello world!");
    mini_os::init();

    // x86_64::instructions::interrupts::int3();
    // let phy_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // let l4_table = unsafe { active_level_4_table(phy_memory_offset) };
    //
    // for (i, entry) in l4_table.iter().enumerate() {
    //     use x86_64::structures::paging::PageTable;
    //
    //     if !entry.is_unused() {
    //         println!("L4 ENTRY {}: {:?}", i, entry);
    //
    //         // get physical address from entry and convert it
    //         let phys = entry.frame().unwrap().start_address();
    //         let virt = phys.as_u64() + boot_info.physical_memory_offset;
    //         let ptr = VirtAddr::new(virt).as_mut_ptr();
    //         let l3_table: &PageTable = unsafe { &*ptr };
    //
    //         // print non-empty entries from page table 3
    //         for (i, entry) in l3_table.iter().enumerate() {
    //             println!("L3 ENTRY {}: {:?}", i, entry);
    //         }
    //     }
    // }
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // new: initialise a mapper
    let mut mapper = unsafe { memory::init(phys_mem_offset) };

    use mini_os::memory::BootInfoFrameAllocator;
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    // let addresses = [
    //     // the identity-mapped vga buffer page
    //     0xb8000,
    //     // some code page
    //     0x201008,
    //     // some stack page
    //     0x0100_0020_1a10,
    //     // virtual address mapped to physical address 0
    //     boot_info.physical_memory_offset,
    // ];
    // for &address in &addresses {
    //     let virt = VirtAddr::new(address);
    //     let phys = mapper.translate_addr(virt);
    //     println!("{:?} -> {:?}", virt, phys);
    //     // notice how last 12 bits (offset) are same for virt and phys addresses
    // }

    // map an unused page
    let page = Page::containing_address(VirtAddr::new(0)); // try with 0xdeadbeef
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // write to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    #[cfg(test)] // using "cfg(test)" for conditional compiling...
    test_main(); // name of the test framework entry function

    println!("It did not crash!"); // try running this statement in a for loop from 0 -> 100
    mini_os::hlt_loop();
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
