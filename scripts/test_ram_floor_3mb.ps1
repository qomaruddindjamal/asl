# =============================================================================
# scripts/test_ram_floor_3mb.ps1 - GERBANG REGRESI MEMORI BOOT (misi v1.0.2
# Bagian 5 + Bagian 1.7, tahap P4). Fail-closed: pipeline tidak boleh hijau
# kalau boot cold di ambang memori yang dijanjikan tidak mencapai prompt shell.
#
# DUA ambang, keduanya diukur ulang pada setiap jalan (G4 anti-drift):
#   [A] Janji misi  : -m 3M  (Bagian 5, "gerbang -m 3M lulus")
#   [B] Ratchet     : -m 3392K (lantai boot TERUKUR pada ISO v1.0.2 + RAMDISK
#       256 KiB, plus margin 32 KiB). Fungsinya menahan REGRESI berikutnya.
#
# Kelas keputusan (kode keluar):
#   0 = HIJAU PENUH       : A dan B dua-duanya boot.
#   1 = REGRESI           : B gagal -> ada pertumbuhan footprint baru. MERAH.
#   2 = DEVIASI TERCATAT  : A gagal, B lulus. Gerbang misi 3M MERAH dan status
#       merah itu TIDAK boleh dinyatakan hijau; ia dicatat sebagai deviasi yang
#       sudah diketahui (ACCOUNTABILITY_FAULT_LOG) dan wajib muncul di laporan
#       akhir. ci_local.ps1 meneruskan pipeline dengan banner deviasi.
#
# Angka lantai yang tercatat saat naskah ini ditulis (VERIFIED-ON-VM 2026-10-21,
# ISO SHA256 756b87b057ec2f7be0617da5b1ca463eb774e895b8ad44c8f45bf7dea367f1e8):
#   3072K gagal | 3328K gagal | 3360K lulus | 3392K lulus | 3456K lulus
# Kontrol pembanding: ISO pra-RAMDISK (tahap P2) lulus di 3072K dan gagal di
# 2944K, jadi sisa margin 3 MB sebelum misi ini sudah kurang dari 128 KiB.
#
# Prompt shell TIDAK di-hardcode di sini: dibaca dari satu-sumbernya,
# konstanta SHELL_PROMPT di asl-kernel/asl-cockpit-shell/src/lib.rs.
#
# Artefak serial per-jalan memakai NAMA BERTimestamp dan ditulis ke logs/
# (scratch, di-ignore git) sehingga tidak pernah menimpa bukti yang sudah
# ter-commit di Analysis/reports seperti yang dilakukan ci_local.ps1 lama.
#
#usage:
#   powershell -ExecutionPolicy Bypass -File scripts\test_ram_floor_3mb.ps1
#   powershell ... -MissionRam 3M -RatchetRam 3392K -TimeoutSec 120
# =============================================================================
param(
    [string]$MissionRam  = "3M",
    [string]$RatchetRam  = "3392K",
    [string]$IsoPath     = "",
    [string]$QemuExe     = "C:\Program Files\qemu\qemu-system-x86_64.exe",
    [string]$OutDir      = "",
    [int]$TimeoutSec     = 120
)

$ErrorActionPreference = "Continue"
$RepoRoot = Split-Path -Parent $PSScriptRoot
if (-not $IsoPath) { $IsoPath = Join-Path $RepoRoot "asl-kernel\target\asl-core-kernel.iso" }
if (-not $OutDir)  { $OutDir  = Join-Path $RepoRoot "logs" }
$Stamp = Get-Date -Format "yyyy-MM-dd_HH-mm-ss"
if (-not (Test-Path $OutDir)) { New-Item -ItemType Directory -Path $OutDir | Out-Null }

if (-not (Test-Path $IsoPath)) {
    Write-Host "GERBANG MERAH - ISO tidak ada: $IsoPath"
    exit 1
}
if (-not (Test-Path $QemuExe)) {
    Write-Host "GERBANG MERAH - QEMU tidak ada: $QemuExe"
    exit 1
}

# --- Satu-sumber prompt: parse konstanta SHELL_PROMPT dari kode Rust. ----------
$PromptSource = Join-Path $RepoRoot "asl-kernel\asl-cockpit-shell\src\lib.rs"
$Prompt = $null
Select-String -Path $PromptSource -Pattern 'pub const SHELL_PROMPT\s*:\s*&str\s*=\s*"([^"]*)"' |
    ForEach-Object { $Prompt = $_.Matches[0].Groups[1].Value }
