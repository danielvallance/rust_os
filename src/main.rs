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

use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;
use rust_os::println;

/// This is a custom panic handler, as we do not have access to the default
/// one in the standard library. This panic handler just loops forever.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    rust_os::hlt_loop()
}

// Panic handler in test mode which is a wrapper around rust_os::test_panic_handler
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info);
}

// Specifies kernel_main as the entry point for the freestanding executable
entry_point!(kernel_main);

/// Entry point for the freestanding kernel executable. It takes a BootInfo struct
/// from the bootloader as an argument.
fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    // Invokes the vga module's println! macro to write "Hello world!" to the VGA text buffer
    println!("Hello world!");

    // Initialise and load IDT with breakpoint exception handler
    rust_os::init();

    use x86_64::registers::control::Cr3;

    // Print physical address of the active level 4 page table. This is stored in the CR3 register
    let (level_4_page_table, _) = Cr3::read();
    println!(
        "Level 4 page table at: {:?}",
        level_4_page_table.start_address()
    );

    // Run tests
    #[cfg(test)]
    test_main();

    rust_os::hlt_loop()
}
