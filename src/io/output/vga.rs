// File: src/vga_outputs.rs
// Project: Crysalis OS
// Creation date: Thursday 18 July 2024
// Author: Vincent Berthier <test.test>
// -----
// Last modified: Thursday 18 July 2024 @ 23:53:30
// Modified by: Vincent Berthier
// -----
// Copyright (c) 2024 <Vincent Berthier>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the 'Software'), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED 'AS IS', WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use core::{array::from_fn, fmt};

use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use x86_64::instructions::interrupts;

const VGA_MMIO: u64 = 0xb8000;

lazy_static! {
    /// Static text writer to the screen.
    ///
    /// [`lazy_static`] is necessary here to allow the mut Buffer,
    /// and [`spin::Mutex`] to allow for the writer to be mutable.
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Black, Color::Green),
        // SAFETY:
        // MMIO mapping
        buffer: unsafe { &mut *(VGA_MMIO as *mut Buffer) },
    });
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

/// Color codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    /// Black.
    Black = 0,
    /// Blue
    Blue = 1,
    /// Green
    Green = 2,
    /// Cyan
    Cyan = 3,
    /// Red
    Red = 4,
    /// Magenta
    Magenta = 5,
    /// Brown
    Brown = 6,
    /// `LightGray`
    LightGray = 7,
    /// `DarkGray`
    DarkGray = 8,
    /// `LightBlue`
    LightBlue = 9,
    /// `LightGreen`
    LightGreen = 10,
    /// `LightCyan`
    LightCyan = 11,
    /// `LightRed`
    LightRed = 12,
    /// Pink
    Pink = 13,
    /// Yellow
    Yellow = 14,
    /// White
    White = 15,
}

/// Represents a color code.
///
/// Colors are actually encoded such that:
/// - bytes 0-3 are for the foreground,
/// - bytes 4-7 are for the background
///
/// Each of those values come from [`Color`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    /// Create a new color code from background and foreground color.
    ///
    /// Each of those two colours fit into 4 bits. The background
    /// color is shifted four bits to the left then has the
    /// foreground color appended to get a valid u8 VGA color code.
    ///
    /// # Parameters
    /// * `foreground` - Foreground color,
    /// * `background` - Background color.
    const fn new(foreground: Color, background: Color) -> Self {
        Self((background as u8) << 4 | (foreground as u8))
    }
}

/// A character on the screen is an ascii value and a color.
///
/// Note however that it's not a *true* ascii value, but
/// a [code page 437](https://en.wikipedia.org/wiki/Code_page_437)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

impl ScreenChar {
    pub const fn blank(color: ColorCode) -> Self {
        Self {
            ascii_character: b' ',
            color_code: color,
        }
    }
}

/// Representation of the VGA screen.
#[repr(transparent)]
struct Buffer {
    /// Two dimensional array of [`BUFFER_WIDTH`] by [`BUFFER_HEIGHT`]
    /// characters.
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// Structure actually writing text on the screen.
///
/// Text is always written on the *last* line,
/// and everything is shifted up when writting on a new line.
pub struct Writer {
    /// Current column position on the last row.
    column_position: usize,
    /// The current color code.
    color_code: ColorCode,
    /// The VGA buffer where text is displayed.
    buffer: &'static mut Buffer,
}

impl Writer {
    /// Writes a single character (byte) on the screen.
    ///
    /// It will be written with the current [`Writer::color_code`].
    ///
    /// # Parameters
    /// * `byte` - The byte to write.
    pub fn write_byte(&mut self, byte: u8) {
        if byte == b'\n' {
            return self.new_line();
        }

        if self.column_position >= BUFFER_WIDTH {
            self.new_line();
        }

        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;
        let color_code = self.color_code;
        self.buffer.chars[row][col].write(ScreenChar {
            ascii_character: byte,
            color_code,
        });
        self.column_position += 1;
    }

    /// Prints a string on the screen.
    ///
    /// Non printable characters (other than the newline) are
    /// replaced by a black square.
    ///
    /// # Parameters
    /// * `s` - String to write.
    pub fn write_string(&mut self, string: &str) {
        string.bytes().for_each(|byte| match byte {
            // Printable character or a new line
            0x20..=0x7e | b'\n' => self.write_byte(byte),
            // A non printable character (replaced by a black square)
            _ => self.write_byte(0xfe),
        });
    }

    fn new_line(&mut self) {
        self.buffer.chars.rotate_left(1);
        let blank = ScreenChar::blank(self.color_code);
        self.buffer.chars[BUFFER_HEIGHT - 1] = from_fn(|_| Volatile::new(blank));
        self.column_position = 0;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        self.write_string(string);
        Ok(())
    }
}

/// Print formtatted text to the screen.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::io::output::vga::_print(format_args!($($arg)*)));
}

/// Print formatted text to the screen, then goes to the next line.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    #[expect(
        clippy::unwrap_used,
        reason = "it can never fail since we only return Ok(())"
    )]
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

/// # Panics
/// If the test fails…
#[test_case]
fn direct_output() {
    use core::fmt::Write;

    let string = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{string}").expect("writeln failed");
        for (i, character) in string.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(
                char::from(screen_char.ascii_character),
                character,
                "mismatch between characters"
            );
        }
    });
}

#[test_case]
fn println() {
    println!("test_println! output");
}

#[test_case]
fn print_many_lines() {
    (0..200).for_each(|_| println!("print_many_lines output"));
}

#[test_case]
fn print_long_line() {
    let string = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    println!("{string}{string}{string}{string}");
}

#[expect(clippy::missing_panics_doc, reason = "…")]
#[test_case]
fn check_output() {
    let string = "Some test string that fits on a single line";
    println!("{}", string);
    string.chars().enumerate().for_each(|(i, character)| {
        let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(
            char::from(screen_char.ascii_character),
            character,
            "printed values are different…"
        );
    });
}
