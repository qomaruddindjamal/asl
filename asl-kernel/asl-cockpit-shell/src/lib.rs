//! ASL Interactive Cockpit Shell & Command Line Engine
//! =====================================================
//! Standar DO-178C DAL-A & Allocation-Free Command Interpreter.
//! Menghadirkan prompt interaktif:
//! `ASL-KERNEL> ` (satu-sumber: konstanta `SHELL_PROMPT` di berkas ini)
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

/// Identitas rilis kernel: SATU-SUMBER untuk lima situs tampil
/// (logo, VGA, framebuffer, serial, uname) di asl-kernel-core.
pub const KERNEL_VERSION: &str = "v1.0.2";

/// Prompt shell: SATU-SUMBER untuk seluruh situs tampil. Tanpa posisi folder.
pub const SHELL_PROMPT: &str = "ASL-KERNEL> ";

/// Perintah Shell yang dikenali
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ShellCommand {
    Help,
    Clear,
    MemInfo,
    CpuInfo,
    LsPci,
    LsBlk,
    Fdisk,
    Mount,
    Uname,
    Failover,
    TrapDe,
    TrapBp,
    TrapPf,
    TrapDf,
    TestPml4,
    TestApic,
    TestTask,
    Reboot,
    Halt,
    Unknown,
    Empty,
}

/// Parser dan Pengelola Baris Perintah Shell (Statis tanpa alokasi heap)
pub struct CockpitShell {
    pub line_buffer: [u8; SHELL_BUFFER_MAX],
    /// Posisi sisip (kursor). Tetap identik dengan akhir baris selama pengguna
    /// tidak memakai tombol panah, sehingga perilaku jalur lama tidak berubah.
    pub cursor_pos: usize,
    /// Panjang baris aktif saat ini.
    pub line_len: usize,
    pub is_active: bool,
    /// Mesin sekuens escape: 0 = normal, 1 = ESC, 2 = ESC [, 3 = ESC [ <angka>.
    esc_state: u8,
    esc_digit: u8,
}

impl CockpitShell {
    pub const fn new() -> Self {
        Self {
            line_buffer: [0u8; SHELL_BUFFER_MAX],
            cursor_pos: 0,
            line_len: 0,
            is_active: true,
            esc_state: 0,
            esc_digit: 0,
        }
    }

    /// Menerima karakter masukan dan menangani Backspace / Enter.
    ///
    /// Kontrak lama dipertahankan persis: mengembalikan `Some(ShellCommand)`
    /// hanya saat ENTER, memakai parser enumerasi lama. Penyuntingan barisnya
    /// kini berbagi mesin yang sama dengan `handle_input` supaya kedua jalur
    /// tidak memiliki perilaku kursor yang berbeda.
    pub fn handle_char(&mut self, c: u8) -> Option<ShellCommand> {
        match self.edit(c) {
            Edit::Submitted => {
                let cmd = self.parse_current_line();
                self.clear_line();
                Some(cmd)
            }
            _ => None,
        }
    }

    /// Membaca string baris perintah saat ini
    pub fn current_line(&self) -> &str {
        if self.line_len == 0 {
            return "";
        }
        core::str::from_utf8(&self.line_buffer[..self.line_len]).unwrap_or("")
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
            "lsblk" => ShellCommand::LsBlk,
            "fdisk" => ShellCommand::Fdisk,
            "mount" => ShellCommand::Mount,
            "uname" => ShellCommand::Uname,
            "failover" => ShellCommand::Failover,
            "trap_de" => ShellCommand::TrapDe,
            "trap_bp" => ShellCommand::TrapBp,
            "trap_pf" => ShellCommand::TrapPf,
            "trap_df" => ShellCommand::TrapDf,
            "test_pml4" => ShellCommand::TestPml4,
            "test_apic" => ShellCommand::TestApic,
            "test_task" => ShellCommand::TestTask,
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

    /// Eksekusi halt prosesor x86_64
    #[cfg(target_arch = "x86_64")]
    pub unsafe fn trigger_x86_halt() -> ! {
        loop {
            core::arch::asm!("cli; hlt", options(nomem, nostack, preserves_flags));
        }
    }

    /// Eksekusi reboot AArch64 via PSCI SYSTEM_RESET (HVC / SMC fid 0x84000009)
    #[cfg(target_arch = "aarch64")]
    pub unsafe fn trigger_aarch64_reboot() -> ! {
        // PSCI SYSTEM_RESET: fn_id 0x84000009
        core::arch::asm!(
            "mov x0, #0x0009",
            "movk x0, #0x8400, lsl #16",
            "hvc #0",
            options(nomem, nostack)
        );
        // Fallback: spinning wfi loop
        loop {
            core::arch::asm!("wfi", options(nomem, nostack, preserves_flags));
        }
    }

    /// Eksekusi halt prosesor AArch64 via spinning wfi loop
    #[cfg(target_arch = "aarch64")]
    pub unsafe fn trigger_aarch64_halt() -> ! {
        loop {
            core::arch::asm!("wfi", options(nomem, nostack, preserves_flags));
        }
    }
}

// ============================================================================
// KERANGKA PERINTAH KANONIK (misi v1.0.2 Bagian 1.2, 1.3, 1.5)
// ----------------------------------------------------------------------------
// Kontrak: SATU trait self-describing + SATU registry statis + `help` yang
// dibangun dari registry (bukan dari teks tetap). Sintaks pada registry adalah
// janji abadi (Bagian 1.2): pesan galat dan dokumen PANDUAN dibaca dari sini,
// dan uji sinkron dokumen (P5) membandingkan berkas Markdown terhadap registry.
//
// Perintah lama (`meminfo`, `cpuinfo`, `lsblk`, ...) TIDAK diubah dan TIDAK
// dihapus: jalur dispatch lama tetap utuh di asl-kernel-core. Parser sekarang
// mencoba registry kanonik lebih dulu, lalu jatuh ke enumerasi lama. Dual-path
// ini adalah keputusan sadar (Bagian 6: refactor berisiko pada perintah lama =
// biarkan jalur ganda + catat), dicatat di laporan final.
// ============================================================================

/// Jumlah perintah kanonik yang dijanjikan misi v1.0.2 Bagian 1.2.
pub const CANONICAL_COMMAND_COUNT: usize = 11;

/// Jumlah maksimum argumen yang bisa ditangkap satu baris perintah.
pub const MAX_ARGS: usize = 2;

/// Panjang maksimum satu argumen (nama/path/teks) dalam byte.
pub const MAX_ARG_LEN: usize = 64;

/// Lebar kolom nama pada keluaran `help` (panjang `create-folder` + 1).
pub const HELP_NAME_COLUMN: usize = 14;

/// Kelompok perintah untuk menyusun keluaran `help`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CommandCategory {
    Help,
    Workspace,
    File,
    Status,
    Display,
}

