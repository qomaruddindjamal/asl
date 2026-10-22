# PETA KANONIK v1.0.2 - SATU FAKTA, SATU TEMPAT

Dokumen ini memetakan tempat tinggal setiap fakta yang dipakai rilis v1.0.2 dan
siapa yang menjaganya supaya tidak menyimpang. Isinya bukan laporan hasil ukur;
laporan hasil ukur ada di `Analysis/reports`, dan status klaim ada di `README.md`.

## Pasal 1 - Cara memakai peta ini

Bila sebuah angka, teks, atau nama perintah muncul di lebih dari satu tempat,
hanya baris "sumber" pada tabel berikut yang boleh diubah. Tempat lain menyusun
dirinya dari sumber itu, atau dijaga sebuah test yang menjadi MERAH ketika
keduanya berbeda. Nomor baris pada kolom sumber bisa bergeser; label teks yang
dicetak atau nama konstanta adalah penanda yang stabil.

## Pasal 2 - Identitas dan tampilan shell

| Fakta | Sumber satu-satunya | Yang menyusun dari sumber | Penjaga |
| --- | --- | --- | --- |
| Versi rilis `v1.0.2` | `KERNEL_VERSION` di `asl-kernel/asl-cockpit-shell/src/lib.rs` | banner, `uname`, `status`, prompt bantuan | `test_release_identity_single_source` |
| Prompt `ASL-KERNEL />_ ` (akar) dan `ASL-KERNEL <posisi>>_ ` (subfolder) | `SHELL_PROMPT_PREFIX` + `SHELL_PROMPT_SUFFIX`, disusun `render_prompt`; `SHELL_PROMPT` adalah bentuk akar | seluruh situs tampil lewat `ShellCwd::prompt`; dibaca juga oleh skrip gerbang memori | `test_release_identity_single_source`, `test_f1_prompt_shows_active_position`, `test_f1_prompt_truncates_long_position_and_keeps_suffix`, `test_shell_prompt_shows_active_position`, `test_shell_cwd_parent_naik_satu_tingkat_dan_tetap_di_akar`, parser `scripts/test_ram_floor_3mb.ps1` |
| Daftar perintah kanonik | `REGISTRY` + `CANONICAL_COMMAND_COUNT` di berkas yang sama | keluaran `help`, jumlah pada `status`, `docs/PANDUAN/REFERENSI-PERINTAH.md` | `test_registry_has_thirteen_canonical_commands`, `test_f2_up_command_is_canonical_and_takes_no_argument`, `test_docs_command_reference_matches_registry` |
| Alias parser | `aliases()` tiap entri registry | resolusi parser, baris `alias:` pada `help` | `test_parser_resolves_every_documented_alias` |
| Judul kelompok perintah | `CommandCategory::label()` | seksi pada keluaran `help`, kolom Kelompok pada dokumen panduan | `test_help_text_is_generated_from_registry` |
| Label keluaran shell (48 konstanta `MSG_*`) | `MSG_*` di `asl-kernel/asl-cockpit-shell/src/lib.rs` | setiap baris galat dan baris sukses yang dicetak dispatch di `asl-kernel/asl-kernel-core/src/lib.rs`, `docs/PANDUAN/REFERENSI-PERINTAH.md` Pasal 7 | `test_f3_labels_keluaran_satu_sumber`, `test_f3_dokumen_memuat_setiap_label_keluaran` |
| Batas baris input | `SHELL_BUFFER_MAX` | penyangga shell, uji limpahan | `test_shell_buffer_overflow_protection` |
| Batas satu token argumen | `MAX_ARG_LEN` = 64 di `asl-kernel/asl-cockpit-shell/src/lib.rs` | penampung argumen parser, flag `Invocation::arg_cut`, penolakan nama terpotong dan pengumuman isi terpotong di dispatch, Pasal 6 dan 7 `docs/PANDUAN/REFERENSI-PERINTAH.md` | `test_f4_argumen_terpotong_ditandai_bukan_disembunyikan`, `test_f4_nama_sampai_batas_argumen_tetap_utuh`, `test_f4_dokumen_memuat_batas_argumen`, `test_f4_peta_memuat_batas_argumen`, `test_f6_rename_argumen_terpotong_dapat_dibaca_per_posisi` |

