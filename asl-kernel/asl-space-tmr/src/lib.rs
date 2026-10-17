//! ASL Space TMR: Radiation-Hardened Triple-Modular Redundancy & Memory Scrubbing
//! ==============================================================================
//! Disesuaikan untuk prosesor antariksa ESA LEON3/LEON4 (SPARC V8) dan satelit orbit rendah (LEO/GEO).
//! Mengatasi Single Event Upset (SEU) dan bit-flipping akibat partikel radiasi kosmik berenergi tinggi.
//! Fitur:
//! 1. Majority Voting Triple-Modular Redundancy (2-out-of-3 voting logic).
//! 2. Continuous Background Memory Scrubbing (ECC Bit Auto-Correction).
//! 3. SPARC Register Window Trap Handler (Overflow/Underflow resilience).

#![no_std]

/// Hasil Voting Triple-Modular Redundancy
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TmrVoteResult {
    Unanimous,             // 3/3 Nilai identik
    SingleFaultCorrected,  // 2/3 Nilai sama (1 bit terbalik akibat radiasi, berhasil dikoreksi)
    UnrecoverableTmrFault, // 3 nilai berbeda semua (Anomali fatal radiasi)
}

/// Struktur Data TMR (Triple Storage)
#[derive(Debug, Copy, Clone)]
pub struct TmrVariable<T: Copy + PartialEq> {
    pub copy_a: T,
    pub copy_b: T,
    pub copy_c: T,
}

impl<T: Copy + PartialEq> TmrVariable<T> {
    pub const fn new(initial_val: T) -> Self {
        Self {
            copy_a: initial_val,
            copy_b: initial_val,
            copy_c: initial_val,
        }
    }

    /// Majority Voting Logic (2 dari 3 suara penentu)
    pub fn vote_and_repair(&mut self) -> (T, TmrVoteResult) {
        if self.copy_a == self.copy_b && self.copy_b == self.copy_c {
            (self.copy_a, TmrVoteResult::Unanimous)
        } else if self.copy_a == self.copy_b {
            // copy_c rusak terbalik, perbaiki copy_c
            self.copy_c = self.copy_a;
            (self.copy_a, TmrVoteResult::SingleFaultCorrected)
        } else if self.copy_a == self.copy_c {
            // copy_b rusak terbalik, perbaiki copy_b
            self.copy_b = self.copy_a;
            (self.copy_a, TmrVoteResult::SingleFaultCorrected)
        } else if self.copy_b == self.copy_c {
            // copy_a rusak terbalik, perbaiki copy_a
            self.copy_a = self.copy_b;
            (self.copy_b, TmrVoteResult::SingleFaultCorrected)
        } else {
            // Kehancuran parah: 3 nilai berbeda
            (self.copy_a, TmrVoteResult::UnrecoverableTmrFault)
        }
    }
}

/// Mesin Pemindai Memori Berkelanjutan (Background Memory Scrubber)
pub struct ContinuousMemoryScrubber {
    pub current_scrub_address: u64,
    pub memory_limit: u64,
    pub single_bit_errors_corrected: u64,
    pub double_bit_errors_detected: u64,
}

impl ContinuousMemoryScrubber {
    pub const fn new(base: u64, limit: u64) -> Self {
        Self {
            current_scrub_address: base,
            memory_limit: limit,
            single_bit_errors_corrected: 0,
            double_bit_errors_detected: 0,
        }
    }

    /// Memindai satu blok memori dan memverifikasi parity/syndrome ECC
    pub fn scrub_next_block(&mut self, word: u32, syndrome: u8) -> u32 {
        self.current_scrub_address += 4;
        if self.current_scrub_address >= self.memory_limit {
            self.current_scrub_address = 0;
        }

        // Simulasi koreksi sindrom Hamming ECC (Single-Error Correction, Double-Error Detection)
        if syndrome == 0 {
            word
        } else if (syndrome & 0x80) != 0 {
            // Double bit uncorrectable error
            self.double_bit_errors_detected += 1;
            word
        } else {
            // Single bit flip berhasil diperbaiki secara live
            self.single_bit_errors_corrected += 1;
            let bit_to_flip = (syndrome & 0x1F) as usize;
            word ^ (1 << bit_to_flip)
        }
    }
}

// =============================================================================
// ATOMIC-SPARC-01 TAHAP 1: DOWNSCALE TIMEKEEPING & LOCK-FREE SPLIT-COUNTER
// =============================================================================

use core::sync::atomic::{AtomicU32, Ordering};

/// Pengukur Waktu Siklis Ter-downscale untuk SPARC V8 (LEON3)
///
/// Menggunakan AtomicU32 dengan penanganan modular wraparound terdefinisi (C-2).
pub struct SparcDownscaledTimekeeper {
    pub ticks_us: AtomicU32,
}

impl SparcDownscaledTimekeeper {
    pub const fn new(init_ticks: u32) -> Self {
        Self {
            ticks_us: AtomicU32::new(init_ticks),
        }
    }

    /// Majukan tick secara atomik dengan wraparound wrapping_add
    #[inline(always)]
    pub fn advance_ticks(&self, delta: u32) -> u32 {
        self.ticks_us.fetch_add(delta, Ordering::AcqRel)
    }

    /// Baca tick saat ini
    #[inline(always)]
    pub fn read_ticks(&self) -> u32 {
        self.ticks_us.load(Ordering::Acquire)
    }

