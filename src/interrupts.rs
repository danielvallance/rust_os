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

/// Address of PS/2 controller's data port
const PS2_DATA_PORT_ADDR: u16 = 0x60;

/// Index to the IDT
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
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
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
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

/// Keyboard interrupt handler which handles the user entering numbers by printing them to the VGA buffer
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    // Read scancode which can be used to determine which key was pressed.
    // The PS2 keyboard controller will not send another interrupt until the scancode has been read.
    let mut port = Port::new(PS2_DATA_PORT_ADDR);
    let scancode: u8 = unsafe { port.read() };

    // Determine which key was pressed. Only digits 0-9 are implemented right now.
    let key = match scancode {
        0x02 => Some('1'),
        0x03 => Some('2'),
        0x04 => Some('3'),
        0x05 => Some('4'),
        0x06 => Some('5'),
        0x07 => Some('6'),
        0x08 => Some('7'),
        0x09 => Some('8'),
        0x0a => Some('9'),
        0x0b => Some('0'),
        _ => None,
    };

    // Print the pressed key
    if let Some(key) = key {
        print!("{}", key);
    }

    // Send EOI signal to notify PIC that the interrupt has been handled
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
