//! This module provides an interface to write to the VGA text buffer.
//!
//! The unsafe operations of writing to a raw pointer are restricted
//! to this module, therefore callers of this module do not have
//! to use unsafe blocks.

use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

// Dimensions of the VGA text buffer
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

// Memory map address of VGA text buffer
const VGA_BUF_ADDR: usize = 0xb8000;

/// Colour for the foreground/background of VGA characters
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Colour {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// Unit struct which represents the colour byte written to the VGA
/// text buffer.
///
/// #[repr(transparent)] ensures that this is represented as a
/// single byte in memory which is what the VGA interface expects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColourCode(u8);

impl ColourCode {
    /// Takes the foreground and background colours and stores
    /// background in the first 4 bits, and foreground in the
    /// latter 4 bits as the VGA spec defines.
    fn new(foreground: Colour, background: Colour) -> ColourCode {
        ColourCode((background as u8) << 4 | (foreground as u8))
    }
}

/// This struct defines the two bytes required to describe
/// a character in the VGA text buffer (the ASCII character,
/// and the colour byte)
///
/// #[repr(C)] ensures that the ASCII byte precedes the colour
/// byte in memory
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    colour_code: ColourCode,
}

/// This struct has a single element which is a 2D array of
/// ScreenChar structs of dimensions {BUFFER_WIDTH, BUFFER_HEIGHT}
/// representing the VGA screen.
///
/// The ScreenChars are wrapped in the Volatile struct to instruct
/// the compiler to not optimise out any reads/writes as they have
/// side effects on which we rely.
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// Maintains the state of a writer to the VGA text buffer
pub struct Writer {
    column_position: usize,
    colour_code: ColourCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    /// Writes a single byte to the VGA text buffer
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            // Starts a newline on '\n'
            b'\n' => self.new_line(),
            byte => {
                // Starts a newline if this line is full
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                // We always write on the last row. Once it is full the rows above shift up one.
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                // Write the character into the VGA text buffer, then increment column position.
                let colour_code = self.colour_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    colour_code,
                });
                self.column_position += 1;
            }
        }
    }

    /// Starts a new line by shifting every existing line upwards.
    fn new_line(&mut self) {
        // Move every character to the space in the line above.
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }

        // Clear the final row and reset column_position to 0
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    // Clears the specified row by writing a blank character to every space in that row
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            colour_code: self.colour_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    // Write string to VGA text buffer by writing each individual byte to it
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            // Strings are made of UTF-8 code points, and if the code point
            // is longer than a byte, its constituent bytes will not be valid
            // printable ASCII, so those are ignored by writing an "unknown" byte
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }
}

impl fmt::Write for Writer {
    /// Wrapper around write_string which returns fmt::Result to implement fmt::Write trait
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// lazy_static means the WRITER is initialised when it is used for
// the first time, as opposed to at compile time.
//
// This is required as the VGA_BUF_ADDR raw pointer cannot be
// converted to a reference at compile time.
lazy_static! {

    // This writer is protected by a spinlock mutex (the
    // std::sync::Mutex is unavailable)
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        colour_code: ColourCode::new(Colour::Yellow, Colour::Black),
        buffer: unsafe { &mut *(VGA_BUF_ADDR as *mut Buffer) },
    });
}

// Defines print! and println! macros which call this module's
// print functionality. These macros are available to the whole crate.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}
