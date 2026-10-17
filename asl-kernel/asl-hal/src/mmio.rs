//! ASL HAL: Hardware MMIO Register Volatile Wrapper (AG-01 Mitigation)
//! ====================================================================
//! Menjamin bahwa pembacaan dan penulisan register hardware memory-mapped (MMIO)
//! tidak di-elide, di-reorder secara sembarangan, atau di-cache oleh compiler pass LLVM.
//!
//! # Aturan Kesatuan Memori (DO-178C DAL-A):
//! - `MmioRegister` HANYA digunakan untuk register MMIO dan bit status yang ditulis eksklusif
//!   oleh perangkat keras eksternal (misal: bit DMA DONE, UART TX/RX FIFO status).
//! - Variabel komunikasi antar-core CPU WAJIB tetap menggunakan AtomicU32/AtomicUsize dengan
//!   memory ordering Acquire/Release (SEC-03). DILARANG mencampur volatile dan atomic pada sel yang sama.

use core::marker::PhantomData;

/// Pembungkus Volatile Bertipe Aman untuk Register MMIO Hardware
#[repr(transparent)]
pub struct MmioRegister<T: Copy> {
    address: usize,
    _marker: PhantomData<T>,
}

impl<T: Copy> MmioRegister<T> {
    /// Membuat accessor register MMIO dari alamat memori fisik / virtual yang dipetakan
    ///
    /// # Safety
    /// Pemanggil menjamin bahwa `address` adalah pointer valid yang terpetakan ke kontrol register MMIO
    /// dan memenuhi persyaratan keselarasan memori (memory alignment) tipe `T`.
    #[inline(always)]
    pub const unsafe fn from_address(address: usize) -> Self {
        Self {
            address,
            _marker: PhantomData,
        }
    }

    /// Membaca nilai register secara volatile tanpa optimasi elision compiler
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { core::ptr::read_volatile(self.address as *const T) }
    }

    /// Menulis nilai ke register secara volatile
    #[inline(always)]
    pub fn write(&self, value: T) {
        unsafe { core::ptr::write_volatile(self.address as *mut T, value) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mmio_register_read_write() {
        let mut simulated_hardware_reg: u32 = 0x1234_5678;
        let reg = unsafe {
            MmioRegister::<u32>::from_address(&mut simulated_hardware_reg as *mut u32 as usize)
        };

        assert_eq!(reg.read(), 0x1234_5678);

        reg.write(0xDEAD_BEEF);
        assert_eq!(simulated_hardware_reg, 0xDEAD_BEEF);
        assert_eq!(reg.read(), 0xDEAD_BEEF);
    }
}
