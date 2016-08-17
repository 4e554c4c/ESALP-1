/*
 * Copyright 2016 JJ Garzella and Calvin Lee. See the README.md
 * file at the top-level directory of this distribution.
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>, at your option.
 * This file may not be copied, modified, or distributed
 * except according to those terms.
 */
use core::ptr::Unique;
use spin::Mutex;

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

pub static WRITER: Mutex<Writer> = Mutex::new(Writer {
	column_position: 0,
	color_code: ColorCode::new(Color::Pink, Color::Black),
	buffer: unsafe { Unique::new( 0xb8000 as *mut _ ) },
});

macro_rules! println {
	($fmt:expr) => (print!(concat!($fmt, "\n")));
	($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

macro_rules! print {
	($(arg:tt)*) => ({
		use core::fmt::Write;
		let mut writer = $crate::vga_buffer::Writer.lock();
		writer.write_fmt(format_args!($($arg)*).unwrap());
	});
}

pub fn clear_screen() {
	for _ in 0..BUFFER_HEIGHT {
		println!("");
	}
}

#[allow(dead_code)]
#[repr(u8)]
pub enum Color {
	Black      = 0x0,
	Blue       = 0x1,
	Green      = 0x2,
	Cyan       = 0x3,
	Red        = 0x4,
	Magenta    = 0x5,
	Brown      = 0x6,
	LightGray  = 0x7,
	DarkGray   = 0x8,
	LightBlue  = 0x9,
	LightGreen = 0xa,
	LightCyan  = 0xb,
	LightRed   = 0xc,
	Pink       = 0xd,
	Yellow     = 0xe,
	White      = 0xf,
}

pub struct Writer {
	column_position: usize,
	color_code: ColorCode,
	buffer: Unique<Buffer>,
}

impl Writer {
	pub fn write_byte(&mut self, byte: u8) {
		match byte {
			b'\n' => self.new_line(),
			byte  => {
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

	pub fn write_str(&mut self, s: &str) {
		for byte in s.bytes() {
			self.write_byte(byte);
		}
	}

	fn buffer(&mut self) -> &mut Buffer {
		unsafe { self.buffer.get_mut() }
	}

	fn new_line(&mut self) {
		for row in 0..(BUFFER_HEIGHT-1) {
			let buffer = self.buffer();
			buffer.chars[row] = buffer.chars[row+1]
		}
		self.clear_row(BUFFER_HEIGHT-1);
		self.column_position = 0;
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
