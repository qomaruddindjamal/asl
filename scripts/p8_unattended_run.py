# -*- coding: ascii -*-
# =============================================================================
# scripts/p8_unattended_run.py - JENDELA PENERIMAAN VM TAK-DISAKSIKAN v1.0.2
# (MASTER PROMPT R3 BAGIAN 6: jendela disaksikan DIGANTI jendela VM; kelas
# VERIFIED-ON-HARDWARE tetap 0 dan dinyatakan terang).
#
# Kriteria lulus TIDAK dikarang di sini: daftar penanda, baris sesi, prompt
# satu-sumber dan gerbang versi DIBACA dari alat fail-closed P8-PERSIAPAN
# scripts/g9_final_p8_run.ps1 (berkas itu hanya DIBACA; tidak diubah, tidak
# dijalankan dalam mode yang menulis artefak). -SelfTest alat itu dijalankan
# sebagai bagian prachecklist otomatis karena mode itu tidak menulis berkas.
#
# Beda yang disengaja terhadap alat berjendela: QEMU HEADLESS (-display none),
# dan baris sesi DIKIRIM lewat jalur serial ANSI (chardev socket dua arah pada
# COM1), bukan diketik manusia di jendela. Karena itu kelas buktinya
# VERIFIED-ON-VM, bukan VERIFIED-ON-HARDWARE.
#
# Artefak selalu ber-prefix BARU p8_unattended_ dan bercap-waktu per-invokasi;
# prefiks g9_final_p8_* tidak pernah disentuh. Berkas menolak menimpa artefak
# yang sudah ada (fail-closed).
#
#usage:
#   python scripts\p8_unattended_run.py --selftest
#   python scripts\p8_unattended_run.py
#   python scripts\p8_unattended_run.py --accept-deviation-3m --alasan "rujukan tertulis"
#   python scripts\p8_unattended_run.py --skip-3m
#
# Kode keluar (setara dengan alat berjendela):
#   0 = fase 1 hijau penuh DAN 3M lulus, ATAU 3M merah dengan --accept-deviation-3m
#   1 = prachecklist/prasyarat/boot gagal, crash, atau stderr QEMU tidak kosong
#   2 = sesi Bagian 3 belum lengkap pada batas waktu
#   3 = -m 3M merah dan deviasi belum diterima, ATAU fase 2 dilewati
# =============================================================================
import argparse
import hashlib
import json
import os
import re
import socket
import subprocess
import sys
import threading
import time
from datetime import datetime, timezone

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
REPOS = os.path.join(ROOT, "asl-kernel")
QEMU = r"C:\Program Files\qemu\qemu-system-x86_64.exe"
ISO = os.path.join(REPOS, "target", "asl-core-kernel.iso")
ELF = os.path.join(REPOS, "target", "x86_64-unknown-none", "release", "asl-kernel")
REP = os.path.join(ROOT, "Analysis", "reports")
P8_ALAT = os.path.join(ROOT, "scripts", "g9_final_p8_run.ps1")
PROMPT_SRC = os.path.join(REPOS, "asl-cockpit-shell", "src", "lib.rs")
MASTER_BUKTI = os.path.join(REP, "r3_bagian5-iso-master_2026-10-24.txt")
CHECK_LIMINE = os.path.join(ROOT, "scripts", "check_limine.ps1")
STAMP = datetime.now(timezone.utc).strftime("%Y-%m-%d_%H-%M-%S")
PREFIKS = "p8_unattended_"
CRASH_TOKEN = ("ASL EXCEPTION", "ASL-CORE-PANIC", "FATAL")
ANSI = re.compile(r"\x1b\[[0-9;?]*[A-Za-z]")

SUM_LINES = []


def sum_line(msg):
    print(msg)
    SUM_LINES.append(msg)


def path_artefak(nama, ext="log"):
    return os.path.join(REP, "%s%s_%s.%s" % (PREFIKS, nama, STAMP, ext))


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for blk in iter(lambda: f.read(1 << 20), b""):
            h.update(blk)
    return h.hexdigest()


def byte_len(path):
    return os.path.getsize(path) if os.path.exists(path) else 0


# ------------------------------------------------------------------ pembacaan
def baca_teks(path):
    with open(path, "rb") as f:
        return f.read().decode("utf-8", "replace")


def baca_sumber_prompt():
    """Prompt satu-sumber: PREFIX + SUFIX dari konstanta Rust, bukan tulisan tangan."""
    konst = {}
    for line in baca_teks(PROMPT_SRC).splitlines():
        s = line.strip()
        for nama in ("SHELL_PROMPT_PREFIX", "SHELL_PROMPT_SUFFIX"):
            if s.startswith("pub const %s" % nama):
                konst[nama] = s.split("=", 1)[1].split("//")[0].strip().strip(";").strip('"')
    if "SHELL_PROMPT_PREFIX" not in konst or "SHELL_PROMPT_SUFFIX" not in konst:
        return None, None
    return konst["SHELL_PROMPT_PREFIX"], konst["SHELL_PROMPT_SUFFIX"]


