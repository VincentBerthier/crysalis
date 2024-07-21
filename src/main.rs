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
#![feature(custom_test_frameworks)] // Use a custom test framework since test is in std
#![test_runner(crysalis::tests::test_runner)] // define the test runner as being a custom one.
#![reexport_test_harness_main = "test_main"] // otherwise it launches the kernel

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use crysalis::{
    hlt_loop, init,
    memory::{self, BootInfoFrameAllocator},
    println,
};
use x86_64::{structures::paging::Page, VirtAddr};

entry_point!(kernel_main);

/// The (true) entrypoint of the OS
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    let mut mapper = init(boot_info);
    println!("Hello world!");

    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // map an unused page
    let page = Page::containing_address(VirtAddr::new(0x0dea_dbee_f000));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // write the string `New!` to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe {
        page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e);
    }

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
