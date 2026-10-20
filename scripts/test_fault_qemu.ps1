param()
$ErrorActionPreference = "Continue"
if (Test-Path "C:\asl\logs\test_serial.log") { Remove-Item "C:\asl\logs\test_serial.log" -Force }
if (Test-Path "C:\asl\logs\test_qemu.log") { Remove-Item "C:\asl\logs\test_qemu.log" -Force }

$qemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$proc = Start-Process $qemuExe -ArgumentList "-cdrom C:\asl\asl-kernel\target\asl-core-kernel.iso -serial file:C:\asl\logs\test_serial.log -display none -no-reboot -d int,cpu_reset -D C:\asl\logs\test_qemu.log" -PassThru
Start-Sleep -Seconds 75
if (-not $proc.HasExited) {
    Stop-Process -Id $proc.Id -Force
}

if (Test-Path "C:\asl\logs\test_serial.log") {
    Write-Host "=== SERIAL LOG OUTPUT ==="
    Get-Content "C:\asl\logs\test_serial.log"
} else {
    Write-Host "No serial log produced."
}
