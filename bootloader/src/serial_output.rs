use x86_64::instructions::port::Port;
use core::fmt::{self, Write};

pub const COM1: u16 = 0x3F8;

pub struct SerialPort {
    data: Port<u8>,
    interrupt_enable: Port<u8>,
    fifo_control: Port<u8>,
    line_control: Port<u8>,
    modem_control: Port<u8>,
    line_status: Port<u8>,
}

impl SerialPort {
    pub const fn new(base: u16) -> Self {
        SerialPort {
            data: Port::new(base),
            interrupt_enable: Port::new(base + 1),
            fifo_control: Port::new(base + 2),
            line_control: Port::new(base + 3),
            modem_control: Port::new(base + 4),
            line_status: Port::new(base + 5),
        }
    }

    pub fn init(&mut self) {
        unsafe {
            self.interrupt_enable.write(0x00);
            self.line_control.write(0x80);
    
            self.data.write(0x03);
            self.interrupt_enable.write(0x00);
    
            self.line_control.write(0x03);
            self.fifo_control.write(0xC7);
            self.modem_control.write(0x0B);
        }
    }

    fn is_transmit_ready(&mut self) -> bool {
        unsafe {
            self.line_status.read() & 0x20 != 0
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        while !self.is_transmit_ready() {}

        unsafe { self.data.write(byte) }
    }
}

impl Write for SerialPort {
    
    fn write_str(&mut self, s: &str) -> fmt::Result {
        
        for byte in s.bytes() {
            
            if byte == b'\n' {
                self.write_byte(b'\r');
            }

            self.write_byte(byte);
        }

        Ok(())
    }
}
