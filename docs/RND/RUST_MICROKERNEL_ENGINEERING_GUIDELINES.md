# PANDUAN STANDAR REKAYASA MICROKERNEL RUST: EFISIENSI LOW-SPEC, RESPONSIVITAS & KEAMANAN SISTEM (DO-178C / BARE-METAL)

Dokumen ini adalah **Pedoman Resmi Rekayasa Perangkat Lunak Kernel ASL** yang disintesis dari riset mendalam terhadap repositori microkernel dan embedded OS kelas dunia di GitHub (**seL4/rust-sel4**, **redox-os/kernel**, **theseus-os/Theseus**, **tock/tock**, dan **betrusted-io/xous-core**), digabungkan dengan standar avionik **DO-178C DAL-A** dan **ARINC 653 IMA**.

---

## 1. DOKTRIN UTAMA: MEMORI MINIMALIS & KESTABILAN SISTEM LOW-SPEC

Pada sistem berspesifikasi rendah (RAM 4 MB – 64 MB, mikrokontroler, modem router, atau embedded device), kegagalan alokasi memori (*Out Of Memory / OOM*) adalah ancaman fatal. Berdasarkan analisis arsitektur **Tock OS (Grants pattern)** dan **Xous Memory Manager**:

### A. Kebijakan "Zero Heap Allocation at Boot & Driver Path"
1. **Dilarang Alokasi Dinamis Tak Terbatas:** Driver hardware (`pci`, `ps2`, `uart`, `nic`) tidak boleh memanggil alokator heap dinamis (`alloc::vec::Vec`, `alloc::boxed::Box`) secara sembarangan dalam jalur penanganan data kritis (*critical path*).
2. **Fixed-Capacity Bounded Buffers:** Seluruh struktur data antrean driver harus berupa array berkapasitas tetap (*statically bounded ring buffer*) yang ukurannya ditentukan saat waktu kompilasi (*compile-time constant*).
   ```rust
   // Standar ASL: Ring buffer terikat tanpa alokasi heap
   pub struct BoundedQueue<T, const N: usize> {
       buffer: [T; N],
       head: usize,
       tail: usize,
       count: usize,
   }
   ```
3. **Penyelarasan Cacheline DMA 64-Byte:** Sesuai audit `DMA-CACHE-01`, semua buffer yang diakses kontroler perangkat keras (PCI/USB/Ethernet) wajib dialokasikan dengan atribut `#[repr(align(64))]` guna mencegah korupsi memori akibat operasi invalidasi cache CPU.

### B. Segmentasi Memori Partisi Statis (ARINC 653 IMA)
1. **Spatial Vault Partitioning:** Memori fisik dibagi menjadi jendela memori (*Memory Windows*) yang terisolasi. Jika satu driver mengalami anomali atau buffer overflow, dampaknya tertahan di partisinya sendiri tanpa menyentuh memori partisi kernel utama.
2. **Deterministic Buddy Allocator:** Alokator buddy mengelola halaman fisik dalam ukuran $2^k$ (4 KB, 8 KB, 16 KB) dengan waktu eksekusi berbatas tetap $O(1)$ atau $O(\log N)$, menghilangkan jeda fragmentasi memori (*non-deterministic latency*).

---

## 2. PEDOMAN RESPONSIVITAS SISTEM & LATENSI SUB-MILIDETIK

Mengacu pada studi arsitektur **Theseus OS** dan **Redox OS (pcid & event scheme)**:

### A. Non-Blocking I/O Polling & Event Loop
1. **Anti-Spin-Lock Berkepanjangan:** Driver yang beroperasi dalam mode polling tidak boleh melakukan perulangan hampa tanpa batas (*infinite tight loop*). Setiap putaran polling wajib menyisipkan `core::hint::spin_loop()` dan dibatasi oleh counter batas waktu (*bounded timeout*).
2. **Pemulihan Otomatis Hardware Hang:** Jika hardware tidak merespons dalam jumlah siklus yang ditentukan, fungsi driver harus mengembalikan error eksplisit (`Result<T, DriverError>`), bukan membekukan sistem (*freeze*).