impl CommandCategory {
    /// Judul seksi berbahasa Indonesia (G10).
    pub const fn label(self) -> &'static str {
        match self {
            CommandCategory::Help => "Bantuan",
            CommandCategory::Workspace => "Ruang kerja",
            CommandCategory::File => "Berkas dan direktori (RAMDISK volatil)",
            CommandCategory::Status => "Status dan identitas",
            CommandCategory::Display => "Tampilan",
        }
    }
}

/// Identitas perintah kanonik hasil resolusi parser.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CanonicalId {
    Help,
    Clear,
    Where,
    List,
    Open,
    CreateFolder,
    CreateFile,
    WriteFile,
    DeleteFile,
    DeleteFolder,
    Status,
}

/// Kontrak satu perintah shell yang menjelaskan dirinya sendiri.
///
/// Semua rujukan berumur `'static` karena registry disusun dari konstanta:
/// nol alokasi heap, nol `alloc`, target `no_std` (G1).
pub trait ShellEntry: Sync {
    fn id(&self) -> CanonicalId;
    fn name(&self) -> &'static str;
    /// Nama samar parser (Bagian 1.3). Kosong bila perintah tidak ber-alias.
    fn aliases(&self) -> &'static [&'static str];
    /// Baris sintaks yang menjadi janji abadi, mis. `write-file <nama> <teks>`.
    fn syntax(&self) -> &'static str;
    /// Ringkasan satu baris dalam bahasa Indonesia (G10).
    fn summary(&self) -> &'static str;
    fn category(&self) -> CommandCategory;
    fn min_args(&self) -> usize;
    fn max_args(&self) -> usize;
    /// `true` bila argumen terakhir harus menangkap SELURUH sisa baris apa
    /// adanya (teks bebas spasi), bukan satu kata saja.
    fn rest_is_one_arg(&self) -> bool {
        false
    }
}

#[derive(Debug, Copy, Clone)]
pub struct HelpCmd;
impl ShellEntry for HelpCmd {
    fn id(&self) -> CanonicalId {
        CanonicalId::Help
    }
    fn name(&self) -> &'static str {
        "help"
    }
    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }
    fn syntax(&self) -> &'static str {
        "help [perintah]"
    }
    fn summary(&self) -> &'static str {
        "Daftar perintah yang disusun langsung dari registry; dengan argumen menampilkan satu perintah."
    }
    fn category(&self) -> CommandCategory {
        CommandCategory::Help
    }
    fn min_args(&self) -> usize {
        0
    }
    fn max_args(&self) -> usize {
        1
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ClearCmd;
impl ShellEntry for ClearCmd {
    fn id(&self) -> CanonicalId {
        CanonicalId::Clear
    }
    fn name(&self) -> &'static str {
        "clear"
    }
    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }
    fn syntax(&self) -> &'static str {
        "clear"
    }
    fn summary(&self) -> &'static str {
        "Bersihkan area tampilan konsol framebuffer dan kirim sekuens bersih layar ke serial."
    }
    fn category(&self) -> CommandCategory {
        CommandCategory::Display
    }
    fn min_args(&self) -> usize {
        0
    }
    fn max_args(&self) -> usize {
        0
    }
}

#[derive(Debug, Copy, Clone)]
pub struct WhereCmd;
impl ShellEntry for WhereCmd {
    fn id(&self) -> CanonicalId {
        CanonicalId::Where
    }
    fn name(&self) -> &'static str {
        "where"
    }
    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }
    fn syntax(&self) -> &'static str {
        "where"
    }
    fn summary(&self) -> &'static str {
        "Tampilkan direktori kerja saat ini; prompt sengaja polos tanpa posisi folder."
    }
    fn category(&self) -> CommandCategory {
        CommandCategory::Workspace
    }
    fn min_args(&self) -> usize {
        0
    }
    fn max_args(&self) -> usize {
        0
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ListCmd;
impl ShellEntry for ListCmd {
    fn id(&self) -> CanonicalId {
        CanonicalId::List
    }
    fn name(&self) -> &'static str {
        "list"
    }
    fn aliases(&self) -> &'static [&'static str] {
        &["ls", "dir"]
    }
    fn syntax(&self) -> &'static str {
        "list [jalan]"
    }
    fn summary(&self) -> &'static str {
        "Daftar isi direktori pada RAMDISK volatil; tanpa argumen memakai direktori kerja."
    }
    fn category(&self) -> CommandCategory {
        CommandCategory::Workspace
    }
    fn min_args(&self) -> usize {
        0
    }
    fn max_args(&self) -> usize {
        1
    }
}