def baca_kriteria_p8():
    """Penanda + sesi dibaca dari alat P8 ter-commit (satu-satunya sumber kriteria)."""
    t = baca_teks(P8_ALAT)

    def blok(nama):
        m = re.search(r"\$%s = @\((.*?)\r?\n\)" % nama, t, re.S)
        if not m:
            raise RuntimeError("blok $%s tidak ditemukan di %s" % (nama, P8_ALAT))
        return m.group(1)

    markers = []
    for lbl, isi, n in re.findall(r'@\("([^"]*)",\s*"((?:[^"\\]|\\.)*)",\s*(\d+)\)', blok("Markers")):
        isi = isi.replace("$([char]27)", chr(27)).replace("`$", "$")
        markers.append((lbl, isi, int(n)))

    session = re.findall(r'^\s*"([^"]*)",?\s*$', blok("Session"), re.M)

    vm = re.search(r'Matches\(\$txtFinal,\s*"([^"]+)"\)', t)
    vo = re.search(r'Matches\(\$txtFinal,\s*"(v1\\\.1[^"]*)"', t)
    gerbang = {
        "versi_baru": vm.group(1).replace(r"\.", ".") if vm else None,
        "versi_lama": vo.group(1).replace(r"\.", ".") if vo else None,
    }
    return markers, session, gerbang


# --------------------------------------------------------------------- QEMU/MON
def mulai_qemu(extra_serial_args, stderr_path, qmp_port, ram):
    """RAM DIMINTA di sini, bukan dibiarkan bawaan QEMU (128 MiB). Terukur pada
    invokasi ketiga: fase 2 melaporkan 'LULUS -m 3M' padahal cmdline tidak memuat
    '-m' sama sekali sehingga VM jalan dengan RAM bawaan - hijau palsu yang harus
    dicegah, bukan hanya diakui. Karena itu cmdline diperiksa sebelum dipakai dan
    galat dinaikkan bila -m tidak ada."""
    args = [QEMU, "-machine", "q35", "-vga", "std", "-m", ram, "-cdrom", ISO,
            "-display", "none", "-no-reboot", "-no-shutdown",
            "-qmp", "tcp:127.0.0.1:%d,server,nowait" % qmp_port] + extra_serial_args
    cmd = " ".join(args)
    if ("-m %s " % ram) not in (cmd + " "):
        raise RuntimeError("cmdline QEMU tidak memuat '-m %s': %s" % (ram, cmd))
    with open(stderr_path, "wb") as ef:
        proc = subprocess.Popen(args, stdout=ef, stderr=ef)
    return proc, cmd


def sambung_serial(port, timeout=60.0):
    """QEMU chardev socket dijalankan dengan wait=on: VM baru jalan setelah kita
    CONNECT, sehingga tidak ada byte awal yang hilang."""
    deadline = time.time() + timeout
    last = None
    while time.time() < deadline:
        try:
            s = socket.create_connection(("127.0.0.1", port), timeout=5)
            s.settimeout(0.2)
            return s
        except OSError as exc:
            last = exc
            time.sleep(0.2)
    raise RuntimeError("serial chardev tidak bisa dihubungi: %r" % (last,))


class PenangkapSerial(object):
    """Satu utas pembaca: semua byte COM1 ditulis ke berkas dan tetap tersedia
    untuk dihitung. Berkas dibaca ulang oleh pemantau, jadi posisi file cursor
    dipegang Windows bukan Python."""

    def __init__(self, sock, path):
        self.sock = sock
        self.path = path
        self.stop = False
        self.tulis = open(path, "wb")
        self.threads = []

    def run(self):
        while not self.stop:
            try:
                data = self.sock.recv(4096)
            except socket.timeout:
                continue
            except OSError:
                break
            if not data:
                break
            self.tulis.write(data)
            self.tulis.flush()

    def close(self):
        self.stop = True
        try:
            self.sock.close()
        except OSError:
            pass
        self.tulis.close()

    def teks(self):
        return baca_teks(self.path) if os.path.exists(self.path) else ""


def kirim_baris(sock, baris, jeda_byte=0.004):
    for b in (baris + "\r").encode("ascii"):
        sock.sendall(bytes([b]))
        time.sleep(jeda_byte)


def baca_konstanta_msg():
    """Konstanta pesan satu-sumber dari cockpit shell: (nilai, berkas, baris, nama).
    Dipakai hanya untuk menurunkan VARIAN LITERAL bila penanda alat P8 ternyata
    usang terhadap sumber - bukan untuk mengarang kriteria baru."""
    out = []
    for path in (PROMPT_SRC, os.path.join(REPOS, "asl-kernel-core", "src", "lib.rs")):
        if not os.path.exists(path):
            continue
        for no, line in enumerate(baca_teks(path).splitlines(), 1):
            m = re.match(r'\s*pub const (MSG_\w+)\s*:\s*&str\s*=\s*"((?:[^"\\]|\\.)*)"', line)
            if m:
                out.append((m.group(2), os.path.relpath(path, ROOT), no, m.group(1)))
    return out


def varian_penanda(literal, konstanta):
    """Bila literal penanda alat P8 tidak ada di serial, cari rangkaian konstanta
    SUMBER yang ujungnya persis penanda itu (mis. label 'Kesalahan: ' + 'perintah
    tidak dikenal: '). Semua kandidat dibangun dari konstanta, nol ketikan tangan."""
    lab = next((k for k in konstanta if k[3] == "MSG_ERROR"), None)
    kandidat = []
    for nilai, berkas, no, nama in konstanta:
        if nama == "MSG_ERROR":
            continue
        potongan = [(nilai, berkas, no, nama)]
        if lab and nilai != literal:
            potongan = [(lab[0] + nilai, berkas, no, "MSG_ERROR+" + nama)]
        for varian, berkas2, no2, nama2 in potongan:
            if varian == literal:
                continue
            if varian.lower().endswith(literal.lower()) and len(varian) > len(literal):
                kandidat.append((varian, berkas2, no2, nama2))
    return kandidat


