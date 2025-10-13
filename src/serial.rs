//! This module provides an interface to write to the serial port.
//!
//! The unsafe operations of writing to a raw pointer are restricted
//! to this module, therefore callers of this module do not have
//! to use unsafe blocks.

use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

// There are many ports used in serial communication, however the
// SerialPort::new function can calculate them all from this
const SERIAL_PORT_ADDR: u16 = 0x3F8;

// Spinlock protected SerialPort struct which users of this module
// should use for all writes to the serial port.
lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(SERIAL_PORT_ADDR) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

/// Print formatted strings to serial port
#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // Disable interrupts to avoid the interrupt handler and _print function
    // deadlocking over the serial port lock
    interrupts::without_interrupts(|| {
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    });
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}