#[derive(Debug, Copy, Clone)]
pub struct OpenCmd;
impl ShellEntry for OpenCmd {
    fn id(&self) -> CanonicalId {
        CanonicalId::Open
    }
    fn name(&self) -> &'static str {
        "open"
    }
    fn aliases(&self) -> &'static [&'static str] {
        &["cd", "cat", "type", "read-file", "open-file"]
    }
    fn syntax(&self) -> &'static str {
        "open <jalan>"
    }
    fn summary(&self) -> &'static str {
        "Masuk ke direktori bila tujuannya direktori, atau tampilkan isi berkas bila tujuannya berkas."
    }
    fn category(&self) -> CommandCategory {
        CommandCategory::Workspace
    }
    fn min_args(&self) -> usize {
        1
    }
    fn max_args(&self) -> usize {
        1
    }
}

#[derive(Debug, Copy, Clone)]
pub struct CreateFolderCmd;
impl ShellEntry for CreateFolderCmd {
    fn id(&self) -> CanonicalId {
        CanonicalId::CreateFolder
    }
    fn name(&self) -> &'static str {
        "create-folder"
    }
    fn aliases(&self) -> &'static [&'static str] {
        &["mkdir"]
    }
    fn syntax(&self) -> &'static str {
        "create-folder <nama>"
    }
    fn summary(&self) -> &'static str {
        "Buat direktori baru di dalam direktori kerja saat ini."
    }
    fn category(&self) -> CommandCategory {
        CommandCategory::File
    }
    fn min_args(&self) -> usize {
        1
    }
    fn max_args(&self) -> usize {
        1
    }
}

#[derive(Debug, Copy, Clone)]
pub struct CreateFileCmd;
impl ShellEntry for CreateFileCmd {
    fn id(&self) -> CanonicalId {
        CanonicalId::CreateFile
    }
    fn name(&self) -> &'static str {
        "create-file"
    }
    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }
    fn syntax(&self) -> &'static str {
        "create-file <nama>"
    }
    fn summary(&self) -> &'static str {
        "Buat berkas kosong di dalam direktori kerja saat ini."
    }
    fn category(&self) -> CommandCategory {
        CommandCategory::File
    }
    fn min_args(&self) -> usize {
        1
    }
    fn max_args(&self) -> usize {
        1
    }
}

#[derive(Debug, Copy, Clone)]
pub struct WriteFileCmd;
impl ShellEntry for WriteFileCmd {
    fn id(&self) -> CanonicalId {
        CanonicalId::WriteFile
    }
    fn name(&self) -> &'static str {
        "write-file"
    }
    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }
    fn syntax(&self) -> &'static str {
        "write-file <nama> <teks>"
    }
    fn summary(&self) -> &'static str {
        "Tulis satu baris teks ke berkas dan memotong sisa isi lama; isi hidup selama RAMDISK."
    }
    fn category(&self) -> CommandCategory {
        CommandCategory::File
    }
    fn min_args(&self) -> usize {
        1
    }
    fn max_args(&self) -> usize {
        2
    }
    fn rest_is_one_arg(&self) -> bool {
        true
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DeleteFileCmd;
impl ShellEntry for DeleteFileCmd {
    fn id(&self) -> CanonicalId {
        CanonicalId::DeleteFile
    }
    fn name(&self) -> &'static str {
        "delete-file"
    }
    fn aliases(&self) -> &'static [&'static str] {
        &["rm"]
    }
    fn syntax(&self) -> &'static str {
        "delete-file <nama>"
    }
    fn summary(&self) -> &'static str {
        "Hapus satu berkas dari direktori kerja saat ini; slot datanya ternol dan bisa dipakai ulang."
    }
    fn category(&self) -> CommandCategory {
        CommandCategory::File
    }
    fn min_args(&self) -> usize {
        1
    }
    fn max_args(&self) -> usize {
        1
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DeleteFolderCmd;
impl ShellEntry for DeleteFolderCmd {
    fn id(&self) -> CanonicalId {
        CanonicalId::DeleteFolder
    }
    fn name(&self) -> &'static str {
        "delete-folder"
    }
    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }
    fn syntax(&self) -> &'static str {
        "delete-folder <nama>"
    }
    fn summary(&self) -> &'static str {
        "Hapus satu direktori kosong dari direktori kerja saat ini; direktori berisi ditolak."
    }
    fn category(&self) -> CommandCategory {
        CommandCategory::File
    }
    fn min_args(&self) -> usize {
        1
    }
    fn max_args(&self) -> usize {
        1
    }
}

#[derive(Debug, Copy, Clone)]
pub struct StatusCmd;
impl ShellEntry for StatusCmd {
    fn id(&self) -> CanonicalId {
        CanonicalId::Status
    }
    fn name(&self) -> &'static str {
        "status"
    }
    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }
    fn syntax(&self) -> &'static str {
        "status"
    }
    fn summary(&self) -> &'static str {
        "Ringkas identitas rilis, berkas kerja shell, direktori kerja, dan pemakaian RAMDISK."
    }
    fn category(&self) -> CommandCategory {
        CommandCategory::Status
    }
    fn min_args(&self) -> usize {
        0
    }
    fn max_args(&self) -> usize {
        0
    }
}

/// Registry statis: SATU-SUMBER daftar perintah kanonik, urutan tampil pada `help`.
pub static REGISTRY: [&dyn ShellEntry; CANONICAL_COMMAND_COUNT] = [
    &HelpCmd,
    &ClearCmd,
    &WhereCmd,
    &ListCmd,
    &OpenCmd,
    &CreateFolderCmd,
    &CreateFileCmd,
    &WriteFileCmd,
    &DeleteFileCmd,
    &DeleteFolderCmd,
    &StatusCmd,
];

/// Cari entri registry lewat nama kanonik atau nama samar (Bagian 1.3).
pub fn find_entry(token: &str) -> Option<&'static dyn ShellEntry> {
    let mut i = 0;
    while i < REGISTRY.len() {
        let e = REGISTRY[i];
        if e.name() == token {
            return Some(e);
        }
        let mut a = 0;
        while a < e.aliases().len() {
            if e.aliases()[a] == token {
                return Some(e);
            }
            a += 1;
        }
        i += 1;
    }
    None
}