## Pasal 2A - MODE BERPANDUAN dan penanda kursor (F5)

Mesin state mode berpanduan hidup di `asl-cockpit-shell` (murni, dapat diuji
dari host); jembatan yang mencetak label dan menyentuh VFS hidup di
`asl-kernel-core`. Pintunya hanya satu: baris galat arity tercetak lebih dulu,
lalu untuk tiga perintah pembuatan shell menerima baris berikutnya sebagai
argumen yang hilang.

| Fakta | Sumber satu-satunya | Yang menyusun dari sumber | Penjaga |
| --- | --- | --- | --- |
| Perintah yang boleh dibimbing | `resolusi_pemandu()` membaca `REGISTRY` (sintaks -> tujuan), bukan daftar tangan | jembatan `buka_pemandu` di `asl-kernel-core` | `test_f5_mode_hanya_masuk_setelah_galat_arity` |
| Label `Nama folder baru: ` | `MSG_LABEL_FOLDER` | baris permintaan nama direktori pada kedua penampil | `test_f5_pemanduan_nama_folder`, `test_f3_dokumen_memuat_setiap_label_keluaran` |
| Label `Isi <nama> (baris <n>): ` | `MSG_CONTENT` + `MSG_BARIS_OPEN` + nomor baris + `MSG_BARIS_TAIL`, disusun `Pemandu::label` | baris permintaan isi bernomor pada kedua penampil | `test_f5_pemanduan_isi_multi_baris_dan_nomor` |
| Baris kosong berarti batal (mode nama) atau selesai (mode isi) | `Pemandu::terima` | `terima_baris_panduan` + `pieces_batal_pemandu` | `test_f5_pemanduan_batal_baris_kosong`, `test_f5_pieces_panduan_memakai_label_satu_sumber` |
| Batas nama panduan = batas satu token | `GUIDED_NAME_MAX` (= `MAX_ARG_LEN`) | penampung nama dan penolakan `NamaTerlaluPanjang` | `test_f5_nama_panduan_terlalu_panjang_ditolak`, `test_f5_batas_pemanduan_sama_dengan_slot_ramdisk` |
| Batas isi panduan = satu slot RAMDISK | `GUIDED_ISI_MAKS` = 8192 byte, diikat ke `RAMDISK_FILE_SLOT_BYTES` oleh test lintas krates | penampung isi dan pengumuman `Isi berhenti pada kapasitas satu slot ...` | `test_f5_batas_pemanduan_sama_dengan_slot_ramdisk`, `test_f5_isi_panduan_berhenti_di_satu_slot` |
| State panduan satu-satu, di `.bss` | `static mut PEMANDU` di `asl-kernel-core` (bukan lokal: penyangganya selebar satu slot) | `terima_baris_panduan`, `jalankan_aksi_pemandu` | `test_f5_state_pemandu_cukup_besar_untuk_diletakkan_di_bss` |
| Jalur pembuatan berkas SATU aturan | `jalankan_buat_folder`, `jalankan_buat_berkas`, `jalankan_tulis_berkas` dipakai dispatch kanonik DAN panduan | `run_canonical_command`, `jalankan_aksi_pemandu` | `test_f4_nama_sampai_batas_argumen_tetap_utuh` dan sesi QEMU F5 |
| Glif penanda kursor | `CURSOR_IDLE_GLYPH` (sudah hidup di `SHELL_PROMPT_SUFFIX`) | `gambar_penanda_kursor` pada framebuffer saja | `test_f5_kursor_nyala_beralternasi` |
| Fase penanda kursor | `kursor_nyala(detik)` dengan `CURSOR_BLINK_PERIODE` = 2 | galang shell idle membaca `read_cmos_reg(0)` lewat `baca_detik_rtc` | `test_f5_kursor_nyala_beralternasi`, `screendump` QEMU pada berkas bukti F5 |
| Umpan balik kemajuan Gate 3D di layar | `DualOut` dipakai `execute_gate3d_suite` (entri #38) | setiap baris progres suite yang semula hanya serial | sesi QEMU F5; varians durasi 64,0 s sampai 310,7 s tercatat di `docs/RND/ASL-V1.0.2-E8-LAPORAN-PENUTUPAN.md` |

## Pasal 2B - Ganti nama (F6)

Ganti nama adalah satu operasi pada satu direktori: sumber dan tujuan berbagi
direktori induk. Penolakan terjadi penuh sebelum VFS disentuh, jadi tidak ada
keadaan setengah terganti. `rename` sengaja BUKAN perintah berpanduan - perintah
ini butuh dua argumen sekaligus dan tidak ada baris yang boleh ditafsirkan
setelahnya.

| Fakta | Sumber satu-satunya | Yang menyusun dari sumber | Penjaga |
| --- | --- | --- | --- |
| Entri registry `rename <lama> <baru>` + alias `mv` | `RenameCmd` di `REGISTRY` (`CANONICAL_COMMAND_COUNT` = 13) | keluaran `help`, jumlah pada `status`, blok `### rename` di `docs/PANDUAN/REFERENSI-PERINTAH.md` | `test_registry_has_thirteen_canonical_commands`, `test_docs_command_reference_matches_registry`, `test_parser_resolves_every_documented_alias` |
| Satu perubahan nama di tingkat VFS | `FileSystem::rename` (bawaan `ReadOnly`) di `asl-kernel/asl-storage/src/vfs.rs`, diimplementasikan `RamFileSystem::rename`; jembatan `Vfs::rename` memanggil lewat induk-oleh-nama | dispatch `jalankan_rename` di `asl-kernel/asl-kernel-core/src/lib.rs` | `test_f6_rename_berkas_mengganti_nama_saja`, `test_f6_rename_lewat_jembatan_vfs` |
| Nama tujuan tidak boleh menimpa | `rename` mengembalikan `AlreadyExists` untuk nama tujuan yang sudah dipakai maupun nama yang sama dengan sumber | baris galat `... sudah ada - sumber tidak diubah` | `test_f6_rename_dilarang_menimpa_nama_yang_sudah_ada`, `test_f6_rename_nama_sama_dengan_sumber_ditolak` |
| Nama tujuan harus satu nama, bukan jalur | `rename` menolak `new_name` yang memuat `/` (plus `""`, `.`, `..`, di atas `MAX_NAME_LEN`) | `AlasanRenameGalat::Miring` + label `MSG_RENAME_MIRING` | `test_f6_rename_menolak_nama_tujuan_tidak_sah`, `test_f6_pieces_tolakan_rename_pakai_satu_label_galat` |
| Kedua nama rename dijaga dari pemotongan | `Invocation::arg_terpotong(i)` (flag per posisi) + kebijakan `rename_argumen_terpotong` di `asl-kernel-core`; posisi pertama sudah dijaga penolakan F4 | baris galat `MSG_ARG_CUT` yang sama, dengan potongan argumen kedua sebagai teks yang disebut | `test_f6_rename_argumen_terpotong_dapat_dibaca_per_posisi`, `test_f6_rename_argumen_kedua_terpotong_punya_kebijakan_sendiri` |
| Subtree anak ikut bergeser tanpa sentuhan | anak menunjuk induk lewat inode, sehingga hanya byte nama induk yang berubah; jumlah turunan dilaporkan `FileSystem::subtree_count` (bawaan `0`) | baris sukses direktori `... (<n> entri subtree ikut bergeser)` | `test_f6_rename_direktori_yang_berisi_sah`, `test_f6_rename_direktori_menggeser_seluruh_subtree` |
| Ruang kerja ditulis ulang bila terdampak | `geser_cwd_rename` di `asl-kernel-core` (cocok tepat atau awalan `<sumber>/`) | prompt sesudah rename, pengumuman `MSG_WORKDIR` | `test_f6_geser_cwd_hanya_bila_posisi_terkena`, `test_f6_geser_cwd_tidak_terkena_rename_berkas`, `test_f6_geser_cwd_menolak_memotong_diam_diam` |
| Objek akar tidak dapat diganti nama | basis kosong pada jalur hasil normalisasi di `jalankan_rename` | label `MSG_RENAME_AKAR` | `test_f6_pieces_tolakan_rename_pakai_satu_label_galat` + sesi QEMU F6 |
| rename tidak masuk mode berpanduan | `resolusi_pemandu()` hanya memetakan tiga perintah pembuatan; `rename` -> `TujuanPemandu::TidakAda` | jembatan `buka_pemandu` di `asl-kernel-core` berhenti sebelum mode terbuka | `test_f6_rename_tidak_dibimbing_mode_panduan` |

## Pasal 3 - Penyimpanan dan memori

| Fakta | Sumber satu-satunya | Yang menyusun dari sumber | Penjaga |
| --- | --- | --- | --- |
| Kapasitas RAMDISK 262.144 byte | `RAMDISK_CAPACITY_DEFAULT_BYTES` di `asl-kernel/asl-storage/src/ramdisk.rs` | keluaran `status`, angka pada `docs/PANDUAN/REFERENSI-PERINTAH.md` dan `KONFIGURASI-MEMORI.md` | `test_ram_filesystem_capacity_bounds`, `test_docs_capacity_figures_match_constants` |
| Ukuran slot berkas 8.192 byte | `RAMDISK_FILE_SLOT_BYTES` | batas tulis `write-file`, dokumen | dua test di atas |
| Batas bawah/atas kapasitas | `RAMDISK_CAPACITY_MIN_BYTES`, `RAMDISK_CAPACITY_MAX_BYTES` | keputusan konfigurasi misi | `test_ram_filesystem_capacity_bounds` |
| Batas panjang path 128 byte | `MAX_PATH_LEN` di `asl-kernel/asl-storage/src/vfs.rs` | penyangga normalisasi jalur dan pemotongan posisi prompt di `asl-kernel-core`, baris galat batas pada Pasal 7 `docs/PANDUAN/REFERENSI-PERINTAH.md` | `test_f3_dokumen_memuat_batas_panjang_path`, `test_f1_prompt_truncates_long_position_and_keeps_suffix` |
| Batas nama berkas = batas satu token argumen | `MAX_NAME_LEN` di `asl-kernel/asl-storage/src/vfs.rs` (diikat sama dengan batas argumen parser) | setiap nama yang muat diketik harus muat dibuat; nama di luar batas ditolak sebelum VFS tersentuh | `test_f4_batas_argumen_sama_dengan_batas_nama` |
| Instance yang dipakai kernel | `RAMDISK_FS` (statis `.bss`, `new_bss_zeroed()` + `ensure_root()`) di `asl-kernel/asl-kernel-core/src/lib.rs` | jembatan shell ke VFS | `test_zeroed_static_requires_ensure_root` |
| Antarmuka berkas | `trait FileSystem` di `asl-kernel/asl-storage/src/vfs.rs` | implementasi ramdisk dan jembatan shell | test ramdisk + test jalur ganda shell |
| Ambang gerbang memori | `-MissionRam 3M`, `-RatchetRam 3392K` di `scripts/test_ram_floor_3mb.ps1` | langkah `[4/5]` `scripts/ci_local.ps1` | kode keluar 0/1/2 skrip |
| Tanggal dan GUID volume ISO | konstanta epoch bawaan di `scripts/build_kernel.ps1` | reproduksibilitas SHA256 ISO | dua build byte-identik pada tahap rilis |

## Pasal 4 - Tahap boot yang tercetak di serial

Label berikut adalah penanda yang dibaca checker gerbang memori dan pipeline
smoke. `M6` plus prompt shell adalah syarat lulus smoke boot.

| Label | Dicetak di | Arti |
| --- | --- | --- |
| `[ASL-LAUNCHER] M1` | `asl-kernel-bin/src/main.rs` | masuk stub Limine |
| `[ASL-LAUNCHER] M2` | `asl-kernel-bin/src/main.rs` | handover diadaptasi, peta memori dibersihkan |
| `[ASL-LAUNCHER] M3` | `asl-kernel-bin/src/main.rs` | kendali pindah ke inti mikrokernel |
| `[ASL-CORE] M0` | `asl-kernel-core/src/lib.rs` | infrastruktur galat minimum (IDT/TSS/IST1) |
| `[ASL-CORE] M0+` | `asl-kernel-core/src/lib.rs` | carve-out core sekunder tervalidasi, latch failover terpasang |
| `[ASL-CORE] M4` | `asl-kernel-core/src/lib.rs` | buddy allocator dan heap aktif |
| `[ASL-CORE] M4+` | `asl-kernel-core/src/lib.rs` | guard page stack kernel ditegakkan |
| `[ASL-CORE] M5` | `asl-kernel-core/src/lib.rs` | partisi ARINC 653 dan scheduler deterministik aktif |
| `[ASL-CORE] M6` | `asl-kernel-core/src/lib.rs` | cold boot selesai, masuk idle loop dan shell |

## Pasal 5 - Status klaim pada rilis ini

| Area | Kelas bukti | Catatan |
| --- | --- | --- |
| 13 perintah kanonik + 10 alias, jembatan shell ke RAMDISK | `VERIFIED-ON-HOST` | test registry, parser, editor baris, dan sinkron dokumen |
| Keluaran shell eksplisit: satu label `Kesalahan:`, jalur yang gagal disebut, jumlah byte/entri diumumkan | `VERIFIED-ON-HOST` untuk teks label dan susunan baris; `VERIFIED-ON-VM` untuk baris yang sungguh tercetak | dua test penjaga label dan satu test angka path di sisi host; sesi QEMU 256M pada `Analysis/reports/f3_keluaran_uji_2026-10-22.txt` bagian [7] |
| Boot sampai prompt dan sesi shell di QEMU | `VERIFIED-ON-VM` | berkas log serial; sesi yang disaksikan pemilik dijadwalkan pada gerbang G9 |
| Uji host dan build release di runner GitHub | `VERIFIED-ON-CI` | workflow hanya menempuh langkah uji host dan build; boot QEMU tidak berjalan di runner |
| Gerbang boot `-m 3M` | MERAH, `DEVIASI TERCATAT` | lihat `docs/PANDUAN/KONFIGURASI-MEMORI.md` Pasal 4 dan `ACCOUNTABILITY_FAULT_LOG` entri ke-23 |
| Buat/hapus berkas pada FAT32 | `RANCANGAN`, sengaja dilepas dari v1.0.2 | lingkup STRETCH; catatan di laporan akhir misi |
| Riwayat baris (panah atas/bawah) | `RANCANGAN`, tidak dijanjikan | mesin penyunting baris sudah teruji untuk sisip/hapus/pindah |
| Perangkat keras fisis | tidak ada klaim | belum diukur; kelas `VERIFIED-ON-HARDWARE` tidak dipakai |

## Pasal 6 - Artefak yang terikat sidik jari

- Build ELF rilis pada tree ini: 325.528 byte, 0 error, 0 warning
  (`VERIFIED-ON-HOST`).
- Suntingan yang hanya hidup di dalam `#[cfg(test)]` tetap mengubah SHA256 berkas
  ELF, walaupun segmen yang dimuat tidak berubah. Pengukuran pada tree ini:
  program header identik dan `.text`, `.rodata`, `.data` byte-identik; yang
  berbeda hanya `.symtab` dan `.strtab` akibat 11 dari 327 simbol mengganti
  sufiks internal `.llvm.<hash>` pada alamat dan ukuran yang sama.
- Karena itu SHA256 ELF dan ISO hanya boleh dikutip bersama tree tempat ia diukur,
  dan setiap tahap yang mengandalkan angka boot harus mengukur ulang pada build yang
  akan diterbitkan.
- KOREKSI TERUKUR untuk tahap P7 (menggantikan klaim bahwa direktori build mengubah
  SHA256): tiga direktori build berbeda pada tree yang sama
  (`asl-kernel\target`, `logs\target_rustflags_trap`, `logs\target_p7check` yang
  dibangun dari nol) menghasilkan ELF 325.528 byte dengan SHA256 yang SAMA,
  `9203864ea6feae5bd3d51abd0cd3dc69ddcec0b8e91cd5401b1c63074a27b216`
  (`VERIFIED-ON-HOST`). Satu-satunya faktor yang terbukti mengubah SHA256 adalah
  ISI tree. Angka `494a814a...` yang dikutip pada pesan commit tahap P5 dan P6 tidak
  dapat direproduksi pada pengukuran ulang dan dinyatakan batal; lihat
  `ACCOUNTABILITY_FAULT_LOG` entri ke-24.
