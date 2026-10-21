# =============================================================================
# scripts/g9_final_p8_run.ps1 - JENDELA PENERIMAAN AKHIR v1.0.2 (misi v1.0.2
# Bagian 5 + Bagian 3, tahap P8). G9 penuh: run WAJIB berjendela - "-display
# none" DILARANG pada kedua fase, dan pemilik repo menyaksikan jendela sampai
# M6 + prompt + sesi shell selesai.
#
# Diturunkan dari scripts/g9_lite_t4_run.ps1. Empat perbedaan yang disengaja:
#   1. Nama keluaran ber-prefix g9_final_p8_ dan bercap-waktu per-invokasi,
#      sehingga tidak dapat menimpa artefak G9/G9-lite yang sudah ter-commit.
#   2. Gerbang versi DIBALIK dari T4: serial wajib memuat "v1.0.2" dan NOL
#      "v1.1.0" (identitas rilis v1.0.2 yang diterbitkan pada P9).
#   3. Menagih SESI BAGIAN 3: sebelas perintah kanonik, satu alias, satu
#      kesalahan aritas, dan satu perintah tak dikenal - masing-masing ditandai
#      dari STRING KELUARAN yang dibaca dari kode, bukan dari angka karangan.
#   4. Menambah fase kedua: percobaan boot berjendela pada -m 3M (janji Bagian
#      5). Fase ini diketahui MERAH pada tree ini (ACCOUNTABILITY_FAULT_LOG
#      entri ke-23, result OPEN) sehingga kode keluar default-nya 3; pemilik
#      yang menerima deviasi harus memberi -AcceptDeviation3M secara eksplisit.
#
# Lantai boot TIDAK di-bisect di sini: itu tugas scripts/test_ram_floor_3mb.ps1
# (tiga-valued, sudah terdaftar sebagai langkah [4/5] ci_local.ps1).
#
#usage:
#   powershell -ExecutionPolicy Bypass -File scripts\g9_final_p8_run.ps1 -SelfTest
#   powershell -ExecutionPolicy Bypass -File scripts\g9_final_p8_run.ps1
#   powershell -ExecutionPolicy Bypass -File scripts\g9_final_p8_run.ps1 -AcceptDeviation3M
#   powershell -ExecutionPolicy Bypass -File scripts\g9_final_p8_run.ps1 -StopQemu
#
# Kode keluar:
#   0 = jendela penuh lulus (normal boot + gerbang versi + sesi + 3M lulus,
#       ATAU 3M merah dengan -AcceptDeviation3M dari pemilik)
#   1 = prasyarat/boot normal gagal, crash, atau stderr QEMU tidak kosong
#   2 = sesi Bagian 3 belum lengkap pada batas waktu
#   3 = -m 3M merah dan deviasi belum diterima pemilik (-AcceptDeviation3M)
# =============================================================================
param(
    [switch]$StopQemu,
    [switch]$SelfTest,
    [switch]$AcceptDeviation3M,
    [switch]$Skip3M,
    [int]$BootTimeoutSeconds = 180,
    [int]$SessionTimeoutSeconds = 1200,
    [int]$Probe3MSeconds = 90,
    [string]$MissionRam = "3M"
)

$ErrorActionPreference = "Continue"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$Workspace = Join-Path $RepoRoot "asl-kernel"
$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$Iso = Join-Path $Workspace "target\asl-core-kernel.iso"
$Stamp = Get-Date -Format "yyyy-MM-dd_HH-mm-ss"
$Rep = Join-Path $RepoRoot "Analysis\reports"
$Serial = Join-Path $Rep "g9_final_p8_serial_$Stamp.log"
$Stderr = Join-Path $Rep "g9_final_p8_qemu_stderr_$Stamp.log"
$Serial3M = Join-Path $Rep "g9_final_p8_serial_3m_$Stamp.log"
$Stderr3M = Join-Path $Rep "g9_final_p8_qemu_stderr_3m_$Stamp.log"
$Summary = Join-Path $Rep "g9_final_p8_summary_$Stamp.log"
$PidFile = Join-Path $RepoRoot "logs\g9_final_p8_qemu_pid.txt"
$PromptSource = Join-Path $Workspace "asl-cockpit-shell\src\lib.rs"

