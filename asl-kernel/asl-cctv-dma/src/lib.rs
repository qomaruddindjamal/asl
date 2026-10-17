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

    /// Daftarkan buffer fisik untuk channel kamera tertentu
    pub fn setup_channel_buffer(
        &mut self,
        channel: u8,
        slot: usize,
        phys_y: u64,
        phys_uv: u64,
        width: u32,
        height: u32,
    ) -> Result<(), &'static str> {
        let ch = channel as usize;
        if ch >= MAX_VIDEO_CHANNELS || slot >= FRAMES_PER_CHANNEL_RING {
            return Err("ERR_INVALID_VIDEO_CHANNEL_OR_SLOT");
        }

        self.channels[ch][slot] = VideoFrameBuffer {
            frame_id: 0,
            channel_id: channel,
            physical_address_y: phys_y,
            physical_address_uv: phys_uv,
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut engine = VideoDmaPipelineEngine::new();
        assert!(engine
            .setup_channel_buffer(0, 0, 0x1000_0000, 0x1008_0000, 1920, 1080)
            .is_ok());
        assert!(engine
            .setup_channel_buffer(0, 1, 0x1010_0000, 0x1018_0000, 1920, 1080)
            .is_ok());

        // Notifikasi frame complete
        let f0 = engine.on_hardware_frame_complete(0);
        assert!(f0.is_some());
        assert_eq!(f0.unwrap().physical_address_y, 0x1000_0000);
        assert_eq!(engine.total_frames_captured, 1);
        assert_eq!(engine.active_write_index[0], 1);
    }
}
