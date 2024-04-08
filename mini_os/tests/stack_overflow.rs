#![no_main]
#![no_std]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use mini_os::serial_print;
#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");

    mini_os::gdt::init();
    init_test_idt();
    // we call this function instead of interrupts::init_idt function because we want to register
    // a custom double fault that exits qemu [exit_qemu(QemuExitCode::Success)] instead of
    // panicking...

    // trigger a stack overflow
    stack_overflow();
    panic!("Execution continued after stack overflow");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    mini_os::test_panic_handler(info);
}

#[allow(unconditional_recursion)] // silence compiler warning that function recurses endlessly
fn stack_overflow() {
    stack_overflow(); // for each recursion, return address is pushed
    volatile::Volatile::new(0).read();
    // to prevent tail recursion optimizations, we add dummy volatile read statement
    // which compiler is not allowed to remove hence making this non-tail recursive.
}

// we disable test harness for integration test like this because we can't continue execution after
// double fault. So there is no point of test runner (default and custom). We run directly from
// start() function.

use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptDescriptorTable;

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(mini_os::gdt::DOUBLE_FAULT_INDEX);
            // comment out set_stack_index line to fail the test!
        }
        idt
    };
}

pub fn init_test_idt() {
    TEST_IDT.load();
}

use mini_os::{exit_qemu, serial_println, QemuExitcode};
use x86_64::structures::idt::InterruptStackFrame;

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitcode::Success);
    mini_os::hlt_loop();
}
