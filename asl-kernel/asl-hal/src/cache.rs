//! ASL HAL: Cache Maintenance Operations & DMA Coherency Barriers
//! ====================================================================
//! Mendukung mitigasi DMA-CACHE-01 untuk prosesor non-coherent.
//! Menegakkan granularitas batas 64-byte hardware cache line.

pub const CACHE_LINE: usize = 64;

/// Trait Abstraksi Operasi Pemeliharaan Cache Hardware Universal
///
/// # Safety & Partial-Line Hazard:
/// Pada prosesor non-coherent, memanggil `dma_rx_invalidate` pada buffer yang tidak selaras
/// 64-byte atau berukuran bukan kelipatan 64-byte berpotensi membuang (menghapus) dirty cacheline
/// milik variabel atau data struktur tetangga.
/// Panggilan ini mensyaratkan gerbang ALIGN-01 terpenuhi atau penggunaan bounce-buffer.
pub trait CacheMaintenanceOps {
    /// Invalidate cache range sebelum CPU membaca hasil DMA RX dari memori (evict tanpa flush)
    fn dma_rx_invalidate(&self, buf: u64, len: usize);

    /// Clean cache range setelah CPU menulis buffer dan sebelum DMA TX berjalan (writeback dirty)
    fn dma_tx_clean(&self, buf: u64, len: usize);

    /// Clean & Invalidate bersamaan
    fn dma_clean_and_invalidate(&self, buf: u64, len: usize);

    /// Hardware memory synchronization barrier (serialization fence)
    fn memory_sync_barrier(&self);
}

/// Backend x86_64: Coherent No-op (Hardware Snooping & Bus Coherency)
pub struct CoherentNoopOps;

impl CacheMaintenanceOps for CoherentNoopOps {
    #[inline(always)]
    fn dma_rx_invalidate(&self, _buf: u64, _len: usize) {
        // x86_64 menjamin cache coherency secara hardware via bus snooping
    }

    #[inline(always)]
    fn dma_tx_clean(&self, _buf: u64, _len: usize) {
        // x86_64 menjamin cache coherency secara hardware
    }

    #[inline(always)]
    fn dma_clean_and_invalidate(&self, _buf: u64, _len: usize) {
        // No-op pada sistem hardware-coherent
    }

    #[inline(always)]
    fn memory_sync_barrier(&self) {
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }
}

/// Backend AArch64: ARMv8-A Cache Maintenance via Point of Coherency (PoC)
pub struct ArmV8CacheOps;

impl CacheMaintenanceOps for ArmV8CacheOps {
    fn dma_rx_invalidate(&self, buf: u64, len: usize) {
        #[cfg(target_arch = "aarch64")]
        unsafe {
            let mut addr = buf & !(CACHE_LINE as u64 - 1);
            let end = buf + len as u64;
            while addr < end {
                // Safety: DC CIVAC (Clean and Invalidate by VA to PoC) aman pada alamat memori valid
                core::arch::asm!("dc civac, {0}", in(reg) addr, options(nostack, preserves_flags));
                addr += CACHE_LINE as u64;
            }
            core::arch::asm!("dsb sy", options(nostack, preserves_flags));
        }
        #[cfg(not(target_arch = "aarch64"))]
        let _ = (buf, len);
    }

    fn dma_tx_clean(&self, buf: u64, len: usize) {
        #[cfg(target_arch = "aarch64")]
        unsafe {
            let mut addr = buf & !(CACHE_LINE as u64 - 1);
            let end = buf + len as u64;
            while addr < end {
                // Safety: DC CVAC (Clean by VA to PoC) menulis dirty line ke physical DRAM
                core::arch::asm!("dc cvac, {0}", in(reg) addr, options(nostack, preserves_flags));
                addr += CACHE_LINE as u64;
            }
            core::arch::asm!("dsb sy", options(nostack, preserves_flags));
        }
        #[cfg(not(target_arch = "aarch64"))]
        let _ = (buf, len);
    }