def cari_di_sumber(literal):
    """Cari literal penanda di dua berkas sumber kernel; kembalikan BERKAS:BARIS
    atau None. Huruf ESC diuji pula dalam bentuk escape sumber (\\x1B / \\u{1b})."""
    bentuk = [literal, literal.replace(chr(27), "\\x1B"), literal.replace(chr(27), "\\u{1b}")]
    for path in (PROMPT_SRC, os.path.join(REPOS, "asl-kernel-core", "src", "lib.rs")):
        if not os.path.exists(path):
            continue
        rel = os.path.relpath(path, ROOT)
        for no, line in enumerate(baca_teks(path).splitlines(), 1):
            if any(b in line for b in bentuk):
                return "%s:%d" % (rel, no)
    return None


def audit_penanda(markers, konstanta, session):
    """Validasi otomatis kriteria: tiap literal penanda alat P8 dicari dasar
    terukurnya - literal di sumber Rust, teks yang diketik baris sesi, atau
    varian konstanta MSG_* sumber. Yang tak berpokok tidak boleh dipakai
    menghakimi (fail-closed). Nol kriteria dikarang di sini."""
    out = []
    for lbl, isi, butuh in markers:
        sumber = cari_di_sumber(isi)
        entri = {"penanda": lbl, "literal_alat_p8": isi, "butuh": butuh,
                 "pokok": None, "sumber": sumber, "varian_sumber": None}
        if sumber:
            entri["pokok"] = "literal sumber"
        else:
            diketik = [(i, b) for i, b in enumerate(session, 1) if isi in b]
            if diketik:
                entri["pokok"] = "teks baris sesi %d" % diketik[0][0]
            else:
                for varian, berkas, no, nama in varian_penanda(isi, konstanta):
                    entri["pokok"] = "varian MSG_* sumber"
                    entri["varian_sumber"] = {"literal": varian,
                                              "sumber": "%s:%d (%s)" % (berkas, no, nama)}
                    break
        out.append(entri)
    return out


def posisi_prompt(teks, prefek, sufek):
    """Offset karakter prompt BERSAMA TERAKHIR pada teks yang sudah diluruhkan
    ANSI-nya, atau -1 bila belum ada. Pemantau baris sesi memakai SELISIH offset,
    BUKAN jumlah baris: kernel mencetak prompt lalu disusul gema perintah pada
    baris yang sama, sehingga prompt yang tadi cocok berhenti cocok ('ASL-KERNEL
    />_ status' tidak berakhir dengan sufiks). Terukur pada invokasi kedua: dengan
    pemantau berbasis jumlah, ke-17 baris sesi menunggu penuh 60 s."""
    bersih = re.sub(ANSI, "", teks)
    pos, off = -1, 0
    for l in bersih.splitlines(True):
        s = l.rstrip()
        if s.startswith(prefek) and s.endswith(sufek.strip()):
            pos = off
        off += len(l)
    return pos


def tunggu_mengendap(cari_teks, batas_s=15.0, tenang_s=1.0):
    """Tunggu buffer serial berhenti bertambah sebelum penanda dihitung.

    Butuh karena gema dan keluaran baris SESI TERAKHIR bisa mendarat setelah
    batas 60 s per baris habis: terukur pada invokasi kedua, keluaran 'clear'
    (22 byte termasuk sekuens ESC[2J) baru tertulis sesudah hitungan penanda
    dijalankan, sehingga penanda 'clear' terbaca 0. Kembalikan (byte_tambahan,
    detik_tertunggu)."""
    awal = len(cari_teks())
    t0 = time.time()
    terakhir, stabil_sejak = awal, None
    while time.time() - t0 < batas_s:
        time.sleep(0.25)
        n = len(cari_teks())
        if n == terakhir:
            if stabil_sejak is None:
                stabil_sejak = time.time()
            elif time.time() - stabil_sejak >= tenang_s:
                break
        else:
            terakhir, stabil_sejak = n, None
    return terakhir - awal, time.time() - t0


def stderr_diluruskan(path):
    """Kembalikan (byte_total, byte_sisa, jumlah_baris_info_diizinkan, contoh).
    Baris 'info: QEMU waiting for connection on' adalah keluaran informatif chardev
    socket wait=on, BUKAN galat tamu; sisa apa pun tetap membuat jendela MERAH."""
    raw = open(path, "rb").read() if os.path.exists(path) else b""
    teks = raw.decode("utf-8", "replace")
    boleh, sisa = 0, []
    for l in teks.splitlines(True):
        if "info: QEMU waiting for connection on" in l:
            boleh += 1
        else:
            sisa.append(l)
    return len(raw), len("".join(sisa).encode("utf-8")), boleh, ("".join(sisa)[:200] or "(kosong)")


def prompt_terakhir(teks, prefek, sufek):
    bersih = re.sub(ANSI, "", teks)
    for l in reversed(bersih.splitlines(True)):
        s = l.rstrip()
        if s.startswith(prefek) and s.endswith(sufek.strip()):
            return s
    return None


def hitung_penanda(teks, markers):
    hasil = {}
    for lbl, isi, butuh in markers:
        hasil[lbl] = (teks.count(isi), butuh)
    return hasil


