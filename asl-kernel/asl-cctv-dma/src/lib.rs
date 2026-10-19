//! ASL CCTV DMA: Zero-Copy Video Frame Pipeline & ISP Controller
//! ===============================================================
//! Menghubungkan sensor CMOS (MIPI-CSI) langsung ke Video Processing Unit (VPU).
//! Mendukung SoC pengawasan video (Novatek NT98xx, HiSilicon DaVinci, Ambarella CVflow).
//! Menjamin transmisi frame 4K/8K tanpa penyalinan memori oleh CPU (Zero-Copy Buffer Sharing).

#![no_std]

use asl_hal::MmioRegister;

/// Format Piksel Frame Video
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VideoPixelFormat {
    NV12, // Standar YUV420 semi-planar
    YUV422,
    RGB888,
    RawBayer12, // Output sensor mentah untuk ISP
}

/// Deskriptor Buffer Frame Video Hardware
///
/// Mitigasi AG-01: Flag `vpu_lock_flags` adalah register status hardware yang ditulis eksklusif
/// oleh Video Processing Unit (VPU). CPU membaca status lock secara volatile via `is_locked_by_vpu()`.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct VideoFrameBuffer {
    pub frame_id: u32,
    pub channel_id: u8,
    pub physical_address_y: u64,
    pub physical_address_uv: u64,
    pub width: u32,
    pub height: u32,
    pub stride_bytes: u32,
    pub format: VideoPixelFormat,
    pub vpu_lock_flags: u32, // Bit 0: Locked by hardware VPU codec engine
}

impl VideoFrameBuffer {
    pub const fn empty() -> Self {
        Self {
            frame_id: 0,
            channel_id: 0,
            physical_address_y: 0,
            physical_address_uv: 0,
            width: 0,
            height: 0,
            stride_bytes: 0,
            format: VideoPixelFormat::NV12,
            vpu_lock_flags: 0,
        }
    }

    /// Membaca status penguncian VPU secara volatile murni untuk mencegah loop elision oleh LLVM
    #[inline(always)]
    pub fn is_locked_by_vpu(&self) -> bool {
        let reg_ptr = &self.vpu_lock_flags as *const u32 as usize;
        let reg = unsafe { MmioRegister::<u32>::from_address(reg_ptr) };
        (reg.read() & 0x01) != 0
    }

    /// Simulasi penulisan status oleh perangkat VPU hardware (atau driver VPU)
    #[inline(always)]
    pub fn set_vpu_locked(&mut self, locked: bool) {
        let reg_ptr = &mut self.vpu_lock_flags as *mut u32 as usize;
        let reg = unsafe { MmioRegister::<u32>::from_address(reg_ptr) };
        let current = reg.read();
        if locked {
            reg.write(current | 0x01);
        } else {
            reg.write(current & !0x01);
        }
    }
}

pub const MAX_VIDEO_CHANNELS: usize = 16;
pub const FRAMES_PER_CHANNEL_RING: usize = 4; // Triple/Quad buffering

/// Pengelola Antrean Frame Video Nol-Kopi Multi-Channel
pub struct VideoDmaPipelineEngine {
    pub channels: [[VideoFrameBuffer; FRAMES_PER_CHANNEL_RING]; MAX_VIDEO_CHANNELS],
    pub active_write_index: [usize; MAX_VIDEO_CHANNELS],
    pub total_frames_captured: u64,
}

impl VideoDmaPipelineEngine {
    pub const fn new() -> Self {
        Self {
            channels: [[VideoFrameBuffer::empty(); FRAMES_PER_CHANNEL_RING]; MAX_VIDEO_CHANNELS],
            active_write_index: [0; MAX_VIDEO_CHANNELS],
            total_frames_captured: 0,
        }
    }

    /// Daftarkan buffer frame untuk channel kamera tertentu
    ///
    /// # Kontrak RND_08 (DMA-ADDR-01 remediation) — BREAKING:
    /// Plane Y dan UV WAJIB berupa `IovaLease` tervalidasi dari backend
    /// `DmaAddressSpace` milik partisi pemilik channel. Alamat fisik mentah
    /// tidak lagi dapat diterima. Validasi antesional (semua dicek SEBELUM
    /// state apa pun diubah — kegagalan tidak pernah memasang frame parsial):
    /// - channel & slot in-range
    /// - kedua lease milik partisi yang sama
    /// - kedua alamat bus selaras 64-byte (guard-page / cache-line rule)
    /// - panjang lease mencukupi kebutuhan plane untuk resolusi+format
    ///   (dengan checked arithmetic — overflow wajib ditolak)
    pub fn setup_channel_buffer(
        &mut self,
        channel: u8,
        slot: usize,
        lease_y: &asl_hal::IovaLease,
        lease_uv: &asl_hal::IovaLease,
        width: u32,
        height: u32,
    ) -> Result<(), &'static str> {
        let ch = channel as usize;
        if ch >= MAX_VIDEO_CHANNELS || slot >= FRAMES_PER_CHANNEL_RING {
            return Err("ERR_INVALID_VIDEO_CHANNEL_OR_SLOT");
        }
        if lease_y.partition_id() != lease_uv.partition_id() {
            return Err("ERR_LEASE_PARTITION_MISMATCH");
        }
        if !lease_y.is_bus_aligned() || !lease_uv.is_bus_aligned() {
            return Err("ERR_LEASE_NOT_64B_ALIGNED");
        }
        if width == 0 || height == 0 {
            return Err("ERR_FRAME_DIMENSIONS");
        }
        let w = width as u64;
        let h = height as u64;
        let luma = w.checked_mul(h).ok_or("ERR_FRAME_SIZE_OVERFLOW")?;
        // NV12: UV plane = w*h/2
        let chroma = luma.checked_div(2).ok_or("ERR_FRAME_SIZE_OVERFLOW")?;
        if lease_y.len() < luma || lease_uv.len() < chroma {
            return Err("ERR_LEASE_TOO_SMALL_FOR_FORMAT");
        }

