//! ASL CCTV DMA: Zero-Copy Video Frame Pipeline & ISP Controller
//! ===============================================================
//! Menghubungkan sensor CMOS (MIPI-CSI) langsung ke Video Processing Unit (VPU).
//! Mendukung SoC pengawasan video (Novatek NT98xx, HiSilicon DaVinci, Ambarella CVflow).
//! Menjamin transmisi frame 4K/8K tanpa penyalinan memori oleh CPU (Zero-Copy Buffer Sharing).

#![no_std]

/// Format Piksel Frame Video
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VideoPixelFormat {
    NV12,       // Standar YUV420 semi-planar
    YUV422,
    RGB888,
    RawBayer12, // Output sensor mentah untuk ISP
}

/// Deskriptor Buffer Frame Video Hardware
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
    pub is_locked_by_vpu: bool,
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
            is_locked_by_vpu: false,
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
            is_locked_by_vpu: false,
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
