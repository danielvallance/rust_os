//! Integration test which demonstrates how to test for an
//! expected panic in the absence of the should_panic macro
//! which depends on the standard library.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rust_os::{QemuExitCode, exit_qemu, serial_print, serial_println};

/// Entry point for the test. Runs the function which should panic, and if it does not,
/// prints a failure to the serial interface, and exits QEMU with an error.
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    should_fail();
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

/// Function which should panic, by asserting 0 != 1
fn should_fail() {
    serial_print!("should_panic::should_fail...\t");
    assert_eq!(0, 1);
}

/// Panic handler which prints success message, and exits QEMU with
/// a success code (as we expect a panic)
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}
