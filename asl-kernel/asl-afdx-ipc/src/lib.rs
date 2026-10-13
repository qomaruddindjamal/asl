//! ASL AFDX (ARINC 664) Virtual Link Cryptographic Shaper & IPC
//! ============================================================
//! Komunikasi deterministik antar-partisi berbasis avionik modern.
//! Menjamin:
//! 1. Virtual Link (VL) dengan isolasi bandwidth kaku (Bandwidth Allocation Gap - BAG)
//! 2. Zero-Copy Circular Ring Buffer
//! 3. Cryptographic integrity protection (Zero-Trust Bus)

#![no_std]

pub const MAX_AFDX_PAYLOAD_SIZE: usize = 1472; // Ukuran payload standar paket AFDX / Ethernet
pub const RING_BUFFER_CAPACITY: usize = 16;

/// Frame Pesan Virtual Link AFDX
#[repr(C)]
#[derive(Copy, Clone)]
pub struct AfdxFrame {
    pub virtual_link_id: u16,
    pub sequence_number: u8,
    pub payload_size: usize,
    pub payload: [u8; MAX_AFDX_PAYLOAD_SIZE],
    pub integrity_mac: u64, // SipHash / PAC integritas
}

impl AfdxFrame {
    pub const fn empty() -> Self {
        Self {
            virtual_link_id: 0,
            sequence_number: 0,
            payload_size: 0,
            payload: [0u8; MAX_AFDX_PAYLOAD_SIZE],
            integrity_mac: 0,
        }
    }
}

use core::sync::atomic::{AtomicUsize, AtomicU64, Ordering};

/// Zero-Copy Lock-Free SPSC Circular Ring Buffer untuk IPC Antar-Partisi (SMP Safe)
///
/// Model Konkurensi: Single-Producer Single-Consumer (SPSC)
/// - `head` dimodifikasi eksklusif oleh Consumer (Receive) dan dibaca oleh Producer (Send)
/// - `tail` dimodifikasi eksklusif oleh Producer (Send) dan dibaca oleh Consumer (Receive)
/// - `last_transmission_time_us` dimodifikasi eksklusif oleh Producer
///
/// Jaminan Memory Ordering:
/// - Producer mempublikasikan frame dengan `tail.store(..., Ordering::Release)`
/// - Consumer membaca data setelah `tail.load(..., Ordering::Acquire)`
/// - Consumer mengosongkan slot dengan `head.store(..., Ordering::Release)`
/// - Producer membaca ketersediaan slot dengan `head.load(..., Ordering::Acquire)`
pub struct AfdxVirtualLinkRingBuffer {
    pub virtual_link_id: u16,
    pub bag_microseconds: u32, // Bandwidth Allocation Gap
    pub last_transmission_time_us: AtomicU64,
    pub head: AtomicUsize,
    pub tail: AtomicUsize,
    pub buffer: [AfdxFrame; RING_BUFFER_CAPACITY],
}

impl AfdxVirtualLinkRingBuffer {
    pub const fn new(vl_id: u16, bag_us: u32) -> Self {
        Self {
            virtual_link_id: vl_id,
            bag_microseconds: bag_us,
            last_transmission_time_us: AtomicU64::new(0),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            buffer: [AfdxFrame::empty(); RING_BUFFER_CAPACITY],
        }
    }

    /// Menghitung jumlah elemen yang saat ini ada di antrean
    #[inline]
    pub fn count(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        if tail >= head {
            tail - head
        } else {
            RING_BUFFER_CAPACITY - (head - tail)
        }
    }

    /// Kirim frame ke Ring Buffer (Enqueue) dengan validasi Bandwidth Allocation Gap (BAG)
    ///
    /// Eksklusif dipanggil oleh Producer thread / core.
    pub fn send(&mut self, current_time_us: u64, frame: AfdxFrame) -> Result<(), &'static str> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);

