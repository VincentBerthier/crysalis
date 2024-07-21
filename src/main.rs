// File: src/main.rs
// Project: Crysalis OS
// Creation date: Monday 15 July 2024
// Author: Vincent Berthier <test.test>
// -----
// Last modified: Thursday 18 July 2024 @ 23:55:34
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

//! Started with [Writing an OS in Rust](https://os.phil-opp.com/)

#![no_std] // don't link to the standard library
#![no_main] // disable all Rust-level entry points
#![warn(missing_docs)]
#![feature(custom_test_frameworks)] // Use a custom test framework since test is in std
#![test_runner(crysalis::tests::test_runner)] // define the test runner as being a custom one.
#![reexport_test_harness_main = "test_main"] // otherwise it launches the kernel

use core::panic::PanicInfo;
use crysalis::{hlt_loop, init, println};

/// The entrypoint of the OS
#[no_mangle] // Don't mangle the name of the function, needed to be able to launch it.
pub extern "C" fn _start() -> ! {
    init();

    println!("Hello world!");

    #[cfg(test)]
    test_main();

    println!("all good!");
    hlt_loop();
}

/// Function called on a panic
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crysalis::test_panic_handler(info)
}

#[expect(clippy::missing_panics_doc, reason = "…")]
#[test_case]
fn trivial_assertion() {
    assert_eq!(env!("CARGO_PKG_VERSION"), "0.1.0", "wrong version");
}