    fn dma_clean_and_invalidate(&self, buf: u64, len: usize) {
        self.dma_rx_invalidate(buf, len);
    }

    fn memory_sync_barrier(&self) {
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("dsb sy; isb", options(nostack, preserves_flags));
        }
    }
}

/// Backend AArch32: ARMv7-A Cache Maintenance via CP15
pub struct ArmV7CacheOps;

impl CacheMaintenanceOps for ArmV7CacheOps {
    fn dma_rx_invalidate(&self, buf: u64, len: usize) {
        #[cfg(target_arch = "arm")]
        unsafe {
            let mut addr = (buf as u32) & !(CACHE_LINE as u32 - 1);
            let end = (buf as u32) + len as u32;
            while addr < end {
                // Safety: CP15 c7 DCCIMVAC (Clean and Invalidate Data Cache Line by MVA to PoC)
                core::arch::asm!("mcr p15, 0, {0}, c7, c14, 1", in(reg) addr, options(nostack, preserves_flags));
                addr += CACHE_LINE as u32;
            }
            core::arch::asm!("dsb; isb", options(nostack, preserves_flags));
        }
        #[cfg(not(target_arch = "arm"))]
        let _ = (buf, len);
    }

    fn dma_tx_clean(&self, buf: u64, len: usize) {
        #[cfg(target_arch = "arm")]
        unsafe {
            let mut addr = (buf as u32) & !(CACHE_LINE as u32 - 1);
            let end = (buf as u32) + len as u32;
            while addr < end {
                // Safety: CP15 c7 DCCMVAC (Clean Data Cache Line by MVA to PoC)
                core::arch::asm!("mcr p15, 0, {0}, c7, c10, 1", in(reg) addr, options(nostack, preserves_flags));
                addr += CACHE_LINE as u32;
            }
            core::arch::asm!("dsb", options(nostack, preserves_flags));
        }
        #[cfg(not(target_arch = "arm"))]
        let _ = (buf, len);
    }

    fn dma_clean_and_invalidate(&self, buf: u64, len: usize) {
        self.dma_rx_invalidate(buf, len);
    }

    fn memory_sync_barrier(&self) {
        #[cfg(target_arch = "arm")]
        unsafe {
            core::arch::asm!("dsb; isb", options(nostack, preserves_flags));
        }
    }
}

/// Backend RISC-V: Zicbom Cache Management Operation Extension
pub struct RiscvZicbomOps;

impl CacheMaintenanceOps for RiscvZicbomOps {
    fn dma_rx_invalidate(&self, buf: u64, len: usize) {
        #[cfg(all(target_arch = "riscv64", target_feature = "zicbom"))]
        unsafe {
            let mut addr = buf & !(CACHE_LINE as u64 - 1);
            let end = buf + len as u64;
            while addr < end {
                // Safety: cbo.inval membatalkan cache block
                core::arch::asm!("cbo.inval ({0})", in(reg) addr, options(nostack, preserves_flags));
                addr += CACHE_LINE as u64;
            }
            core::arch::asm!("fence rw, io", options(nostack, preserves_flags));
        }
        #[cfg(not(all(target_arch = "riscv64", target_feature = "zicbom")))]
        let _ = (buf, len);
    }

    fn dma_tx_clean(&self, buf: u64, len: usize) {
        #[cfg(all(target_arch = "riscv64", target_feature = "zicbom"))]
        unsafe {
            let mut addr = buf & !(CACHE_LINE as u64 - 1);
            let end = buf + len as u64;
            while addr < end {
                // Safety: cbo.clean menulis dirty cache block ke memory
                core::arch::asm!("cbo.clean ({0})", in(reg) addr, options(nostack, preserves_flags));
                addr += CACHE_LINE as u64;
            }
            core::arch::asm!("fence rw, io", options(nostack, preserves_flags));
        }
        #[cfg(not(all(target_arch = "riscv64", target_feature = "zicbom")))]
        let _ = (buf, len);
    }

