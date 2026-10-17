//! ASL PS/2 Keyboard Driver & Set 1 Scan-Code Decoder
//! ===================================================
//! Standar DO-178C DAL-A & Bounded Ring Buffer Input Processing.
//! Mendukung:
//! - Membaca status I/O port 0x64 (Output Buffer Full bit 0)
//! - Membaca data I/O port 0x60
//! - Scan-Code Set 1 (Make code & Break code)
//! - Tombol Modifiers: Left Shift, Right Shift, Caps Lock
//! - Printable ASCII, Enter, Backspace, Tab, Space
//! - Bounded Ring Buffer anti-overflow dan anti-hanging polling loops

#![no_std]

pub const PS2_DATA_PORT: u16 = 0x60;
pub const PS2_STATUS_PORT: u16 = 0x64;

pub const PS2_STATUS_OUTPUT_BUFFER_FULL: u8 = 0x01;
pub const PS2_STATUS_INPUT_BUFFER_FULL: u8 = 0x02;

/// Kapasitas ring buffer keyboard
pub const KEYBOARD_BUFFER_CAPACITY: usize = 64;

/// Bounded Ring Buffer untuk antrean karakter ASCII
pub struct KeyboardRingBuffer {
    buffer: [u8; KEYBOARD_BUFFER_CAPACITY],
    head: usize,
    tail: usize,
    count: usize,
}