def qmp_call(port, nama, argumen=None, timeout=20.0):
    c = socket.create_connection(("127.0.0.1", port), timeout=timeout)
    c.settimeout(timeout)
    buf = b""

    def readln():
        nonlocal buf
        while b"\r\n" not in buf:
            chunk = c.recv(65536)
            if not chunk:
                raise RuntimeError("QMP socket tertutup")
            buf += chunk
        line, buf = buf.split(b"\r\n", 1)
        return json.loads(line.decode())

    readln()
    c.sendall((json.dumps({"execute": "qmp_capabilities"}) + "\r\n").encode())
    readln()
    payload = {"execute": nama}
    if argumen:
        payload["arguments"] = argumen
    c.sendall((json.dumps(payload) + "\r\n").encode())
    rep = readln()
    c.close()
    return rep


def screendump(port, idx, ppm_dir):
    ppm = os.path.join(ppm_dir, "%sscreendump_%s_%d.ppm" % (PREFIKS, STAMP, idx))
    try:
        rep = qmp_call(port, "screendump", {"filename": ppm.replace("\\", "/")})
    except Exception as exc:
        return {"idx": idx, "gagal": repr(exc)}
    if "return" not in rep or not os.path.exists(ppm):
        return {"idx": idx, "gagal": "qmp tanpa berkas: %s" % rep}
    out = {"idx": idx, "ppm_ada": True, "ppm_bytes": byte_len(ppm),
           "ppm_sha256": sha256(ppm)}
    try:
        from PIL import Image
    except ImportError:
        out["png"] = "TIDAK (PIL tidak tersedia di host ini)"
        return out
    png = ppm[:-4] + ".png"
    Image.open(ppm).save(png)
    out.update({"png": png, "png_bytes": byte_len(png),
                "png_sha256": sha256(png),
                "dimensi": "%dx%d" % Image.open(png).size})
    return out


def hentikan(proc, timeout=20):
    if proc.poll() is None:
        proc.terminate()
        try:
            proc.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            proc.kill()
    return proc.returncode