    fn dma_clean_and_invalidate(&self, buf: u64, len: usize) {
        #[cfg(all(target_arch = "riscv64", target_feature = "zicbom"))]
        unsafe {
            let mut addr = buf & !(CACHE_LINE as u64 - 1);
            let end = buf + len as u64;
            while addr < end {
                core::arch::asm!("cbo.flush ({0})", in(reg) addr, options(nostack, preserves_flags));
                addr += CACHE_LINE as u64;
            }
            core::arch::asm!("fence rw, io", options(nostack, preserves_flags));
        }
        #[cfg(not(all(target_arch = "riscv64", target_feature = "zicbom")))]
        let _ = (buf, len);
    }

    fn memory_sync_barrier(&self) {
        #[cfg(target_arch = "riscv64")]
        unsafe {
            core::arch::asm!("fence rw, io", options(nostack, preserves_flags));
        }
    }
}

// -----------------------------------------------------------------------------
// BOUNCE BUFFER POLICY & KEPEMILIKAN SATU-LINE-SATU-AKTOR
// -----------------------------------------------------------------------------

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BounceDecision {
    DirectZeroCopy,
    RequiresBounceBuffer,
}

pub struct BounceBufferPolicy;

impl BounceBufferPolicy {
    /// Mengevaluasi apakah transfer DMA dapat zero-copy langsung atau wajib bounce buffer
    #[inline(always)]
    pub fn evaluate(addr: u64, len: usize) -> BounceDecision {
        // Alamat dan panjang wajib kelipatan persis 64-byte untuk zero-copy langsung
        if (addr & (CACHE_LINE as u64 - 1)) == 0 && (len & (CACHE_LINE - 1)) == 0 {
            BounceDecision::DirectZeroCopy
        } else {
            BounceDecision::RequiresBounceBuffer
        }
    }

    /// Menghitung ukuran buffer bounce yang dibulatkan ke atas ke batas CACHE_LINE terdekat
    #[inline(always)]
    pub fn round_up_cache_line(len: usize) -> usize {
        (len + (CACHE_LINE - 1)) & !(CACHE_LINE - 1)
    }
}

/// Struktur Bounce Buffer terisolasi pada batas 64-byte
#[repr(C, align(64))]
pub struct IsolatedBounceBuffer<const CAPACITY: usize = 2048> {
    pub data: [u8; CAPACITY],
}

impl<const CAPACITY: usize> IsolatedBounceBuffer<CAPACITY> {
    pub const fn new() -> Self {
        Self { data: [0u8; CAPACITY] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounce_buffer_policy_decisions() {
        // Aligned 64-byte address dan length -> DirectZeroCopy
        assert_eq!(BounceBufferPolicy::evaluate(0x1000_0000, 128), BounceDecision::DirectZeroCopy);
        assert_eq!(BounceBufferPolicy::evaluate(0x1000_0040, 64), BounceDecision::DirectZeroCopy);

        // Misaligned address (offset 1) -> RequiresBounceBuffer
        assert_eq!(BounceBufferPolicy::evaluate(0x1000_0001, 128), BounceDecision::RequiresBounceBuffer);
        // Misaligned length (panjang ganjil misal 1518 byte frame ethernet) -> RequiresBounceBuffer
        assert_eq!(BounceBufferPolicy::evaluate(0x1000_0000, 1518), BounceDecision::RequiresBounceBuffer);
    }

    #[test]
    fn test_bounce_buffer_round_up_cache_line() {
        assert_eq!(BounceBufferPolicy::round_up_cache_line(1518), 1536); // 1536 kelipatan 64
        assert_eq!(BounceBufferPolicy::round_up_cache_line(64), 64);
        assert_eq!(BounceBufferPolicy::round_up_cache_line(1), 64);
    }

    #[test]
    fn test_isolated_bounce_buffer_alignment_64() {
        let buf: IsolatedBounceBuffer<2048> = IsolatedBounceBuffer::new();
        assert_eq!(core::mem::align_of_val(&buf), 64);
    }
}
