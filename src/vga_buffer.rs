// Copyright 2016 Phillip Oppermann, Calvin Lee and JJ Garzella.
// See the README.md file at the top-level directory of this
// distribution.
//
// Licensed under the MIT license <LICENSE or
// http://opensource.org/licenses/MIT>, at your option.
// This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(dead_code)]

use core::ptr::Unique;
use spin::Mutex;
use log_buffer::LogBuffer;
use core::sync::atomic::{ATOMIC_BOOL_INIT, ATOMIC_USIZE_INIT};
use core::cell::UnsafeCell;
use core::convert::AsMut;

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

static WRITER: Mutex<Writer> = Mutex::new(Writer {
    column_position: 0,
    color_code: ColorCode::new(Color::Pink, Color::Black),
    buffer: unsafe { Unique::new(0xb8000 as *mut _) },
});

pub struct BufWrapper<T>([T; 4096]);

impl<T> AsMut<[T]> for BufWrapper<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.0.as_mut()
    }
}

pub static WRITE_BUF: LogBuffer<BufWrapper<u8>> = LogBuffer {
    buffer: UnsafeCell::new(BufWrapper::<u8> { 0: [0xff; 4096] }),
    position: ATOMIC_USIZE_INIT,
    lock: ATOMIC_BOOL_INIT,
};

macro_rules! println {
	($fmt:expr) => (print!(concat!($fmt, "\n")));
	($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

macro_rules! print {
	($($arg:tt)*) => ({
		use core::fmt::Write;
		let mut wb = &($crate::vga_buffer::WRITE_BUF);
		wb.write_fmt(format_args!($($arg)*)).unwrap();
	});
}

pub fn clear_screen() {
    for _ in 0..BUFFER_HEIGHT {
        println!("");
    }
}

pub fn flush_screen() {
    WRITER.lock().write_str(WRITE_BUF.extract());
    WRITE_BUF.clear();
}

pub fn change_color(fg: Color, bg: Color) {
    WRITER.lock().color(fg, bg);
}

#[allow(dead_code)]
#[repr(u8)]
pub enum Color {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xa,
    LightCyan = 0xb,
    LightRed = 0xc,
    Pink = 0xd,
    Yellow = 0xe,
    White = 0xf,
}

struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: Unique<Buffer>,
}

impl Writer {
    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\0' => (),
            b'\n' => self.new_line(),
            b'\t' => self.write_str("    "),
            b'\x08' => self.back_space(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                self.buffer().chars[row][col] = ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                };
                self.column_position += 1;
            }
        }
    }

    fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }

    fn color(&mut self, fg: Color, bg: Color) {
        self.color_code = ColorCode::new(fg, bg);
    }

    fn buffer(&mut self) -> &mut Buffer {
        unsafe { self.buffer.get_mut() }
    }

    fn new_line(&mut self) {
        for row in 0..(BUFFER_HEIGHT - 1) {
            let buffer = self.buffer();
            buffer.chars[row] = buffer.chars[row + 1]
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn back_space(&mut self) {
        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;
        if col == 0 {
            return;
        } else {
            self.buffer().chars[row][col - 1] = ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            };
            self.column_position = col - 1;
        }
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        self.buffer().chars[row] = [blank; BUFFER_WIDTH];
    }
}

impl ::core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct ColorCode(u8);

impl ColorCode {
    const fn new(fg: Color, bg: Color) -> ColorCode {
        ColorCode((bg as u8) << 4 | (fg as u8))
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}
