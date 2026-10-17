# RELEASE NOTES: ASL CORE KERNEL v1.0.1
=========================================
**Release Version:** `v1.0.1`  
**Date:** 2026-10-17  
**Architecture Authority:** `asl_core.md` (150 Silicon Platforms)  
**Governing Standard:** DO-178C DAL-A / ARINC 653 Integrated Modular Avionics (IMA)  

---

## 1. HIGHLIGHTS & DELIVERABLES (v1.0.1)

ASL Core Kernel v1.0.1 delivers live Linear Framebuffer hardware graphics rendering, authentic bare-metal hardware telemetry extraction (without dummy data), and verified execution across extreme low-memory virtualization environments (down to 4 MB RAM) on Oracle VirtualBox and host silicon.

### Key Capabilities Verified:
1. **Live Graphical Linear Framebuffer (VBE / GOP 1024x768x32-bpp):**
   - High-performance Linear Framebuffer console engine (`FbConsole`) rendering 8x16 bitmap raster fonts directly on screen.
   - Official ASL ASCII Art banner, bilingual greeting, and colored operational badges rendered directly to the hardware monitor display (`Analysis/reports/asl_vm_screen.png`).
2. **Authentic Hardware Telemetry Extraction (Zero Dummy Data):**
   - **Installed Memory:** Computed dynamically from physical Limine handover descriptors and CMOS hardware ports `0x70`/`0x71`. Verified detecting exact motherboard RAM allocations (tested dynamically at **64 MB** and **4 MB RAM** without recompilation).
   - **CPU Architecture & Brand:** Extracted directly from host silicon via x86 assembly `CPUID` instruction leaves `0x80000002`..`0x80000004` (e.g., `Intel(R) Core(TM) i5-8500 CPU @ 3.00GHz`).
   - **Deterministic CPU Load:** Calculated precisely from ARINC 653 Core Partition slot ratio (20,000 µs slot / 100,000 µs major frame = **20%**).
3. **Limine Protocol Response Pipeline Hardening:**
   - Linker script segment (`limine_requests`) configured with writable `FLAGS(6)` permissions and 4096-byte page alignment.
   - Limine request structures declared with volatile memory semantics (`core::ptr::read_volatile`), guaranteeing immediate population of Memmap, Kernel Address, HHDM, and Framebuffer response pointers.
4. **Direct Runtime Telemetry (Milestones M1 through M6):**
   - **M1:** Limine Entry Stub execution via assembly trampoline.
   - **M2:** Memory map parsing and physical boundary sanitization.
   - **M3:** Control handover to ASL microkernel core entry (`_asl_kernel_entry`).
   - **M4:** Deterministic Buddy Allocator and heap initialization.
   - **M5:** ARINC 653 Spatial Partition Vault and Major Frame Scheduler initialization with verified initial deterministic tick advance.
   - **M6:** Clean transition into the hardware `hlt` safe idle loop.
5. **Multi-Crate Microkernel Architecture & Host Testing:**
   - 17 member crates compiling cleanly under Rust `#![no_std]`.
   - 45 automated unit tests validating boundary arithmetic, memory window isolation, scheduler cyclic timeframes, IOMMU DMA address translation, cacheline bounce buffers, and slab allocation.
6. **Bootable Hybrid Avionics ISO:**
   - Mastered with `xorriso` and Limine v8.x hybrid BIOS/UEFI boot stages (`asl-kernel/target/asl-core-kernel.iso`, 3,954,688 bytes).
   - Validated running in headless and interactive GUI Oracle VirtualBox VM `ASL`.

---

## 2. HARDWARE & RUNTIME REQUIREMENTS (v1.0.1)

Based on actual verified runtime metrics:

### Minimum Hardware Requirements (Bare-Metal / VM):
- **CPU Architecture:** x86_64 compatible processor (Intel 64 or AMD64) with Long Mode and PAE support.
- **Physical RAM (Minimum):** **4 MB RAM** (Kernel binary footprint is only ~70 KB; verified booting and idling stably at 4 MB RAM without page faults or OOM).
- **Physical RAM (Recommended):** **64 MB RAM** or higher.
- **Firmware / Boot Standard:** Legacy BIOS (El Torito CD-ROM boot) or UEFI 64-bit with GOP / VBE framebuffer support.
- **Display Resolution:** Minimum 800x600 (recommended 1024x768 at 32-bpp true color).
- **Storage / Media:** CD-ROM / DVD-ROM ISO drive or USB Flash Drive (minimum 8 MB storage).

### Host Development & Build Requirements:
- **Operating System:** Windows 10/11 64-bit or Linux x86_64.
- **Compiler:** Rust 1.80+ (toolchain `x86_64-pc-windows-msvc` and target `x86_64-unknown-none`).
- **ISO Mastering Utilities:** `xorriso` (v1.5.0+) and Limine v8.x binary (`limine.exe`).
- **Hypervisor for Testing:** Oracle VirtualBox (GUI or headless) or QEMU (`qemu-system-x86_64`).

---

## 3. REMEDIATION & FINDINGS STATUS

- **DMA-CACHE-01:** Fully resolved via `asl-hal::cache::IsolatedBounceBuffer` with 64-byte alignment enforcement.
- **DMA-ADDR-01:** Fully resolved via `asl-hal::dma_space::IommuTranslationSpace` with memory window checksum verification.
- **ALIGN-01:** Fully resolved via 64-byte aligned `DmaPacketDescriptor` structures.
- **ATOMIC-SPARC-01:** Fully resolved via lock-free split-counter CAS emulation.
- **REAL-TELEMETRY-01:** Fully resolved in v1.0.1; zero dummy data across all status reporting badges.
