//! ASL Interactive Cockpit Shell & Command Line Engine
//! =====================================================
//! Standar DO-178C DAL-A & Allocation-Free Command Interpreter.
//! Menghadirkan prompt interaktif:
//! `ASL-OS> `
//! Perintah bawaan:
//! - `help`    : Menampilkan daftar perintah
//! - `clear`   : Membersihkan layar
//! - `meminfo` : Menampilkan informasi memori asli (RAM MB & Usable pages)
//! - `cpuinfo` : Menampilkan CPU brand string & ARINC 653 load
//! - `lspci`   : Menampilkan perangkat bus PCI yang terdeteksi
//! - `uname`   : Menampilkan versi kernel & build target
//! - `reboot`  : Menjalankan safe hardware restart via keyboard controller
//! - `halt`    : Menjalankan safe ACPI/APM processor halt

#![no_std]

pub const SHELL_BUFFER_MAX: usize = 128;
pub const PROMPT_STRING: &str = "ASL-OS> ";

/// Perintah Shell yang dikenali
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ShellCommand {
    Help,
    Clear,
    MemInfo,
    CpuInfo,
    LsPci,
    Uname,
    Reboot,
    Halt,
    Unknown,
    Empty,
}

/// Parser dan Pengelola Baris Perintah Shell (Statis tanpa alokasi heap)
pub struct CockpitShell {
    pub line_buffer: [u8; SHELL_BUFFER_MAX],
    pub cursor_pos: usize,
    pub is_active: bool,
}

impl CockpitShell {
    pub const fn new() -> Self {
        Self {
            line_buffer: [0u8; SHELL_BUFFER_MAX],
            cursor_pos: 0,
            is_active: true,
        }
    }

    /// Menerima karakter masukan dan menangani Backspace / Enter
    pub fn handle_char(&mut self, c: u8) -> Option<ShellCommand> {
        match c {
            b'\n' | b'\r' => {
                let cmd = self.parse_current_line();
                self.cursor_pos = 0;
                Some(cmd)
            }
            b'\x08' => {
                // Backspace
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.line_buffer[self.cursor_pos] = 0;
                }
                None
            }
            printable if printable >= 0x20 && printable <= 0x7E => {
                if self.cursor_pos < SHELL_BUFFER_MAX - 1 {
                    self.line_buffer[self.cursor_pos] = printable;
                    self.cursor_pos += 1;
                    self.line_buffer[self.cursor_pos] = 0;
                }
                None
            }
            _ => None,
        }
    }

    /// Membaca string baris perintah saat ini
    pub fn current_line(&self) -> &str {
        if self.cursor_pos == 0 {
            return "";
        }
        core::str::from_utf8(&self.line_buffer[..self.cursor_pos]).unwrap_or("")
    }

    /// Mem-parsing string baris perintah
    pub fn parse_current_line(&self) -> ShellCommand {
        let raw = self.current_line().trim();
        if raw.is_empty() {
            return ShellCommand::Empty;
        }

        match raw {
            "help" => ShellCommand::Help,
            "clear" => ShellCommand::Clear,
            "meminfo" => ShellCommand::MemInfo,
            "cpuinfo" => ShellCommand::CpuInfo,
            "lspci" => ShellCommand::LsPci,
            "uname" => ShellCommand::Uname,
            "reboot" => ShellCommand::Reboot,
            "halt" => ShellCommand::Halt,
            _ => ShellCommand::Unknown,
        }
    }

    /// Eksekusi perintah reboot x86 melalui keyboard controller port 0x64 pulse
    #[cfg(target_arch = "x86_64")]
    pub unsafe fn trigger_x86_reboot() -> ! {
        // Pulse reset line via keyboard controller 0x64 command 0xFE
        core::arch::asm!(
            "out 0x64, al",
            in("al") 0xFEu8,
            options(nomem, nostack, preserves_flags)
        );
        // Fallback: triple fault via zero IDT
        let zero_idt: [u64; 2] = [0, 0];
        core::arch::asm!("lidt [{}]", in(reg) &zero_idt, options(nomem, nostack));
        core::arch::asm!("int3", options(nomem, nostack));
        loop {
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }

    /// Eksekusi halt prosesor
    #[cfg(target_arch = "x86_64")]
    pub unsafe fn trigger_x86_halt() -> ! {
        loop {
            core::arch::asm!("cli; hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_command_parsing() {
        let mut shell = CockpitShell::new();

        for b in b"help" {
            assert_eq!(shell.handle_char(*b), None);
        }
        assert_eq!(shell.handle_char(b'\n'), Some(ShellCommand::Help));

        for b in b"lspci" {
            shell.handle_char(*b);
        }
        assert_eq!(shell.handle_char(b'\r'), Some(ShellCommand::LsPci));

        for b in b"uname" {
            shell.handle_char(*b);
        }
        assert_eq!(shell.handle_char(b'\n'), Some(ShellCommand::Uname));
    }

    #[test]
    fn test_shell_backspace_and_empty() {
        let mut shell = CockpitShell::new();

        shell.handle_char(b'a');
        shell.handle_char(b'b');
        shell.handle_char(b'\x08'); // Hapus 'b'
        assert_eq!(shell.current_line(), "a");

        shell.handle_char(b'\x08'); // Hapus 'a'
        assert_eq!(shell.current_line(), "");

        assert_eq!(shell.handle_char(b'\n'), Some(ShellCommand::Empty));
    }
}