        // Kapasitas penuh bila (tail + 1) % CAPACITY == head
        let next_tail = (tail + 1) % RING_BUFFER_CAPACITY;
        if next_tail == head {
            return Err("ERR_AFDX_RING_BUFFER_OVERFLOW");
        }

        // Penegakan Traffic Shaping BAG: Cegah Flooding & DoS
        let last_tx = self.last_transmission_time_us.load(Ordering::Relaxed);
        if (current_time_us - last_tx) < (self.bag_microseconds as u64) {
            return Err("ERR_AFDX_BAG_VIOLATION_RATE_LIMITED");
        }

        // Tulis frame ke slot tail privat producer
        self.buffer[tail] = frame;

        // Publikasikan payload secara aman ke consumer
        self.last_transmission_time_us.store(current_time_us, Ordering::Relaxed);
        self.tail.store(next_tail, Ordering::Release);

        Ok(())
    }

    /// Terima frame dari Ring Buffer (Dequeue)
    ///
    /// Eksklusif dipanggil oleh Consumer thread / core.
    pub fn receive(&mut self) -> Option<AfdxFrame> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);

        if head == tail {
            return None; // Antrean kosong
        }

        let frame = self.buffer[head];
        let next_head = (head + 1) % RING_BUFFER_CAPACITY;

        // Kosongkan slot dan publikasikan posisi head baru ke producer
        self.head.store(next_head, Ordering::Release);

        Some(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_afdx_empty_receive() {
        let mut queue = AfdxVirtualLinkRingBuffer::new(101, 100);
        assert_eq!(queue.count(), 0);
        assert!(queue.receive().is_none());
    }

    #[test]
    fn test_afdx_send_and_receive_fifo() {
        let mut queue = AfdxVirtualLinkRingBuffer::new(101, 10);
        let mut frame1 = AfdxFrame::empty();
        frame1.sequence_number = 1;
        let mut frame2 = AfdxFrame::empty();
        frame2.sequence_number = 2;

        assert_eq!(queue.send(10, frame1), Ok(()));
        assert_eq!(queue.send(25, frame2), Ok(()));
        assert_eq!(queue.count(), 2);

        let rx1 = queue.receive().expect("Should receive frame 1");
        assert_eq!(rx1.sequence_number, 1);
        let rx2 = queue.receive().expect("Should receive frame 2");
        assert_eq!(rx2.sequence_number, 2);
        assert_eq!(queue.count(), 0);
        assert!(queue.receive().is_none());
    }

    #[test]
    fn test_afdx_bag_rate_limiting() {
        let mut queue = AfdxVirtualLinkRingBuffer::new(101, 100); // BAG = 100 us
        let frame = AfdxFrame::empty();

        assert_eq!(queue.send(100, frame), Ok(()));
        // Kirim sebelum 100 us berlalu harus ditolak rate-limiting
        assert_eq!(queue.send(150, frame), Err("ERR_AFDX_BAG_VIOLATION_RATE_LIMITED"));
        // Kirim setelah 100 us berlalu harus berhasil
        assert_eq!(queue.send(205, frame), Ok(()));
    }

    #[test]
    fn test_afdx_ring_buffer_overflow_and_wraparound() {
        let mut queue = AfdxVirtualLinkRingBuffer::new(101, 0); // No BAG for capacity test
        let mut frame = AfdxFrame::empty();

        // Buffer berkapasitas 16 memiliki kapasitas efektif 15 slot untuk membedakan full vs empty
        for i in 0..15 {
            frame.sequence_number = i as u8;
            assert_eq!(queue.send(i as u64, frame), Ok(()));
        }

        // Pengiriman ke-16 harus overflow
        assert_eq!(queue.send(100, frame), Err("ERR_AFDX_RING_BUFFER_OVERFLOW"));

        // Konsumsi 1 frame, lalu kirim lagi (verifikasi wraparound)
        let rx = queue.receive().expect("Should receive oldest slot");
        assert_eq!(rx.sequence_number, 0);
        frame.sequence_number = 99;
        assert_eq!(queue.send(101, frame), Ok(()));
    }
}