impl KeyboardRingBuffer {
    pub const fn new() -> Self {
        Self {
            buffer: [0u8; KEYBOARD_BUFFER_CAPACITY],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    pub fn push(&mut self, byte: u8) -> bool {
        if self.count >= KEYBOARD_BUFFER_CAPACITY {
            return false; // Bounded: Tolak jika penuh (anti memory overflow)
        }
        self.buffer[self.tail] = byte;
        self.tail = (self.tail + 1) % KEYBOARD_BUFFER_CAPACITY;
        self.count += 1;
        true
    }

    pub fn pop(&mut self) -> Option<u8> {
        if self.count == 0 {
            return None;
        }
        let byte = self.buffer[self.head];
        self.head = (self.head + 1) % KEYBOARD_BUFFER_CAPACITY;
        self.count -= 1;
        Some(byte)
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn len(&self) -> usize {
        self.count
    }
}

/// State modifier keyboard (Shift & Caps Lock)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ModifierState {
    pub left_shift: bool,
    pub right_shift: bool,
    pub caps_lock: bool,
}

impl ModifierState {
    pub const fn new() -> Self {
        Self {
            left_shift: false,
            right_shift: false,
            caps_lock: false,
        }
    }

    pub fn is_shift_active(&self) -> bool {
        self.left_shift || self.right_shift
    }
}

/// Driver PS/2 Keyboard Utama
pub struct Ps2Keyboard {
    pub modifiers: ModifierState,
    pub ring_buffer: KeyboardRingBuffer,
    pub extended_scancode: bool,
}

impl Ps2Keyboard {
    pub const fn new() -> Self {
        Self {
            modifiers: ModifierState::new(),
            ring_buffer: KeyboardRingBuffer::new(),
            extended_scancode: false,
        }
    }

    /// Menerjemahkan scan-code Set 1 ke karakter ASCII
    pub fn process_scancode(&mut self, scancode: u8) -> Option<u8> {
        if scancode == 0xE0 {
            self.extended_scancode = true;
            return None;
        }

        if self.extended_scancode {
            self.extended_scancode = false;
            // Abaikan sementara tombol navigasi extended (arrow/page keys)
            return None;
        }

        let is_break = (scancode & 0x80) != 0;
        let make_code = scancode & 0x7F;

        if is_break {
            // Key release
            match make_code {
                0x2A => self.modifiers.left_shift = false,
                0x36 => self.modifiers.right_shift = false,
                _ => {}
            }
            None
        } else {
            // Key press
            match make_code {
                0x2A => {
                    self.modifiers.left_shift = true;
                    None
                }
                0x36 => {
                    self.modifiers.right_shift = true;
                    None
                }
                0x3A => {
                    self.modifiers.caps_lock = !self.modifiers.caps_lock;
                    None
                }
                _ => {
                    let ascii = self.map_set1_to_ascii(make_code);
                    if let Some(c) = ascii {
                        self.ring_buffer.push(c);
                    }
                    ascii
                }
            }
        }
    }

    fn map_set1_to_ascii(&self, make_code: u8) -> Option<u8> {
        let shift = self.modifiers.is_shift_active();
        let caps = self.modifiers.caps_lock;
        let letters_uppercase = shift ^ caps;

        let char_opt = match make_code {
            0x01 => None, // Escape
            0x02 => Some(if shift { b'!' } else { b'1' }),
            0x03 => Some(if shift { b'@' } else { b'2' }),
            0x04 => Some(if shift { b'#' } else { b'3' }),
            0x05 => Some(if shift { b'$' } else { b'4' }),
            0x06 => Some(if shift { b'%' } else { b'5' }),
            0x07 => Some(if shift { b'^' } else { b'6' }),
            0x08 => Some(if shift { b'&' } else { b'7' }),
            0x09 => Some(if shift { b'*' } else { b'8' }),
            0x0A => Some(if shift { b'(' } else { b'9' }),
            0x0B => Some(if shift { b')' } else { b'0' }),
            0x0C => Some(if shift { b'_' } else { b'-' }),
            0x0D => Some(if shift { b'+' } else { b'=' }),
            0x0E => Some(b'\x08'), // Backspace
            0x0F => Some(b'\t'),   // Tab
            0x10 => Some(if letters_uppercase { b'Q' } else { b'q' }),
            0x11 => Some(if letters_uppercase { b'W' } else { b'w' }),
            0x12 => Some(if letters_uppercase { b'E' } else { b'e' }),
            0x13 => Some(if letters_uppercase { b'R' } else { b'r' }),
            0x14 => Some(if letters_uppercase { b'T' } else { b't' }),
            0x15 => Some(if letters_uppercase { b'Y' } else { b'y' }),
            0x16 => Some(if letters_uppercase { b'U' } else { b'u' }),
            0x17 => Some(if letters_uppercase { b'I' } else { b'i' }),
            0x18 => Some(if letters_uppercase { b'O' } else { b'o' }),
            0x19 => Some(if letters_uppercase { b'P' } else { b'p' }),
            0x1A => Some(if shift { b'{' } else { b'[' }),
            0x1B => Some(if shift { b'}' } else { b']' }),
            0x1C => Some(b'\n'), // Enter
            0x1E => Some(if letters_uppercase { b'A' } else { b'a' }),
            0x1F => Some(if letters_uppercase { b'S' } else { b's' }),
            0x20 => Some(if letters_uppercase { b'D' } else { b'd' }),
            0x21 => Some(if letters_uppercase { b'F' } else { b'f' }),
            0x22 => Some(if letters_uppercase { b'G' } else { b'g' }),
            0x23 => Some(if letters_uppercase { b'H' } else { b'h' }),
            0x24 => Some(if letters_uppercase { b'J' } else { b'j' }),
            0x25 => Some(if letters_uppercase { b'K' } else { b'k' }),
            0x26 => Some(if letters_uppercase { b'L' } else { b'l' }),
            0x27 => Some(if shift { b':' } else { b';' }),
            0x28 => Some(if shift { b'"' } else { b'\'' }),
            0x29 => Some(if shift { b'~' } else { b'`' }),
            0x2B => Some(if shift { b'|' } else { b'\\' }),
            0x2C => Some(if letters_uppercase { b'Z' } else { b'z' }),
            0x2D => Some(if letters_uppercase { b'X' } else { b'x' }),
            0x2E => Some(if letters_uppercase { b'C' } else { b'c' }),
            0x2F => Some(if letters_uppercase { b'V' } else { b'v' }),
            0x30 => Some(if letters_uppercase { b'B' } else { b'b' }),
            0x31 => Some(if letters_uppercase { b'N' } else { b'n' }),
            0x32 => Some(if letters_uppercase { b'M' } else { b'm' }),
            0x33 => Some(if shift { b'<' } else { b',' }),
            0x34 => Some(if shift { b'>' } else { b'.' }),
            0x35 => Some(if shift { b'?' } else { b'/' }),
            0x39 => Some(b' '), // Space
            _ => None,
        };

        char_opt
    }

    /// Polling hardware non-blocking: membaca data port 0x60 hanya jika port 0x64 bit 0 menyala
    #[cfg(target_arch = "x86_64")]
    pub unsafe fn poll_hardware(&mut self) -> Option<u8> {
        let status: u8;
        core::arch::asm!(
            "in al, dx",
            out("al") status,
            in("dx") PS2_STATUS_PORT,
            options(nomem, nostack, preserves_flags)
        );

        if (status & PS2_STATUS_OUTPUT_BUFFER_FULL) != 0 {
            let scancode: u8;
            core::arch::asm!(
                "in al, dx",
                out("al") scancode,
                in("dx") PS2_DATA_PORT,
                options(nomem, nostack, preserves_flags)
            );
            return self.process_scancode(scancode);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ps2_ring_buffer_fifo_and_overflow() {
        let mut buf = KeyboardRingBuffer::new();
        assert!(buf.is_empty());

        for i in 0..KEYBOARD_BUFFER_CAPACITY {
            assert!(buf.push(i as u8));
        }

        // Harus tolak jika penuh
        assert!(!buf.push(0xFF));
        assert_eq!(buf.len(), KEYBOARD_BUFFER_CAPACITY);

        for i in 0..KEYBOARD_BUFFER_CAPACITY {
            assert_eq!(buf.pop(), Some(i as u8));
        }

        assert_eq!(buf.pop(), None);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_ps2_scancode_letters_and_modifiers() {
        let mut kbd = Ps2Keyboard::new();

        // Tekan huruf 'a' (make code 0x1E)
        assert_eq!(kbd.process_scancode(0x1E), Some(b'a'));

        // Lepas huruf 'a' (break code 0x9E) -> None
        assert_eq!(kbd.process_scancode(0x9E), None);

        // Tekan Left Shift (make code 0x2A)
        assert_eq!(kbd.process_scancode(0x2A), None);
        assert!(kbd.modifiers.left_shift);

        // Tekan huruf 'a' saat shift -> 'A'
        assert_eq!(kbd.process_scancode(0x1E), Some(b'A'));

        // Lepas Left Shift (break code 0xAA)
        assert_eq!(kbd.process_scancode(0xAA), None);
        assert!(!kbd.modifiers.left_shift);

        // Tekan Enter (make code 0x1C)
        assert_eq!(kbd.process_scancode(0x1C), Some(b'\n'));

        // Tekan Backspace (make code 0x0E)
        assert_eq!(kbd.process_scancode(0x0E), Some(b'\x08'));
    }

    #[test]
    fn test_ps2_caps_lock_and_wraparound() {
        let mut kbd = Ps2Keyboard::new();

        // Caps Lock toggle on (make code 0x3A)
        assert_eq!(kbd.process_scancode(0x3A), None);
        assert!(kbd.modifiers.caps_lock);
        // Huruf 'b' menjadi 'B'
        assert_eq!(kbd.process_scancode(0x30), Some(b'B'));

        // Tekan Shift saat Caps Lock aktif -> XOR -> huruf kecil 'b'
        assert_eq!(kbd.process_scancode(0x2A), None);
        assert_eq!(kbd.process_scancode(0x30), Some(b'b'));

        // Lepas Shift
        assert_eq!(kbd.process_scancode(0xAA), None);
        // Toggle Caps Lock off
        assert_eq!(kbd.process_scancode(0x3A), None);
        assert!(!kbd.modifiers.caps_lock);
        assert_eq!(kbd.process_scancode(0x30), Some(b'b'));

        // Test Ring Buffer Wraparound
        let mut buf = KeyboardRingBuffer::new();
        for i in 0..50 {
            assert!(buf.push(i));
        }
        for _ in 0..30 {
            buf.pop();
        }
        // Sekarang head di 30, tail di 50. Tambahkan 40 elemen lagi untuk memicu wraparound
        for i in 50..90 {
            assert!(buf.push(i));
        }
        assert_eq!(buf.len(), 60);
    }
}
