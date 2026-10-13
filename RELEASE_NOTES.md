# RELEASE NOTES: ASL CORE KERNEL v1.0.0
=========================================
**Release Version:** `v1.0.0`  
**Date:** 2026-10-13  
**Architecture Authority:** `asl_core.md` (150 Silicon Platforms)  
**Governing Standard:** DO-178C DAL-A / ARINC 653 Integrated Modular Avionics (IMA)  

---

## 1. HIGHLIGHTS & DELIVERABLES

ASL Core Kernel v1.0.0 delivers the foundational bare-metal microkernel implementation and verification suite for the ASL operating system ecosystem.

### Key Capabilities Verified:
1. **Limine Base Protocol Revision 2 Boot Pipeline:**
   - Higher-half bare-metal launcher (`asl-kernel-bin`) executing with a dedicated page-aligned 512 KB static stack.
   - Handover protocol adapter (`AslBootHandover`) sanitizing memory maps and isolating physical kernel ranges from usable RAM.
2. **Direct Runtime Telemetry (Milestones M1 through M6):**
   - **M1:** Limine Entry Stub execution via assembly trampoline.
   - **M2:** Memory map parsing and physical boundary sanitization.
   - **M3:** Control handover to ASL microkernel core entry (`_asl_kernel_entry`).
   - **M4:** Deterministic Buddy Allocator and heap initialization.
   - **M5:** ARINC 653 Spatial Partition Vault and Major Frame Scheduler initialization with verified initial deterministic tick advance.
   - **M6:** Clean transition into the hardware `hlt` safe idle loop.
3. **Multi-Crate Microkernel Architecture:**
   - 17 member crates compiling cleanly under Rust `#![no_std]`.
   - 31 automated unit tests validating boundary arithmetic, memory window isolation, scheduler cyclic timeframes, and slab allocation.
4. **Bootable Hybrid Avionics ISO:**
   - ISO mastered with `xorriso` and Limine v8.x hybrid BIOS/UEFI boot stages.
   - Validated running in headless Oracle VirtualBox VM `ASL`.

---

## 2. VERIFIED SUPPORT SCOPE

- **Primary Reference Execution Target:** `x86_64-unknown-none` on BIOS/UEFI x86_64 hypervisors (VirtualBox, QEMU, bare-metal PC).
- **Host Testing & Toolchain Target:** `x86_64-pc-windows-msvc` / Rust 1.80+.
- **Architectural Specification & Crate Framework:** Complete multi-arch abstractions for ARM, RISC-V, MIPS, SPARC, POWER, and s390x.

---

## 3. OPEN FINDINGS & KNOWN LIMITATIONS

In adherence to strict engineering transparency:
- The 4 Stage C.1 risk findings (`ATOMIC-SPARC-01`, `DMA-CACHE-01`, `DMA-ADDR-01`, `ALIGN-01`) remain formally **OPEN** in `audit/asl-core-kernel/SECURITY_FINDINGS.md` as target-specific constraints for non-coherent architectures and legacy 32-bit PCI bridges.
- Silicon-level hardware deployment across all 150 architectures is staged along the 36-month engineering roadmap in `docs/ROADMAP/ROADMAP_MASTER.md`.
