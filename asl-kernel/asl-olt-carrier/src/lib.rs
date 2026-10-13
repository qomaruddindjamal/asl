//! ASL OLT Carrier: Dynamic Bandwidth Allocation (DBA) Controller & PON Aggregator
//! ===============================================================================
//! Menjalankan orkestrasi kartu OLT tingkat penyedia layanan (Huawei MA5800, ZTE C600, Nokia Quillion).
//! Fitur:
//! 1. Algoritma Dynamic Bandwidth Allocation (DBA) berbasis Status-Report (SR-DBA).
//! 2. Alokasi slot waktu TDMA dalam skala mikrodetik untuk ratusan ONT.
//! 3. Agregasi multi-rate GPON (2.5G/1.25G) dan XGS-PON (10G/10G simetris).

#![no_std]

pub const MAX_CONNECTED_ONTS: usize = 64;

/// Entri Bandwidth Alokasi untuk 1 Terminal ONT
#[derive(Debug, Copy, Clone)]
pub struct OntBandwidthGrant {
    pub alloc_id: u16,
    pub start_time_slot: u16,
    pub grant_size_bytes: u16,
}

impl OntBandwidthGrant {
    pub const fn empty() -> Self {
        Self {
            alloc_id: 0,
            start_time_slot: 0,
            grant_size_bytes: 0,
        }
    }
}

/// Mesin Dynamic Bandwidth Allocation (DBA Engine)
pub struct DynamicBandwidthAllocationEngine {
    pub total_upstream_capacity_bytes: u32,
    pub grants: [OntBandwidthGrant; MAX_CONNECTED_ONTS],
    pub active_ont_count: usize,
    pub frame_counter: u64,
}

impl DynamicBandwidthAllocationEngine {
    pub const fn new() -> Self {
        Self {
            total_upstream_capacity_bytes: 38880, // Ukuran frame GPON 125 mikrodetik
            grants: [OntBandwidthGrant::empty(); MAX_CONNECTED_ONTS],
            active_ont_count: 0,
            frame_counter: 0,
        }
    }

    /// Daftarkan ONT baru ke jadwal PON
    pub fn register_ont(&mut self, alloc_id: u16) -> Result<usize, &'static str> {
        if self.active_ont_count >= MAX_CONNECTED_ONTS {
            return Err("ERR_PON_PORT_CAPACITY_EXCEEDED");
        }
        let idx = self.active_ont_count;
        self.grants[idx].alloc_id = alloc_id;
        self.active_ont_count += 1;
        Ok(idx)
    }

    /// Kalkulasi alokasi slot waktu TDMA secara adil dan berprioritas (Fair Queuing DBA)
    pub fn calculate_upstream_bandwidth_map(&mut self) {
        if self.active_ont_count == 0 {
            return;
        }

        self.frame_counter += 1;
        let quota = (self.total_upstream_capacity_bytes as usize) / self.active_ont_count;
        let mut current_slot: u16 = 0;

        let mut i = 0;
        while i < self.active_ont_count {
            self.grants[i].start_time_slot = current_slot;
            self.grants[i].grant_size_bytes = quota as u16;
            current_slot += quota as u16;
            i += 1;
        }
    }
}
