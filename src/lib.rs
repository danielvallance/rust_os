//! This library contains the infrastructure for running unit tests and integration tests

#![no_std]
// The entry point defined in main.rs is not compiled with this library in test
// mode. Therefore library conditionally defines its own entry point when run in test mode
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
// Instructing the compiler to use the x86-interrupt calling convention is
// an unstable feature, so enable it here
#![feature(abi_x86_interrupt)]

// Link this crate with the alloc crate
extern crate alloc;

pub mod allocator;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod vga;

use core::panic::PanicInfo;

#[cfg(test)]
use bootloader::{BootInfo, entry_point};

// Port address of isa-debug-exit as defined in Cargo.toml
const ISA_DEBUG_EXIT_PORT: u16 = 0xf4;

/// General kernel initialisation function
pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

/// Trait for functions which can be passed to our test runner
pub trait Testable {
    fn run(&self) -> ();
}

// Testable is implemented for all "Fn()" and it simply
// runs the function, while printing the test name and
// "[ok]" on success to the serial port.
impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

/// Custom test runner. This does not depend on the standard library.
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }

    // Exit QEMU with the success code defined in Cargo.toml
    exit_qemu(QemuExitCode::Success);
}

/// Prints failure information to the serial port, and exits QEMU with a failure
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);

    // Exit QEMU with a failure code
    exit_qemu(QemuExitCode::Failed);
    hlt_loop()
}

// Specifies the entry point of the test executable
#[cfg(test)]
entry_point!(test_kernel_main);

/// Entry point for 'cargo test'. This is necessary as the entry point defined in
/// main.rs cannot be used by this library in test mode. It takes a BootInfo struct
/// from the bootloader as an argument.
#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    // Initialise kernel
    init();
    test_main();
    hlt_loop()
}

/// Panic handler for this library in test mode. The one defined in main.rs
/// cannot be used by this library in test mode.
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

/// 4 byte exit code which isa-debug-exit expects when it is used
/// to exit QEMU. isa-debug-exit is configured to expect 4 bytes
/// in Cargo.toml so #[repr(32)] is used to guarantee this is 4 bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    // QEMU will exit with (val << 1) | 1 where val is what is
    // written to isa-debug-exit. Therefore we must define
    // 33 ((0x10 << 1) | 1) as a success code in the Cargo.toml
    // as normally any non-zero value is interpreted as failure.
    //
    // We use values not well-known by QEMU to distinguish test exits
    // from normal exits
    Success = 0x10,
    Failed = 0x11,
}

/// Open a port to isa-debug-exit and write the 4 byte exit code to it to exit QEMU
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(ISA_DEBUG_EXIT_PORT);
        port.write(exit_code as u32);
    }
}

/// Executes the hlt instruction in a loop to let the CPU
/// sleep until it receives an interrupt.
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/// Tests the breakpoint exception handler by invoking a breakpoint
/// instruction (int3) and checking that the kernel continues.
#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}