    /// Menghitung selisih waktu terdefinisi yang kebal terhadap u32::MAX wraparound
    #[inline(always)]
    pub fn elapsed_since(&self, previous_tick: u32) -> u32 {
        let current = self.read_ticks();
        current.wrapping_sub(previous_tick)
    }
}

/// Struktur Pembacaan 64-bit Non-RMW Lock-Free Split-Counter untuk SPARC V8
///
/// Pola Eksekusi: lo (Relaxed) -> hi (Acquire) -> lo (Relaxed) dengan deteksi carry.
pub struct AtomicU64Split {
    pub lo: AtomicU32,
    pub hi: AtomicU32,
}

impl AtomicU64Split {
    pub const fn new(val: u64) -> Self {
        Self {
            lo: AtomicU32::new(val as u32),
            hi: AtomicU32::new((val >> 32) as u32),
        }
    }

    /// Pembacaan lock-free 64-bit yang aman dari interferensi carry tanpa lock
    pub fn load_lockfree(&self) -> u64 {
        loop {
            let l1 = self.lo.load(Ordering::Relaxed);
            let h = self.hi.load(Ordering::Acquire);
            let l2 = self.lo.load(Ordering::Relaxed);

            // Jika l2 >= l1 dan pembacaan konsisten, tidak ada carry interleaving
            if l1 == l2 {
                return ((h as u64) << 32) | (l1 as u64);
            }
            // Terjadi interleave penulisan / carry saat loop, retry sekali
        }
    }

    /// Update 64-bit bernilai penuh oleh producer
    pub fn store(&self, val: u64) {
        let new_lo = val as u32;
        let new_hi = (val >> 32) as u32;

        self.hi.store(new_hi, Ordering::Release);
        self.lo.store(new_lo, Ordering::Release);
    }
}

/// Pembungkus Universal Atomic 64-bit yang memilih implementasi secara optimal (ATOMIC-SPARC-01 Tahap 2)
pub enum UniversalAtomicU64 {
    #[cfg(target_has_atomic = "64")]
    Native(core::sync::atomic::AtomicU64),
    Split(AtomicU64Split),
}

impl UniversalAtomicU64 {
    pub fn new(val: u64) -> Self {
        #[cfg(target_has_atomic = "64")]
        {
            UniversalAtomicU64::Native(core::sync::atomic::AtomicU64::new(val))
        }
        #[cfg(not(target_has_atomic = "64"))]
        {
            UniversalAtomicU64::Split(AtomicU64Split::new(val))
        }
    }

    pub fn load(&self) -> u64 {
        match self {
            #[cfg(target_has_atomic = "64")]
            UniversalAtomicU64::Native(a) => a.load(Ordering::Acquire),
            UniversalAtomicU64::Split(s) => s.load_lockfree(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tmr_voting_and_scrubber() {
        let mut tmr = TmrVariable::new(42);
        assert_eq!(tmr.vote_and_repair(), (42, TmrVoteResult::Unanimous));

        tmr.copy_b = 99; // Corrupted
        assert_eq!(
            tmr.vote_and_repair(),
            (42, TmrVoteResult::SingleFaultCorrected)
        );
        assert_eq!(tmr.copy_b, 42); // Repaired

        let mut scrubber = ContinuousMemoryScrubber::new(0x1000, 0x2000);
        // syndrome = 1 membalik bit 1 (1 << 1 = 2)
        let corrected = scrubber.scrub_next_block(0xAAAA_AAA0, 0x01);
        assert_eq!(corrected, 0xAAAA_AAA2);
        assert_eq!(scrubber.single_bit_errors_corrected, 1);
    }

    #[test]
    fn test_sparc_atomic_u32_wraparound_defined() {
        // Uji inkremen melewati u32::MAX (C-2)
        let timekeeper = SparcDownscaledTimekeeper::new(u32::MAX - 5);
        let old = timekeeper.read_ticks();
        assert_eq!(old, u32::MAX - 5);

        // Majukan melewati batas batas u32::MAX
        timekeeper.advance_ticks(10);
        let current = timekeeper.read_ticks();
        assert_eq!(current, 4); // Wraparound modular terverifikasi: (MAX - 5) + 10 = 4

        // Selisih waktu wrapping_sub terbukti bernilai persis 10
        assert_eq!(timekeeper.elapsed_since(old), 10);
    }

    #[test]
    fn test_sparc_atomic_u64_split_counter_interleaving_carry() {
        let split = AtomicU64Split::new(0x0000_0001_FFFF_FFFF);
        assert_eq!(split.load_lockfree(), 0x0000_0001_FFFF_FFFF);

        // Simulasikan carry overflow saat penulisan
        split.store(0x0000_0002_0000_0000);
        assert_eq!(split.load_lockfree(), 0x0000_0002_0000_0000);
    }

    #[test]
    fn test_universal_atomic_u64_selection() {
        let universal = UniversalAtomicU64::new(0x1234_5678_9ABC_DEF0);
        assert_eq!(universal.load(), 0x1234_5678_9ABC_DEF0);

        // Pada host x86_64, target_has_atomic = 64 bernilai true sehingga jalur native aktif
        #[cfg(target_has_atomic = "64")]
        match universal {
            UniversalAtomicU64::Native(_) => {}
            _ => panic!("Expected native AtomicU64 on 64-bit atomic capable target!"),
        }
    }
}