function Sum($m) {
    Write-Host $m
    Add-Content -Path $Summary -Value $m -Encoding ascii
}

# ------------------------------------------------------------------ stop mode
if ($StopQemu) {
    if (Test-Path $PidFile) {
        $procId = (Get-Content $PidFile -Raw).Trim()
        $p = Get-Process -Id ([int]$procId) -ErrorAction SilentlyContinue
        if ($p) { Stop-Process -Id $p.Id -Force; Write-Host "[OK] jendela QEMU (PID $($p.Id)) ditutup." }
        else { Write-Host "[i] PID $procId sudah tidak berjalan." }
        Remove-Item $PidFile -Force
    } else {
        Write-Host "[i] Tidak ada file PID g9_final_p8 di logs\ - tidak ada yang ditutup."
    }
    exit 0
}

# ------------------------------------------------- satu-sumber prompt + marker
# Prompt dibaca dari konstanta, bukan ditulis tangan di sini.
$Prompt = $null
Select-String -Path $PromptSource -Pattern 'pub const SHELL_PROMPT\s*:\s*&str\s*=\s*"([^"]*)"' |
    ForEach-Object { $Prompt = $_.Matches[0].Groups[1].Value }
if (-not $Prompt) {
    Write-Host "GERBANG MERAH - SHELL_PROMPT tidak terbaca dari $PromptSource"
    exit 1
}

# Penanda keluaran: setiap baris = (label, string yang harus muncul di serial,
# jumlah kemunculan minimum). String diambil dari jalur cetak kanonik:
#   - asl-kernel-core/src/lib.rs  : "Status Cockpit Shell " (1273), "Direktori dibuat: " (1196),
#     "Berkas dibuat: ", "Tertulis ", "Berkas dihapus: ", "Direktori dihapus: ", "Daftar isi " (1119),
#     "Ruang kerja: " (1151), "Perintah tidak dikenal: " (1095), "Sintaks: " (1837),
#     dan sekuens bersih layar "\x1B[2J\x1B[H" (1103 dan 1862) - karena itu penanda 'clear' di bawah
#     ditulis sebagai karakter ESC (27) diikuti "[2J", bukan sebagai teks sumber.
#   - asl-cockpit-shell/src/lib.rs: "Perintah kanonik Cockpit Shell" (776, keluaran `help`).
# Penanda "halo dari sesi p8" BUKAN string kernel: itu isi yang operator ketik pada baris
# write-file, dan dibutuhkan dua kemunculan (gema masukan + isi berkas hasil `cat`) supaya
# jalur tulis-baca RAMDISK terbukti lewat serial, bukan lewat tebakan.
# Penanda "Daftar isi /" adalah hasil SAMBUNGAN dua cetak di sumber: literal "Daftar isi "
# (asl-kernel-core/src/lib.rs:1119) lalu `path` yang pada build ini selalu diawali '/' karena
# resolve_target menghasilkan jalur absolut. Huruf '/' sengaja ikut diuji: string registry
# "Daftar isi direktori pada RAMDISK volatil..." (asl-cockpit-shell/src/lib.rs:378) juga tercetak
# oleh `help`, jadi penanda tanpa '/' bisa terpenuhi hanya dengan mengetik `help`.
$Markers = @(
    @("help (registry)", "Perintah kanonik Cockpit Shell", 1),
    @("status", "Status Cockpit Shell ", 1),
    @("create-folder", "Direktori dibuat: ", 1),
    @("list + alias dir", "Daftar isi /", 2),
    @("cd (alias open) + open /", "Ruang kerja: ", 2),
    @("create-file", "Berkas dibuat: ", 1),
    @("write-file", "Tertulis ", 1),
    @("isi berkas via alias cat", "halo dari sesi p8", 2),
    @("delete-file", "Berkas dihapus: ", 1),
    @("delete-folder", "Direktori dihapus: ", 1),
    @("help token tak dikenal", "Perintah tidak dikenal: ", 1),
    @("aritas salah -> Sintaks", "Sintaks: ", 1),
    @("clear (sekuens ANSI)", "$([char]27)[2J", 1)
)