/// Satu perintah kanonik yang sudah terurai beserta argumennya.
///
/// Argumen disalin dari penyangga baris ke penyangga tetap milik `Invocation`
/// supaya pemanggil tetap bisa memegang argumen setelah baris dibersihkan.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Invocation {
    pub id: CanonicalId,
    pub name: &'static str,
    pub argc: usize,
    pub arg_len: [usize; MAX_ARGS],
    args: [[u8; MAX_ARG_LEN]; MAX_ARGS],
}

impl Invocation {
    /// Isi argumen ke-`i` sebagai `&str`; `""` bila indeks di luar argc.
    pub fn arg(&self, i: usize) -> &str {
        if i >= self.argc || i >= MAX_ARGS {
            return "";
        }
        core::str::from_utf8(&self.args[i][..self.arg_len[i]]).unwrap_or("")
    }

    /// Semua argumen yang tersedia, untuk pemeriksaan di sisi jembatan.
    pub fn args(&self) -> [&str; MAX_ARGS] {
        [self.arg(0), self.arg(1)]
    }

    /// Entri registry yang melahirkan panggilan ini.
    pub fn entry(&self) -> &'static dyn ShellEntry {
        let mut i = 0;
        while i < REGISTRY.len() {
            if REGISTRY[i].id() == self.id {
                return REGISTRY[i];
            }
            i += 1;
        }
        REGISTRY[0]
    }
}

/// Hasil satu baris yang selesai diketik.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LineOutcome {
    /// Token pertama tidak ada di registry: serahkan ke jalur perintah lama.
    NotCanonical,
    /// Perintah kanonik siap dieksekusi.
    Ready(Invocation),
    /// Miris perintah kanonik tapi jumlah argumen tidak sesuai janji sintaks.
    BadArity(&'static str),
}

/// Salin satu argumen ke penyangga tetap `Invocation`.
fn push_arg(inv: &mut Invocation, tok: &str) {
    if inv.argc >= MAX_ARGS {
        return;
    }
    let bytes = tok.as_bytes();
    let n = if bytes.len() > MAX_ARG_LEN {
        MAX_ARG_LEN
    } else {
        bytes.len()
    };
    let mut i = 0;
    while i < n {
        inv.args[inv.argc][i] = bytes[i];
        i += 1;
    }
    inv.arg_len[inv.argc] = n;
    inv.argc += 1;
}

/// Uraikan satu baris menjadi panggilan kanonik, atau nyatakan bukan kanonik.
///
/// Pemisahan memakai spasi kosong (satu atau lebih). Nilai `NotCanonical`
/// sengaja bukan `Unknown`: perintah lama tetap punya kesempatan dicocokkan.
pub fn parse_line(line: &str) -> LineOutcome {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return LineOutcome::NotCanonical;
    }
    let mut tokens = trimmed.split(' ');
    let first = match tokens.next() {
        Some(t) => t,
        None => return LineOutcome::NotCanonical,
    };
    let entry = match find_entry(first) {
        Some(e) => e,
        None => return LineOutcome::NotCanonical,
    };

    let mut inv = Invocation {
        id: entry.id(),
        name: entry.name(),
        argc: 0,
        arg_len: [0usize; MAX_ARGS],
        args: [[0u8; MAX_ARG_LEN]; MAX_ARGS],
    };
    let mut extra_words = 0usize;
    if entry.rest_is_one_arg() {
        // Mode teks bebas: sisa baris setelah nama argumen pertama = satu argumen.
        let after_name = trimmed[first.len()..].trim_start();
        if !after_name.is_empty() {
            match after_name.find(' ') {
                Some(p) => {
                    push_arg(&mut inv, &after_name[..p]);
                    let rest = after_name[p..].trim_start();
                    if rest.is_empty() {
                        extra_words = 1;
                    } else {
                        push_arg(&mut inv, rest);
                        extra_words = 2;
                    }
                }
                None => {
                    push_arg(&mut inv, after_name);
                    extra_words = 1;
                }
            }
        }
    } else {
        for tok in tokens {
            if tok.is_empty() {
                continue;
            }
            extra_words += 1;
            push_arg(&mut inv, tok);
        }
    }

    if inv.argc < entry.min_args() {
        return LineOutcome::BadArity(entry.syntax());
    }
    if extra_words > entry.max_args() {
        return LineOutcome::BadArity(entry.syntax());
    }
    LineOutcome::Ready(inv)
}

/// Bantalan spasi untuk kolom nama `help`.
const HELP_PAD: &str = "                  ";

/// Bangun keluaran `help` dari registry. `sink` menerima potongan teks,
/// sehingga jembatan pemanggil cukup menyalinnya ke serial dan framebuffer.
pub fn write_help<W: FnMut(&str)>(mut sink: W) {
    sink("Perintah kanonik Cockpit Shell (disusun dari registry, satu-sumber):\n");
    let categories = [
        CommandCategory::Help,
        CommandCategory::Display,
        CommandCategory::Workspace,
        CommandCategory::File,
        CommandCategory::Status,
    ];
    let mut k = 0;
    while k < categories.len() {
        let wanted = categories[k];
        let mut section_shown = false;
        let mut i = 0;
        while i < REGISTRY.len() {
            let e = REGISTRY[i];
            if e.category() == wanted {
                if !section_shown {
                    sink("  ");
                    sink(e.category().label());
                    sink(":\n");
                    section_shown = true;
                }
                sink("    ");
                sink(e.name());
                if e.name().len() < HELP_NAME_COLUMN {
                    let mut take = HELP_NAME_COLUMN - e.name().len();
                    if take > HELP_PAD.len() {
                        take = HELP_PAD.len();
                    }
                    sink(&HELP_PAD[..take]);
                } else {
                    sink(" ");
                }
                sink(e.syntax());
                sink("  --  ");
                sink(e.summary());
                sink("\n");
                if !e.aliases().is_empty() {
                    sink("      alias: ");
                    let mut a = 0;
                    while a < e.aliases().len() {
                        sink(e.aliases()[a]);
                        if a + 1 < e.aliases().len() {
                            sink(", ");
                        }
                        a += 1;
                    }
                    sink("\n");
                }
            }
            i += 1;
        }
        k += 1;
    }
    sink("Perintah lama (meminfo, cpuinfo, uname, lspci, lsblk, fdisk, mount, failover, reboot, halt, test_*, trap_*) tetap dikenali.\n");
    sink("Detail satu perintah: help <perintah>\n");
}

