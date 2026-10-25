# -*- coding: ascii -*-
"""K-4 - salinan READ-ONLY keadaan media SASARAN SEBELUM di-flash.

NOL kata kerja tulis: yang dijalankan hanya Get-Disk / Get-Partition / Get-Volume /
Get-CimInstance dan `cmd /c dir`. Tujuan: keadaan awal media yang boleh dihapus
(label, filesystem, kapasitas, isi tingkat atas) tercatat SEBELUM pemilik menjalankan
perkakas flash, supaya klaim "media ini memang milik saya dan boleh dihapus" punya
dasar ukur, bukan ingatan.

Nomor disk WAJIB diberikan lewat --disk (tidak ada nilai bawaan): alat ini tidak boleh
menebak media mana yang akan di-flash. Disk yang menjadi boot atau sistem Windows
DITOLAK sebagai sasaran, meski nomornya diberikan.

Catatan cacat alat (ACCOUNTABILITY_FAULT_LOG entri ke-66): versi naskah scratch
memanggil `cmd /c dir` dengan satu argumen "%s:\\ /a /-p /o:gn", yang diperlakukan cmd
sebagai SATU nama direktori sehingga keluarannya salah (`Directory of F:\\ \\a \\-p`,
exit 1). Versi ini memecah argumen perbutir dan memvalidasi huruf volume.
"""
import argparse
import hashlib
import json
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

PS = r"""
$n = __NOMOR__
$d = Get-Disk -Number $n
$p = @(Get-Partition -DiskNumber $n -ErrorAction SilentlyContinue)
$v = @()
foreach ($x in $p) {
  $q = Get-Volume -Partition $x -ErrorAction SilentlyContinue
  if ($q) { $v += [pscustomobject]@{L=[string]$q.DriveLetter; FS=$q.FileSystem; LS=$q.FileSystemLabel;
                                     H=[string]$q.HealthStatus; Size=$q.Size; Free=$q.SizeRemaining;
                                     P=$x.PartitionNumber; PT=$x.PartitionType; Off=$x.Offset} }
}
$w = Get-CimInstance Win32_DiskDrive -Filter "Index=$n"
[pscustomobject]@{
  Nomor=$n; Model=$w.Model; Interface=$w.InterfaceType; UkuranDisk=$d.Size;
  Style=$d.PartitionStyle; ReadOnly=$d.IsReadOnly; Offline=$d.IsOffline;
  IsBoot=[string]$d.IsBoot; IsSystem=[string]$d.IsSystem;
  LogSec=$d.LogicalSectorSize; FisSec=$d.PhysicalSectorSize; Alert=$d.AllocationUnitSize;
  JumlahPartisi=$p.Count; Volume=$v
} | ConvertTo-Json -Compress -Depth 4
"""


def ps_disk(nomor):
    if not re.fullmatch(r"\d+", str(nomor)):
        raise RuntimeError("nomor disk harus angka, bukan %r" % nomor)
    return PS.replace("__NOMOR__", str(nomor))


def ambil_identitas(nomor):
    p = subprocess.run(["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass",
                        "-Command", ps_disk(nomor)], cwd=ROOT, shell=False,
                       capture_output=True)
    if p.returncode != 0:
        raise RuntimeError("powershell gagal (exit %d): %s" % (
            p.returncode, p.stderr.decode("utf-8", "replace")[:400]))
    raw = p.stdout.decode("utf-8", "replace").strip()
    m = re.search(r"\{.*\}", raw, re.S)
    if not m:
        raise RuntimeError("keluaran powershell tidak berbentuk JSON: %r" % raw[:200])
    d = json.loads(m.group(0))
    for k in ("IsBoot", "IsSystem"):
        if str(d.get(k)).strip().lower() == "true":
            raise RuntimeError("disk %s berlabel %s=true - bukan sasaran flash yang sah" % (
                nomor, k))
    return d


def isi_volume(huruf):
    if not re.fullmatch(r"[A-Za-z]", str(huruf)):
        raise RuntimeError("huruf volume tidak terbentuk satu huruf: %r" % huruf)
    q = subprocess.run(["cmd", "/c", "dir", "%s:\\" % huruf, "/a", "/-p", "/o:gn"],
                       shell=False, capture_output=True)
    return {"exit": q.returncode, "teks": q.stdout.decode("cp437", "replace")}


def utama(argv=None):
    ap = argparse.ArgumentParser(description="keadaan media READ-ONLY sebelum flash (K-4)")
    ap.add_argument("--disk", required=True, help="nomor disk sasaran (Get-Disk -Number)")
    ap.add_argument("--out", default=os.path.join(ROOT, "logs", "k4_media_state.json"))
    a = ap.parse_args(argv)

    d = ambil_identitas(a.disk)
    n = int(d["Nomor"])
    huruf = sorted({(v["L"] or "").strip(":") for v in (d["Volume"] or []) if v.get("L")})
    isi = {h: isi_volume(h) for h in huruf}
    hasil = {"nomor_disk": n, "identitas": d, "huruf_volume": huruf,
             "isi_tingkat_atas": {h: {"exit": isi[h]["exit"],
                                      "baris": len(isi[h]["teks"].splitlines()),
                                      "teks": isi[h]["teks"]} for h in huruf},
             "sumber": "Get-Disk/Get-Partition/Get-Volume/Get-CimInstance + cmd dir (READ-ONLY)",
             "nol_tulis": True,
             "isol_ascii": {h: sum(1 for x in isi[h]["teks"].encode("utf-8", "replace")
                                   if x > 127) for h in huruf}}
    os.makedirs(os.path.dirname(a.out), exist_ok=True)
    json.dump(hasil, open(a.out, "w", encoding="utf-8"), ensure_ascii=False, indent=1,
              sort_keys=True)
    mentah = open(a.out, "rb").read()

    print("disk %d | %s | %s | ukuran=%d B | style=%s | readonly=%s | sektor logik=%s" % (
        n, d["Interface"], (d["Model"] or "").strip(), d["UkuranDisk"], d["Style"],
        d["ReadOnly"], d["LogSec"]))
    print("partisi=%d volume=%d huruf=%s" % (d["JumlahPartisi"], len(d["Volume"] or []), huruf))
    for v in (d["Volume"] or []):
        print("  partisi %s tipe=%r offset=%s | %s: label=%r fs=%s sehat=%s ukuran=%s "
              "tersedia=%s" % (v["P"], v["PT"], v["Off"], v["L"], v["LS"], v["FS"], v["H"],
                               v["Size"], v["Free"]))
    for h in huruf:
        baris = [l for l in isi[h]["teks"].splitlines() if l.strip()]
        print("  isi %s: (exit %d, %d baris keluaran dir)" % (h, isi[h]["exit"], len(baris)))
        for l in baris[:24]:
            print("    %s" % l.strip()[:110])
    print("nol tulis ke media: benar (hanya baca) | berkas: %s" % a.out)
    print("SHA256 berkas keluaran: %s | byte=%d" % (
        hashlib.sha256(mentah).hexdigest(), len(mentah)))
    return 0


if __name__ == "__main__":
    sys.exit(utama())
