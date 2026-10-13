//! ASL ONT GPON: GEM Framing, T-CONT Engine & Hardware NAT Driver
//! ==============================================================
//! Mengabstraksikan silikon modem fiber optik (ZTE ZX279128S, HiSilicon SD5116).
//! Menangani:
//! 1. GEM (GPON Encapsulation Method) framing (Port-ID mapping).
//! 2. T-CONT (Transmission Container) QoS queueing upstream.
//! 3. HNAT (Hardware Network Address Translation) line-rate routing.

#![no_std]

/// Status Transceiver Optik BOSA (Bidirectional Optical Sub-Assembly)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OpticalLinkState {
    LossOfSignal,     // Tidak ada cahaya terdeteksi (Kabel terputus)
    O1_InitialState,
    O2_StandbyState,
    O3_SerialNumState,
    O4_RangingState,
    O5_OperationState, // Terkoneksi stabil ke OLT, siap kirim data
}

/// Header Frame GEM (GPON Encapsulation Method) standar ITU-T G.984.3
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct GemFrameHeader {
    pub payload_length: u16, // 12-bit PLI (Payload Length Indicator)
    pub port_id: u16,        // 12-bit GEM Port ID
    pub flags: u8,
    pub header_error_control: u8, // HEC CRC
}

impl GemFrameHeader {
    pub const fn new(port_id: u16, len: u16) -> Self {
        Self {
            payload_length: len & 0x0FFF,
            port_id: port_id & 0x0FFF,
            flags: 0,
            header_error_control: 0,
        }
    }
}

/// Transmission Container (T-CONT) untuk Alokasi Upstream Bandwidth
pub struct TContQueue {
    pub alloc_id: u16,
    pub queue_depth_bytes: u32,
    pub transmitted_bytes: u64,
}

impl TContQueue {
    pub const fn new(alloc_id: u16) -> Self {
        Self {
            alloc_id,
            queue_depth_bytes: 0,
            transmitted_bytes: 0,
        }
    }
}

pub const MAX_TCONT_QUEUES: usize = 8;

/// Driver Silikon ONT FTTH
pub struct GponOntDriver {
    pub link_state: OpticalLinkState,
    pub rx_optical_power_dbm: i16, // misal -1900 = -19.00 dBm
    pub tconts: [TContQueue; MAX_TCONT_QUEUES],
    pub hardware_nat_enabled: bool,
}

impl GponOntDriver {
    pub const fn new() -> Self {
        Self {
            link_state: OpticalLinkState::LossOfSignal,
            rx_optical_power_dbm: -4000,
            tconts: [
                TContQueue::new(256), TContQueue::new(257),
                TContQueue::new(258), TContQueue::new(259),
                TContQueue::new(260), TContQueue::new(261),
                TContQueue::new(262), TContQueue::new(263),
            ],
            hardware_nat_enabled: true,
        }
    }

    /// Update status link optik dari sinyal SerDes
    pub fn update_signal(&mut self, power_dbm: i16) {
        self.rx_optical_power_dbm = power_dbm;
        if power_dbm > -2800 && power_dbm < -800 {
            self.link_state = OpticalLinkState::O5_OperationState;
        } else {
            self.link_state = OpticalLinkState::LossOfSignal;
        }
    }

    /// Mock Hardware NAT (HNAT) Acceleration Stub
    ///
    /// CATATAN ARSITEKTUR & STATUS IMPLEMENTASI:
    /// Metode ini saat ini merupakan **STUB MOCK UNTUK SIMULASI PIPELINE FIBER**.
    /// Pada platform silikon produksi riil (seperti ZTE Sanechips ZX279128S atau HiSilicon SD5116),
    /// akselerasi NAT dieksekusi langsung oleh mesin routing silikon on-chip melalui tabel 5-tuple flow hardware
    /// (Source IP, Destination IP, Source Port, Destination Port, Protocol) tanpa intervensi CPU core.
    ///
    /// Untuk pengujian host/simulasi, metode ini mengembalikan pasangan IP dengan flag simulasi aktif,
    /// atau identitas asli saat `hardware_nat_enabled` bernilai false.
    #[inline]
    pub fn accelerate_hnat(&self, src_ip: u32, dst_ip: u32) -> (u32, u32) {
        if self.hardware_nat_enabled {
            // Simulasi translasi mock: toggle bit kontrol LSB untuk pengujian pipeline simulasi
            (src_ip ^ 0x00000001, dst_ip)
        } else {
            (src_ip, dst_ip)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpon_optical_signal_states() {
        let mut ont = GponOntDriver::new();
        assert_eq!(ont.link_state, OpticalLinkState::LossOfSignal);

        // Daya sinyal normal (-19.00 dBm) memicu O5 Operation State
        ont.update_signal(-1900);
        assert_eq!(ont.link_state, OpticalLinkState::O5_OperationState);

        // Kabel terputus / sinyal terlalu lemah (-35.00 dBm)
        ont.update_signal(-3500);
        assert_eq!(ont.link_state, OpticalLinkState::LossOfSignal);
    }

    #[test]
    fn test_gpon_mock_hnat_behavior() {
        let mut ont = GponOntDriver::new();
        let src = 0x0A000001; // 10.0.0.1
        let dst = 0xC0A80101; // 192.168.1.1

        // Saat hardware_nat_enabled = true (default)
        let (trans_src, trans_dst) = ont.accelerate_hnat(src, dst);
        assert_eq!(trans_src, src ^ 0x00000001);
        assert_eq!(trans_dst, dst);

        // Saat disabled, IP dikembalikan tanpa modifikasi
        ont.hardware_nat_enabled = false;
        let (orig_src, orig_dst) = ont.accelerate_hnat(src, dst);
        assert_eq!(orig_src, src);
        assert_eq!(orig_dst, dst);
    }
}