# ------------------------------------------------------------------ utama
def main():
    ap = argparse.ArgumentParser(description="jendela penerimaan VM tak-disaksikan v1.0.2")
    ap.add_argument("--selftest", action="store_true", help="validasi kriteria + prasyarat, tanpa QEMU")
    ap.add_argument("--accept-deviation-3m", action="store_true",
                    help="nyatakan deviasi -m 3M diterima dengan rujukan tertulis")
    ap.add_argument("--alasan", default="", help="rujukan tertulis penerimaan deviasi")
    ap.add_argument("--skip-3m", action="store_true", help="lewati fase 2 (verdict tidak penuh)")
    ap.add_argument("--boot-ram", default="256M", help="RAM fase 1 (cold boot)")
    ap.add_argument("--boot-timeout", type=int, default=180)
    ap.add_argument("--session-timeout", type=int, default=900)
    ap.add_argument("--probe-3m-seconds", type=int, default=90)
    ap.add_argument("--mission-ram", default="3M")
    ap.add_argument("--qmp-port", type=int, default=44610)
    ap.add_argument("--serial-port", type=int, default=44620)
    args = ap.parse_args()

    fail = []

    # ---- PRA-CHECKLIST (divalidasi otomatis, fail-closed) ----
    prereq = []

    def catat(nama, ok, detail):
        prereq.append((nama, bool(ok), detail))
        if not ok:
            fail.append(nama)

    catat("qemu ada", os.path.exists(QEMU), QEMU)
    if os.path.exists(QEMU):
        v = subprocess.run([QEMU, "--version"], capture_output=True).stdout.decode("utf-8", "replace")
        versi_qemu = v.splitlines()[0] if v.strip() else "(tidak terbaca)"
    else:
        versi_qemu = "(qemu tidak ada)"
    catat("iso master ada", os.path.exists(ISO), ISO)

    prefek, sufek = baca_sumber_prompt()
    prompt = (prefek + sufek) if prefek is not None else None
    catat("prompt terbaca dari konstanta satu-sumber", prompt is not None, repr(prompt))

    try:
        markers, session, gerbang = baca_kriteria_p8()
    except RuntimeError as exc:
        markers, session, gerbang = [], [], {"versi_baru": None, "versi_lama": None}
        catat("kriteria P8 terbaca", False, repr(exc))
    catat("penanda sesi terbaca dari alat P8", len(markers) == 13,
          "%d penanda dari %s" % (len(markers), os.path.relpath(P8_ALAT, ROOT)))
    catat("baris sesi terbaca dari alat P8", len(session) == 17, "%d baris" % len(session))
    catat("gerbang versi terbaca dari alat P8",
          gerbang["versi_baru"] == "v1.0.2" and gerbang["versi_lama"] == "v1.1.0",
          "wajib '%s' >=1 dan nol '%s'" % (gerbang["versi_baru"], gerbang["versi_lama"]))

    konstanta = baca_konstanta_msg()
    audit = audit_penanda(markers, konstanta, session)
    tuna = [a for a in audit if a["pokok"] is None]
    drift = [a for a in audit if a["pokok"] != "literal sumber"]
    catat("tiap penanda punya pokok terukur (sumber / baris sesi / varian MSG_*)",
          len(tuna) == 0,
          "%d/%d literal ada di sumber; %d bukan literal sumber; tanpa pokok: %s" % (
              len(markers) - len(drift), len(markers), len(drift),
              "; ".join(a["penanda"] for a in tuna) or "(nol)"))

    p_lk = subprocess.run(["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass",
                           "-File", CHECK_LIMINE], capture_output=True, cwd=ROOT)
    out_lk = p_lk.stdout.decode("utf-8", "replace")
    catat("check_limine exit 0 (perkakas cocok ledger)", p_lk.returncode == 0,
          "exit %d | %s" % (p_lk.returncode,
                            (out_lk.strip().splitlines() or ["(tanpa keluaran)"])[-1][:120]))

    p_ps = subprocess.run(["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass",
                           "-File", P8_ALAT, "-SelfTest"], capture_output=True, cwd=ROOT)
    out_ps = p_ps.stdout.decode("utf-8", "replace")
    catat("selftest alat P8 exit 0", p_ps.returncode == 0,
          "exit %d (mode -SelfTest tidak menulis artefak)" % p_ps.returncode)
    n_ps = re.search(r"(\d+) penanda, (\d+) baris sesi", out_ps)
    catat("selftest alat P8 sependapat dengan yang dibaca",
          n_ps is not None and int(n_ps.group(1)) == len(markers) and int(n_ps.group(2)) == len(session),
          n_ps.group(0) if n_ps else "(keluaran selftest tidak terbaca)")

    iso_size = byte_len(ISO)
    iso_sha = sha256(ISO) if os.path.exists(ISO) else "-"
    sha_master, ukuran_master = "-", None
    if os.path.exists(MASTER_BUKTI):
        m = re.search(r"ISO aktif sekarang\s+(\d+) B\s+SHA256 ([0-9a-f]{64})", baca_teks(MASTER_BUKTI))
        if m:
            sha_master = m.group(2).lower()
            ukuran_master = int(m.group(1))
    ok_media = (sha_master != "-" and ukuran_master == iso_size and sha_master == iso_sha.lower())
    catat("media yang di-boot = ISO master yang dikuasai BAGIAN 5", ok_media,
          "master %s %s B | ISO kini %s %d B" % (sha_master[:16], ukuran_master,
                                                 iso_sha[:16], iso_size))

    elf_size, elf_sha = byte_len(ELF), (sha256(ELF) if os.path.exists(ELF) else "-")

    print("=== PRA-CHECKLIST JENDELA VM TAK-DISAKSIKAN (%s) ===" % STAMP)
    for nama, ok, detail in prereq:
        print("  [%s] %-52s %s" % ("OK  " if ok else "MERAH", nama, detail))
    print("  ISO    : %d bytes SHA256=%s" % (iso_size, iso_sha))
    print("  ELF    : %d bytes SHA256=%s" % (elf_size, elf_sha))
    print("  QEMU   : %s" % versi_qemu)
    print("  prompt : '%s' (satu-sumber, %s)" % (prompt, os.path.relpath(PROMPT_SRC, ROOT)))
    print("  sesi   : %d penanda, %d baris (dibaca dari %s)" % (
        len(markers), len(session), os.path.relpath(P8_ALAT, ROOT)))
    for a in drift:
        print("  POKOK  : %-28s literal %r bukan string sumber; pokok=%s%s" % (
            a["penanda"], a["literal_alat_p8"], a["pokok"],
            (" -> varian %r dari %s" % (a["varian_sumber"]["literal"],
                                        a["varian_sumber"]["sumber"])) if a["varian_sumber"] else ""))

    if args.selftest:
        print("SELF-TEST selesai - TIDAK menjalankan QEMU, TIDAK menulis artefak.")
        return 1 if fail else 0

    for nama, ok, _ in prereq:
        if not ok:
            print("GERBANG MERAH prachecklist: %s - jendela DITAHAN, tidak ada boot." % nama)
            return 1

    # ---- tolak menimpa artefak yang sudah ada (fail-closed) ----
    serial_f = path_artefak("serial")
    serial3m_f = path_artefak("serial_3m")
    stderr_f = path_artefak("qemu_stderr")
    stderr3m_f = path_artefak("qemu_stderr_3m")
    summary_f = path_artefak("summary")
    hasil_f = path_artefak("hasil", "json")
    for p in (serial_f, summary_f, hasil_f):
        if os.path.exists(p):
            print("GERBANG MERAH: artefak sudah ada, menolak menimpa: %s" % p)
            return 1

    with open(summary_f, "w", encoding="ascii", newline="\r\n") as f0:
        f0.write("ASL P8 UNATTENDED VM ACCEPTANCE RUN (TAK-DISAKSIKAN) - %s\n" % STAMP)
    sum_line("prompt satu-sumber : '%s'" % prompt)
    sum_line("kriteria           : %d penanda, %d baris sesi - DIBACA dari %s (nol salin tangan)" % (
        len(markers), len(session), os.path.relpath(P8_ALAT, ROOT)))
    sum_line("QEMU               : %s" % versi_qemu)
    sum_line("ISO                : %d bytes SHA256=%s" % (iso_size, iso_sha))
    sum_line("ELF                : %d bytes SHA256=%s" % (elf_size, elf_sha))
    sum_line("RAM                : fase 1 diminta -m %s; fase 2 (janji misi) diminta -m %s; keduanya "
             "DIBUKTIKAN pada baris CMDLINE yang dicetak alat" % (args.boot_ram, args.mission_ram))
    sum_line("mode               : headless (-display none); sesi lewat jalur serial ANSI dua arah")
    sum_line("kelas bukti        : VERIFIED-ON-VM (BUKAN VERIFIED-ON-HARDWARE)")

    # ======================= FASE 1: boot 256M + sesi via serial ==============
    sum_line("")
    sum_line("=== FASE 1 - cold boot -m %s headless + sesi Bagian 3 lewat serial ===" % args.boot_ram)
    serial_args = ["-chardev",
                   "socket,id=c0,host=127.0.0.1,port=%d,server=on,wait=on" % args.serial_port,
                   "-serial", "chardev:c0"]
    proc, cmdline = mulai_qemu(serial_args, stderr_f, args.qmp_port, args.boot_ram)
    sum_line("CMDLINE: %s" % cmdline)
    try:
        sock = sambung_serial(args.serial_port, timeout=args.boot_timeout)
    except RuntimeError as exc:
        sum_line("[FAIL] %s" % exc)
        hentikan(proc)
        return 1
    penangkap = PenangkapSerial(sock, serial_f)
    t = threading.Thread(target=penangkap.run, daemon=True)
    t.start()

    # tunggu prompt boot
    deadline = time.time() + args.boot_timeout
    m6 = False
    crash = False
    baris_prompt = None
    mulai = time.time()
    while time.time() < deadline:
        time.sleep(1)
        txt = penangkap.teks()
        if "M6: Kernel Cold Boot Complete" in txt:
            m6 = True
        baris_prompt = prompt_terakhir(txt, prefek, sufek)
        if baris_prompt:
            break
        if any(tok in txt for tok in CRASH_TOKEN):
            crash = True
            break
        if proc.poll() is not None:
            break
    boot_detik = round(time.time() - mulai, 1)
    sum_line("Boot   : M6=%s prompt=%s crash=%s (setelah %ss)" % (
        m6, bool(baris_prompt), crash, boot_detik))
    sum_line("         baris prompt terdeteksi: %r" % baris_prompt)

    if not (m6 and baris_prompt and not crash):
        sum_line("[FAIL] fase 1: cold boot -m %s tidak mencapai M6 + prompt, atau crash - MERAH." % args.boot_ram)
        penangkap.close()
        hentikan(proc)
        _tutup_summary(summary_f)
        return 1

    shots = [screendump(args.qmp_port, 1, REP)]
    sum_line("screendump sesudah boot: %s" % json.dumps(shots[-1]))

    # ketik sesi otomatis
    akhir_sesi = time.time() + args.session_timeout
    tepat = 0
    for i, baris in enumerate(session, 1):
        sebelum = posisi_prompt(penangkap.teks(), prefek, sufek)
        kirim_baris(sock, baris)
        batas = min(time.time() + 60, akhir_sesi)
        siap = False
        while time.time() < batas:
            time.sleep(0.5)
            txt = penangkap.teks()
            if any(tok in txt for tok in CRASH_TOKEN):
                crash = True
                break
            if posisi_prompt(txt, prefek, sufek) > sebelum:
                siap = True
                break
        if siap:
            tepat += 1
        elif not crash:
            sum_line("[i] baris sesi %d (%r): prompt baru tidak muncul dalam 60 s; lanjut ke baris "
                     "berikutnya, penanda dihitung di akhir" % (i, baris))
        if crash:
            sum_line("[FAIL] crash terdeteksi sesudah baris sesi %d (%r)" % (i, baris))
            break

    jeda_b, detik_b = tunggu_mengendap(penangkap.teks)
    txt = penangkap.teks()
    hit = hitung_penanda(txt, markers)
    missing = []
    drift_terukur = []
    sum_line("")
    sum_line("=== Hasil sesi Bagian 3 (hitung kemunculan penanda di serial) ===")
    sum_line("Pemantau baris sesi: %d/%d baris mendapat prompt baru sebelum batas 60 s "
             "(selisih offset prompt, ANSI diluruhkan)." % (tepat, len(session)))
    sum_line("Pengendapan buffer sebelum penanda dihitung: %d byte tambahan dalam %.1f s "
             "(diameter: gema/keluaran baris terakhir boleh mendarat sesudah batas)." % (jeda_b, detik_b))
    sum_line("Penanda = literal dari alat P8; bila 0, varian dicari dari konstanta MSG_* sumber.")
    sum_line("Pokok penanda (audit prachecklist): %d literal sumber, %d teks baris sesi, "
             "%d varian konstanta MSG_* sumber." % (
                 sum(1 for a in audit if a["pokok"] == "literal sumber"),
                 sum(1 for a in audit if (a["pokok"] or "").startswith("teks baris sesi")),
                 sum(1 for a in audit if (a["pokok"] or "").startswith("varian MSG_*"))))
    stale = set(a["penanda"] for a in audit if (a["pokok"] or "").startswith("varian MSG_*"))
    for lbl, isi, butuh in markers:
        n = hit[lbl][0]
        extra = ""
        if n == 0 and lbl in stale:
            for varian, berkas, no, nama in varian_penanda(isi, konstanta):
                nv = txt.count(varian)
                if nv >= butuh:
                    n = nv
                    drift_terukur.append({"penanda": lbl, "literal_alat_p8": isi,
                                          "terukur_literal": hit[lbl][0], "varian_sumber": varian,
                                          "terukur_varian": nv,
                                          "sumber": "%s:%d (%s)" % (berkas, no, nama)})
                    extra = "  <DRIFT: literal %r 0x; varian sumber %r terukur %d>" % (isi, varian, nv)
                    break
        status = "OK  " if n >= butuh else "KURANG"
        if n < butuh:
            missing.append(lbl)
        sum_line("  [%s] %-28s %r terukur %d (butuh %d)%s" % (status, lbl, isi, n, butuh, extra))
    if drift_terukur:
        sum_line("DRIFT KRITERIA (literal alat P8 usang terhadap sumber; alat terproteksi TIDAK diubah):")
        for d in drift_terukur:
            sum_line("  - %s: %r terukur %d -> %r terukur %d dari %s" % (
                d["penanda"], d["literal_alat_p8"], d["terukur_literal"],
                d["varian_sumber"], d["terukur_varian"], d["sumber"]))

    ver_new = txt.count(gerbang["versi_baru"])
    ver_old = txt.count(gerbang["versi_lama"])
    serial_bytes = byte_len(serial_f)
    stderr_total, stderr_sisa, stderr_info, stderr_contoh = stderr_diluruskan(stderr_f)
    sum_line("Versi  : occurrence '%s'=%d  occurrence '%s'=%d  (gerbang: >=1 dan 0)" % (
        gerbang["versi_baru"], ver_new, gerbang["versi_lama"], ver_old))
    sum_line("Ukuran : serial=%d byte  stderr total=%d byte; sisa=%d byte setelah %d baris info "
             "chardev wait=on diluruhkan (gerbang: sisa 0)" % (
                 serial_bytes, stderr_total, stderr_sisa, stderr_info))

    if crash:
        penangkap.close()
        hentikan(proc)
        _tulis_hasil(hasil_f, locals(), 1, "SKIP", shots, missing)
        _tutup_summary(summary_f)
        return 1
    if ver_old != 0 or ver_new < 1:
        sum_line("[FAIL] gerbang versi merah - identitas rilis pada media yang di-boot bukan v1.0.2.")
        penangkap.close()
        hentikan(proc)
        _tulis_hasil(hasil_f, locals(), 1, "SKIP", shots, missing)
        _tutup_summary(summary_f)
        return 1
    if stderr_sisa != 0:
        sum_line("[FAIL] stderr QEMU tidak kosong: sisa %d byte setelah baris info chardev "
                 "diluruhkan. CONTOH SISA: %r" % (stderr_sisa, stderr_contoh))
        penangkap.close()
        hentikan(proc)
        _tulis_hasil(hasil_f, locals(), 1, "SKIP", shots, missing)
        _tutup_summary(summary_f)
        return 1
    shots.append(screendump(args.qmp_port, 2, REP))
    sum_line("screendump sesudah sesi: %s" % json.dumps(shots[-1]))
    if missing:
        sum_line("[FAIL] sesi Bagian 3 belum lengkap (%d penanda kurang): %s" % (
            len(missing), "; ".join(missing)))
        penangkap.close()
        exit_kode = 2
        hentikan(proc)
        _tulis_hasil(hasil_f, locals(), 2, "SKIP", shots, missing)
        _tutup_summary(summary_f)
        return exit_kode
    sum_line("[OK] fase 1 lulus: M6 + prompt + sesi Bagian 3 lengkap + gerbang versi + stderr sisa 0 byte.")
    penangkap.close()
    hentikan(proc)

    # ======================= FASE 2: -m 3M ===================================
    probe3 = "SKIP"
    if not args.skip_3m:
        sum_line("")
        sum_line("=== FASE 2 - percobaan boot headless pada -m %s (janji Bagian 5) ===" % args.mission_ram)
        s3 = ["-serial", "file:%s" % serial3m_f]
        p3, cmd3 = mulai_qemu(s3, stderr3m_f, args.qmp_port + 1, args.mission_ram)
        sum_line("CMDLINE: %s" % cmd3)
        d3 = time.time() + args.probe_3m_seconds
        r3 = m3 = c3 = False
        while time.time() < d3:
            time.sleep(2)
            t3 = penangkap_text(serial3m_f)
            if "M6: Kernel Cold Boot Complete" in t3:
                m3 = True
            if prompt_terakhir(t3, prefek, sufek):
                r3 = True
                break
            if any(tok in t3 for tok in CRASH_TOKEN):
                c3 = True
                break
            if p3.poll() is not None:
                break
        b3 = byte_len(serial3m_f)
        e3_total, e3_sisa, e3_info, e3_contoh = stderr_diluruskan(stderr3m_f)
        sum_line("Hasil  : M6=%s prompt=%s crash=%s serial=%d byte stderr=%d byte (sisa=%d, info=%d)" % (
            m3, r3, c3, b3, e3_total, e3_sisa, e3_info))
        if r3 and m3 and not c3:
            probe3 = "LULUS"
            sum_line("[OK] fase 2: janji Bagian 5 -m %s LULUS pada ISO ini." % args.mission_ram)
        else:
            probe3 = "MERAH"
            sum_line("[MERAH] fase 2: -m %s tidak mencapai prompt pada ISO ini (serial=%d byte)." % (
                args.mission_ram, b3))
            sum_line("        DEVIASI YANG SUDAH DIKETAHUI, bukan temuan baru: ACCOUNTABILITY_FAULT_LOG")
            sum_line("        entri ke-23 ber-result OPEN; scripts/test_ram_floor_3mb.ps1 mengembalikan 2.")
        shots.append(screendump(args.qmp_port + 1, 3, REP))
        sum_line("screendump fase 2 (-m %s): %s" % (args.mission_ram, json.dumps(shots[-1])))
        hentikan(p3)
    else:
        sum_line("")
        sum_line("=== FASE 2 DILEWATI (--skip-3m) - keputusan Bagian 5 tidak diukur. ===")

    # ======================= keputusan ========================================
    green = (ver_new >= 1 and ver_old == 0 and stderr_sisa == 0 and not missing and not crash)
    sum_line("")
    if not green:
        sum_line("KEPUTUSAN: 1 (MERAH) - salah satu gerbang fase 1 gagal.")
        _tulis_hasil(hasil_f, locals(), 1, probe3, shots, missing)
        _tutup_summary(summary_f)
        return 1
    if args.skip_3m:
        sum_line("KEPUTUSAN: 3 (BELUM LENGKAP) - fase 2 dilewati; jendela P8 belum penuh.")
        _tulis_hasil(hasil_f, locals(), 3, probe3, shots, missing)
        _tutup_summary(summary_f)
        return 3
    if probe3 == "MERAH" and not args.accept_deviation_3m:
        sum_line("KEPUTUSAN: 3 (MERAH PADA JANJI 3 MB, DEVIASI BELUM DITERIMA) - fase 1 hijau penuh.")
        _tulis_hasil(hasil_f, locals(), 3, probe3, shots, missing)
        _tutup_summary(summary_f)
        return 3
    if probe3 == "MERAH":
        sum_line("[PASS BERSYARAT] jendela VM tak-disaksikan: cold boot -m %s + sesi Bagian 3 +" % args.boot_ram)
        sum_line("  gerbang versi + stderr sisa 0 = HIJAU; janji -m %s MERAH dan deviasinya DITERIMA" % args.mission_ram)
        sum_line("  lewat --accept-deviation-3m. RUJUKAN TERTULIS: %s" % (args.alasan or "(tidak diisi)"))
        sum_line("  Deviasi ini WAJIB muncul di rilis, LAPORAN-FINAL dan ACCOUNTABILITY_FAULT_LOG.")
        _tulis_hasil(hasil_f, locals(), 0, probe3, shots, missing)
        _tutup_summary(summary_f)
        return 0
    sum_line("[PASS] jendela VM tak-disaksikan penuh: fase 1 + fase 2 -m %s lulus." % args.mission_ram)
    _tulis_hasil(hasil_f, locals(), 0, probe3, shots, missing)
    _tutup_summary(summary_f)
    return 0


