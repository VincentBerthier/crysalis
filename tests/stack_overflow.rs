#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

use crysalis::{exit_qemu, interrupts::gdt, serial_print, serial_println, QemuExitCode};
use lazy_static::lazy_static;
use volatile::Volatile;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}

/// Start function for the test
///
/// # Panics
/// If the test ran successfully
#[expect(clippy::panic, reason = "test")]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");

    gdt::init();
    init_test_idt();

    // trigger a stack overflow
    stack_overflow();

    panic!("Execution continued after stack overflow");
}

pub fn init_test_idt() {
    TEST_IDT.load();
}

#[expect(unconditional_recursion, reason = "test")]
fn stack_overflow() {
    stack_overflow(); // for each recursion, the return address is pushed
    Volatile::new(0).read(); // prevent tail recursion optimizations
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crysalis::test_panic_handler(info)
}

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}