### B. Time-Triggered Major/Minor Frame Scheduling (ARINC 653)
1. **Jaminan Slot Waktu:** CPU dibagi menjadi Major Frame siklis (misal 100 ms) yang dipecah menjadi Minor Frame partisi yang kaku.
2. **Zero-Jitter Execution:** Tugas-tugas kritis (misalnya kontrol penerbangan, transmisi laser SFP, dan dekode masukan keyboard) dijamin mendapatkan jatah CPU pada slot waktu yang tepat tanpa terganggu oleh proses background.

---

## 3. KAIDAH PEMROGRAMAN RUST KHUSUS KERNEL BARE-METAL (`#![no_std]`)

### A. Penanganan `unsafe` Block yang Disiplin
1. **Safety Contract Invariant:** Setiap blok `unsafe` wajib disertai komentar dokumentasi `// SAFETY:` yang menjelaskan secara eksplisit mengapa akses memori mentah, port I/O, atau pemanggilan instruksi assembly tersebut terjamin valid dan tidak melanggar kepemilikan Rust.
2. **Volatile Memory Semantics:** Semua interaksi dengan Memory-Mapped I/O (MMIO) register perangkat keras dan pointer Limine handover wajib menggunakan `core::ptr::read_volatile` dan `core::ptr::write_volatile` agar compiler tidak mengabaikan pembacaan akibat optimasi register caching.

### B. Error Handling Tanpa Panic
1. **Dilarang `.unwrap()` dan `.expect()`:** Dalam mode kernel `#![no_std]`, pemanggilan panic akan mematikan sistem operasi. Seluruh fungsi driver harus menggunakan enum `Result<T, KernelError>` atau `Option<T>` dengan pencocokan pola (*pattern matching*) yang lengkap.
2. **Fail-Safe Trap:** Panic handler hanya difungsikan sebagai benteng terakhir (*ultimate fail-safe trap*) yang mencatat lokasi file/baris ke port serial sebelum memarkir core prosesor ke status aman `hlt`.

---

## 4. MATRIKS IMPLEMENTASI DRIVER v1.0.2 BERDASARKAN HASIL RISET GITHUB

| Driver Subsystem | Referensi Repositori | Pola Kode Terpilih | Penyesuaian ke Kernel ASL |
|---|---|---|---|
| **PCI Enumerator** (`asl-driver-pci`) | `theseus-os/kernel/pci` & `redox-os/pcid` | Memindai bus I/O port `0xCF8`/`0xCFC` dengan bit enable `0x80000000`. Membaca BAR0..BAR5 dan memeriksa tipe (Memory vs I/O space). | Dibuat statis `#![no_std]` tanpa heap. Hasil scan disimpan dalam `[Option<PciDeviceDescriptor>; MAX_PCI_DEVICES]`. |
| **PS/2 Keyboard** (`asl-driver-ps2`) | `theseus-os/kernel/ps2` | Membaca status port `0x64` bit 0 (Output Buffer Full) sebelum membaca port `0x60`. Decode scan-code Set 1 Make/Break. | Dipadukan dengan bounded ring buffer interaktif untuk melayani Cockpit Shell `ASL-OS> `. |
| **VGA / Framebuffer** (`asl-kernel-core`) | `redox-os/kernel/devices/graphical_debug` | Menulis piksel langsung ke Linear Framebuffer GOP/VBE 32-bpp dengan pitch stride memory mapping. | Menggunakan raster font bitmap asli 8x16 dengan penskalaan bersih tanpa floating point. |
| **Low-Memory Alloc** (`asl-memory`) | `tock/kernel/src/grant.rs` & `xous-core/mem.rs` | Menggunakan fixed-size tracking array dan boundary checking fisik. | Buddy Allocator dan slab cache berordo 2 dengan proteksi ARINC 653. |
