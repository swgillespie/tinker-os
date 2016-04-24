#![feature(lang_items, const_fn, asm, nonzero)]
#![allow(dead_code, unused_imports)]
#![no_std]

extern crate rlibc;
extern crate spin;

#[macro_use]
pub mod vga;
mod lang;

#[no_mangle]
pub extern "C" fn kernel_main() {
    vga::VGA_TERMINAL.lock().clear_screen();
    println!("Initial boot sequence complete");
    for i in 0..10000 {
        println!("scrolling works: {}", i);
    }
}
