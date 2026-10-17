//! ASL UART COM1 Telemetry Driver
//! ==================================
//! Direct Port I/O (0x3F8) untuk pancaran telemetri milestone boot avionik M1..M6.

const COM1_PORT: u16 = 0x3F8;

#[inline(always)]
unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") val,
        options(nomem, nostack, preserves_flags)
    );
}

#[inline(always)]
unsafe fn inb(port: u16) -> u8 {
    let ret: u8;
    core::arch::asm!(
        "in al, dx",
        out("al") ret,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    ret
}

pub struct SerialPort;

impl SerialPort {
    pub const fn new() -> Self {
        Self
    }

    pub fn init(&self) {
        unsafe {
            outb(COM1_PORT + 1, 0x00); // Disable interrupts
            outb(COM1_PORT + 3, 0x80); // Enable DLAB (set baud rate divisor)
            outb(COM1_PORT + 0, 0x03); // Set divisor to 3 (38400 baud)
            outb(COM1_PORT + 1, 0x00);
            outb(COM1_PORT + 3, 0x03); // 8 bits, no parity, one stop bit
            outb(COM1_PORT + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
            outb(COM1_PORT + 4, 0x0B); // IRQs enabled, RTS/DSR set
        }
    }

    pub fn is_transmit_empty(&self) -> bool {
        unsafe { (inb(COM1_PORT + 5) & 0x20) != 0 }
    }

    pub fn write_byte(&self, b: u8) {
        while !self.is_transmit_empty() {
            core::hint::spin_loop();
        }
        unsafe {
            outb(COM1_PORT, b);
        }
    }

    pub fn write_str(&self, s: &str) {
        for b in s.bytes() {
            if b == b'\n' {
                self.write_byte(b'\r');
            }
            self.write_byte(b);
        }
    }

    pub fn write_u64(&self, mut n: u64) {
        if n == 0 {
            self.write_byte(b'0');
            return;
        }
        let mut buf = [0u8; 20];
        let mut i = 0;
        while n > 0 {
            buf[i] = b'0' + (n % 10) as u8;
            n /= 10;
            i += 1;
        }
        while i > 0 {
            i -= 1;
            self.write_byte(buf[i]);
        }
    }
}
