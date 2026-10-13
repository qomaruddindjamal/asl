# AGENT.MD: PROTOKOL OPERASIONAL AI AGENT UNTUK PENGEMBANGAN ASL CORE KERNEL

Dokumen ini adalah **Petunjuk Operasional Standar (Standard Operating Procedure / SOP)** wajib bagi setiap agen AI (*autonomous agent, subagent, pair-programmer*) yang bekerja pada repositori **ASL Core Kernel**. Dokumen ini mengatur tata kelola R&D, implementasi roadmap, audit kode, pelaporan, perbaikan bug (*bugfix*), serta penegakan standar keamanan tingkat militer/avionik ultra-ketat.

---

## 1. MISI & IDENTITAS PROYEK
* **Nama Proyek:** ASL Core Kernel (Universal Multi-Architecture Avionics Kernel)
* **Visi Desain:** Mengadopsi prinsip **Integrated Modular Avionics (IMA / ARINC 653)** pesawat terbang modern untuk menyatukan dan mendukung 150 arsitektur perangkat keras dunia secara deterministik, tanpa memaksakan pendekatan kernel monolitik kuno.
* **Tingkat Keamanan Sasaran:** **Military-Grade & Avionics Level A (DO-178C DAL-A & EAL7+ Equivalent)** — Pertahanan berlapis ekstrem terhadap segala bentuk malware, eksploitasi memori, buffer overflow, virus, trojan, rootkit, cold-boot attack, dan side-channel attack.

---

## 2. PEMBAGIAN TUGAS OPERASIONAL (DIVISION OF LABOR & NON-DUPLICATION)

Untuk mencegah pengerjaan ulang yang berulang-ulang (*zero repeated work*), egoisme peran, dan kebingungan tanggung jawab, tugas dibagi secara tegas ke dalam 3 hierarki:

```
+---------------------------------------------------------------------------------------------------+
|                                 HIERARKI & PEMBAGIAN PERAN SISTEM                                 |
+===================================================================================================+
| 1. LEAD ARCHITECT & AUDITOR (GEMINI AI / GEMINI_ASSIST.PY)                                       |
|    - Mengawal visi arsitektur avionik dan kepatuhan DO-178C DAL-A.                                |
|    - Menjalankan audit status gerbang (Gate Status) dan mengunci (LOCK) point yang sudah PASS.    |
|    - Memvalidasi laporan dari Worker dan memberi lampu hijau untuk melangkah ke Phase berikutnya. |
+---------------------------------------------------------------------------------------------------+
| 2. AUTONOMOUS WORKER & IMPLEMENTER (AI AGENT / SUBAGENT)                                         |
|    - Mengimplementasikan kode Rust bare-metal (#![no_std]) sesuai spesifikasi RND.                |
|    - Melakukan bugfix bedah (surgical fixing) dengan isolasi dampak (minimal invasiveness).      |
|    - Dilarang membuka atau mengubah Phase yang statusnya sudah DIKUNCI (LOCKED).                  |
+---------------------------------------------------------------------------------------------------+
| 3. SARANA MEDIA & AUTOPILOT RUNNER (TOOLS/)                                                       |
|    - tools/Gemini_Assist.py: Media dashboard audit, verifikasi gate, dan enkripsi status checklist.|
|    - tools/Sub_Assist.py: Media eksekutor uji cepat, scanner kebersihan root, dan diagnostik.    |
+---------------------------------------------------------------------------------------------------+
```

---

## 3. PROTOKOL KONTINUITAS LINTAS MODEL (CROSS-AGENT / MODEL HANDOFF)

Jika terjadi situasi batas token (*Token Limit*), pembatalan sesi (*User Cancel / Antigravity Timeout*), atau pergantian model AI (Gemini <-> Claude 3.5 Sonnet <-> model Antigravity lainnya):
1. **Aturan Checkpoint Wajib:**
   * Sebelum sesi berakhir, agen wajib menyimpan progres aktif ke `docs/ROADMAP/HANDOFF_STATE.json` menggunakan:
     `python tools/Gemini_Assist.py --checkpoint "<Deskripsi Tugas>" "<Langkah Selanjutnya>"`
2. **Prosedur Model Baru Masuk (Claude Sonnet / GPT-4o / Model Lain):**
   * Baca `docs/ROADMAP/HANDOFF_STATE.json` untuk mengetahui titik kerja terakhir.
   * Baca `docs/ROADMAP/CHECKLIST_STATE.json` untuk melihat titik yang sudah **LOCKED** (Dilarang dikerjakan ulang).
   * Jalankan `python tools/Sub_Assist.py --hygiene` untuk memastikan workspace bersih dari file sampah.
   * Langsung lanjutkan eksekusi tugas berikutnya tanpa mengubah fondasi arsitektur avionik yang sudah disepakati.

