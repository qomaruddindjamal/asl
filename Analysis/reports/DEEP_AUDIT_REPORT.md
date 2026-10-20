# DEEP AUDIT REPORT: ASL CORE KERNEL (INDEPENDENT REVIEW)
================================================================================
**Auditor:** Software Architect & Security Auditor  
**Date:** 2026-10-19  
**Target:** ASL Core Avionics Microkernel (`C:\asl\asl-kernel`)  
**Standard Acuan:** DO-178C DAL-A / ARINC 653 IMA  
**Scope:** Read-Only Audit & Verification Report

---

## 1. EXECUTIVE SUMMARY & DOKUMEN BASIS
- **README.md & CANONICAL_MASTER.md:** Mengklaim kernel IMA `#![no_std]` deterministik x86_64 higher-half Limine v2, 45 unit test lulus, 25 gates Phase 1..6 locked, serta 4 temuan teknis berstatus OPEN (`ATOMIC-SPARC-01`, `DMA-CACHE-01`, `DMA-ADDR-01`, `ALIGN-01`).
- **Kontradiksi Arsitektural Dokumen vs Kode:**
  - `asl_avionics_kernel_blueprint.md` masih menggunakan spesifikasi pohon file bahasa C (`fms_exec.c`, `cpu_interface.h`, dll.), sedangkan implementasi aktual adalah workspace Rust 31 crates modern `#![no_std]`.
  - `docs/ROADMAP/CHECKLIST_STATE.json` mencatat 25 gate Phase 1..6 `PASS & LOCKED` (level simulasi/arsitektur), namun milestone `Milestone_03` (Storage Subsystem) berstatus `PARTIAL` (Gate 3.6 AHCI DMA & Gate 3.20 Rebuild berstatus GAP_IDENTIFIED).
  - Status `ALIGN-01`: Pada `SECURITY_FINDINGS.md` dan `REMEDIATION_PLAN.md`, `ALIGN-01` sudah **CLOSED** (dengan 4 unit test di `asl-pmd`), tetapi `README.md §4` dan `CANONICAL_MASTER.md §3` masih mencatatnya sebagai **OPEN**.

---

## 2. INVESTIGASI CRATE `asl-kernel-core-secondary`

### 2.1 Analisis Komparasi dengan `asl-kernel-core`
- **Diff & Kemiripan Substantif:** Kemiripan kode: **< 5%** (Bukan klon/duplikasi basi).
  - `asl-kernel-core` (2.225 baris): Microkernel runtime primer (`_asl_kernel_entry`, IDT, GDT, TSS IST1, VGA, GOP Linear Framebuffer, Buddy Allocator, ARINC 653 Vault, Scheduler, CLI).
  - `asl-kernel-core-secondary` (311 baris): Recovery Finite State Machine (`SecondaryCoreEngine`) dengan state `Standby -> FaultDetected -> IsolationRequested -> RecoveryEntry -> Validation -> Operational / RecoveryFailed`.
- **Referensi di Workspace:**
  - `asl-kernel/Cargo.toml`: Terdaftar sebagai workspace member.
  - `asl-kernel-core/Cargo.toml`: Terdaftar sebagai dependency.
  - `asl-kernel-core/src/lib.rs:610-611`: Diinstansiasi sebagai `pub static mut GLOBAL_SECONDARY_ENGINE`.
  - Terkoneksi ke `asl_fault_notifier()` (line 663) dan `panic_handler` (line 675) untuk merekam anomali via `notify_fault()`.
  - `asl-kernel-bin/src/main.rs`: Tidak mereferensikan secondary secara langsung; kontrol handover bootloader langsung ke entry `asl-kernel-core`.
- **Kesimpulan Hipotesis:**
  - **Fungsi Aktual:** **Hot-Standby Disaster Recovery Engine / Failover State Machine Hook** (Bukan duplikasi basi, bukan boot core AP/SMP mandiri).
- **Rekomendasi Arsitektural:** **PERTAHANKAN DAN REFINE**.
  - *Justifikasi:* Sesuai doktrin DO-178C DAL-A self-healing. Namun, perlu penegakan isolasi memori fisik (enclave terpisah) agar saat kernel primer mengalami memory corruption fatal, instance static `GLOBAL_SECONDARY_ENGINE` tidak berada dalam ruang address space yang ikut korup.

---

## 3. TEXT-BASED BOOT FLOW DIAGRAM

