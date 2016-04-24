//! Rust interface to the VGA text buffer system.
use core::nonzero::NonZero;
use rlibc;

const VGA_TEXT_BUFFER : *mut u16 = 0xb8000 as *mut u16;
const VGA_TEXT_HEIGHT : u8 = 25;
const VGA_TEXT_WIDTH : u8 = 80;
const VGA_BUFFER_SIZE : usize = (VGA_TEXT_HEIGHT as usize) * (VGA_TEXT_WIDTH as usize);

/// A VGA buffer, defined to be 80 columns wide
/// and 25 columns tall. The pointer contained in
/// this struct must point to a buffer that is
/// an array of 80*25 u16s. This may point to the
/// actual VGA buffer, or it may point to buffers
/// that the kernel allocates to multiplex access to the
/// VGA buffer.
struct Buffer {
    pub ptr: NonZero<*mut u16>
}

impl Buffer {
    pub const unsafe fn new(ptr: *mut u16) -> Buffer {
        Buffer {
            ptr: NonZero::new(ptr)
        }
    }
    
    pub const unsafe fn new_physical_vga_buffer() -> Buffer {
        Buffer {
            ptr: NonZero::new(VGA_TEXT_BUFFER)
        }
    }
    
    pub fn is_physical_vga_buffer(&self) -> bool {
        *self.ptr == VGA_TEXT_BUFFER
    }
    
    pub fn scroll_up(&mut self, lines: u8) {
        // we want to shift everything lines * VGA_TEXT_WIDTH to left.
        // this can be done with a memmove and a memset.
        let front_offset = (lines * VGA_TEXT_WIDTH) as usize;
        let back_offset = VGA_BUFFER_SIZE - front_offset;
        unsafe {
            let offset = VGA_TEXT_BUFFER.offset(front_offset as isize);
            rlibc::memmove(VGA_TEXT_BUFFER as *mut u8, offset as *mut u8, back_offset * 2);
            let memset_offset = VGA_TEXT_BUFFER.offset(back_offset as isize);
            rlibc::memset(memset_offset as *mut u8, 0, back_offset * 2);
        }
    }
}

/// Color represents the various colors that are available
/// to the VGA device to display.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    Black,
    Blue,
    Green,
    Cyan,
    Red,
    Magenta,
    Brown,
    LightGray
}

#[derive(Copy, Clone)]
pub struct Position {
    pub col: u8,
    pub row: u8
}

impl Position {
    pub fn as_offset(self) -> isize {
        return (self.row as isize) * (VGA_TEXT_WIDTH as isize) + (self.col as isize);
    }
    
    pub fn is_valid(self) -> bool {
        self.row < VGA_TEXT_HEIGHT && self.col < VGA_TEXT_WIDTH
    }
    
    pub const fn new(row: u8, col: u8) -> Position {
        Position {
            col: col,
            row: row
        }
    }
}

pub struct Device {
    bg_color: Color,
    fg_color: Color,
    is_bright: bool,
    buffer: Buffer
}


impl Device {
    pub const unsafe fn new_actual_vga_device() -> Device {
        Device {
            bg_color: Color::Black,
            fg_color: Color::LightGray,
            is_bright: true,
            buffer: Buffer::new_physical_vga_buffer()
        }
    }
    
    pub const unsafe fn new(ptr: *mut u16) -> Device {
        Device {
            bg_color: Color::Black,
            fg_color: Color::LightGray,
            is_bright: true,
            buffer: Buffer::new(ptr)
        }
    }
    
    pub fn is_actual_vga_device(&self) -> bool {
        self.buffer.is_physical_vga_buffer()
    }
    
    pub fn set_background_color(&mut self, color: Color) {
        self.bg_color = color;
    }
    
    pub fn set_foreground_color(&mut self, color: Color) {
        self.fg_color = color;
    }
    
    pub fn toggle_bright(&mut self) {
        self.is_bright = !self.is_bright;
    }
    
    pub fn write_byte(&mut self, byte: u8, position: Position) {
        assert!(position.is_valid());
        unsafe {
            self.write_byte_unchecked(byte, position);
        }
    }
    
    pub fn height(&self) -> u8 {
        VGA_TEXT_HEIGHT
    }
    
    pub fn width(&self) -> u8 {
        VGA_TEXT_WIDTH
    }
    
    pub unsafe fn write_byte_unchecked(&mut self, byte: u8, position: Position) {
        let offset = position.as_offset();
        let place = VGA_TEXT_BUFFER.offset(offset);
        // VGA color byte, from osdev:
        // Bit 76543210
        //     ||||||||
        //     |||||^^^-fore colour
        //     ||||^----fore colour bright bit
        //     |^^^-----back colour
        //     ^--------back colour bright bit OR enables blinking Text
        let brightness_bit = if self.is_bright { 1 } else { 0 };
        let color_byte : u8 = ((self.bg_color as u8) << 4) | brightness_bit << 3 | (self.fg_color as u8);
        *place = ((color_byte as u16) << 8) | byte as u16;
    }
    
    pub fn clear_screen(&mut self) {
        for row in 0..VGA_TEXT_HEIGHT {
            for col in 0..VGA_TEXT_WIDTH {
                self.write_byte(b' ', Position::new(row, col));
            }
        }
    }
    
    pub fn scroll_up(&mut self, lines: u8) {
        self.buffer.scroll_up(lines);
    }
}
