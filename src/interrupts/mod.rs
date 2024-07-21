use lazy_static::lazy_static;
use pic::{keyboard_interrupt_handler, timer_interrupt_handler, InterruptIndex};
use x86_64::{
    instructions::interrupts as x86_64_interrupts,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame},
};

use crate::println;

/// The Interrupt Stack Tables & Task State Segments definitions
/// for the Global Descriptor Table.
pub mod gdt;
/// Hardware interrupts
pub mod pic;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        // Fault interrupts
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        // Hardware interrupts
        idt[InterruptIndex::Timer.as_u8()]
            .set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_u8()]
            .set_handler_fn(keyboard_interrupt_handler);

        idt
    };
}

/// Loads the Interrupt Definition Table into the memory.
pub fn init() {
    gdt::init();
    IDT.load();
    pic::init();
    x86_64_interrupts::enable();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

#[expect(clippy::panic, reason = "we want it to panic if we reach that step.")]
extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

#[test_case]
fn breakpoint_exception() {
    use x86_64::instructions::interrupts::int3;
    // invoke a breakpoint exception
    int3();
}