# Urutan sesi Bagian 3 yang diketik pemilik di jendela QEMU.
$Session = @(
    "help",
    "status",
    "where",
    "create-folder sesi",
    "list",
    "dir",
    "cd sesi",
    "where",
    "create-file catatan.txt",
    "write-file catatan.txt halo dari sesi p8",
    "cat catatan.txt",
    "delete-file catatan.txt",
    "open /",
    "delete-folder sesi",
    "help tidakada",
    "open",
    "clear"
)

if ($SelfTest) {
    Write-Host "SELF-TEST g9_final_p8_run.ps1 - TIDAK menjalankan QEMU, TIDAK menulis artefak."
    Write-Host "  prompt satu-sumber : '$Prompt'"
    Write-Host "  marker sesi         : $($Markers.Count) penanda, $($Session.Count) baris sesi"
    foreach ($m in $Markers) { Write-Host ("    {0,-28} '{1}' >= {2}" -f $m[0], ($m[1] -replace [string][char]27, '<ESC>'), $m[2]) }
    Write-Host "  ISO yang akan di-boot (bila ada): $Iso"
    if (Test-Path $Iso) {
        Write-Host ("    {0} bytes SHA256={1}" -f (Get-Item $Iso).Length, (Get-FileHash -Algorithm SHA256 $Iso).Hash)
    } else {
        Write-Host "    (belum ada - jalankan build_kernel.ps1 -Action iso sebelum jendela)"
    }
    Write-Host "  versi gerbang       : 'v1.0.2' >= 1 dan 'v1.1.0' == 0 pada serial"
    Write-Host "  fase 3M            : -m $MissionRam berjendela; merah -> kode keluar 3 kecuali -AcceptDeviation3M"
    exit 0
}

"ASL G9 FINAL ACCEPTANCE RUN P8 (VISIBLE) - $Stamp" | Out-File -FilePath $Summary -Encoding ascii
Sum "prompt satu-sumber : '$Prompt' (dari $(Split-Path -Leaf $PromptSource))"
Sum "marker sesi        : $($Markers.Count) penanda keluaran, $($Session.Count) baris sesi"

# ----------------------------------------------------------------- prereqs
if (-not (Test-Path $QemuExe)) { Sum "[FAIL] qemu tidak ditemukan: $QemuExe"; exit 1 }
if (-not (Test-Path $Iso)) { Sum "[FAIL] ISO tidak ditemukan: $Iso (jalankan build_kernel.ps1 -Action iso)"; exit 1 }

$isoSize = (Get-Item $Iso).Length
$isoHash = (Get-FileHash -Algorithm SHA256 $Iso).Hash
$elfPath = Join-Path $Workspace "target\x86_64-unknown-none\release\asl-kernel"
Sum "QEMU   : $(& $QemuExe --version | Select-Object -First 1)"
Sum "ISO    : $Iso"
Sum "ISO    : $isoSize bytes  SHA256=$isoHash"
if (Test-Path $elfPath) {
    Sum "ELF    : $((Get-Item $elfPath).Length) bytes  SHA256=$((Get-FileHash -Algorithm SHA256 $elfPath).Hash)"
}
Sum "Catatan: ISO reproducible per-platform sejak PHASE-7 T1; SHA256 di atas tetap sidik jari PER-TREE dan hanya sah untuk tree yang di-build pada langkah ini."

# ------------------------------------------- FASE 1: boot normal + sesi, berjendela
# SENGAJA tidak ada "-display none" / "-WindowStyle Hidden" (aturan G9).
$qemuArgs = @("-machine", "q35", "-vga", "std", "-m", "256M", "-cdrom", $Iso,
    "-serial", "file:$Serial", "-no-reboot", "-no-shutdown")
