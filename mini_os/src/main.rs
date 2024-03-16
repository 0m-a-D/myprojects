#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(mini_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc; // needed here too because main.rs and lib.rs are treated as separate crates
use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
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
    use mini_os::allocator;
    use mini_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    println!("hello world!");
    mini_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // new: initialise a mapper
    let mut mapper = unsafe { memory::init(phys_mem_offset) };

    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // HEAP IMPLEMENTATION CHECK
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialisation failed!");
    let a = Box::new(41);
    println!("value at {:p} is {}", a, a);

    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    // create reference counted vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!(
        "current reference count is {}",
        Rc::strong_count(&cloned_reference)
    );
    core::mem::drop(reference_counted);
    println!(
        "reference count is now {}",
        Rc::strong_count(&cloned_reference)
    );

    #[cfg(test)] // using "cfg(test)" for conditional compiling...
    test_main(); // name of the test framework entry function

    println!("It did not crash!"); // try running this statement in a for loop from 0 -> 100
    mini_os::hlt_loop();
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
