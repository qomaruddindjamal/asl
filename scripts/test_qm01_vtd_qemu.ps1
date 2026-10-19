# =============================================================================
# scripts/test_qm01_vtd_qemu.ps1 - HV-1 / QM-01 VT-d smoke (q35 + intel-iommu)
# Boot the hybrid ISO on QEMU q35 with the emulated Intel IOMMU enabled and
# capture COM1 serial to Analysis/reports/qemu_qm01_vtd_serial.log.
# Exit criteria observed from the serial log (no interactive display needed).
# =============================================================================
$ErrorActionPreference = "Stop"
$RepoRoot = "C:\asl"
$qemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$iso = Join-Path $RepoRoot "asl-kernel\target\asl-core-kernel.iso"
$serialLog = Join-Path $RepoRoot "Analysis\reports\qemu_qm01_vtd_serial.log"
$qemuErrLog = Join-Path $RepoRoot "Analysis\reports\qemu_qm01_vtd_qemu_stderr.log"

if (-not (Test-Path $qemuExe)) { Write-Error "QEMU not found at $qemuExe"; exit 1 }
if (-not (Test-Path $iso)) { Write-Error "ISO not found: $iso (run build_kernel.ps1 -Action iso first)"; exit 1 }
Remove-Item $serialLog,$qemuErrLog -ErrorAction SilentlyContinue

$args = @(
    "-machine", "q35",
    "-device", "intel-iommu,intremap=off,caching-mode=on",
    "-vga", "std",
    "-m", "256M",
    "-cdrom", $iso,
    "-serial", "file:$serialLog",
    "-display", "none",
    "-no-reboot",
    "-no-shutdown"
)
Write-Host "==> QM-01: launching QEMU q35 + intel-iommu..." -ForegroundColor Cyan
$proc = Start-Process $qemuExe -ArgumentList $args -RedirectStandardError $qemuErrLog -PassThru

$deadline = (Get-Date).AddSeconds(120)
$done = $false
while ((Get-Date) -lt $deadline) {
    Start-Sleep -Seconds 5
    if (Test-Path $serialLog) {
        $txt = Get-Content $serialLog -Raw -ErrorAction SilentlyContinue
        if ($txt -and $txt.Contains("QM-01 probe complete")) { $done = $true; break }
    }
    if ($proc.HasExited) { break }
}
if (-not $proc.HasExited) { Stop-Process -Id $proc.Id -Force }
Write-Host "==> QM-01 run finished (marker_found=$done). Serial evidence: $serialLog" -ForegroundColor Green
if (Test-Path $serialLog) { Get-Content $serialLog } else { Write-Host "NO SERIAL LOG PRODUCED" }
