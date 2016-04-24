//! The VGA module provides an interface to the VGA text buffer system.

use spin::Mutex;

mod device;
mod terminal;

pub use self::device::{Color, Device, Position};
pub use self::terminal::Terminal;

pub static VGA_TERMINAL : Mutex<Terminal> = unsafe {
    Mutex::new(Terminal::new(Device::new_actual_vga_device()))
};

macro_rules! println {
    ($fmt:expr) => {
        {
            use core::fmt::Write;
            let mut term = ::vga::VGA_TERMINAL.lock();
            writeln!(term, $fmt).expect("write to VGA cannot fail");
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        {
            use core::fmt::Write;
            let mut term = ::vga::VGA_TERMINAL.lock();
            writeln!(term, $fmt, $($arg)*).expect("write to VGA cannot fail");
        }
    }
}

macro_rules! print {
    ($fmt:expr) => {
        {
            use core::fmt::Write;
            let mut term = ::vga::VGA_TERMINAL.lock();
            write!(term, $fmt).expect("write to VGA cannot fail");
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        {
            use core::fmt::Write;
            let mut term = ::vga::VGA_TERMINAL.lock();
            write!(term, $fmt, $($arg)*).expect("write to VGA cannot fail");
        }
    }
}
