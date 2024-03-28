use core::ptr::addr_of;

use lazy_static::lazy_static;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5; // 20KiB size
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let start_start = VirtAddr::from_ptr(unsafe { addr_of!(STACK) });

            start_start + STACK_SIZE
        };
        tss
    };
}

use x86_64::structures::gdt::SegmentSelector;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable};

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selectors {
                code_selector,
                tss_selector,
            },
        )
    };
}

pub fn init() {
    // GDT.load();
    // just loading won't solve the stack overflow problem. Also need to modify double
    // fault IDT entry so it uses this new GDT.

    use x86_64::instructions::segmentation::{Segment, CS};
    use x86_64::instructions::tables::load_tss;
    // CS here stands for code selector

    GDT.0.load(); // uses the "lgdt" instruction
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}
