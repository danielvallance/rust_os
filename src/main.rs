//! This is a bare-minimum freestanding Rust executable

// #![no_main] tells rustc that we do not want to use the entry point defined by the
// Rust runtime (as the Rust runtime requires an underlying OS).
#![no_main]
// #![no_std] tells rustc that we do not want to link this executable against the
// standard library (as it relies on an underlying OS).
#![no_std]

use core::panic::PanicInfo;

/// This is a custom panic handler, as we do not have access to the default
/// one in the standard library. This panic handler just loops forever.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
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
    loop {}
}
