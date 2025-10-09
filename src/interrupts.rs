//! This module creates an interrupt descriptor table (IDT)
//! and loads it on the CPU.
//!
//! Currently the only exception which the IDT handles
//! is the breakpoint exception.

use crate::println;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

// Create IDT and set its breakpoint handler to the breakpoint_handler function
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
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
