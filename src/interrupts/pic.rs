use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use pic8259::ChainedPics;
use spin::{self, Mutex};
use x86_64::{instructions::port::Port, structures::idt::InterruptStackFrame};

use crate::print;

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
    print!(".");
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
    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{character}"),
                DecodedKey::RawKey(key) => print!("{key:?}"),
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
