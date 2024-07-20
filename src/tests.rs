// File: src/tests.rs
// Project: Crysalis OS
// Creation date: Thursday 18 July 2024
// Author: Vincent Berthier <test.test>
// -----
// Last modified: Thursday 18 July 2024 @ 23:50:40
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

use core::any::type_name;

use crate::{exit_qemu, serial_print, serial_println, QemuExitCode};

/// Testing driver for the kernel. Will be called externally.
///
/// # Parameters
/// * `tests` - List of tests.
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    serial_println!("\x1B[32mAll tests passed.\x1B[0m");
    exit_qemu(QemuExitCode::Success);
}

/// A trait implemented for all tests.
pub trait Testable {
    /// Run the test.
    fn run(&self);
    /// Display the starting test message.
    fn start(&self);
    /// Display the test success message.
    fn success(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        self.start();
        self();
        self.success();
    }

    fn start(&self) {
        serial_print!("{:<100}...", type_name::<T>());
    }

    fn success(&self) {
        serial_println!("\r\x1B[32m{:<100}[Ok]\x1B[0m", type_name::<T>());
    }
}