/// Bangun detail satu perintah untuk `help <perintah>`; `false` bila tak dikenal.
pub fn write_help_for<W: FnMut(&str)>(token: &str, mut sink: W) -> bool {
    match find_entry(token) {
        Some(e) => {
            sink(e.name());
            sink(": ");
            sink(e.syntax());
            sink("\n  ");
            sink(e.summary());
            sink("\n");
            if !e.aliases().is_empty() {
                sink("  alias: ");
                let mut a = 0;
                while a < e.aliases().len() {
                    sink(e.aliases()[a]);
                    if a + 1 < e.aliases().len() {
                        sink(", ");
                    }
                    a += 1;
                }
                sink("\n");
            }
            sink("  argumen: paling sedikit ");
            sink(match e.min_args() {
                0 => "nol",
                1 => "satu",
                _ => "dua",
            });
            sink(", paling banyak ");
            sink(match e.max_args() {
                0 => "nol",
                1 => "satu",
                _ => "dua",
            });
            sink("\n");
            true
        }
        None => false,
    }
}

// ============================================================================
// PENYUNTING BARIS + JALUR INPUT KANONIK (misi v1.0.2 Bagian 1.5, P3 "line editor")
// ============================================================================

/// Efek satu karakter masukan terhadap penyangga baris, dipakai untuk menentukan
/// gema (echo) yang benar pada serial dan framebuffer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Edit {
    /// Tidak ada perubahan baris (kontrol tak dikenal, sekuens escape, tumpahan).
    Ignored,
    /// Satu karakter sisip: gema karakter itu.
    Inserted,
    /// Satu karakter hapus di belakang kursor: gema backspace.
    Removed,
    /// Satu karakter hapus di depan kursor (tombol Delete). Baris berubah tapi
    /// penampil framebuffer/serial tidak dapat menggeser teks, jadi tanpa gema.
    RemovedForward,
    /// Baris selesai (Enter): gema baris baru lalu eksekusi.
    Submitted,
}

/// Permintaan gema ke penampil.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Echo {
    None,
    Char(u8),
    Backspace,
    Newline,
}

/// Status mesin sekuens escape sederhana (ESC [ <akhir> dan ESC [ <angka> ~).
const ESC_IDLE: u8 = 0;
const ESC_GOT_ESC: u8 = 1;
const ESC_GOT_BRACKET: u8 = 2;
const ESC_GOT_PARAM: u8 = 3;

impl CockpitShell {
    /// Bersihkan baris setelah ENTER; status escape ikut direset.
    pub fn clear_line(&mut self) {
        self.line_len = 0;
        self.cursor_pos = 0;
        self.esc_state = ESC_IDLE;
        self.esc_digit = 0;
        self.line_buffer[0] = 0;
    }

    /// Satu langkah penyuntingan baris: sisip di posisi kursor, hapus, dan
    /// navigasi kursor lewat sekuens escape. Append-only seperti perilaku lama
    /// tetap berlaku bila kursor tidak pernah dipindah.
    pub fn edit(&mut self, c: u8) -> Edit {
        // 1. Mesin sekuens escape.
        if self.esc_state != ESC_IDLE {
            return self.edit_escape(c);
        }
        match c {
            0x1B => {
                self.esc_state = ESC_GOT_ESC;
                self.esc_digit = 0;
                return Edit::Ignored;
            }
            b'\n' | b'\r' => return Edit::Submitted,
            b'\x08' | 0x7F => {
                if self.cursor_pos > 0 {
                    let mut i = self.cursor_pos - 1;
                    while i + 1 < self.line_len {
                        self.line_buffer[i] = self.line_buffer[i + 1];
                        i += 1;
                    }
                    self.cursor_pos -= 1;
                    self.line_len -= 1;
                    self.line_buffer[self.line_len] = 0;
                    return Edit::Removed;
                }
                return Edit::Ignored;
            }
            b' ' | 0x21..=0x7E => {
                if self.line_len >= SHELL_BUFFER_MAX - 1 {
                    return Edit::Ignored;
                }
                let mut i = self.line_len;
                while i > self.cursor_pos {
                    self.line_buffer[i] = self.line_buffer[i - 1];
                    i -= 1;
                }
                self.line_buffer[self.cursor_pos] = c;
                self.cursor_pos += 1;
                self.line_len += 1;
                return Edit::Inserted;
            }
            _ => return Edit::Ignored,
        }
    }