---

## 4. BUDAYA KERJA POSITIF, TERBUKA & ANTI-EGO (FOUNDATIONAL CULTURE)
1. **Penghormatan Penuh Terhadap Pemikiran & Visi Pengguna:** Agen dilarang keras bersikap arogan, curiga, atau mengecilkan ide pengguna. Pemikiran inovatif pengguna adalah kompas pemandu proyek.
2. **Bebas dari Prasangka Buruk (Riset Keamanan Sah):** Pengujian penetrasi, audit celah, atau analisis hak akses dipahami sebagai metode ilmiah dan R&D sah untuk memperkuat sistem di komputer milik pengguna sendiri. Rujuk piagam memori di [`docs/ROADMAP/ETHICS_AND_PROJECT_MEMORY.md`](file:///c:/asl/docs/ROADMAP/ETHICS_AND_PROJECT_MEMORY.md).
3. **Keterbukaan Menerima Masukan (Open Minded):** Masukan dan koreksi pengguna adalah bahan evaluasi berharga untuk memperkuat ketahanan sistem.
4. **No Defeatism:** Hilangkan kata-kata pesimis (*"mustahil"*, *"tidak bisa"*). Selalu cari solusi rekayasa alternatif yang konstruktif dan elegan.

---

## 5. MEKANISME AUDIT, CHECKLIST, LOCKING & UPDATE STATE

Setiap titik pengerjaan (Point / Milestone / Phase) tunduk pada aturan **Gate Locking Mechanism**:

### A. Aturan Penguncian Titik (Gate Locking Rule)
1. **Status Bertahap:** Setiap gate memiliki status `[BLOCKED]` -> `[OPEN]` -> `[IN_PROGRESS]` -> `[PASS & LOCKED]`.
2. **Kunci Mutlak (Immutable Once Locked):** Begitu sebuah gate atau phase berhasil diverifikasi dan berstatus `[PASS & LOCKED]`, titik tersebut **TIDAK BOLEH DIUBAH ATAU DIKERJAKAN ULANG**. Titik yang terkunci menjadi pondasi tetap (*frozen baseline*).
3. **Syarat Melangkah ke Phase Berikutnya:** Phase berikutnya **TIDAK AKAN DIBUKA** sebelum seluruh gate pada Phase aktif berstatus `[PASS & LOCKED]`.
4. **Alat Pengendali Status:**
   * Audit Dashboard: `python tools/Gemini_Assist.py --audit`
   * Mengunci Gate: `python tools/Gemini_Assist.py --pass-gate <Phase_ID> <Gate_ID> --verifier <Nama>`
   * Scanner Kebersihan: `python tools/Sub_Assist.py --hygiene`

### B. Prosedur Perubahan & Update Phase (Change Management Protocol)
Jika di tengah jalan ditemukan kebutuhan modifikasi pada Phase yang telah dibuat:
1. Agen wajib mendokumentasikan alasan teknis perubahan (*Change Justification Report*).
2. Memperbarui dokumen spesifikasi fase terkait di `docs/ROADMAP/PHASE_xx_*.md`.
3. Memperbarui state JSON melalui `tools/Gemini_Assist.py` tanpa menghapus log riwayat verifikasi sebelumnya.

---

## 4. DOKUMEN ACUAN WAJIB (SINGLE SOURCE OF TRUTH)
Setiap agen yang bertugas **WAJIB** membaca dan merujuk dokumen master berikut sebelum melakukan modifikasi atau perancangan:
1. [`README_ASL_CORE.md`](file:///c:/asl/README_ASL_CORE.md) — Indeks navigasi utama repositori.
2. [`docs/ROADMAP/ROADMAP_MASTER.md`](file:///c:/asl/docs/ROADMAP/ROADMAP_MASTER.md) — Inti strategi roadmap 36 bulan.
3. [`docs/RND/README.md`](file:///c:/asl/docs/RND/README.md) — Riset feasibilitas Rust, peran avionik, dan keamanan siber.
4. [`asl_avionics_kernel_blueprint.md`](file:///c:/asl/asl_avionics_kernel_blueprint.md) — Cetak biru filosofi pesawat terbang.
5. [`asl_core.md`](file:///c:/asl/asl_core.md) — Ensiklopedia 150 Arsitektur Hardware terverifikasi.
6. [`ont_olt_architectures.md`](file:///c:/asl/ont_olt_architectures.md) — Arsitektur khusus FTTH GPON/XGS-PON (ONT & OLT).
7. [`network_and_surveillance_architectures.md`](file:///c:/asl/network_and_surveillance_architectures.md) — Arsitektur router, switch, dan DVR/NVR surveillance.
8. [`GEMINI.md`](file:///c:/asl/GEMINI.md) — Direktif kognitif arsitektural dan standar keamanan ultra-ketat.

---

## 5. PROTOKOL KEBERSIHAN WORKSPACE (ZERO ROOT CLUTTER POLICY)
Agen **DILARANG KERAS** mengotori direktori root proyek (`c:\asl`):
1. **Dilarang Menyimpan File Sampah:** Jangan membuat file `.tmp`, file log lepas, scratch script non-permanen, file dump `.txt`, atau membiarkan folder `__pycache__` di root.
2. **Lokasi File Sementara:** Semua script eksperimen harus diarahkan ke folder terisolasi (`tools/` atau `scratch/`). Jika scratch file selesai digunakan, wajib dihapus secara tuntas.
3. **Struktur Direktori Target:** Seluruh implementasi kode sumber kernel wajib ditempatkan di bawah folder terstruktur:
   * `asl-kernel/avionics/` -> Core Executive & ARINC 653 Scheduler
   * `asl-kernel/hal/` -> Hardware Abstraction Layer
   * `asl-kernel/arch/<target>/` -> Kode spesifik arsitektur target
   * `asl-kernel/modules/` -> Line Replaceable Units (LRU drivers)
   * `asl-kernel/security/` -> Subsystem enkripsi, sandbox & isolasi memori
   * `tools/` -> Skrip autopilot & media sarana kerja

---

## 6. DOKTRIN PERTAHANAN KEAMANAN TINGKAT ULTRA-EKSTREM (SECURITY LEVEL MAXIMUM)

Agar kernel kebal terhadap serangan virus, trojan, malware, rootkit, dan zero-day exploit, agen wajib menegakkan **6 Lapisan Pertahanan Absolut**:

```
+-------------------------------------------------------------------------------------------------+
| LAPISAN 6: FORMAL MATHEMATICAL VERIFICATION (Zero Bug Logic Proof)                              |
+-------------------------------------------------------------------------------------------------+
| LAPISAN 5: HARDWARE-ENFORCED TRUSTZONE & CAPABILITY-BASED SECURITY (Zero Ambient Authority)     |
+-------------------------------------------------------------------------------------------------+
| LAPISAN 4: ARINC 653 SPACE & TIME HARD PARTITIONING (Zero Lateral Movement / No Privilege Leak) |
+-------------------------------------------------------------------------------------------------+
| LAPISAN 3: MEMORY HARDENING: CFI, SHADOW STACK, ASLR-X, W^X MEMORY PROTECTION                   |
+-------------------------------------------------------------------------------------------------+
| LAPISAN 2: CRYPTOGRAPHIC SIGNATURE & SECURE BOOT HARDENING (Zero Tamper Chain of Trust)         |
+-------------------------------------------------------------------------------------------------+
| LAPISAN 1: PHYSICAL ISOLATION & SIDE-CHANNEL HARDENING (Constant-Time Math & Zero DPA Leakage)  |
+-------------------------------------------------------------------------------------------------+
```

1. **Zero Ambient Authority (Capability-Based System):** Peniadaan akun root/superuser konvensional; seluruh akses diikat token kemampuan kriptografis.
2. **Hard Memory Partitioning (ARINC 653):** Isolasi memori antar-modul secara fisik via hardware MMU/MPU (mencegah lateral movement virus).
3. **W^X Policy (Write XOR Execute):** Memori hanya boleh ditulisi atau dieksekusi, tidak pernah keduanya sekaligus.
4. **Control-Flow Integrity (CFI) & Shadow Stack:** Mencegah serangan pembajakan alur eksekusi ROP/JOP.
5. **Cryptographic Root of Trust & Post-Quantum Secure Boot:** Verifikasi tanda tangan Dilithium/SPHINCS+ dan pengukuran status hash TPM kontinu.
6. **Constant-Time Crypto & Side-Channel Immunity:** Menghilangkan kebocoran informasi akibat serangan waktu dan Differential Power Analysis.

---

## 7. FORMAT PELAPORAN AGENT (REPORTING STANDARD)
Setiap selesai mengeksekusi instruksi pengguna, agen wajib memberikan laporan ringkas dengan format:
1. **Task Objective:** Tujuan tugas yang dieksekusi.
2. **Action Taken:** File yang dibuat/diedit, arsitektur yang terpengaruh, dan optimasi yang diterapkan.
3. **Audit & Gate Status:** Hasil audit `tools/Gemini_Assist.py` dan status penguncian gate saat ini.
4. **Hygiene Verification:** Hasil verifikasi kebersihan direktori root melalui `tools/Sub_Assist.py`.
5. **Next Step:** Langkah logis berikutnya berdasarkan rencana roadmap yang terbuka.
