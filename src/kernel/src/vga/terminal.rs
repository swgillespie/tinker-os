//! A higher-level terminal interface abstracting
//! over the VGA buffer.

use core::fmt;
use vga::{Color, Device, Position};

pub struct Terminal {
    position: Position,
    device: Device
}

impl Terminal {
    pub const fn new(device: Device) -> Terminal {
        Terminal {
            position: Position::new(0, 0),
            device: device
        }
    }
    
    pub fn write_byte(&mut self, byte: u8) {
        // rules enforced here, in this order:
        //  1. if this write would cause the buffer
        //     to go one byte too wide, move to a new line
        //  2. if this write is a newline (b'\n'), move to a new line
        //  3. else if this write is a carriage return (b'\r'),
        //     move to the start of this line
        //  4. if this write would cause the buffer to go one line
        //     too high, shift all lines up by one line.
        //
        //  The position field of this struct points to the position
        //  where the next byte will go, so it's potentially invalid here.
        //  It must be valid after we enforce all of these rules.
        if self.position.col >= self.device.width() {
            self.position.col = 0;
            self.position.row += 1;
        }
        
        // neither of these characters print anything on the screen,
        // so we return early here.
        if byte == b'\n' {
            self.position.col = 0;
            self.position.row += 1;
            return;
        } else if byte == b'\r' {
            self.position.col = 0;
            return;
        }
        
        if self.position.row >= self.device.height() {
            let scroll_lines = self.position.row - self.device.height() + 1;
            self.device.scroll_up(scroll_lines);
            self.position.row -= scroll_lines;
        }
        
        // at this point, our position should be valid.
        // TODO(swgillespie) this could cause a panic.
        // This code is called from panic_fmt, so panicking here
        // would cause an infinite loop or other badness.
        //
        // In theory panic_fmt could have an atomic bool that it sets
        // when it is panicking, and a panic while panicking could just halt.
        self.device.write_byte(byte, self.position);
        self.position.col += 1;
    }
    
    pub fn set_foreground_color(&mut self, color: Color) {
        self.device.set_foreground_color(color);
    }
    
    pub fn set_background_color(&mut self, color: Color) {
        self.device.set_background_color(color);
    }
    
    pub fn toggle_bright(&mut self) {
        self.device.toggle_bright();
    }
    
    pub fn clear_screen(&mut self) {
        self.device.clear_screen();
    }
}

impl fmt::Write for Terminal {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.as_bytes().into_iter().cloned() {
            self.write_byte(byte);
        }
        
        Ok(())
    }
}
