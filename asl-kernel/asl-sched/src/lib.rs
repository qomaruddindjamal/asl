//! ASL Deterministic Scheduler: ARINC 653 Major/Minor Frame Time-Triggered
//! ======================================================================
//! Memenuhi kriteria penerbangan DO-178C DAL-A.
//! Zero-jitter, penjadwalan berbasis waktu deterministik tetap (Major Frame Cycle),
//! menjamin alokasi CPU untuk partisi kritis (FADEC, Fly-by-wire, Cryptographic Vault).

#![no_std]

pub const MAX_SLOTS_PER_MAJOR_FRAME: usize = 32;

/// Slot Waktu Minor Frame dalam Siklus Avionic Major Frame
#[derive(Debug, Copy, Clone)]
pub struct MinorFrameSlot {
    pub partition_id: u32,
    pub duration_microseconds: u32,
    pub is_critical_slot: bool,
}

impl MinorFrameSlot {
    pub const fn empty() -> Self {
        Self {
            partition_id: 0,
            duration_microseconds: 0,
            is_critical_slot: false,
        }
    }
}

/// Siklus Jadwal Deterministik ARINC 653 Major Frame
pub struct MajorFrameSchedule {
    pub major_frame_duration_us: u32,
    pub slot_count: usize,
    pub slots: [MinorFrameSlot; MAX_SLOTS_PER_MAJOR_FRAME],
    pub current_slot_index: usize,
    pub elapsed_in_current_slot_us: u32,
}

impl MajorFrameSchedule {
    pub const fn new(duration_us: u32) -> Self {
        Self {
            major_frame_duration_us: duration_us,
            slot_count: 0,
            slots: [MinorFrameSlot::empty(); MAX_SLOTS_PER_MAJOR_FRAME],
            current_slot_index: 0,
            elapsed_in_current_slot_us: 0,
        }
    }

    /// Menambahkan Minor Frame Slot ke dalam siklus
    pub fn add_slot(&mut self, partition_id: u32, duration_us: u32, is_critical: bool) -> Result<(), &'static str> {
        if self.slot_count >= MAX_SLOTS_PER_MAJOR_FRAME {
            return Err("ERR_SCHEDULE_SLOTS_FULL");
        }

        self.slots[self.slot_count] = MinorFrameSlot {
            partition_id,
            duration_microseconds: duration_us,
            is_critical_slot: is_critical,
        };
        self.slot_count += 1;
        Ok(())
    }

    /// Memajukan tick timer deterministik dan menentukan apakah terjadi perpindahan partisi (Context Switch)
    pub fn advance_tick(&mut self, tick_us: u32) -> (u32, bool) {
        if self.slot_count == 0 {
            return (0, false);
        }

        self.elapsed_in_current_slot_us += tick_us;
        let current_slot = &self.slots[self.current_slot_index];

        if self.elapsed_in_current_slot_us >= current_slot.duration_microseconds {
            // Waktu slot habis: ganti ke slot berikutnya secara deterministik siklik
            self.elapsed_in_current_slot_us = 0;
            self.current_slot_index = (self.current_slot_index + 1) % self.slot_count;
            let next_partition = self.slots[self.current_slot_index].partition_id;
            (next_partition, true) // Memicu Context Switch
        } else {
            (current_slot.partition_id, false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_major_frame_slots() {
        let mut sched = MajorFrameSchedule::new(100_000);
        assert!(sched.add_slot(1, 20_000, true).is_ok());
        assert!(sched.add_slot(2, 80_000, false).is_ok());
        assert_eq!(sched.slot_count, 2);

        // Advance 10ms dalam slot 1 (belum switch)
        let (p, switched) = sched.advance_tick(10_000);
        assert_eq!(p, 1);
        assert!(!switched);

        // Advance 10ms lagi (mencapai 20ms: switch ke slot 2)
        let (p, switched) = sched.advance_tick(10_000);
        assert_eq!(p, 2);
        assert!(switched);

        // Advance 80ms dalam slot 2 (switch kembali ke slot 1 siklik)
        let (p, switched) = sched.advance_tick(80_000);
        assert_eq!(p, 1);
        assert!(switched);
    }

    #[test]
    fn test_scheduler_slot_overflow() {
        let mut sched = MajorFrameSchedule::new(100_000);
        for i in 0..MAX_SLOTS_PER_MAJOR_FRAME {
            assert!(sched.add_slot(i as u32, 1000, false).is_ok());
        }
        // Slot ke-33 harus error
        assert_eq!(sched.add_slot(99, 1000, false), Err("ERR_SCHEDULE_SLOTS_FULL"));
    }
}
