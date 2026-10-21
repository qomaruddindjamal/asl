# PANDUAN ASL CORE KERNEL v1.0.2 - INDEKS

Panduan berbahasa Indonesia untuk manusia yang menjalankan, membangun, dan
memeriksa ASL Core Kernel pada rilis v1.0.2. Folder ini hanya berisi cara
pemakaian dan cara ukur. Klaim status proyek ada di `docs/ROADMAP` dan
`README.md`; peta fakta ada di `docs/ROADMAP/PETA-KANONIK.md`.

## Pasal 1 - Isi folder

| Berkas | Menjawab pertanyaan | Penjaga kebenaran |
| --- | --- | --- |
| `MULAI-CEPAT.md` | Bagaimana membangun, mem-boot, dan masuk shell dari nol? | Pipeline `scripts/ci_local.ps1` dan test host |
| `REFERENSI-PERINTAH.md` | Perintah apa saja yang dijanjikan shell dan apa bentuknya? | `test_docs_command_reference_matches_registry` (membaca dokumen ini lewat `include_str!`) |
| `KONFIGURASI-MEMORI.md` | Berapa RAM yang dibutuhkan boot dan mengapa janji `-m 3M` masih merah? | `scripts/test_ram_floor_3mb.ps1` dan `test_docs_capacity_figures_match_constants` |
| `../ROADMAP/PETA-KANONIK.md` | Di mana satu-sumber tiap fakta, dan mana yang masih rancangan? | Disusun dari kode; tiap baris menyebut berkas sumbernya |

## Pasal 2 - Cara membaca angka di panduan ini

Setiap angka di folder ini bersandar pada salah satu kelas bukti berikut. Kelas
bukti ditulis di samping angka, tidak boleh dibuang saat mengutip.

- `VERIFIED-ON-HOST` - diukur oleh `cargo test`/`cargo build` di mesin develop.
- `VERIFIED-ON-CI` - diukur runner GitHub Actions; hanya menutup langkah uji host
  dan build release, bukan boot.
- `VERIFIED-ON-CI-API` - metadata run CI yang diambil lewat API GitHub dan
  di-commit sebagai berkas log.
- `VERIFIED-ON-VM` - diukur dari keluaran serial QEMU pada berkas log.
- `VERIFIED-ON-VM-VISIBLE` - sama seperti di atas tetapi jendela QEMU terbuka dan
  disaksikan pemilik repo (gerbang G9).
- `RANCANGAN` - sudah diputuskan bentuknya, belum diuji. Bukan klaim hasil.

Yang TIDAK pernah dipakai: `VERIFIED-ON-HARDWARE` (belum ada perangkat fisis yang
diuji) dan klaim tangkapan layar dari mesin virtual headless.

## Pasal 3 - Prinsip gerbang ASL yang berlaku di seluruh panduan

1. Gerbang merah tetap dilaporkan merah. Tidak ada langkah dokumentasi yang
   mengubah status merah menjadi hijau; yang boleh hanya memperbaiki penyebabnya
   lalu mengukur ulang (`ACCOUNTABILITY_FAULT_LOG`, Bagian 6 misi).
2. Angka selalu diukur ulang pada tree yang sedang dipakai. Angka dari tree lain
   hanya sah bila sidik jari tree itu disebut.
3. Satu fakta, satu sumber. Tampilan (dokumen, keluaran `help`, keluaran `status`)
   disusun dari sumbernya, bukan ditulis tangan ulang.
4. Riwayat bersifat append-only. Kesalahan dikoreksi lewat entri berikutnya, tidak
   pernah dengan mengubah commit yang sudah terbit.
5. Konten baru berbahasa Indonesia; identifier kode tetap bahasa Inggris.

## Pasal 4 - Batasan panduan ini

- Menyelaskan build `x86_64-unknown-none` pada host Windows dan QEMU `q35`.
  Jalur aarch64 punya skrip sendiri (`-Action build-aarch64`, `-Action
  run-qemu-aarch64`) dan tidak dijanjikan pada rilis ini.
- Menyimpan berkas panduannya di repositori, tetapi tidak menyimpan artefak hasil
  boot di folder ini; artefak bukti berada di `Analysis/reports`.
- Tidak menjanjikan penyimpanan permanen: berkas shell hidup di RAMDISK volatil.