Sum ""
Sum "=== FASE 1 - cold boot 256M berjendela + sesi Bagian 3 ==="
Sum "CMDLINE: `"$QemuExe`" $($qemuArgs -join ' ')"

$proc = Start-Process $QemuExe -ArgumentList $qemuArgs -RedirectStandardError $Stderr -PassThru
if (-not $proc) { Sum "[FAIL] Start-Process mengembalikan null"; exit 1 }
New-Item -ItemType Directory -Path (Join-Path $RepoRoot "logs") -Force | Out-Null
$proc.Id | Out-File -FilePath $PidFile -Encoding ascii
Sum "PID    : $($proc.Id)  (tutup dengan: g9_final_p8_run.ps1 -StopQemu)"

# tunggu prompt
$deadline = (Get-Date).AddSeconds($BootTimeoutSeconds)
$m6 = $false; $shell = $false; $crash = $false
while ((Get-Date) -lt $deadline) {
    Start-Sleep -Seconds 3
    $txt = $null
    if (Test-Path $Serial) { $txt = Get-Content $Serial -Raw -ErrorAction SilentlyContinue }
    if ($txt) {
        if ($txt.Contains("M6: Kernel Cold Boot Complete")) { $m6 = $true }
        if ($txt.Contains($Prompt)) { $shell = $true; break }
        if ($txt.Contains("ASL EXCEPTION") -or $txt.Contains("ASL-CORE-PANIC") -or $txt.Contains("FATAL")) { $crash = $true; break }
    }
    if ($proc.HasExited) { break }
}
Sum "Boot   : M6=$m6 prompt=$shell crash=$crash (setelah $([int]((Get-Date) - $proc.StartTime).TotalSeconds)s)"

if (-not ($m6 -and $shell -and -not $crash)) {
    Sum "[FAIL] fase 1: cold boot 256M tidak mencapai M6 + prompt, atau crash - pipeline MERAH (G5)."
    if (-not $proc.HasExited) { Stop-Process -Id $proc.Id -Force }
    Remove-Item $PidFile -Force -ErrorAction SilentlyContinue
    exit 1
}

# lembaran sesi untuk pemilik
Sum ""
Sum "==> Jendela QEMU sudah pada prompt '$Prompt'. KETIK baris-baris berikut di jendela itu, ENTER setiap baris:"
foreach ($line in $Session) {
    Sum "      $line"
}
Sum "    (satu baris sengaja tanpa argumen untuk menguji jalur aritas; baris terakhir 'clear' mengirim sekuens ANSI)"
Sum "    Mesin penunggu membaca penanda keluaran dari berkas serial, bukan dari tebakan."

$sessionDeadline = (Get-Date).AddSeconds($SessionTimeoutSeconds)
$found = @{}
while ((Get-Date) -lt $sessionDeadline) {
    Start-Sleep -Seconds 4
    $txt = $null
    if (Test-Path $Serial) { $txt = Get-Content $Serial -Raw -ErrorAction SilentlyContinue }
    if ($txt) {
        if ($txt.Contains("ASL EXCEPTION") -or $txt.Contains("ASL-CORE-PANIC") -or $txt.Contains("FATAL")) { $crash = $true; break }
        $pending = $false
        foreach ($m in $Markers) {
            $n = ([regex]::Matches($txt, [regex]::Escape($m[1]))).Count
            if ($n -lt $m[2]) { $pending = $true }
            $found[$m[0]] = $n
        }
        if (-not $pending) { break }
    }
    if ($proc.HasExited) { break }
}

Sum ""
Sum "=== Hasil sesi Bagian 3 (hitung kemunculan penanda di serial) ==="
$missing = @()
foreach ($m in $Markers) {
    $n = 0
    if ($found.ContainsKey($m[0])) { $n = $found[$m[0]] }
    $status = if ($n -ge $m[2]) { "OK  " } else { "KURANG" }
    if ($n -lt $m[2]) { $missing += $m[0] }
    Sum ("  [{0}] {1,-28} '{2}' terukur {3} (butuh {4})" -f $status, $m[0], $m[1], $n, $m[2])
}

$txtFinal = if (Test-Path $Serial) { Get-Content $Serial -Raw } else { "" }
$verNew = ([regex]::Matches($txtFinal, "v1\.0\.2")).Count
$verOld = ([regex]::Matches($txtFinal, "v1\.1\.0")).Count
$serialBytes = if (Test-Path $Serial) { (Get-Item $Serial).Length } else { 0 }
$stderrBytes = if (Test-Path $Stderr) { (Get-Item $Stderr).Length } else { 0 }
Sum "Versi  : occurrence 'v1.0.2'=$verNew  occurrence 'v1.1.0'=$verOld  (gerbang: >=1 dan 0)"
Sum "Ukuran : serial=$serialBytes byte  stderr QEMU=$stderrBytes byte (gerbang: 0)"

$verOk = ($verNew -ge 1 -and $verOld -eq 0)
$stderrOk = ($stderrBytes -eq 0)
if (-not $verOk) {
    Sum "[FAIL] gerbang versi merah - identitas rilis pada artefak yang di-boot bukan v1.0.2."
    if (-not $proc.HasExited) { Stop-Process -Id $proc.Id -Force }
    Remove-Item $PidFile -Force -ErrorAction SilentlyContinue
    exit 1
}
if (-not $stderrOk) {
    Sum "[FAIL] stderr QEMU tidak kosong ($stderrBytes byte) pada fase 1."
    if (-not $proc.HasExited) { Stop-Process -Id $proc.Id -Force }
    Remove-Item $PidFile -Force -ErrorAction SilentlyContinue
    exit 1
}
if ($crash) {
    Sum "[FAIL] crash terdeteksi pada sesi."
    if (-not $proc.HasExited) { Stop-Process -Id $proc.Id -Force }
    Remove-Item $PidFile -Force -ErrorAction SilentlyContinue
    exit 1
}
if ($missing.Count -gt 0) {
    Sum "[FAIL] sesi Bagian 3 belum lengkap ($($missing.Count) penanda kurang): $($missing -join '; ')"
    Sum "       Jendela dibiarkan TERBUKA - lanjutkan mengetik, lalu jalankan ulang dengan batas waktu lebih besar."
    exit 2
}
Sum "[OK] fase 1 lulus: M6 + prompt + sesi Bagian 3 lengkap + gerbang versi + stderr 0 byte."

# -------------------------------------------- FASE 2: percobaan -m 3M, berjendela
$probe3Result = "SKIP"
if (-not $Skip3M) {
    Sum ""
    Sum "=== FASE 2 - percobaan boot berjendela pada -m $MissionRam (janji Bagian 5) ==="
    if (-not $proc.HasExited) {
        Stop-Process -Id $proc.Id -Force
        Sum "Jendela fase 1 ditutup untuk memberi tempat fase 2 (proses QEMU kedua tidak dipakai bersamaan)."
    }
    Remove-Item $PidFile -Force -ErrorAction SilentlyContinue
    $q3 = @("-machine", "q35", "-vga", "std", "-m", $MissionRam, "-cdrom", $Iso,
        "-serial", "file:$Serial3M", "-no-reboot", "-no-shutdown")
    Sum "CMDLINE: `"$QemuExe`" $($q3 -join ' ')"
    $p3 = Start-Process $QemuExe -ArgumentList $q3 -RedirectStandardError $Stderr3M -PassThru
    $d3 = (Get-Date).AddSeconds($Probe3MSeconds)
    $r3 = $false; $m3 = $false; $c3 = $false
    while ((Get-Date) -lt $d3) {
        Start-Sleep -Seconds 3
        $t3 = $null
        if (Test-Path $Serial3M) { $t3 = Get-Content $Serial3M -Raw -ErrorAction SilentlyContinue }
        if ($t3) {
            if ($t3.Contains("M6: Kernel Cold Boot Complete")) { $m3 = $true }
            if ($t3.Contains($Prompt)) { $r3 = $true; break }
            if ($t3.Contains("ASL EXCEPTION") -or $t3.Contains("ASL-CORE-PANIC") -or $t3.Contains("FATAL")) { $c3 = $true; break }
        }
        if ($p3.HasExited) { break }
    }
    $bytes3 = if (Test-Path $Serial3M) { (Get-Item $Serial3M).Length } else { 0 }
    $err3 = if (Test-Path $Stderr3M) { (Get-Item $Stderr3M).Length } else { 0 }
    Sum "Hasil  : M6=$m3 prompt=$r3 crash=$c3 serial=$bytes3 byte stderr=$err3 byte"
    if ($r3 -and $m3 -and -not $c3) {
        $probe3Result = "LULUS"
        Sum "[OK] fase 2: janji Bagian 5 -m $MissionRam LULUS pada ISO ini."
    } else {
        $probe3Result = "MERAH"
        Sum "[MERAH] fase 2: -m $MissionRam tidak mencapai prompt pada ISO ini (serial=$bytes3 byte)."
        Sum "        Ini DEVIASI YANG SUDAH DIKETAHUI, bukan temuan baru: ACCOUNTABILITY_FAULT_LOG entri ke-23"
        Sum "        ber-result OPEN dan scripts/test_ram_floor_3mb.ps1 mengembalikan kode keluar 2 (DEVIASI TERCATAT)."
        Sum "        Pemilik yang menerima deviasi ini harus menjalankan ulang dengan -AcceptDeviation3M."
    }
    if (-not $p3.HasExited) { Stop-Process -Id $p3.Id -Force }
} else {
    Sum ""
    Sum "=== FASE 2 DILEWATI (-Skip3M) - keputusan Bagian 5 tidak diukur pada invokasi ini. ==="
}

