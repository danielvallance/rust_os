//! This module contains our kernel's global descriptor table (GDT) which we
//! will define and load onto the CPU.
//!
//! The kernel's GDT contains a reference to the kernel's task state segment
//! (TSS) which contains an interrupt stack table (IST) in which a known good
//! stack is created for use by the double fault handler.

use lazy_static::lazy_static;
use x86_64::VirtAddr;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;

/// The double fault handler will use the first stack defined in the IST
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

/// Segment selectors for a code segment and TSS.
///
/// This will be used to load the CS register and task register
/// with indices to the correct segments in the new GDT.
struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

// Define new TSS with interrupt stack table that contains
// known good stack for the double fault interrupt. This
// is useful for when the double fault handler is invoked
// when the current stack is in a bad state, for example a stack
// overflow.
lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            // Currently using static mut for stack allocation until proper stack allocation is implemented.
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(&raw const STACK);
            stack_start + STACK_SIZE
        };
        tss
    };
}

// New GDT which has a reference to the above TSS.
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

/// Loads the GDT onto the CPU.
pub fn init() {
    use x86_64::instructions::segmentation::{CS, Segment};
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();

    // Update the state of the CS and task registers to refer
    // to the correct segments in the newly loaded GDT.
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}
