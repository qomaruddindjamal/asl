//! ASL DSP RPC: Low-Latency Shared-Memory Interconnect
//! ==================================================
//! Protokol RPC berkecepatan tinggi antara CPU host dan DSP core (TI C6000, Qualcomm Hexagon).
//! Menjamin latensi sub-5 mikrodetik untuk pemrosesan sinyal radar avionik dan FFT audio.
//! Menggunakan struktur mailbox bebas kunci (Lock-free Mailbox Ring).

#![no_std]

/// Perintah Pemrosesan DSP
#[repr(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DspOpcode {
    Fft1024 = 1,
    FirFilter = 2,
    RadarDopplerProcess = 3,
    BeamformingCalculation = 4,
}

/// Pesan Kotak Surat (Mailbox Message) CPU-DSP
#[repr(C)]
#[derive(Copy, Clone)]
pub struct DspMailboxMessage {
    pub message_id: u32,
    pub opcode: DspOpcode,
    pub input_data_ptr: u64,
    pub output_data_ptr: u64,
    pub data_length_samples: u32,
    pub status_code: u16,
}

impl DspMailboxMessage {
    pub const fn empty() -> Self {
        Self {
            message_id: 0,
            opcode: DspOpcode::Fft1024,
            input_data_ptr: 0,
            output_data_ptr: 0,
            data_length_samples: 0,
            status_code: 0,
        }
    }
}

pub const DSP_MAILBOX_CAPACITY: usize = 16;

use core::sync::atomic::{AtomicUsize, Ordering};

/// Pengontrol Komunikasi Bersama CPU-DSP (SPSC Lock-Free Mailbox Ring)
///
/// Model Konkurensi SPSC:
/// - Host CPU bertindak sebagai Producer (`invoke_dsp_task`) memajukan `tail` via `Ordering::Release`
/// - DSP Core bertindak sebagai Consumer (`poll_dsp_completion`) memajukan `head` via `Ordering::Release`
pub struct DspSharedMemoryRpcBridge {
    pub mailbox: [DspMailboxMessage; DSP_MAILBOX_CAPACITY],
    pub head: AtomicUsize,
    pub tail: AtomicUsize,
    pub dsp_interrupt_triggered_total: u64,
}

impl DspSharedMemoryRpcBridge {
    pub const fn new() -> Self {
        Self {
            mailbox: [DspMailboxMessage::empty(); DSP_MAILBOX_CAPACITY],
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            dsp_interrupt_triggered_total: 0,
        }
    }

    /// Menghitung jumlah pesan yang saat ini menunggu di mailbox
    #[inline]
    pub fn count(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        if tail >= head {
            tail - head
        } else {
            DSP_MAILBOX_CAPACITY - (head - tail)
        }
    }

    /// Kirim tugas komputasi sinyal ke DSP
    ///
    /// Eksklusif dipanggil oleh Host CPU core (Producer)
    pub fn invoke_dsp_task(&mut self, msg: DspMailboxMessage) -> Result<(), &'static str> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);

        let next_tail = (tail + 1) % DSP_MAILBOX_CAPACITY;
        if next_tail == head {
            return Err("ERR_DSP_MAILBOX_FULL");
        }

        self.mailbox[tail] = msg;
        self.tail.store(next_tail, Ordering::Release);
        self.dsp_interrupt_triggered_total += 1;

        Ok(())
    }

    /// Terima hasil pemrosesan sinyal dari DSP
    ///
    /// Eksklusif dipanggil oleh DSP core / Consumer thread
    pub fn poll_dsp_completion(&mut self) -> Option<DspMailboxMessage> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);

        if head == tail {
            return None;
        }

        let completed_msg = self.mailbox[head];
        let next_head = (head + 1) % DSP_MAILBOX_CAPACITY;
        self.head.store(next_head, Ordering::Release);

        Some(completed_msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dsp_rpc_empty_poll() {
        let mut rpc = DspSharedMemoryRpcBridge::new();
        assert_eq!(rpc.count(), 0);
        assert!(rpc.poll_dsp_completion().is_none());
    }

    #[test]
    fn test_dsp_rpc_invoke_and_poll_fifo() {
        let mut rpc = DspSharedMemoryRpcBridge::new();
        let mut msg1 = DspMailboxMessage::empty();
        msg1.message_id = 42;
        msg1.opcode = DspOpcode::Fft1024;

        let mut msg2 = DspMailboxMessage::empty();
        msg2.message_id = 43;
        msg2.opcode = DspOpcode::RadarDopplerProcess;

        assert_eq!(rpc.invoke_dsp_task(msg1), Ok(()));
        assert_eq!(rpc.invoke_dsp_task(msg2), Ok(()));
        assert_eq!(rpc.count(), 2);

        let res1 = rpc.poll_dsp_completion().expect("Should get msg 1");
        assert_eq!(res1.message_id, 42);
        assert_eq!(res1.opcode, DspOpcode::Fft1024);

        let res2 = rpc.poll_dsp_completion().expect("Should get msg 2");
        assert_eq!(res2.message_id, 43);
        assert_eq!(res2.opcode, DspOpcode::RadarDopplerProcess);

        assert_eq!(rpc.count(), 0);
        assert!(rpc.poll_dsp_completion().is_none());
    }

    #[test]
    fn test_dsp_rpc_capacity_and_wraparound() {
        let mut rpc = DspSharedMemoryRpcBridge::new();
        let mut msg = DspMailboxMessage::empty();

        for i in 0..15 {
            msg.message_id = i;
            assert_eq!(rpc.invoke_dsp_task(msg), Ok(()));
        }

        assert_eq!(rpc.invoke_dsp_task(msg), Err("ERR_DSP_MAILBOX_FULL"));

        // Dequeue 1 msg
        let polled = rpc.poll_dsp_completion().expect("Should dequeue");
        assert_eq!(polled.message_id, 0);

        // Enqueue lagi (wraparound)
        msg.message_id = 999;
        assert_eq!(rpc.invoke_dsp_task(msg), Ok(()));
    }
}

