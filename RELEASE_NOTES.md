# RELEASE NOTES: ASL CORE KERNEL v1.0.1
=========================================
**Release Version:** `v1.0.1`  
**Date:** 2026-10-17  
**Architecture Authority:** `asl_core.md` (150 Silicon Platforms)  
**Governing Standard:** DO-178C DAL-A / ARINC 653 Integrated Modular Avionics (IMA)  

---

## 1. HIGHLIGHTS & DELIVERABLES (v1.0.1)

ASL Core Kernel v1.0.1 delivers live Linear Framebuffer hardware graphics rendering and full bare-metal execution verification under Oracle VirtualBox hardware virtualization.

### Key Capabilities Verified:
1. **Live Graphical Linear Framebuffer (VBE / GOP 1024x768x32-bpp):**
   - High-performance Linear Framebuffer console engine (`FbConsole`) rendering 8x16 bitmap raster fonts directly on screen.
   - Official ASL ASCII Art banner, bilingual greeting, and colored operational badges rendered directly to the hardware monitor display (`Analysis/reports/asl_vm_screen.png`).
2. **Limine Protocol Response Pipeline Hardening:**
   - Linker script segment (`limine_requests`) configured with writable `FLAGS(6)` permissions.
   - Limine request structures declared mutable with volatile memory semantics (`core::ptr::read_volatile`), guaranteeing immediate population of Memmap, Kernel Address, HHDM, and Framebuffer response pointers.
3. **Direct Runtime Telemetry (Milestones M1 through M6):**
   - **M1:** Limine Entry Stub execution via assembly trampoline.
   - **M2:** Memory map parsing and physical boundary sanitization.
   - **M3:** Control handover to ASL microkernel core entry (`_asl_kernel_entry`).
   - **M4:** Deterministic Buddy Allocator and heap initialization.
   - **M5:** ARINC 653 Spatial Partition Vault and Major Frame Scheduler initialization with verified initial deterministic tick advance.
   - **M6:** Clean transition into the hardware `hlt` safe idle loop.
4. **Multi-Crate Microkernel Architecture & Host Testing:**
   - 17 member crates compiling cleanly under Rust `#![no_std]`.
   - 45 automated unit tests validating boundary arithmetic, memory window isolation, scheduler cyclic timeframes, IOMMU DMA address translation, cacheline bounce buffers, and slab allocation.
5. **Bootable Hybrid Avionics ISO:**
   - Mastered with `xorriso` and Limine v8.x hybrid BIOS/UEFI boot stages (`asl-kernel/target/asl-core-kernel.iso`, 3,917,824 bytes, SHA-256: `42d931d2031f64520364222ee1dd01fddd96dc80582ffb0d5297253f02fb747f`).
   - Validated running in headless and GUI Oracle VirtualBox VM `ASL`.

---

## 2. VERIFIED SUPPORT SCOPE

- **Primary Reference Execution Target:** `x86_64-unknown-none` on BIOS/UEFI x86_64 hypervisors (VirtualBox, QEMU, bare-metal PC).
- **Host Testing & Toolchain Target:** `x86_64-pc-windows-msvc` / Rust 1.80+.
- **Architectural Specification & Crate Framework:** Complete multi-arch abstractions for ARM, RISC-V, MIPS, SPARC, POWER, and s390x.

---

## 3. REMEDIATION & FINDINGS STATUS

- **DMA-CACHE-01:** Fully resolved via `asl-hal::cache::IsolatedBounceBuffer` with 64-byte alignment enforcement.
- **DMA-ADDR-01:** Fully resolved via `asl-hal::dma_space::IommuTranslationSpace` with memory window checksum verification.
- **ALIGN-01:** Fully resolved via 64-byte aligned `DmaPacketDescriptor` structures.
- **ATOMIC-SPARC-01:** Fully resolved via lock-free split-counter CAS emulation.