if (-not $Prompt) {
    Write-Host "GERBANG MERAH - SHELL_PROMPT tidak terbaca dari $PromptSource"
    exit 1
}
Write-Host "prompt satu-sumber : '$Prompt'"
Write-Host "ISO                : $IsoPath"
Write-Host "SHA256             : $((Get-FileHash -Algorithm SHA256 $IsoPath).Hash)"

# --- Satu percobaan boot cold pada satu ukuran RAM. ---------------------------
function Probe-Boot {
    param([string]$Ram)
    $Serial = Join-Path $OutDir "ram_floor_${Stamp}_m${Ram}_serial.log"
    $Stderr = Join-Path $OutDir "ram_floor_${Stamp}_m${Ram}_qemu_stderr.log"
    Remove-Item $Serial, $Stderr -ErrorAction SilentlyContinue
    $qargs = @("-machine", "q35", "-vga", "std", "-m", $Ram, "-cdrom", $IsoPath,
        "-serial", "file:$Serial", "-display", "none", "-no-reboot", "-no-shutdown")
    $proc = Start-Process $QemuExe -ArgumentList $qargs -RedirectStandardError $Stderr -PassThru
    $deadline = (Get-Date).AddSeconds($TimeoutSec)
    $reached = $false
    $crash = $false
    $m6 = $false
    while ((Get-Date) -lt $deadline) {
        Start-Sleep -Seconds 2
        $txt = $null
        if (Test-Path $Serial) { $txt = Get-Content $Serial -Raw -ErrorAction SilentlyContinue }
        if ($txt) {
            if ($txt.Contains("M6: Kernel Cold Boot Complete")) { $m6 = $true }
            if ($txt.Contains($Prompt)) { $reached = $true; break }
            if ($txt.Contains("ASL EXCEPTION") -or $txt.Contains("ASL-CORE-PANIC") -or $txt.Contains("FATAL")) {
                $crash = $true; break
            }
        }
        if ($proc.HasExited) { break }
    }
    if (-not $proc.HasExited) { Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue }
    Start-Sleep -Seconds 1
    $serialBytes = if (Test-Path $Serial) { (Get-Item $Serial).Length } else { 0 }
    $stderrBytes = if (Test-Path $Stderr) { (Get-Item $Stderr).Length } else { 0 }
    return [pscustomobject]@{
        Ram = $Ram; Reached = $reached; M6 = $m6; Crash = $crash
        SerialBytes = $serialBytes; StderrBytes = $stderrBytes
        SerialPath = $Serial
    }
}

Write-Host ""
Write-Host "=== [A] janji misi: -m $MissionRam ==="
$a = Probe-Boot -Ram $MissionRam
Write-Host ("  reached={0} m6={1} crash={2} serial={3} B stderr={4} B" -f `
    $a.Reached, $a.M6, $a.Crash, $a.SerialBytes, $a.StderrBytes)

Write-Host "=== [B] ratchet regresi: -m $RatchetRam ==="
$b = Probe-Boot -Ram $RatchetRam
Write-Host ("  reached={0} m6={1} crash={2} serial={3} B stderr={4} B" -f `
    $b.Reached, $b.M6, $b.Crash, $b.SerialBytes, $b.StderrBytes)

Write-Host ""
$okB = ($b.Reached -and $b.M6 -and -not $b.Crash -and $b.StderrBytes -eq 0)
$okA = ($a.Reached -and $a.M6 -and -not $a.Crash -and $a.StderrBytes -eq 0)

if (-not $okB) {
    Write-Host "KEPUTUSAN: 1 (REGRESI) - boot tidak mencapai prompt pada ratchet $RatchetRam."
    Write-Host "  Pipeline MERAH. Tidak boleh merge sampai footprint dikurangi."
    Write-Host "  artefak: $($b.SerialPath)"
    exit 1
}
if ($okA) {
    Write-Host "KEPUTUSAN: 0 (HIJAU PENUH) - janji misi $MissionRam dan ratchet $RatchetRam dua-duanya boot."
    Write-Host "  artefak: $($a.SerialPath)"
    exit 0
}
Write-Host "KEPUTUSAN: 2 (DEVIASI TERCATAT) - ratchet $RatchetRam lulus, janji misi $MissionRam TIDAK."
Write-Host "  Gerbang Bagian 5 'gerbang -m 3M lulus' berstatus MERAH pada tree ini."
Write-Host "  Status merah TIDAK diubah menjadi hijau; deviasi dicatat di"
Write-Host "  ACCOUNTABILITY_FAULT_LOG dan diulang di LAPORAN-FINAL v1.0.2."
Write-Host "  Bukti: $MissionRam serial=$($a.SerialBytes) B reached=$($a.Reached); $RatchetRam lulus penuh."
Write-Host "  artefak: $($a.SerialPath)"
exit 2
