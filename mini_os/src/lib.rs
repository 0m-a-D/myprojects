#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)] // mutable references in const functions are unstable
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
extern crate alloc; // using alloc which is a subset of "std" like "core"...
pub mod allocator;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod task;
pub mod vga_buffer;
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[repr(u32)]
pub enum QemuExitcode {
    Success = 0x10, // 16 --> mapped to success exit code 33 [(16 << 1) | 1]...
    Failure = 0x11, // 17
}
pub fn exit_qemu(exit_code: QemuExitcode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        // 0xf4 is chosen as it is generally unused on x86 architecture...
        port.write(exit_code as u32);
    }
}

use core::panic::PanicInfo;
pub trait Testable {
    fn run(&self);
}
impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("running {} tests!", tests.len());
    for test in tests {
        test.run();
    }
    // update to exit qemu
    exit_qemu(QemuExitcode::Success);
}
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[Failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitcode::Failure);
    hlt_loop();
}

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() }; // initializing 8259 PIC
    x86_64::instructions::interrupts::enable(); // enables interrupts using "sti" (set instruction)
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

#[cfg(test)]
entry_point!(test_kernel_main);

#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