    /// Lanjutan sekuens escape: panah kiri/kanan, Home, End, Delete.
    /// Panah atas/bawah sengaja tidak melakukan apa pun: riwayat baris tidak
    /// dijanjikan misi v1.0.2 Bagian 1 (dicatat sebagai lingkup tertunda).
    fn edit_escape(&mut self, c: u8) -> Edit {
        match self.esc_state {
            ESC_GOT_ESC => {
                self.esc_state = if c == b'[' { ESC_GOT_BRACKET } else { ESC_IDLE };
                Edit::Ignored
            }
            ESC_GOT_BRACKET => {
                match c {
                    b'D' => {
                        if self.cursor_pos > 0 {
                            self.cursor_pos -= 1;
                        }
                        self.esc_state = ESC_IDLE;
                    }
                    b'C' => {
                        if self.cursor_pos < self.line_len {
                            self.cursor_pos += 1;
                        }
                        self.esc_state = ESC_IDLE;
                    }
                    b'H' => {
                        self.cursor_pos = 0;
                        self.esc_state = ESC_IDLE;
                    }
                    b'F' => {
                        self.cursor_pos = self.line_len;
                        self.esc_state = ESC_IDLE;
                    }
                    b'A' | b'B' => {
                        self.esc_state = ESC_IDLE;
                    }
                    b'0'..=b'9' => {
                        self.esc_digit = c - b'0';
                        self.esc_state = ESC_GOT_PARAM;
                    }
                    _ => {
                        self.esc_state = ESC_IDLE;
                    }
                }
                Edit::Ignored
            }
            _ => {
                // ESC_GOT_PARAM: ESC [ 3 ~ = Delete (hapus di depan kursor).
                if c == b'~' {
                    if self.esc_digit == 3 && self.cursor_pos < self.line_len {
                        let mut i = self.cursor_pos;
                        while i + 1 < self.line_len {
                            self.line_buffer[i] = self.line_buffer[i + 1];
                            i += 1;
                        }
                        self.line_len -= 1;
                        self.line_buffer[self.line_len] = 0;
                        self.esc_state = ESC_IDLE;
                        self.esc_digit = 0;
                        return Edit::RemovedForward;
                    }
                }
                self.esc_state = ESC_IDLE;
                self.esc_digit = 0;
                Edit::Ignored
            }
        }
    }

    /// Jalur input lengkap untuk kernel: sunting baris dan, saat ENTER, uraikan
    /// lewat registry kanonik lebih dulu sebelum menyerahkan ke enumerasi lama.
    ///
    /// Nilai ENTER membawa PASANGAN `(perintah lama, hasil kanonik)` supaya
    /// jembatan kernel dapat memilih jalur tanpa menyunting ulang baris yang
    /// sudah dibersihkan.
    pub fn handle_input(&mut self, c: u8) -> (Echo, Option<(ShellCommand, LineOutcome)>) {
        match self.edit(c) {
            Edit::Inserted => (Echo::Char(c), None),
            Edit::Removed => (Echo::Backspace, None),
            Edit::RemovedForward => (Echo::None, None),
            Edit::Ignored => (Echo::None, None),
            Edit::Submitted => {
                let legacy = self.parse_current_line();
                let outcome = parse_line(self.current_line());
                self.clear_line();
                (Echo::Newline, Some((legacy, outcome)))
            }
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

    #[test]
    fn test_all_shell_commands_and_unknown() {
        let commands = [
            ("clear", ShellCommand::Clear),
            ("meminfo", ShellCommand::MemInfo),
            ("cpuinfo", ShellCommand::CpuInfo),
            ("lspci", ShellCommand::LsPci),
            ("lsblk", ShellCommand::LsBlk),
            ("fdisk", ShellCommand::Fdisk),
            ("mount", ShellCommand::Mount),
            ("failover", ShellCommand::Failover),
            ("reboot", ShellCommand::Reboot),
            ("halt", ShellCommand::Halt),
            ("unknown_cmd", ShellCommand::Unknown),
        ];

        for (cmd_str, expected_cmd) in commands {
            let mut shell = CockpitShell::new();
            for b in cmd_str.bytes() {
                shell.handle_char(b);
            }
            assert_eq!(shell.handle_char(b'\n'), Some(expected_cmd));
            // Buffer harus di-reset setelah enter
            assert_eq!(shell.current_line(), "");
        }
    }

    #[test]
    fn test_shell_buffer_overflow_protection() {
        let mut shell = CockpitShell::new();
        // Coba masukkan 200 karakter 'x' (melebihi SHELL_BUFFER_MAX 128)
        for _ in 0..200 {
            shell.handle_char(b'x');
        }
        // Pastikan panjang string terpotong aman pada SHELL_BUFFER_MAX - 1 (127)
        assert_eq!(shell.current_line().len(), SHELL_BUFFER_MAX - 1);
        assert_eq!(shell.handle_char(b'\n'), Some(ShellCommand::Unknown));
    }

    #[test]
    fn test_release_identity_single_source() {
        // Janji misi v1.0.2 Bagian 1.1: prompt polos, TANPA posisi folder di prompt.
        assert_eq!(SHELL_PROMPT, "ASL-KERNEL> ");
        assert!(SHELL_PROMPT.ends_with("> "));
        assert!(!SHELL_PROMPT.contains('/'));
        assert!(!SHELL_PROMPT.contains('\\'));
        // Identitas rilis dibaca dari satu konstanta oleh seluruh situs tampil.
        assert_eq!(KERNEL_VERSION, "v1.0.2");
        assert!(KERNEL_VERSION.starts_with('v'));
    }
    /// Bangun keluaran `help` (daftar atau detail) ke penyangga milik pemanggil.
    /// Modul tes tidak boleh menuntut `alloc`, jadi penampungnya array tetap.
    fn capture_help<'a>(token: Option<&str>, buf: &'a mut [u8; 8192]) -> &'a str {
        let mut n = 0usize;
        {
            let mut sink = |s: &str| {
                let b = s.as_bytes();
                let mut i = 0;
                while i < b.len() && n + i < buf.len() {
                    buf[n + i] = b[i];
                    i += 1;
                }
                n += i;
            };
            match token {
                None => write_help(&mut sink),
                Some(t) => {
                    write_help_for(t, &mut sink);
                }
            }
        }
        core::str::from_utf8(&buf[..n]).unwrap_or("")
    }

