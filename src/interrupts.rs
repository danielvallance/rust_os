//! This module creates an interrupt descriptor table (IDT)
//! and loads it on the CPU.
//!
//! Currently the only exception which the IDT handles
//! is the breakpoint exception.

use crate::{gdt, print, println};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

/// PIC1 will send interrupt vector indices 32-39
pub const PIC_1_OFFSET: u8 = 32;

/// PIC2 will send interrupt vector indices 40-47
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// Index to the IDT
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

/// Spinlock protected interface to 2 chained programmable interrupt controllers (PICs)
pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

// Create IDT and set its breakpoint handler to the breakpoint_handler function
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);

        // Double fault handler uses known good stack in the IST
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);
        idt
    };
}

/// Load the IDT onto the CPU
pub fn init_idt() {
    IDT.load();
}

/// Handles breakpoint exception by pretty printing the stack frame.
///
/// Handling exceptions does not require the use of naked functions as
/// the compiler can be instructed to use the x86-interrupt calling convention
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

/// Handles double fault by pretty printing the stack frame using the panic macro.
///
/// The panic macro is used as this function is diverging as x86-64 does
/// not allow double fault handlers to return.
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

/// Timer interrupt handler
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    print!(".");

    // Send 'end-of-interrupt' (EOI) signal to PIC, so it knows the interrupt has been
    // processed, and that it can send more.
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}
