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
| Prompt `ASL-KERNEL> ` | `SHELL_PROMPT` di berkas yang sama | seluruh situs tampil; dibaca juga oleh skrip gerbang memori | `test_release_identity_single_source`, parser `scripts/test_ram_floor_3mb.ps1` |
| Daftar perintah kanonik | `REGISTRY` + `CANONICAL_COMMAND_COUNT` di berkas yang sama | keluaran `help`, jumlah pada `status`, `docs/PANDUAN/REFERENSI-PERINTAH.md` | `test_registry_has_eleven_canonical_commands`, `test_docs_command_reference_matches_registry` |
| Alias parser | `aliases()` tiap entri registry | resolusi parser, baris `alias:` pada `help` | `test_parser_resolves_every_documented_alias` |
| Judul kelompok perintah | `CommandCategory::label()` | seksi pada keluaran `help`, kolom Kelompok pada dokumen panduan | `test_help_text_is_generated_from_registry` |
| Batas baris input | `SHELL_BUFFER_MAX` | penyangga shell, uji limpahan | `test_shell_buffer_overflow_protection` |

## Pasal 3 - Penyimpanan dan memori

| Fakta | Sumber satu-satunya | Yang menyusun dari sumber | Penjaga |
| --- | --- | --- | --- |
| Kapasitas RAMDISK 262.144 byte | `RAMDISK_CAPACITY_DEFAULT_BYTES` di `asl-kernel/asl-storage/src/ramdisk.rs` | keluaran `status`, angka pada `docs/PANDUAN/REFERENSI-PERINTAH.md` dan `KONFIGURASI-MEMORI.md` | `test_ram_filesystem_capacity_bounds`, `test_docs_capacity_figures_match_constants` |
| Ukuran slot berkas 8.192 byte | `RAMDISK_FILE_SLOT_BYTES` | batas tulis `write-file`, dokumen | dua test di atas |
| Batas bawah/atas kapasitas | `RAMDISK_CAPACITY_MIN_BYTES`, `RAMDISK_CAPACITY_MAX_BYTES` | keputusan konfigurasi misi | `test_ram_filesystem_capacity_bounds` |
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
| 11 perintah kanonik + 9 alias, jembatan shell ke RAMDISK | `VERIFIED-ON-HOST` | test registry, parser, editor baris, dan sinkron dokumen |
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
- Karena itu SHA256 ELF dan ISO hanya boleh dikutip bersama tree dan direktori
  build tempat ia diukur, dan setiap tahap yang mengandalkan angka boot harus
  mengukur ulang pada build yang akan diterbitkan.