    #[test]
    fn test_registry_has_eleven_canonical_commands() {
        assert_eq!(REGISTRY.len(), CANONICAL_COMMAND_COUNT);
        assert_eq!(CANONICAL_COMMAND_COUNT, 11);

        let mut alias_total = 0usize;
        let mut i = 0;
        while i < REGISTRY.len() {
            let e = REGISTRY[i];
            assert!(!e.name().is_empty());
            assert!(!e.summary().is_empty());
            assert!(!e.syntax().is_empty());
            assert!(e.min_args() <= e.max_args());
            assert!(e.max_args() <= MAX_ARGS);
            alias_total += e.aliases().len();
            for a in e.aliases() {
                // Setiap alias harus menunjuk kembali ke entri yang sama.
                let hit = find_entry(a).expect("alias harus terurai");
                assert_eq!(hit.id(), e.id());
            }
            i += 1;
        }
        // Empat perintah ber-alias: list(2), open(5), create-folder(1), delete-file(1).
        assert_eq!(alias_total, 9);

        // Nama dan identitas kanonik tidak boleh kembar.
        let mut j = 0;
        while j < REGISTRY.len() {
            let mut k = j + 1;
            while k < REGISTRY.len() {
                assert_ne!(REGISTRY[j].name(), REGISTRY[k].name());
                assert_ne!(REGISTRY[j].id(), REGISTRY[k].id());
                k += 1;
            }
            j += 1;
        }
    }

    #[test]
    fn test_parser_resolves_every_documented_alias() {
        let pasangan = [
            ("ls", "list"),
            ("dir", "list"),
            ("list", "list"),
            ("cd", "open"),
            ("cat", "open"),
            ("type", "open"),
            ("read-file", "open"),
            ("open-file", "open"),
            ("open", "open"),
            ("mkdir", "create-folder"),
            ("create-folder", "create-folder"),
            ("rm", "delete-file"),
            ("delete-file", "delete-file"),
            ("create-file", "create-file"),
            ("write-file", "write-file"),
            ("delete-folder", "delete-folder"),
            ("where", "where"),
            ("status", "status"),
            ("help", "help"),
            ("clear", "clear"),
        ];
        for (token, nama) in pasangan {
            let e = find_entry(token).unwrap_or_else(|| panic!("token {} tak terurai", token));
            assert_eq!(e.name(), nama);
        }
        // Nama lama TIDAK boleh diambil alih registry agar jalur lama tetap hidup.
        for lama in [
            "meminfo", "cpuinfo", "uname", "lspci", "lsblk", "fdisk", "mount", "failover",
            "reboot", "halt", "test_pml4", "test_apic", "test_task", "trap_de", "trap_bp",
            "trap_pf", "trap_df",
        ] {
            assert!(find_entry(lama).is_none(), "registry mengambil alih {}", lama);
        }
    }

    #[test]
    fn test_parser_line_outcomes_and_arity() {
        assert!(matches!(parse_line("help"), LineOutcome::Ready(_)));
        assert!(matches!(parse_line("  where  "), LineOutcome::Ready(_)));

        match parse_line("open") {
            LineOutcome::BadArity(syntax) => assert_eq!(syntax, "open <jalan>"),
            other => panic!("perlu BadArity, dapat {:?}", other),
        }
        assert!(matches!(
            parse_line("help satu dua tiga"),
            LineOutcome::BadArity(_)
        ));
        // Nama di luar registry diserahkan ke jalur lama, bukan dianggap Unknown.
        assert_eq!(parse_line("meminfo"), LineOutcome::NotCanonical);
        assert_eq!(parse_line(""), LineOutcome::NotCanonical);
        assert_eq!(parse_line("   "), LineOutcome::NotCanonical);

        match parse_line("create-file   catatan.txt") {
            LineOutcome::Ready(inv) => {
                assert_eq!(inv.id, CanonicalId::CreateFile);
                assert_eq!(inv.argc, 1);
                assert_eq!(inv.arg(0), "catatan.txt");
                assert_eq!(inv.arg(1), "");
                assert_eq!(inv.entry().name(), "create-file");
            }
            other => panic!("perlu Ready, dapat {:?}", other),
        }
    }

    #[test]
    fn test_parser_write_file_takes_remainder_as_one_text() {
        // Sintaks `write-file <nama> <teks>` berjanji teks boleh berisi spasi.
        match parse_line("write-file pesan.txt halo dunia ASL") {
            LineOutcome::Ready(inv) => {
                assert_eq!(inv.id, CanonicalId::WriteFile);
                assert_eq!(inv.argc, 2);
                assert_eq!(inv.arg(0), "pesan.txt");
                assert_eq!(inv.arg(1), "halo dunia ASL");
            }
            other => panic!("perlu Ready, dapat {:?}", other),
        }
        // Tanpa teks tetap sah (berkas dikosongkan): argc = 1.
        match parse_line("write-file kosong.txt") {
            LineOutcome::Ready(inv) => assert_eq!(inv.argc, 1),
            other => panic!("perlu Ready argc 1, dapat {:?}", other),
        }
        // Argumen panjang dipotong aman ke MAX_ARG_LEN, bukan panik.
        let mut baris = [0u8; 200];
        let pre = b"create-file ";
        let mut i = 0;
        while i < pre.len() {
            baris[i] = pre[i];
            i += 1;
        }
        while i < baris.len() {
            baris[i] = b'x';
            i += 1;
        }
        let teks_baris = core::str::from_utf8(&baris).unwrap_or("");
        assert!(teks_baris.len() > MAX_ARG_LEN + pre.len());
        match parse_line(teks_baris) {
            LineOutcome::Ready(inv) => assert_eq!(inv.arg(0).len(), MAX_ARG_LEN),
            other => panic!("perlu Ready, dapat {:?}", other),
        }
    }

