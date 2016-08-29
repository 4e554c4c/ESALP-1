/*
 * Copyright 2016 JJ Garzella and Calvin Lee. See the README.md
 * file at the top-level directory of this distribution.
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>, at your option.
 * This file may not be copied, modified, or distributed
 * except according to those terms.
*/

use spin::Mutex;
use interrupts::cpuio::Port;
use core::sync::atomic::AtomicBool;
use core::{mem,ptr};

pub static KBDUS: [char; 128] =
[
	'\0',  '\x27', '1', '2', '3', '4', '5', 
	'6', '7', '8',	'9', '0', '-', '=', '\x08',/* Backspace */
	'\t',/* Tab */
	'q', 'w', 'e', 'r', 't', 'y', 
	'u', 'i', 'o', 'p', '[', ']', '\n',	/* Enter key */
	 '\0', /* 29   - Control */
	'a', 's', 'd', 'f', 'g', 'h',
	'j', 'k', 'l', ';', /* 39 */
	'\'', '`',   '\0', /* Left shift */
	'\\', 'z', 'x', 'c', 'v', 'b', 'n', /* 49 */
	'm', ',', '.', '/',   '\0',	/* Right shift */
	'*',
	'\0',	/* Alt */
	' ',	/* Space bar */
	'\0',	/* Caps lock */
	'\0',	/* 59 - F1 key ... > */
	'\0','\0','\0','\0','\0','\0','\0','\0',
	'\0',	/* < ... F10 */
	'\0',	/* 69 - Num lock*/
	'\0',	/* Scroll Lock */
	'\0',	/* Home key */
	'\0',	/* Up Arrow */
	'\0',	/* Page Up */
	'-',
	'\0',	/* Left Arrow */
	'\0',
	'\0',	/* Right Arrow */
	'+',
	'\0',	/* 79 - End key*/
	'\0',	/* Down Arrow */
	'\0',	/* Page Down */
	'\0',	/* Insert Key */
	'\0',	/* Delete Key */
	'\0',   '\0',   '\0',
	'\0',	/* F11 Key */
	'\0',	/* F12 Key */
	'\0',	/* All other keys are undefined */
	'\0','\0','\0','\0','\0','\0','\0','\0','\0','\0',
	'\0','\0','\0','\0','\0','\0','\0','\0','\0','\0',
	'\0','\0','\0','\0','\0','\0','\0','\0','\0','\0',
	'\0','\0','\0','\0','\0','\0','\0','\0'
];

lazy_static!{
	pub static ref KEYS: [AtomicBool; 128] = unsafe {
		let mut keys: [AtomicBool; 128] = mem::uninitialized();

		for elem in &mut keys[..] {
			ptr::write(elem, AtomicBool::new(false));
		}
		keys
	};
}

pub static KEYBOARD: Mutex<Port<u8>> = Mutex::new(
	unsafe {
		Port::new(0x60)
	}
);