# ------------------------------------------------------------------ keputusan
$green = $verOk -and $stderrOk -and ($missing.Count -eq 0) -and (-not $crash)
if (-not $green) {
    Sum "KEPUTUSAN: 1 (MERAH) - salah satu gerbang fase 1 gagal."
    exit 1
}
if ($Skip3M) {
    Sum "KEPUTUSAN: 3 (BELUM LENGKAP) - fase 2 dilewati, jadi jendela P8 belum bisa dinyatakan penuh."
    Sum "  Jalankan tanpa -Skip3M (atau dengan -AcceptDeviation3M bila deviasi 3 MB diterima)."
    exit 3
}
if ($probe3Result -eq "MERAH" -and -not $AcceptDeviation3M) {
    Sum "KEPUTUSAN: 3 (MERAH PADA JANJI 3 MB, DEVIASI BELUM DITERIMA) - fase 1 hijau penuh, fase 2 merah."
    Sum "  Gerbang Bagian 5 dinyatakan MERAH dan tidak diubah menjadi hijau (G5, Bagian 6)."
    Sum "  Setelah pemilik menerima deviasi dengan sadar: jalankan ulang dengan -AcceptDeviation3M."
    exit 3
}
if ($probe3Result -eq "MERAH") {
    Sum "[PASS BERSYARAT] jendela P8: cold boot 256M + sesi Bagian 3 + gerbang versi + stderr 0 = HIJAU;"
    Sum "  janji -m 3M MERAH dan deviasinya DITERIMA pemilik lewat -AcceptDeviation3M pada invokasi ini."
    Sum "  Deviasi ini WAJIB muncul di LAPORAN-FINAL v1.0.2 dan di ACCOUNTABILITY_FAULT_LOG."
    Sum "Artefak: $Serial ; $Serial3M ; $Summary"
    exit 0
}
Sum "[PASS] G9 FINAL P8: cold boot berjendela 256M + sesi Bagian 3 lengkap + gerbang versi v1.0.2 + stderr 0 + -m $MissionRam lulus."
Sum "Jendela fase 1 sempat dibiarkan terbuka untuk disaksikan pemilik; artefak serial ter-commit pada commit BERIKUTNYA (G8)."
Sum "Artefak: $Serial ; $Serial3M ; $Summary"
exit 0