```
+-----------------------------------------------------------------------------------+
| 1. LIMINE HANDOVER (Firmware BIOS / UEFI GOP)                                     |
|    - Memuat kernel ELF higher-half (-z max-page-size=0x1000, 4KB page aligned)    |
|    - Mengisi Limine Requests: MEMMAP, KERNEL_ADDR, FRAMEBUFFER, HHDM              |
+-----------------------------------------------------------------------------------+
                                       │
                                       ▼
+-----------------------------------------------------------------------------------+
| 2. ASL PLATFORM LAUNCHER (asl-kernel-bin / _start -> asl_launcher_main)           |
|    - Dedicated static stack: 512 KB aligned 4096 bytes                            |
|    - Inisialisasi UART COM1 (0x3F8)                                               |
|    - Sanitasi memory map: mengecualikan kphys..kphys+ksize dari usable RAM        |
|    - Membentuk kontrak AslBootHandover (magic 0x41534C4B) di BSS                  |
+-----------------------------------------------------------------------------------+
                                       │
                                       ▼ (Handover pointer via RDI)
+-----------------------------------------------------------------------------------+
| 3. ASL CORE ENTRY (asl-kernel-core::_asl_kernel_entry)                            |
|    - [M0] Inisialisasi IDT, GDT, TSS IST1 x86_64 fault handler & fault notifier  |
|    - Inisialisasi VGA text mode & Linear Framebuffer Console (GOP/HHDM)           |
|    - AslAvionicsKernel::cold_boot_init:                                           |
|        ├── asl-bootloader : Handover validation (magic & 52-bit max limit)        |
|        ├── asl-memory     : BuddyAllocator (halaman fisik O(1) & DMA pool)        |
|        ├── asl-security-vault : ARINC 653 Spatial Vault (Core Partition 1)        |
|        └── asl-sched      : MajorFrameSchedule (100ms major frame, 20ms/80ms slot)|
+-----------------------------------------------------------------------------------+
                                       │
                   ┌───────────────────┴───────────────────┐
                   ▼                                       ▼
       [Nominal Deterministic Loop]             [Hardware Trap / Panic]
       - Shell, PCI, Driver PS/2,               - Hook ke SecondaryCoreEngine
         Storage VFS/RAID                        - Controlled Halt (loop spin)
```

---

## 4. VERIFIKASI KLAIM README VS FAKTA RIIL

| Aspek Klaim README | Status | Bukti Eksekusi & Fakta Lapangan |
| :--- | :---: | :--- |
| **Unit Test Workspace**<br>*(Klaim: 45 unit tests passed)* | **PARTIAL** | Perintah `cargo test --workspace --lib` (default target) **FAIL** karena `.cargo/config.toml` menetapkan target default `x86_64-unknown-none` (tanpa std/test crate).<br>Saat dijalankan dengan target host `cargo test --workspace --target x86_64-pc-windows-msvc --lib`: **128 unit tests passed** (45 tests adalah baseline lama per 13 Okt, kini telah bertambah hingga 128 tests termasuk suite storage). |
| **Bare-Metal Compiler**<br>*(Klaim: Clean build x86_64-unknown-none)* | **PASS** | `cargo check --package asl-kernel-bin --target x86_64-unknown-none --release` dan `cargo build --package asl-kernel-bin --target x86_64-unknown-none --release` menghasilkan biner ELF64 bersih tanpa warning maupun error. |
| **25 Phase 1..6 Gates**<br>*(Klaim: All 25 gates locked)* | **PASS** | `docs/ROADMAP/CHECKLIST_STATE.json` mengonfirmasi Phase_01 s/d Phase_06 terkunci (`locked: true`, status `PASSED`). Milestone_03 aktif secara terpisah. |

---

## 5. PEMETAAN TEMUAN DMA KE LOKASI KODE & RE-KLASIFIKASI SEVERITY

