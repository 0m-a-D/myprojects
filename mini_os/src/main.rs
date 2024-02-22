#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(mini_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

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
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("hello world!");
    mini_os::init();

    // x86_64::instructions::interrupts::int3();

    // creating a pagefault //
    let ptr = 0x20555a as *mut u8;
    unsafe {
        let _x = *ptr;
    }
    println!("READ WORKED");
    // 0x20555a is the address that points to a code page. Code pages are mapped read-only by
    // bootloader... so we can read the data that this address points to, but can't write to it...

    unsafe {
        *ptr = 42;
    }
    println!("WRITE WORKED");

    // accessing page tables
    use x86_64::registers::control::Cr3;
    let (level_4_page_table, _) = Cr3::read();
    println!(
        "LEVEL 4 PAGE TABLE AT: {:?}",
        level_4_page_table.start_address()
    ); // comment line 39 -> 42 to see the address of level 4 page table.

    #[cfg(test)] // using "cfg(test)" for conditional compiling...
    test_main(); // name of the test framework entry function

    println!("It did not crash!");
    mini_os::hlt_loop();
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
