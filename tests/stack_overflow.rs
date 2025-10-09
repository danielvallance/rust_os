//! Integration test which initialises and loads a test IDT,
//! and checks that a stack overflow triggers the double
//! fault handler.

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use lazy_static::lazy_static;
use rust_os::{QemuExitCode, exit_qemu, serial_print, serial_println};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

/// Double fault handler which exits QEMU with a success code
extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

// Test IDT which sets the double fault handler to test_double_fault_handler
lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(rust_os::gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}

/// Loads the test IDT onto the CPU
pub fn init_test_idt() {
    TEST_IDT.load();
}

/// Panic handler which is a wrapper around rust_os::test_panic_handler
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info)
}

/// Function that triggers a stack overflow with infinite recursion
#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    // Ensures stack overflow by preventing tail call recursion
    volatile::Volatile::new(0).read();
}

/// Initialises and loads IDT, triggers stack overflow and tests that the
/// double fault handler was triggered.
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");

    rust_os::gdt::init();
    init_test_idt();

    stack_overflow();

    // If the double fault handler was not triggered, panic and fail
    panic!("Execution continued after stack overflow");
}
