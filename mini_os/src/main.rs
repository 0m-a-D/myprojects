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
    loop {}
}
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    mini_os::test_panic_handler(info);
}
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // vga_buffer::write_something();
    // use core::fmt::Write;
    // vga_buffer::WRITER.lock().write_str("Hello world").unwrap();
    // write!(vga_buffer::WRITER.lock(), "some numbers: {} {}", 10, 3.14).unwrap();
    println!("hello world!");
    mini_os::init();

    //triggering a page fault
    // unsafe {
    //     *(0xdeadbeef as *mut u8) = 42;
    // }

    // triggering a stack overflow
    #[allow(unconditional_recursion)]
    fn stack_overflow() {
        stack_overflow();
    }
    stack_overflow();

    //invoke a breakpoint exception...
    x86_64::instructions::interrupts::int3();

    #[cfg(test)] // using "cfg(test)" for conditional compiling...
    test_main(); // name of the test framework entry function

    println!("It did not crash!");

    loop {}
}
