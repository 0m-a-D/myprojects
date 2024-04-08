use crate::gdt;
use crate::{print, println};
use lazy_static::lazy_static;
// lazily initializes a static variable when referenced for the first time
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        // breakpoint handler
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
        idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(gdt::DOUBLE_FAULT_INDEX);
        /*

        set_stack_index is marked unsafe because caller should ensure used index is valid and
        not already used for another exception. In this case, it's set to index 0.

        */
        idt[InterrupIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        // InterruptDescriptorTable implements "IndexMut" trait so we could use array indexing
        // syntax. handler function signarure for interrupts is same as usual exceptions because
        // CPU handles interrupts the same way as exceptions.
        idt[InterrupIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        }

        //page fault handler
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load(); // uses the "lidt" instruction to load interrupt descriptor table...
}

/// CPU EXCEPTION HANDLERS
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:?}", stack_frame);
}
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

use crate::hlt_loop;
use x86_64::structures::idt::PageFaultErrorCode;

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    // CR2 register is set by CPU on page fault and it holds the virtual address that caused the
    // page fault...

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}

/// HARDWARE INTERRUPTS SECTION
use pic8259::ChainedPics;
// use spin;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterrupIndex {
    Timer = PIC_1_OFFSET, // using C-like enum
    Keyboard, // we didn't have to specify value 33, because it defaults to previous value + 1.
}
impl InterrupIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    print!(".");
    // sending and EOI signal that timer interrupt has been processed...
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterrupIndex::Timer.as_u8());
    }
}
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);
    // 0x60 (I/O port) is data port of PS-2 keyboard...
    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode); // new

    // similar to timer interrupt, we send an EOI signal to PICs...
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterrupIndex::Keyboard.as_u8());
    }
}
