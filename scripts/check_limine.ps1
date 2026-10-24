# =============================================================================
# scripts/check_limine.ps1 - K-5 provision verifier (READ-ONLY, fail-closed)
#
# Membandingkan perkakas Limine yang terpasang di jalur pin (luar target/,
# surviving `cargo clean`) terhadap ledger docs/TOOLING/LIMINE-PIN.json.
# Tidak menulis apa pun: hanya query + hitung hash.
#
# Exit 0 hanya bila: jalur pin ada, limine.exe ada dan --version cocok dengan
# pinned_version, keempat payload ada dan byte-identik dengan rujukan ledger,
# dan staging ISO (target/iso_root/boot/limine) memuat berkas yang sama.
# Selain itu exit 1 dengan daftar sebab.
# =============================================================================
$ErrorActionPreference = "Stop"
$ScriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Split-Path -Parent $ScriptRoot
$LedgerPath = Join-Path $RepoRoot "docs\TOOLING\LIMINE-PIN.json"

if (-not (Test-Path $LedgerPath)) {
    Write-Host "LEDGER TIDAK ADA: $LedgerPath"
    exit 1
}
$L = Get-Content -Raw -Encoding UTF8 $LedgerPath | ConvertFrom-Json

function Get-Sha256($Path) {
    (Get-FileHash -Algorithm SHA256 -Path $Path).Hash.ToLower()
}

$sebab = @()
Write-Host "== LEDGER: pinned_version=$($L.pinned_version) install_root=$($L.install_root)"
Write-Host "== Waktu ledger ditulis (UTC): $($L.ledger_ditulis_utc)"

$root = $L.install_root
$pinExe = Join-Path $root "limine.exe"

Write-Host ""
Write-Host "-- 1. Utilitas host pada jalur pin"
if (-not (Test-Path $root)) {
    Write-Host "   $root : TIDAK ADA (direktori pin belum dibuat)"
    $sebab += "direktori pin tidak ada"
} elseif (-not (Test-Path $pinExe)) {
    Write-Host "   $pinExe : TIDAK ADA"
    $sebab += "limine.exe tidak ada di jalur pin"
} else {
    $f = Get-Item $pinExe
    $h = Get-Sha256 $pinExe
    Write-Host ("   limine.exe : ADA, {0} B, SHA256 {1}" -f $f.Length, $h)
    $expected = $null
    try { $expected = $L.hash_utilitas_host_yang_dicatat.sha256 } catch { $expected = $null }
    if ($expected) {
        if ($expected.ToLower() -eq $h) { Write-Host "   cocok dengan ledger." }
        else { Write-Host "   TIDAK cocok dengan ledger ($expected)."; $sebab += "hash limine.exe berbeda dari ledger" }
    } else {
        Write-Host "   ledger belum mencatat hash untuk berkas ini; catatkan sebelum artefak dipakai (perintah pemilik)."
        $sebab += "hash limine.exe belum tercatat di ledger"
    }
    $ver = (& $pinExe --version 2>&1) -join " "
    Write-Host "   --version : $ver"
    if ($ver -notmatch [regex]::Escape($L.pinned_version)) {
        Write-Host "   TIDAK memuat pinned_version '$($L.pinned_version)'."
        $sebab += "versi limine.exe tidak cocok dengan pin"
    }
}

Write-Host ""
Write-Host "-- 2. Payload pada jalur pin vs rujukan ledger (byte-identik)"
# Rujukan aktif dibaca dari ledger (kunci payload_rujukan_aktif) supaya pergantian versi pin
# tidak lagi menuntut suntingan di alat periksa - akar fault log #50/#54 adalah alat yang
# mengasumsi satu versi tertentu.
$refKey = $L.payload_rujukan_aktif
if (-not $refKey) {
    Write-Host "   LEDGER tidak punya kunci payload_rujukan_aktif - pemeriksaan tidak bisa dijalankan."
    exit 1
}
Write-Host "   kunci rujukan aktif: $refKey"
$rujuk = $L.$refKey.berkas
foreach ($nama in @("BOOTX64.EFI", "limine-bios-cd.bin", "limine-bios.sys", "limine-uefi-cd.bin")) {
    $ref = $rujuk.$nama
    $p = Join-Path $root $nama
    if (-not (Test-Path $p)) {
        Write-Host ("   {0,-20} : TIDAK ADA di jalur pin (rujukan {1} B, {2})" -f $nama, $ref.bytes, $ref.sha256.Substring(0, 16) + "...")
        $sebab += "payload $nama tidak ada"
        continue
    }
    $f = Get-Item $p
    $h = Get-Sha256 $p
    if ($f.Length -eq $ref.bytes -and $h -eq $ref.sha256.ToLower()) {
        Write-Host ("   {0,-20} : COCOK ({1} B)" -f $nama, $f.Length)
    } else {
        Write-Host ("   {0,-20} : BEDA  ukuran {1} vs {2} | sha {3} vs {4}" -f $nama, $f.Length, $ref.bytes, $h.Substring(0, 16) + "...", $ref.sha256.Substring(0, 16) + "...")
        $sebab += "payload $nama tidak identik dengan rujukan"
    }
}

Write-Host ""
Write-Host "-- 3. Staging ISO (target/iso_root/boot/limine)"
$staging = $L.payload_panggung_bangun
if (-not (Test-Path $staging)) {
    Write-Host "   $staging : TIDAK ADA - isi ulang setelah setiap `cargo clean` (akar fault log #50/#54)"
    $sebab += "staging ISO tidak ada"
} else {
    foreach ($nama in @("BOOTX64.EFI", "limine-bios-cd.bin", "limine-bios.sys", "limine-uefi-cd.bin")) {
        $p = Join-Path $staging $nama
        if (Test-Path $p) {
            $f = Get-Item $p
            $h = Get-Sha256 $p
            $ref = $rujuk.$nama
            $ok = ($f.Length -eq $ref.bytes) -and ($h -eq $ref.sha256.ToLower())
            Write-Host ("   {0,-20} : {1} ({2} B)" -f $nama, $(if ($ok) { "COCOK" } else { "BEDA" }), $f.Length)
            if (-not $ok) { $sebab += "staging $nama tidak identik dengan rujukan" }
        } else {
            Write-Host ("   {0,-20} : TIDAK ADA di staging" -f $nama)
            $sebab += "staging $nama tidak ada"
        }
    }
    $conf = Join-Path $staging "limine.conf"
    Write-Host ("   limine.conf          : {0}" -f $(if (Test-Path $conf) { "ADA" } else { "TIDAK ADA" }))
}

Write-Host ""
Write-Host "== RINGKASAN"
if ($sebab.Count -eq 0) {
    Write-Host "SIAP: perkakas cocok dengan ledger; boleh menjalankan scripts/build_kernel.ps1 -Action iso."
    exit 0
}
Write-Host ("BELUM SIAP - {0} sebab:" -f $sebab.Count)
foreach ($s in $sebab) { Write-Host "   - $s" }
Write-Host "Proses provisioning: docs/TOOLING/LIMINE-PROVISIONING.md bagian 5."
exit 1
