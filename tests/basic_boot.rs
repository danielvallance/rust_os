//! Integration test module which tests conditions in the same environment
//! as a standard boot of our kernel.

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rust_os::println;

/// Entry point to the integration test executable which just runs "test_main()"
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

/// Panic handler which is a wrapper around "rust_os::test_panic_handler"
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info);
}

/// Example test case which tests that printing to the VGA
/// text buffer does not cause a panic in the basic boot environment
#[test_case]
fn test_println() {
    println!("test_println output");
}