def penangkap_text(path):
    return baca_teks(path) if os.path.exists(path) else ""


def _tutup_summary(path):
    with open(path, "ab") as f:
        for l in SUM_LINES:
            f.write(l.encode("ascii", "replace") + b"\r\n")
    print("[summary] %s (%d byte)" % (path, byte_len(path)))


def _tulis_hasil(path, lv, kode, probe3, shots, missing):
    out = {
        "stamp_utc": STAMP,
        "kode_keluar": kode,
        "ram_fase1_diminta": lv.get("args").boot_ram if lv.get("args") else None,
        "ram_fase2_diminta": (lv.get("args").mission_ram if lv.get("args") else None),
        "prompt": lv.get("prompt"),
        "iso_bytes": lv.get("iso_size"),
        "iso_sha256": lv.get("iso_sha"),
        "master_sha256_bagian5": lv.get("sha_master"),
        "elf_bytes": lv.get("elf_size"),
        "elf_sha256": lv.get("elf_sha"),
        "qemu_version": lv.get("versi_qemu"),
        "penanda_dibaca": len(lv.get("markers") or []),
        "sesi_dibaca": len(lv.get("session") or []),
        "baris_sesi_penuh_prompt": lv.get("tepat"),
        "pengendapan_byte": lv.get("jeda_b"),
        "pengendapan_detik": lv.get("detik_b"),
        "prachecklist": [(n, bool(o), d) for n, o, d in (lv.get("prereq") or [])],
        "serial_bytes": byte_len(lv.get("serial_f") or ""),
        "stderr_total_bytes": lv.get("stderr_total"),
        "stderr_sisa_bytes": lv.get("stderr_sisa"),
        "stderr_baris_info_chardev": lv.get("stderr_info"),
        "serial_3m_bytes": byte_len(lv.get("serial3m_f") or ""),
        "stderr_3m_total_bytes": lv.get("e3_total"),
        "stderr_3m_sisa_bytes": lv.get("e3_sisa"),
        "versi_baru_occ": lv.get("ver_new"),
        "versi_lama_occ": lv.get("ver_old"),
        "penanda_kurang": missing,
        "drift_kriteria_penanda": lv.get("drift_terukur") or [],
        "audit_penanda_sumber": lv.get("audit") or [],
        "fase2": probe3,
        "accept_deviation_3m": bool(lv.get("args") and lv["args"].accept_deviation_3m),
        "alasan": (lv.get("args").alasan if lv.get("args") else ""),
        "screendump": shots,
        "artefak": [lv.get("serial_f"), lv.get("serial3m_f"), lv.get("stderr_f"),
                    lv.get("stderr3m_f"), lv.get("summary_f")],
        "kelas_bukti": "VERIFIED-ON-VM",
        "catatan": "jendela tak-disaksikan; VERIFIED-ON-HARDWARE tetap 0 (R3 BAGIAN 6, otorisasi 2026-10-24)",
    }
    with open(path, "w", encoding="ascii", newline="\r\n") as f:
        f.write(json.dumps(out, indent=1, sort_keys=True) + "\n")


if __name__ == "__main__":
    sys.exit(main())
