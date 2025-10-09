//! This is a bare-minimum freestanding Rust executable

// #![no_main] tells rustc that we do not want to use the entry point defined by the
// Rust runtime (as the Rust runtime requires an underlying OS).
#![no_main]
// #![no_std] tells rustc that we do not want to link this executable against the
// standard library (as it relies on an underlying OS).
#![no_std]
// The default test framework is not available as it relies on the standard
// library. Therefore the custom_test_frameworks feature is used to run
// tests with a custom test framework. The tests will be passed to the specified
// test runner (rust_os::test_runner) for execution.
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
// Configure entry point for test run to be called test_main
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rust_os::println;

/// This is a custom panic handler, as we do not have access to the default
/// one in the standard library. This panic handler just loops forever.
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Panic handler in test mode which is a wrapper around rust_os::test_panic_handler
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info);
}

/// The '#[unsafe(no_mangle)]' attribute directs rustc to not mangle the name,
/// as we need to pass the name of this entry point function to the linker.
///
/// We also specify that it uses the C calling convention as this
/// executable will be called with the C calling convention, not the
/// Rust one. This is because this freestanding executable will not
/// be invoked by the Rust runtime.
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // Invokes the vga module's println! macro to write "Hello world!" to the VGA text buffer
    println!("Hello world!");

    // Initialise and load IDT with breakpoint exception handler
    rust_os::init();

    // Trigger page fault by writing to unmapped address. The IDT
    // does not have handlers for page faults, double faults or triple
    // faults, therefore this page fault will cause a double fault, then
    // a triple fault which causes a system reset on most hardware and QEMU.
    unsafe {
        *(0xdeadbeef as *mut u8) = 42;
    }

    // Run tests
    #[cfg(test)]
    test_main();

    loop {}
}
