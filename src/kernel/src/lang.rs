//! Lang items, as required by the Rust compiler in a freestanding environment.
use core::fmt;
use vga;

#[lang = "panic_fmt"]
extern "C" fn panic_fmt(args: ::core::fmt::Arguments, file: &str, line: u32) -> ! {
    println!("kernel panic!");
    println!("panic at file {}, line {}: {}", file, line, args);
    unsafe {
        asm!("hlt" :::: "volatile");
    }
    
    // control never actually gets here - this is just to please the compiler
    loop {}
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
