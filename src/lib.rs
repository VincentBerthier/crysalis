// File: src/lib.rs
// Project: Crysalis OS
// Creation date: Thursday 18 July 2024
// Author: Vincent Berthier <test.test>
// -----
// Last modified: Thursday 18 July 2024 @ 23:55:16
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

//! Implementation of the Crysalis kernel.
//! Started with [Writing an OS in Rust](https://os.phil-opp.com/)

#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![warn(missing_docs)]

use core::panic::PanicInfo;

use x86_64::instructions::hlt;

/// CPU interrupts handling.
pub mod interrupts;
/// I/O functionalities
pub mod io;
/// Test handlers.
pub mod tests;

/// Entry point for `cargo test`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    hlt_loop();
}

/// Initializes the kernel.
pub fn init() {
    interrupts::init();
}

/// Panic handler for tests.
///
/// # Parameters
/// * `info` - Information for the panic.
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("\x1B[31m[failed]\x1B[0m\n");
    serial_println!("{}", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

/// Function called on a panic during a test
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

/// [Hlt instruction](https://en.wikipedia.org/wiki/HLT_(x86_instruction)) loop.
pub fn hlt_loop() -> ! {
    loop {
        hlt();
    }
}

/// Exit codes for Qemu sent through the port-mapped IO.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    /// Success code
    Success = 0x10,
    /// Failure code
    Failed = 0x11,
}

/// Forces exit of Qemu
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    // SAFETY:
    // Port-mapped IO
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}