    #[test]
    fn test_line_editor_insert_delete_and_navigation() {
        let mut shell = CockpitShell::new();
        for b in b"bc" {
            assert_eq!(shell.edit(*b), Edit::Inserted);
        }
        assert_eq!(shell.current_line(), "bc");

        // Home lalu sisip di depan kursor.
        for b in b"\x1B[H" {
            assert_eq!(shell.edit(*b), Edit::Ignored);
        }
        assert_eq!(shell.edit(b'a'), Edit::Inserted);
        assert_eq!(shell.current_line(), "abc");

        // Left ke awal, lalu Delete (ESC [ 3 ~) membuang karakter DI DEPAN kursor.
        for b in b"\x1B[D" {
            shell.edit(*b);
        }
        for b in b"\x1B[3~" {
            shell.edit(*b);
        }
        assert_eq!(shell.current_line(), "bc");

        // End lalu backspace membuang karakter belakang kursor.
        for b in b"\x1B[F" {
            shell.edit(*b);
        }
        assert_eq!(shell.edit(b'\x08'), Edit::Removed);
        assert_eq!(shell.current_line(), "b");

        // Backspace di awal baris tidak mengubah apa pun.
        for b in b"\x1B[H" {
            shell.edit(*b);
        }
        assert_eq!(shell.edit(b'\x08'), Edit::Ignored);
        assert_eq!(shell.current_line(), "b");

        // Panah atas/bawah tidak dijanjikan Bagian 1: baris tetap utuh.
        for b in b"\x1B[A" {
            shell.edit(*b);
        }
        for b in b"\x1B[B" {
            shell.edit(*b);
        }
        assert_eq!(shell.current_line(), "b");

        // Karakter kontrol tak dikenal diabaikan.
        assert_eq!(shell.edit(0x07), Edit::Ignored);
        assert_eq!(shell.edit(b'\t'), Edit::Ignored);
    }

    #[test]
    fn test_line_editor_overflow_and_submit_contract() {
        let mut shell = CockpitShell::new();
        let (echo, outcome) = shell.handle_input(b'\n');
        assert_eq!(echo, Echo::Newline);
        assert_eq!(
            outcome,
            Some((ShellCommand::Empty, LineOutcome::NotCanonical))
        );

        for _ in 0..200 {
            shell.handle_input(b'x');
        }
        assert!(shell.line_buffer[..shell.line_len]
            .iter()
            .all(|&b| b == b'x'));
        assert_eq!(shell.current_line().len(), SHELL_BUFFER_MAX - 1);

        let (echo, outcome) = shell.handle_input(b'\n');
        assert_eq!(echo, Echo::Newline);
        assert_eq!(shell.current_line(), "");
        assert_eq!(
            outcome,
            Some((ShellCommand::Unknown, LineOutcome::NotCanonical))
        );

        shell.handle_input(b'a');
        let (echo, _) = shell.handle_input(b'\x08');
        assert_eq!(echo, Echo::Backspace);
        let (echo, _) = shell.handle_input(b'\x1B');
        assert_eq!(echo, Echo::None);

        // ESC yang menggantung ditutup oleh sekuens tak dikenal supaya mesin
        // kembali normal sebelum karakter berikutnya disisipkan.
        assert_eq!(shell.handle_input(b'['), (Echo::None, None));
        assert_eq!(shell.handle_input(b'Z'), (Echo::None, None));

        // Delete (ESC [ 3 ~) di depan kursor: baris berubah TANPA gema,
        // karena konsol framebuffer/serial tidak dapat menggeser teks.
        assert_eq!(shell.handle_input(b'b'), (Echo::Char(b'b'), None));
        assert_eq!(shell.handle_input(b'c'), (Echo::Char(b'c'), None));
        assert_eq!(shell.current_line(), "bc");
        for b in b"\x1B[H" {
            assert_eq!(shell.handle_input(*b), (Echo::None, None));
        }
        for b in b"\x1B[3~" {
            assert_eq!(shell.handle_input(*b), (Echo::None, None));
        }
        assert_eq!(shell.current_line(), "c");
    }

    #[test]
    fn test_handle_char_keeps_legacy_contract_through_shared_editor() {
        let mut shell = CockpitShell::new();
        for b in b"lsblk" {
            assert_eq!(shell.handle_char(*b), None);
        }
        assert_eq!(shell.handle_char(b'\n'), Some(ShellCommand::LsBlk));
        assert_eq!(shell.current_line(), "");

        // Nama kanonik juga tetap terurai lewat kontrak lama (perilaku lama dijaga).
        for b in b"help" {
            shell.handle_char(*b);
        }
        assert_eq!(shell.handle_char(b'\r'), Some(ShellCommand::Help));
    }

    #[test]
    fn test_help_text_is_generated_from_registry() {
        let mut buf = [0u8; 8192];
        let text = capture_help(None, &mut buf);
        assert!(text.len() > 400, "keluaran help terlalu pendek: {}", text.len());

        for e in REGISTRY {
            assert!(text.contains(e.name()), "help tidak memuat {}", e.name());
            assert!(
                text.contains(e.syntax()),
                "help tidak memuat sintaks {}",
                e.syntax()
            );
            assert!(
                text.contains(e.summary()),
                "help tidak memuat ringkasan {}",
                e.name()
            );
            for a in e.aliases() {
                assert!(text.contains(a), "help tidak memuat alias {}", a);
            }
        }
        assert!(text.contains("alias: "), "baris alias tidak dibangun");
        assert!(text.contains("registry"), "sumber help tidak dinyatakan");
        assert!(text.contains("Berkas dan direktori"), "seksi berkas hilang");
        assert!(text.contains("Ruang kerja"), "seksi ruang kerja hilang");
    }

    #[test]
    fn test_help_for_one_command_and_unknown_token() {
        let mut buf = [0u8; 8192];
        let text = capture_help(Some("write-file"), &mut buf);
        assert!(text.contains("write-file <nama> <teks>"));
        assert!(text.contains("paling sedikit satu, paling banyak dua"));

        let mut n = 0usize;
        assert!(!write_help_for("bukan-perintah", |t| n += t.len()));
        assert_eq!(n, 0);
    }
}
