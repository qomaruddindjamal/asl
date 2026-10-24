# =============================================================================
# scripts/stage_iso_root.ps1 - pengisi staging payload Limine + limine.conf (R3 BAGIAN 3)
#
# Mengapa ada: `cargo clean` menghapus seluruh target/ termasuk
# target/iso_root/boot/limine, dan repository tidak pernah melacak payload
# (*.sys dan *.bin di-abaikan .gitignore) maupun berkas konfigurasinya. Selama
# ini staging diisi manual, sehingga ISO tidak bisa dikuasai dari pohon bersih.
# Skrip ini menutup lubang itu: payload diambil dari jalur pin yang hash-nya
# tercatat di ledger, konfigurasi diambil dari asl-kernel/config/limine.conf
# yang DI-TRACK, dan keduanya diperiksa byte-per-byte sebelum disalin.
#
# Fail-closed: hanya membaca ledger + menulis ke target/iso_root/boot. Tidak
# menyentuh kernel ELF (itu tugas scripts/build_kernel.ps1), tidak menyentuh
# jalur pin, tidak menghapus berkas lain. Exit 0 hanya bila keempat payload dan
# konfigurasi tersalin dengan hash yang sama persis dengan rujukan ledger.
# =============================================================================
$ErrorActionPreference = "Stop"
$ScriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Split-Path -Parent $ScriptRoot
$LedgerPath = Join-Path $RepoRoot "docs\TOOLING\LIMINE-PIN.json"
$PayloadNames = @("BOOTX64.EFI", "limine-bios-cd.bin", "limine-bios.sys", "limine-uefi-cd.bin")

function Get-Sha256($Path) {
    (Get-FileHash -Algorithm SHA256 -Path $Path).Hash.ToLower()
}

if (-not (Test-Path $LedgerPath)) {
    Write-Host "LEDGER TIDAK ADA: $LedgerPath"
    exit 1
}
$L = Get-Content -Raw -Encoding UTF8 $LedgerPath | ConvertFrom-Json
$pinRoot = $L.install_root
$staging = $L.payload_panggung_bangun
$confSrc = Join-Path $RepoRoot "asl-kernel\config\limine.conf"
$refKey = $L.payload_rujukan_aktif
if (-not $refKey) { Write-Host "LEDGER tidak punya kunci payload_rujukan_aktif"; exit 1 }
$rujuk = $L.$refKey.berkas

Write-Host "== stage_iso_root | pin=$($L.pinned_version) rujukan=$refKey"
Write-Host "   sumber payload : $pinRoot"
Write-Host "   staging        : $staging"

$sebab = @()
if (-not (Test-Path $pinRoot)) {
    Write-Host "   JALUR PIN TIDAK ADA - pasang perkakas lebih dulu (docs/TOOLING/LIMINE-PROVISIONING.md)."
    exit 1
}
if (-not (Test-Path $confSrc)) {
    Write-Host "   KONFIGURASI TIDAK ADA: $confSrc"
    exit 1
}
if (-not (Test-Path $staging)) {
    New-Item -ItemType Directory -Path $staging -Force | Out-Null
    Write-Host "   direktori staging dibuat (sebelumnya tidak ada)"
}

for ($i = 0; $i -lt $PayloadNames.Count; $i++) {
    $nama = $PayloadNames[$i]
    $s = Join-Path $pinRoot $nama
    if (-not (Test-Path $s)) { Write-Host ("   {0,-20}: TIDAK ADA di jalur pin" -f $nama); $sebab += "payload $nama tidak ada di pin"; continue }
    $shaS = Get-Sha256 $s
    $sizeS = (Get-Item $s).Length
    if ($sizeS -ne $rujuk.$nama.bytes -or $shaS -ne $rujuk.$nama.sha256.ToLower()) {
        Write-Host ("   {0,-20}: BEDA dengan ledger ({1} B, {2})" -f $nama, $sizeS, $shaS.Substring(0, 16) + "...")
        $sebab += "payload $nama tidak cocok dengan ledger"
        continue
    }
    Copy-Item $s (Join-Path $staging $nama) -Force
    $t = Join-Path $staging $nama
    if ((Get-Sha256 $t) -ne $shaS) { Write-Host ("   {0,-20}: SALIN TIDAK UJIHASH-IDENTIK" -f $nama); $sebab += "gagal verifikasi hasil salin $nama"; continue }
    Write-Host ("   {0,-20}: TERSALIN {1} B, hash cocok ledger" -f $nama, $sizeS)
}

# Konfigurasi dibakar ke dua lokasi yang sama-sama dipakai ISO yang terbukti boot:
# boot/limine/limine.conf (di dalam direktori payload) dan boot/limine.conf.
$confSha = Get-Sha256 $confSrc
$confSize = (Get-Item $confSrc).Length
$confTujuan = @($staging, (Split-Path -Parent $staging))
foreach ($d in $confTujuan) {
    $t = Join-Path $d "limine.conf"
    Copy-Item $confSrc $t -Force
    if ((Get-Sha256 $t) -ne $confSha) { Write-Host "   limine.conf : GAGAL salin ke $t"; $sebab += "limine.conf tidak identik di $t"; continue }
    Write-Host ("   limine.conf : TERSALIN ke {0} ({1} B, sha {2})" -f $t, $confSize, $confSha.Substring(0, 16) + "...")
}

Write-Host ""
if ($sebab.Count -eq 0) {
    Write-Host "SIAP: staging iso_root cocok dengan ledger. Lanjut scripts/build_kernel.ps1 -Action iso."
    exit 0
}
Write-Host ("BELUM SIAP - {0} sebab:" -f $sebab.Count)
foreach ($s in $sebab) { Write-Host "   - $s" }
exit 1
