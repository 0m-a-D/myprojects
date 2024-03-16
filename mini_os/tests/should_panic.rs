#![no_main]
#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use mini_os::{exit_qemu, hlt_loop, serial_print, serial_println, QemuExitcode};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitcode::Success);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_fail();
    serial_println!("[test did not panic!]");
    exit_qemu(QemuExitcode::Failure);

    hlt_loop()
}

fn should_fail() {
    serial_print!("should_panic::should_fail...\t");
    // manually entered the test function name because we are not implementing the Testabale trait!
    assert_eq!(0, 1);
}
