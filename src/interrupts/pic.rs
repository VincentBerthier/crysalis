// File: src/interrupts/pic.rs
// Project: Crysalis OS
// Creation date: Saturday 10 August 2024
// Author: Vincent Berthier <test.test>
// -----
// Last modified: Saturday 10 August 2024 @ 18:25:08
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

use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use pic8259::ChainedPics;
use spin::{self, Mutex};
use x86_64::{instructions::port::Port, structures::idt::InterruptStackFrame};

use crate::{print, tasks::keyboard};

/// Offset of the Primary Programmable Interrupt Controller.
pub const PIC_1_OFFSET: u8 = 32;
/// Offset of the Secondary Programmable Interrupt Controller.
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// Definition of the Programmable Interrupt Controllers.
pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

/// Inits the PICS
pub fn init() {
    unsafe {
        PICS.lock().initialize();
    }
}

/// Index of the various interrupts in the PIC.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    /// Offset of the Timer Hardware Interrupt
    Timer = PIC_1_OFFSET,
    /// Offset for keyboard interrupts.
    Keyboard,
}

impl InterruptIndex {
    /// Casts the Offset to an u8
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Hardware timer iterrupt handler
pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

/// Keyboard event interrupt
pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Azerty, ScancodeSet1>> = Mutex::new(
            Keyboard::new(ScancodeSet1::new(), layouts::Azerty, HandleControl::Ignore)
        );
    }
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    keyboard::add_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