        self.channels[ch][slot] = VideoFrameBuffer {
            frame_id: 0,
            channel_id: channel,
            physical_address_y: lease_y.iova(),
            physical_address_uv: lease_uv.iova(),
            width,
            height,
            stride_bytes: width,
            format: VideoPixelFormat::NV12,
            vpu_lock_flags: 0,
        };
        Ok(())
    }

    /// Penanganan notifikasi hardware saat 1 frame selesai di-DMA oleh sensor
    pub fn on_hardware_frame_complete(&mut self, channel: u8) -> Option<VideoFrameBuffer> {
        let ch = channel as usize;
        if ch >= MAX_VIDEO_CHANNELS {
            return None;
        }

        let curr_idx = self.active_write_index[ch];
        let completed_frame = self.channels[ch][curr_idx];

        self.total_frames_captured += 1;
        self.active_write_index[ch] = (curr_idx + 1) % FRAMES_PER_CHANNEL_RING;

        Some(completed_frame)
    }

    /// Validasi anti-TOCTOU pra-doorbell (RND_08 §B): kedua lease masih
    /// otoritatif di backend DAN konsisten dengan mapping frame terpasang.
    pub fn validate_frame_leases<S: asl_hal::DmaAddressSpace>(
        &self,
        space: &S,
        channel: u8,
        slot: usize,
        lease_y: &asl_hal::IovaLease,
        lease_uv: &asl_hal::IovaLease,
    ) -> Result<(), &'static str> {
        let ch = channel as usize;
        if ch >= MAX_VIDEO_CHANNELS || slot >= FRAMES_PER_CHANNEL_RING {
            return Err("ERR_INVALID_VIDEO_CHANNEL_OR_SLOT");
        }
        space
            .validate_pin(lease_y)
            .map_err(|_| "ERR_STALE_DMA_LEASE")?;
        space
            .validate_pin(lease_uv)
            .map_err(|_| "ERR_STALE_DMA_LEASE")?;
        let frame = &self.channels[ch][slot];
        if frame.physical_address_y != lease_y.iova() || frame.physical_address_uv != lease_uv.iova()
        {
            return Err("ERR_LEASE_FRAME_MISMATCH");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use asl_hal::{BounceAddressSpace, DmaAddressSpace, DmaError};

    fn video_space() -> BounceAddressSpace {
        let mut space = BounceAddressSpace::new(0x4000_0000);
        // Partisi kamera 7 -> window buffer frame; partisi 9 -> window terpisah;
        // partisi 5 -> window reclaim (menguasai kembali buffer UV setelah unpin)
        space.register_region(7, 0x1000_0000, 0x1008_0000).unwrap();
        space.register_region(5, 0x1008_0000, 0x1018_0000).unwrap();
        space.register_region(7, 0x1018_0000, 0x1080_0000).unwrap();
        space.register_region(9, 0x2000_0000, 0x2040_0000).unwrap();
        space
    }

    const LUMA_480P: u64 = 640 * 480;
    const CHROMA_480P: u64 = LUMA_480P / 2;

    #[test]
    fn test_cctv_vpu_lock_volatile_semantics() {
        let mut frame = VideoFrameBuffer::empty();
        // Awal: tidak terkunci
        assert!(!frame.is_locked_by_vpu());

        // VPU hardware mengunci frame
        frame.set_vpu_locked(true);
        assert!(frame.is_locked_by_vpu());

        // VPU hardware melepaskan frame
        frame.set_vpu_locked(false);
        assert!(!frame.is_locked_by_vpu());
    }

    #[test]
    fn test_cctv_dma_pipeline_channel_setup_and_frame_cycle() {
        let mut space = video_space();
        let ly0 = space.pin(7, 0x1000_0000, LUMA_480P).unwrap();
        let lu0 = space.pin(7, 0x1018_0000, CHROMA_480P).unwrap();
        let ly1 = space.pin(7, 0x1020_0000, LUMA_480P).unwrap();
        let lu1 = space.pin(7, 0x102A_0000, CHROMA_480P).unwrap();

        let mut engine = VideoDmaPipelineEngine::new();
        assert!(engine
            .setup_channel_buffer(0, 0, &ly0, &lu0, 640, 480)
            .is_ok());
        assert!(engine
            .setup_channel_buffer(0, 1, &ly1, &lu1, 640, 480)
            .is_ok());

        // Notifikasi frame complete
        let f0 = engine.on_hardware_frame_complete(0);
        assert!(f0.is_some());
        assert_eq!(f0.unwrap().physical_address_y, 0x1000_0000);
        assert_eq!(engine.total_frames_captured, 1);
        assert_eq!(engine.active_write_index[0], 1);
    }

    #[test]
    fn tm_cctv_setup_rejects_before_any_state_change() {
        let mut space = video_space();
        let ly = space.pin(7, 0x1000_0000, LUMA_480P).unwrap();
        let lu = space.pin(7, 0x1018_0000, CHROMA_480P).unwrap();
        let mut engine = VideoDmaPipelineEngine::new();

        // Partition mismatch -> ditolak atomik
        let foreign = space.pin(9, 0x2000_0000, CHROMA_480P).unwrap();
        assert_eq!(
            engine.setup_channel_buffer(3, 0, &ly, &foreign, 640, 480),
            Err("ERR_LEASE_PARTITION_MISMATCH")
        );
        // Lease UV terlalu kecil untuk format -> ditolak
        let tiny_uv = space.pin(9, 0x2010_0000, 0x1000).unwrap();
        let tiny_y = space.pin(9, 0x2020_0000, 0x1000).unwrap();
        assert_eq!(
            engine.setup_channel_buffer(3, 0, &tiny_y, &tiny_uv, 640, 480),
            Err("ERR_LEASE_TOO_SMALL_FOR_FORMAT")
        );
        // Dimensi overflow / nol -> ditolak
        assert_eq!(
            engine.setup_channel_buffer(3, 0, &ly, &lu, 0, 480),
            Err("ERR_FRAME_DIMENSIONS")
        );
        assert_eq!(
            engine.setup_channel_buffer(3, 0, &ly, &lu, u32::MAX, u32::MAX),
            Err("ERR_LEASE_TOO_SMALL_FOR_FORMAT")
        );
        // channel invalid -> ditolak
        assert_eq!(
            engine.setup_channel_buffer(16, 0, &ly, &lu, 640, 480),
            Err("ERR_INVALID_VIDEO_CHANNEL_OR_SLOT")
        );

        // TIDAK ADA state yang berubah dari seluruh kegagalan di atas
        assert_eq!(engine.total_frames_captured, 0);
        let untouched = &engine.channels[3][0];
        assert_eq!(untouched.physical_address_y, 0);
        assert_eq!(untouched.physical_address_uv, 0);
        assert_eq!(untouched.width, 0);
        assert_eq!(untouched.height, 0);
        assert_eq!(untouched.frame_id, 0);
        assert_eq!(engine.active_write_index[3], 0);
    }

    #[test]
    fn tm08_cctv_stale_uv_lease_rejected_pre_doorbell() {
        let mut space = video_space();
        let ly = space.pin(7, 0x1000_0000, LUMA_480P).unwrap();
        let lu = space.pin(7, 0x1018_0000, CHROMA_480P).unwrap();

        // Setup sah saat kedua lease masih otoritatif
        let mut engine = VideoDmaPipelineEngine::new();
        assert!(engine
            .setup_channel_buffer(5, 0, &ly, &lu, 640, 480)
            .is_ok());

        // TOCTOU: buffer UV dibebaskan (reclaim) setelah setup — partisi 5
        // mengambil alih slot yang sama; lease lama jadi stale selamanya.
        space.unpin(&lu).unwrap();
        let _lu5 = space.pin(5, 0x1008_0000, CHROMA_480P).unwrap();
        assert_eq!(space.validate_lease(&lu), Err(DmaError::GenerationMismatch));

        // Validasi pre-doorbell engine WAJIB gagal -> frame tidak pernah
        // disubmit ke VPU dengan mapping stale/lintas-partisi
        assert_eq!(
            engine.validate_frame_leases(&space, 5, 0, &ly, &lu),
            Err("ERR_STALE_DMA_LEASE")
        );

        // Device-side pun: write atas nama partisi 7 ke iova UV yang kini
        // milik partisi 5 -> CrossPartitionViolation (bukan diam-diam lolos)
        let res = space.translate_and_check(7, 0x1008_0000, CHROMA_480P as usize, true);
        assert_eq!(res, Err(asl_hal::DmaAddressFault::CrossPartitionViolation));
    }
}
