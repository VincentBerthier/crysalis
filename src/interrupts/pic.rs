use pic8259::ChainedPics;
use spin;
use x86_64::structures::idt::InterruptStackFrame;

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
