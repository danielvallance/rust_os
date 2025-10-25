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
use x86_64::structures::paging::Page;

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
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use rust_os::memory;
    use x86_64::VirtAddr;

    // Invokes the vga module's println! macro to write "Hello world!" to the VGA text buffer
    println!("Hello world!");

    // Initialise and load IDT with breakpoint exception handler
    rust_os::init();

    // The kernel maps the entirety of physical memory into virtual memory. The bootloader queries
    // the firmware for the address at which this mapping begins, then passes it to the kernel, which
    // then assigns it to this variable
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    // Initialise OffsetPageTable which implements the Mapper and Translate traits in
    // contexts where the entirety of physical memory is mapped into virtual memory
    let mut mapper = unsafe { memory::init(phys_mem_offset) };

    let mut frame_allocator = memory::EmptyFrameAllocator;

    // map a page which does not already have the required page tables
    // (and so will need the frame allocator to allocate some frames for them)
    let page = Page::containing_address(VirtAddr::new(0xdeadbeaf0000));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // write the string `New!` to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    // Run tests
    #[cfg(test)]
    test_main();

    rust_os::hlt_loop()
}