| Finding ID | Lokasi Kode Riil (File & Fungsi/Item) | Klasifikasi Baru | Severity Baru | Justifikasi Teknis |
| :--- | :--- | :---: | :---: | :--- |
| **`DMA-ADDR-01`** | `asl-pmd/src/lib.rs:151` (`transmit_burst`)<br>`asl-hal/src/dma_space.rs:33` (`DmaAddressSpace`) | **SECURITY** | **CRITICAL** | `transmit_burst()` menerima `phys_addr: u64` mentah tanpa verifikasi partisi IOVA. Komponen untrusted dalam partisi guest dapat memanipulasi DMA controller untuk menimpa kernel atau partisi lain (DMA bypass isolasi ARINC 653). |
| **`DMA-CACHE-01`** | `asl-hal/src/cache.rs:15` (`CacheMaintenanceOps`) | **SAFETY** | **HIGH** | Pada CPU non-coherent (ARMv7/MIPS/LEON3), ketiadaan invalidasi cacheline dapat memicu *stale reads* atau pembuangan dirty data milik task sekitar (*silent memory corruption* pada kontrol avionik). |
| **`ALIGN-01`** | `asl-pmd/src/lib.rs:50` (`DmaPacketDescriptor`)<br>`asl-pmd/src/lib.rs:72` (`init_pool`) | **PERFORMANCE / SAFETY** | **MEDIUM** | Di x86_64 berdampak pada false sharing (performance penalty), tetapi pada CPU non-coherent menjadi safety hazard jika deskriptor berbagi baris cache 64-byte dengan data lain. Telah dimitigasi parsial di `asl-pmd` dengan static assert dan runtime pool check. |
| **`ATOMIC-SPARC-01`** | `asl-space-tmr/src/lib.rs:110` (`SparcDownscaledTimekeeper`) | **SAFETY** | **MEDIUM** | SPARC V8 (LEON3) tidak memiliki instruksi atomik 64-bit native. Ketergantungan RMW 64-bit tanpa masking PSR/LDSTUB dapat memicu race condition pada telemetri antariksa; estimasi WCET belum diukur secara empiris di hardware. |

---

## 6. DAFTAR 10 RISIKO TERBESAR YANG BELUM TERCATAT DI DOKUMEN

1. **Target Default Workspace Broken untuk Testing:** File `.cargo/config.toml` menetapkan default target `x86_64-unknown-none`. Menjalankan `cargo test` biasa langsung crash dengan `error[E0463]: can't find crate for test`. Developer baru akan mengira test suite rusak.
2. **Ketiadaan Runtime IOMMU Driver Aktif (VT-d / AMD-Vi):** Trait `DmaAddressSpace` telah didefinisikan di `asl-hal`, tetapi `asl-pmd` dan driver DMA lainnya belum terintegrasi ke engine IOMMU nyata. `transmit_burst` masih menerima raw physical address.
3. **Kerapuhan State Mutabel Global `GLOBAL_SECONDARY_ENGINE`:** Secondary core engine disimpan sebagai variabel statis mutabel (`static mut`) di dalam address space kernel primer (`asl-kernel-core`). Jika kernel primer mengalami memory corruption liar, state engine recovery sekunder berisiko ikut korup.
4. **`_asl_kernel_entry` Tidak Mengembalikan Kontrol (No-Return Hang):** Ketika terjadi unrecoverable panic, panic handler melakukan spin loop tanpa mekanisme hardware watchdog reset atau reset line assert.
5. **Ketergantungan `read_volatile` untuk MMIO Tanpa Memory Fencing Komprehensif:** Operasi I/O volatile di beberapa crate subsistem mengasumsikan compiler reordering fence, namun belum menyertakan instruksi CPU serialization barrier (`mfence` / `dsb sy`).
6. **Alokasi Framebuffer Direct Map Tanpa Validasi Range HHDM:** Di `asl-kernel-core/src/lib.rs:731`, pemetaan `HHDM_OFFSET + fb_cfg.base_physical_addr` tidak memeriksa apakah rentang memori tersebut telah dipetakan oleh page table Limine di PML4.
7. **Ketiadaan Proteksi Stack Overflow pada Task Switching:** Scheduler ARINC 653 belum mengonfigurasi Guard Pages (halaman non-present di bawah stack partisi) untuk mendeteksi stack exhaustion sebelum merusak heap buddy allocator.
8. **Sinkronisasi Dokumen Kedaluwarsa:** `asl_avionics_kernel_blueprint.md` masih menggunakan arsitektur pohon file C23, berpotensi menyesatkan auditor sertifikasi DO-178C yang mensyaratkan ketertelusuran dokumen-ke-kode 100%.
9. **Ketiadaan Validasi Identitas Perangkat PCI:** Crate `asl-driver-pci` membaca konfigurasi PCI tanpa verifikasi vendor/device ID whitelist DAL-A terisolasi, memungkinkan DMA rogue device terhubung ke bus.
10. **Kompatibilitas Multicore SMP pada Primary-Secondary Handshake:** `SecondaryCoreEngine` belum thread-safe / core-safe untuk skenario multi-core asymmetric multiprocessing (AMP/SMP), berisiko memicu data race saat AP core aktif bersamaan dengan BSP.

---
**Kesimpulan Audit:** Proyek berada pada status implementasi fungsional bare-metal yang sangat solid (128 test lulus, bootloader & kernel Limine bersih), namun membutuhkan penataan konfigurasi target test serta integrasi penuh proteksi IOMMU/DMA sebelum melangkah ke pengujian target silikon fisik.
