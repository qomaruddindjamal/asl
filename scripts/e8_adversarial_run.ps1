# =============================================================================
# e8_adversarial_run.ps1 - ALAT REPRODUKSI E8 (P0): sesi masukan adversarial
# terhadap shell kernel ASL pada QEMU 256M berjendela.
#
# Cara kerja (tanpa dependency eksternal, hanya .NET bawaan cangkang):
#   - Mesin dijalankan dengan satu port serial TCP dua arah pada 127.0.0.1,
#     sehingga baris masukan dapat dikirim secara deterministik dan seluruh
#     keluaran serial tertangkap byte-per-byte ke berkas bukti.
#   - Kernel mempoll COM1 sebagai sumber masukan (asl-kernel-core/src/lib.rs
#   1754-1763) dan menormalisasi 0x0D menjadi 0x0A, jadi satu 0x0A sudah cukup
#     sebagai penutup baris.
#   - Jendela tetap TERLIHAT: sengaja tidak ada "-display none" dan tidak ada
#     opsi penyembunyi jendela (aturan G9 - alat ini bukan alat senyap).
#   - Penanda crash yang dipantau: "[ASL-CORE-PANIC]" (penanganan panic
#     kernel-core), "[ASL EXCEPTION]" (asl-hal/src/x86_fault.rs), dan kata
#     "FATAL". Proses QEMU yang keluar sendiri juga dihitung sebagai crash
#     (-no-reboot -no-shutdown).
#
# Bukti yang dihasilkan (semua bercap-waktu, tidak menimpa artefak ter-commit):
#   Analysis\reports\e8_adversarial_serial_<cap>.log   -> keluaran serial mentah
#   Analysis\reports\e8_adversarial_cases_<cap>.tsv    -> tabel per kasus
#   Analysis\reports\e8_adversarial_summary_<cap>.log  -> ringkasan + keadaan akhir
#
# Kode keluar:
#   0 = sesi selesai, TIDAK ADA kasus memicu panic/hang (E8 tidak ter-reproduksi)
#   1 = prasyarat alat gagal (qemu/ISO/hashing port/konek/boot) - MERAH
#   2 = REPRODUKSI POSITIF: ada kasus yang memicu penanda crash atau shell tidak
#       kembali ke prompt dalam batas waktu
#   3 = proses QEMU keluar tanpa penanda crash (keadaan tak terduga)
#
# Pemakaian:
#   powershell -ExecutionPolicy Bypass -File scripts\e8_adversarial_run.ps1 -SelfTest
#   powershell -ExecutionPolicy Bypass -File scripts\e8_adversarial_run.ps1
#   powershell -ExecutionPolicy Bypass -File scripts\e8_adversarial_run.ps1 -StopQemu
# =============================================================================
param(
    [switch]$StopQemu,
    [switch]$SelfTest,
    [switch]$IncludeDiagnostics,
    [switch]$IncludeFaultDemo,
    [string]$DemoCommand = "",
    [int]$BootTimeoutSeconds = 180,
    [int]$CaseTimeoutSeconds = 45,
    [int]$QuietMilliseconds = 400,
    [int]$BasePort = 14721,
    [string]$ExpectedIsoSha = "aa77806018d7a9009bb705e29e71d33724a6b50fc3a5cf62de24133359c74de9"
)

$ErrorActionPreference = "Continue"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$Workspace = Join-Path $RepoRoot "asl-kernel"
$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$Iso = Join-Path $Workspace "target\asl-core-kernel.iso"
$PromptSource = Join-Path $Workspace "asl-cockpit-shell\src\lib.rs"
$Stamp = Get-Date -Format "yyyy-MM-dd_HH-mm-ss"
$Rep = Join-Path $RepoRoot "Analysis\reports"
$SerialLog = Join-Path $Rep "e8_adversarial_serial_$Stamp.log"
$CasesTsv = Join-Path $Rep "e8_adversarial_cases_$Stamp.tsv"
$SummaryLog = Join-Path $Rep "e8_adversarial_summary_$Stamp.log"
$StderrLog = Join-Path $Rep "e8_adversarial_qemu_stderr_$Stamp.log"
$PidFile = Join-Path $RepoRoot "logs\e8_adversarial_qemu_pid.txt"

function Sum($m) {
    Write-Host $m
    Add-Content -Path $SummaryLog -Value $m -Encoding ascii
}

function Show-Text($s, $max) {
    # Tampilkan teks kendali secara terlihat: ESC -> <ESC>, 0x7F -> <DEL>,
    # bukan-cetak -> <NN>. Tidak ada byte yang disembunyikan dari pembaca bukti.
    $out = New-Object System.Text.StringBuilder
    foreach ($ch in $s.ToCharArray()) {
        $code = [int]$ch
        if ($code -eq 27) { [void]$out.Append("<ESC>") }
        elseif ($code -eq 127) { [void]$out.Append("<DEL>") }
        elseif ($code -eq 8) { [void]$out.Append("<BS>") }
        elseif ($code -ge 32 -and $code -le 126) { [void]$out.Append($ch) }
        else { [void]$out.Append("<" + $code.ToString("D2") + ">") }
    }
    $t = $out.ToString()
    if ($t.Length -gt $max) { $t = $t.Substring(0, $max) + "...[" + $s.Length + " byte]" }
    return $t
}

# --------------------------------------------------------------- mode berhenti
if ($StopQemu) {
    if (Test-Path $PidFile) {
        $procId = (Get-Content $PidFile -Raw).Trim()
        $p = Get-Process -Id ([int]$procId) -ErrorAction SilentlyContinue
        if ($p) { Stop-Process -Id $p.Id -Force; Write-Host "[OK] jendela QEMU E8 (PID $($p.Id)) ditutup." }
        else { Write-Host "[i] PID $procId sudah tidak berjalan." }
        Remove-Item $PidFile -Force
    } else {
        Write-Host "[i] Tidak ada file PID E8 di logs\ - tidak ada yang ditutup."
    }
    exit 0
}

# --------------------------------------------- satu-sumber prompt + penanda crash
$Prompt = $null
Select-String -Path $PromptSource -Pattern 'pub const SHELL_PROMPT\s*:\s*&str\s*=\s*"([^"]*)"' |
    ForEach-Object { $Prompt = $_.Matches[0].Groups[1].Value }
if (-not $Prompt) {
    Write-Host "GERBANG MERAH - SHELL_PROMPT tidak terbaca dari $PromptSource"
    exit 1
}

$CrashMarkers = @("ASL-CORE-PANIC", "ASL EXCEPTION", "FATAL")

# ------------------------------------------------------------- daftar kasus
# Kind:
#   line  -> kirim teks + 0x0A, tunggu prompt kembali
#   burst -> kirim banyak baris berturut-turut tanpa jeda, lalu tunggu tenang
#   bytes -> kirim byte mentah (termasuk kendali) tanpa penutup baris
$Long512 = "A" * 512
$Long1024 = "B" * 1024
$Long5000 = "C" * 5000
$W8192 = "write-file a.txt " + ("D" * 8192)
$W8193 = "write-file a.txt " + ("E" * 8193)
$ManyArgs = "list satu dua tiga empat lima enam tujuh delapan sembilan sepuluh"

$Cases = @(
    @{ Id = "S01"; Kind = "line";  Label = "SIAP: buat direktori uji"; Text = "create-folder e8uji" },
    @{ Id = "S02"; Kind = "line";  Label = "SIAP: masuk direktori uji"; Text = "open e8uji" },
    @{ Id = "S03"; Kind = "line";  Label = "SIAP: buat berkas a.txt"; Text = "create-file a.txt" },
    @{ Id = "S04"; Kind = "line";  Label = "SIAP: isi a.txt"; Text = "write-file a.txt isi awal e8" },

    @{ Id = "C01"; Kind = "line";  Label = "baris kosong"; Text = "" },
    @{ Id = "C02"; Kind = "line";  Label = "hanya spasi"; Text = "     " },
    @{ Id = "C03"; Kind = "line";  Label = "baris 512 karakter"; Text = $Long512 },
    @{ Id = "C04"; Kind = "line";  Label = "baris 1024 karakter"; Text = $Long1024 },
    @{ Id = "C05"; Kind = "line";  Label = "baris 5000 karakter (>=4096)"; Text = $Long5000 },
    @{ Id = "C06"; Kind = "line";  Label = "argumen '..'"; Text = "open .." },
    @{ Id = "C07"; Kind = "line";  Label = "argumen '//'"; Text = "open //" },
    @{ Id = "C08"; Kind = "line";  Label = "trailing slash"; Text = "create-folder namaku/" },
    @{ Id = "C09"; Kind = "line";  Label = "argumen titik"; Text = "open ." },
    @{ Id = "C10"; Kind = "line";  Label = "argumen berkutip"; Text = 'create-folder "nama bertanda"' },
    @{ Id = "C11"; Kind = "line";  Label = "argumen hubung"; Text = "delete-file -" },
    @{ Id = "C12"; Kind = "line";  Label = "open berkas yang ada"; Text = "open a.txt" },
    @{ Id = "C13"; Kind = "line";  Label = "open jalur tidak ada"; Text = "open tidak-ada-e8-9" },
    @{ Id = "C14"; Kind = "line";  Label = "open '/..'"; Text = "open /.." },
    @{ Id = "C15"; Kind = "line";  Label = "up di akar"; Text = "up" },
    @{ Id = "C16"; Kind = "line";  Label = "create-folder nama baru (siap kasus bertumpuk)"; Text = "create-folder sub" },
    @{ Id = "C16B"; Kind = "line"; Label = "create-folder nama sudah ada"; Text = "create-folder sub" },
    @{ Id = "C17"; Kind = "line";  Label = "create-folder berisi garis miring"; Text = "create-folder luar/dalam" },
    @{ Id = "C18"; Kind = "line";  Label = "create-folder tanpa nama"; Text = "create-folder" },
    @{ Id = "C19"; Kind = "line";  Label = "create-file nama sudah ada"; Text = "create-file a.txt" },
    @{ Id = "C20"; Kind = "line";  Label = "write-file ke folder tidak ada"; Text = "write-file tidak-ada-e8/x.txt isi" },
    @{ Id = "C21"; Kind = "line";  Label = "write-file pada folder"; Text = "write-file sub isi" },
    @{ Id = "C22"; Kind = "line";  Label = "write-file isi kosong (tanpa argumen isi)"; Text = "write-file a.txt" },
    @{ Id = "C23"; Kind = "line";  Label = "write-file isi satu spasi"; Text = "write-file a.txt  " },
    @{ Id = "C24"; Kind = "line";  Label = "write-file tepat 8192 byte (di ujung baris)"; Text = $W8192 },
    @{ Id = "C25"; Kind = "line";  Label = "write-file 8193 byte (di ujung baris)"; Text = $W8193 },
    @{ Id = "C26"; Kind = "line";  Label = "rename ke nama sudah ada"; Text = "rename a.txt b.txt" },
    @{ Id = "C27"; Kind = "line";  Label = "rename akar"; Text = "rename / /" },
    @{ Id = "C28"; Kind = "line";  Label = "delete-file pada folder"; Text = "delete-file sub" },
    @{ Id = "C29"; Kind = "line";  Label = "delete-folder pada berkas"; Text = "delete-folder a.txt" },
    @{ Id = "C30"; Kind = "line";  Label = "delete akar"; Text = "delete-folder /" },
    @{ Id = "C31"; Kind = "line";  Label = "perintah tak dikenal"; Text = "perintahane8takdikenal" },
    @{ Id = "C32"; Kind = "line";  Label = "argumen berlebih"; Text = $ManyArgs },
    @{ Id = "C33"; Kind = "line";  Label = "nama panjang 127 byte di ujung buffer"; Text = ("f" * 127) },
    @{ Id = "C34"; Kind = "bytes"; Label = "byte kendali mentah (NUL, BEL, BS, DEL, ESC-A)"; Bytes = @(0, 7, 8, 127, 27, 91, 65) },
    @{ Id = "C35"; Kind = "burst"; Label = "rentetan cepat tanpa jeda (8 baris)"; Lines = @("status", "where", "list", "open ..", "help", "create-file b.txt", "delete-file b.txt", "status") },
    @{ Id = "C36"; Kind = "burst"; Label = "rentetan cepat baris kosong x12"; Lines = @("", "", "", "", "", "", "", "", "", "", "", "") }
)

$SetupBack = @(
    @{ Id = "S91"; Kind = "line"; Label = "SISA: kembali ke akar"; Text = "open /" },
    @{ Id = "S92"; Kind = "line"; Label = "SISA: hapus direktori uji"; Text = "delete-folder e8uji" }
)

# KUMPULAN B (hanya dengan -IncludeDiagnostics): perintah diagnostik lama yang
# masih hidup di jalur legacy. Kandidat panic statis ada di jalur ini
# (unwrap/assert ber-state), jadi daftar utama saja tidak cukup untuk E8.
$Diag = @(
    @{ Id = "D01"; Kind = "line"; Label = "diagnostik meminfo"; Text = "meminfo" },
    @{ Id = "D02"; Kind = "line"; Label = "diagnostik cpuinfo"; Text = "cpuinfo" },
    @{ Id = "D03"; Kind = "line"; Label = "diagnostik lspci"; Text = "lspci" },
    @{ Id = "D04"; Kind = "line"; Label = "diagnostik lsblk"; Text = "lsblk" },
    @{ Id = "D05"; Kind = "line"; Label = "diagnostik fdisk"; Text = "fdisk" },
    @{ Id = "D06"; Kind = "line"; Label = "diagnostik mount"; Text = "mount" },
    @{ Id = "D07"; Kind = "line"; Label = "diagnostik uname"; Text = "uname" },
    @{ Id = "D08"; Kind = "line"; Label = "diagnostik failover"; Text = "failover" },
    @{ Id = "D09"; Kind = "line"; Label = "diagnostik test_pml4"; Text = "test_pml4" },
    @{ Id = "D10"; Kind = "line"; Label = "diagnostik test_apic (mengaktifkan interupsi)"; Text = "test_apic" },
    @{ Id = "D11"; Kind = "line"; Label = "sesudah test_apic: status (shell masih hidup?)"; Text = "status" },
    @{ Id = "D12"; Kind = "burst"; Label = "rentetan cepat saat timer aktif"; Lines = @("where", "list", "status", "uname", "help") },
    @{ Id = "D13"; Kind = "line"; Label = "diagnostik test_task (jalur unwrap/assert)"; Text = "test_task" },
    @{ Id = "D14"; Kind = "line"; Label = "test_task diulang (stateManager berubah?)"; Text = "test_task" },
    @{ Id = "D15"; Kind = "line"; Label = "test_apic setelah test_task"; Text = "test_apic" },
    @{ Id = "D16"; Kind = "line"; Label = "perintah biasa sesudah rangkaian diagnostik"; Text = "status" }
)

# KUMPULAN C (hanya dengan -IncludeFaultDemo): penyuntik fault yang SENGAJA.
# Satu boot hanya menampung satu kasus karena penanganan berakhir di
# 'loop { cli; hlt }'. Pakai -DemoCommand untuk memilih satu token per boot.
$Demo = @(
    @{ Id = "X01"; Kind = "line"; Label = "suntikan fault trap_de"; Text = "trap_de" },
    @{ Id = "X02"; Kind = "line"; Label = "suntikan fault trap_bp"; Text = "trap_bp" },
    @{ Id = "X03"; Kind = "line"; Label = "suntikan fault trap_pf"; Text = "trap_pf" },
    @{ Id = "X04"; Kind = "line"; Label = "suntikan fault trap_df"; Text = "trap_df" }
)

# Rangkai urutan yang akan dijalankan. -IncludeDiagnostics menambahkan kumpulan
# diagnostik lama; -IncludeFaultDemo menambahkan penyuntik fault yang sengaja
# (hanya satu yang muat per boot karena penanganan berakhir macet).
if ($DemoCommand -ne "") {
    $Seq = @(@{ Id = "V01"; Kind = "line"; Label = "verifikasi satu token: $DemoCommand"; Text = $DemoCommand })
} else {
    $Seq = @($Cases) + @($SetupBack)
    if ($IncludeDiagnostics) { $Seq = $Seq + $Diag }
    if ($IncludeFaultDemo) { $Seq = $Seq + $Demo }
}

# ------------------------------------------------------------------ self test
if ($SelfTest) {
    Write-Host "SELF-TEST e8_adversarial_run.ps1 - TIDAK menjalankan QEMU, TIDAK menulis artefak."
    Write-Host "  prompt satu-sumber : '$Prompt' (dari $(Split-Path -Leaf $PromptSource))"
    Write-Host "  penanda crash      : $($CrashMarkers -join ' | ')"
    Write-Host "  kasus              : daftar jalan = $($Seq.Count) (>=30 terpenuhi untuk daftar utama)"
    Write-Host ("    kumpulan utama   : {0} kasus + {1} sisa" -f $Cases.Count, $SetupBack.Count)
    Write-Host ("    diagnostik (B)   : {0} kasus (aktif: {1})" -f $Diag.Count, $IncludeDiagnostics)
    Write-Host ("    suntikan fault(C): {0} kasus (aktif: {1})" -f $Demo.Count, $IncludeFaultDemo)
    Write-Host ("    satu token       : '{0}' (mode verifikasi per boot)" -f $DemoCommand)
    $n = 0
    foreach ($c in $Seq) {
        if ($c.Kind -eq "bytes") {
            $desc = ($c.Bytes | ForEach-Object { $_.ToString("D2") }) -join " "
            Write-Host ("    {0} [{1}] {2} -> bytes {3}" -f $c.Id, $c.Kind, $c.Label, $desc)
        } elseif ($c.Kind -eq "burst") {
            Write-Host ("    {0} [{1}] {2} -> {3} baris" -f $c.Id, $c.Kind, $c.Label, $c.Lines.Count)
        } else {
            $bytes = [System.Text.Encoding]::ASCII.GetByteCount($c.Text)
            Write-Host ("    {0} [{1}] {2} -> '{3}' ({4} byte)" -f $c.Id, $c.Kind, $c.Label, (Show-Text $c.Text 60), $bytes)
        }
        $n++
    }
    Write-Host "  total payload      : $n"
    Write-Host "  ISO yang di-boot   : $Iso"
    if (Test-Path $Iso) {
        $h = (Get-FileHash -Algorithm SHA256 $Iso).Hash.ToLower()
        Write-Host ("    {0} bytes SHA256={1}" -f (Get-Item $Iso).Length, $h)
        Write-Host ("    cocok dengan sidik jari baseline E8 : {0}" -f ($h -eq $ExpectedIsoSha.ToLower()))
    } else {
        Write-Host "    (belum ada - jalankan build_kernel.ps1 -Action iso sebelum sesi)"
    }
    Write-Host "  mesin              : $QemuExe"
    Write-Host "  serial             : chardev socket TCP 127.0.0.1 (porta $BasePort dan seterusnya), berjendela"
    Write-Host "  kode keluar        : 0=tanpa panic, 1=prasyarat merah, 2=REPRODUKSI POSITIF, 3=QEMU keluar tanpa penanda"
    exit 0
}

# ----------------------------------------------------------------- prasyarat
"ASL E8 ADVERSARIAL INPUT RUN - $Stamp" | Out-File -FilePath $SummaryLog -Encoding ascii
Sum "prompt satu-sumber : '$Prompt'"
Sum "penanda crash      : $($CrashMarkers -join ' | ')"
Sum "kasus              : $($Cases.Count) utama + $($SetupBack.Count) sisa; dijalankan $($Seq.Count) kasus (diagnostik=$IncludeDiagnostics faultDemo=$IncludeFaultDemo satuToken='$DemoCommand')"

if (-not (Test-Path $QemuExe)) { Sum "[FAIL] qemu tidak ditemukan: $QemuExe"; exit 1 }
if (-not (Test-Path $Iso)) { Sum "[FAIL] ISO tidak ditemukan: $Iso"; exit 1 }

$isoSize = (Get-Item $Iso).Length
$isoHash = (Get-FileHash -Algorithm SHA256 $Iso).Hash.ToLower()
Sum "QEMU   : $(& $QemuExe --version | Select-Object -First 1)"
Sum "ISO    : $Iso"
Sum "ISO    : $isoSize bytes SHA256=$isoHash"
if ($isoHash -ne $ExpectedIsoSha.ToLower()) {
    Sum "[FAIL] sidik jari ISO tidak cocok dengan baseline E8 ($ExpectedIsoSha) - sesi harus diulang pada ISO yang benar."
    exit 1
}

# cari porta TCP yang bebas (socket chardev dua arah untuk COM1)
$Port = 0
for ($i = 0; $i -lt 12; $i++) {
    $cand = $BasePort + $i
    $probe = New-Object System.Net.Sockets.TcpListener([System.Net.IPAddress]::Loopback, $cand)
    try { $probe.Start(); $Port = $cand; $probe.Stop(); break }
    catch { $Port = 0 }
    finally { $probe = $null }
}
if ($Port -eq 0) { Sum "[FAIL] tidak ada porta TCP bebas mulai $BasePort - prasyarat alat merah."; exit 1 }
Sum "PORTA  : 127.0.0.1:$Port (chardev socket dua arah, COM1)"

# ------------------------------------------------------------- jalankan mesin
$qemuArgs = @(
    "-machine", "q35", "-vga", "std", "-m", "256M", "-cdrom", $Iso,
    "-chardev", "socket,id=ser0,server=on,wait=on,host=127.0.0.1,port=$Port",
    "-serial", "chardev:ser0",
    "-no-reboot", "-no-shutdown"
)
Sum ""
Sum "=== FASE 1 - cold boot 256M berjendela + koneksi serial ==="
Sum "CMDLINE: `"$QemuExe`" $($qemuArgs -join ' ')"

$proc = Start-Process $QemuExe -ArgumentList $qemuArgs -RedirectStandardError $StderrLog -PassThru
if (-not $proc) { Sum "[FAIL] Start-Process mengembalikan null"; exit 1 }
$proc.Id | Out-File -FilePath $PidFile -Encoding ascii
Sum "PID    : $($proc.Id)  (tutup dengan: e8_adversarial_run.ps1 -StopQemu)"

$client = New-Object System.Net.Sockets.TcpClient
$deadline = (Get-Date).AddSeconds(60)
$connected = $false
while ((Get-Date) -lt $deadline) {
    try { $client.Connect("127.0.0.1", $Port); $connected = $true; break } catch { Start-Sleep -Milliseconds 250 }
    if ($proc.HasExited) { break }
}
if (-not $connected) {
    Sum "[FAIL] koneksi ke chardev serial gagal - prasyarat alat merah."
    if (-not $proc.HasExited) { Stop-Process -Id $proc.Id -Force }
    Remove-Item $PidFile -Force -ErrorAction SilentlyContinue
    exit 1
}
$stream = $client.GetStream()
$fs = [System.IO.File]::Open($SerialLog, [System.IO.FileMode]::Append, [System.IO.FileAccess]::Write)
Sum "SERIAL : terhubung, log byte-per-byte -> $SerialLog"

# ---- keadaan baca
$sb = New-Object System.Text.StringBuilder
$rxTail = New-Object System.Text.StringBuilder
$readBuf = New-Object byte[] 8192
$lastByteAt = Get-Date
$bytesIn = 0
$bytesOut = 0

function Pull-Bytes {
    # Baca seluruh byte yang tersedia; tulis ke log + keptakan tails.
    $script:pullCount = 0
    while ($stream.DataAvailable) {
        $n = $stream.Read($readBuf, 0, $readBuf.Length)
        if ($n -le 0) { break }
        $script:bytesIn += $n
        $script:lastByteAt = Get-Date
        $fs.Write($readBuf, 0, $n)
        $script:pullCount += $n
        $s = [System.Text.Encoding]::ASCII.GetString($readBuf, 0, $n)
        [void]$script:sb.Append($s)
        [void]$script:rxTail.Append($s)
        if ($script:rxTail.Length -gt 4096) {
            $keep = $script:rxTail.ToString()
            $script:rxTail.Clear()
            [void]$script:rxTail.Append($keep.Substring($keep.Length - 2048))
        }
    }
    $fs.Flush()
    return $script:pullCount
}

function Send-Line($text) {
    $b = [System.Text.Encoding]::ASCII.GetBytes($text + "`n")
    $stream.Write($b, 0, $b.Length)
    $stream.Flush()
    $script:bytesOut += $b.Length
    return $b.Length
}

function Crash-Found($text) {
    foreach ($m in $CrashMarkers) { if ($text.Contains($m)) { return $m } }
    return $null
}

function Wait-Prompt($needMore, $timeoutSeconds) {
    # Tunggu sampai jumlah prompt bertambah $needMore dan keluaran tenang.
    $base = ([regex]::Matches($sb.ToString(), [regex]::Escape($Prompt))).Count
    $until = (Get-Date).AddSeconds($timeoutSeconds)
    while ((Get-Date) -lt $until) {
        Pull-Bytes | Out-Null
        $now = ([regex]::Matches($sb.ToString(), [regex]::Escape($Prompt))).Count
        $quietMs = ((Get-Date) - $lastByteAt).TotalMilliseconds
        if ($now -ge ($base + $needMore) -and $quietMs -ge $QuietMilliseconds) {
            return $true
        }
        if ($proc.HasExited) { return $false }
        if (Crash-Found $sb.ToString()) { return $false }
        Start-Sleep -Milliseconds 25
    }
    return $false
}

# ---------------------------------------------------- tunggu prompt pertama
$ok = Wait-Prompt 1 $BootTimeoutSeconds
$bootCrash = Crash-Found $sb.ToString()
$booted = $sb.ToString().Contains("M6: Kernel Cold Boot Complete")
if ($proc.HasExited) {
    Sum "[FAIL] proses QEMU keluar sebelum prompt - mesin merah (lihat stderr)."
    Remove-Item $PidFile -Force -ErrorAction SilentlyContinue
    $fs.Close(); $client.Close()
    exit 3
}
Sum "Boot   : M6=$booted prompt=$ok crash=$(if ($bootCrash) { $bootCrash } else { 'TIDAK' })"
if (-not $ok) {
    Sum "[FAIL] boot tidak mencapai prompt '$Prompt' dalam $BootTimeoutSeconds s - mesin merah (G5)."
    if (-not $proc.HasExited) { Stop-Process -Id $proc.Id -Force }
    Remove-Item $PidFile -Force -ErrorAction SilentlyContinue
    $fs.Close(); $client.Close()
    exit 1
}
if ($bootCrash) {
    Sum "[PANIK] penanda crash muncul sebelum sesi: $bootCrash"
}

# ------------------------------------------------------------- fase 2: kasus
Sum ""
Sum "=== FASE 2 - sesi masukan adversarial ==="
Sum "CATATAN bangun: pada ISO ini registry kanonik berisi 11 perintah; 'up' (C15) dan 'rename' (C26,C27)"
Sum "             BELUM ada, jadi kedua kasus itu menguji jalur 'perintah tak dikenal' - bukan jalur"
Sum "             aslinya. Jalur asli keduanya diuji lagi setelah F2/F6, dan jendela E8 diulang."
$rows = New-Object System.Collections.Generic.List[string]
$rows.Add("id`tlabel`thasil_pokok`tjenis_kirim`tbyte_kirim`tdetik_mm`tkiriminput_terlihat`tteks_balasan_terpotong")

$repro = @()
$caseNo = 0
foreach ($c in $Seq) {
    $caseNo++
    $sentText = ""
    $outcome = ""
    $result = "OK"
    $before = $sb.Length
    $sentBytes = 0
    $t0 = Get-Date
    $toSec = $CaseTimeoutSeconds
    if ($c.ContainsKey("Timeout")) { $toSec = [int]$c.Timeout }
    # Kumpulan diagnostik (D09/D10/D13/D15) menjalankan rangkaian uji panjang;
    # batas per kasus dinaikkan lewat -CaseTimeoutSeconds saat -IncludeDiagnostics.

    if ($c.Kind -eq "bytes") {
        $sentText = ($c.Bytes | ForEach-Object { $_.ToString("D2") }) -join " "
        $b = New-Object byte[] ($c.Bytes.Count)
        for ($i = 0; $i -lt $c.Bytes.Count; $i++) { $b[$i] = [byte]$c.Bytes[$i] }
        $stream.Write($b, 0, $b.Length); $stream.Flush()
        $sentBytes = $b.Length
        $bytesOut += $b.Length
        Start-Sleep -Milliseconds 600
        # kirim enter agar baris yang belum lengkap tidak menggantung
        $sentBytes += (Send-Line "")
        $settled = Wait-Prompt 1 $toSec
    } elseif ($c.Kind -eq "burst") {
        $sentText = ($c.Lines | ForEach-Object { "'$_'" }) -join " "
        foreach ($ln in $c.Lines) { $sentBytes += (Send-Line $ln) }
        $settled = Wait-Prompt $c.Lines.Count $toSec
    } else {
        $sentText = $c.Text
        $sentBytes = Send-Line $c.Text
        $settled = Wait-Prompt 1 $toSec
    }
    Pull-Bytes | Out-Null
    $elapsedMs = [int]((Get-Date) - $t0).TotalMilliseconds

    $crash = Crash-Found $sb.ToString()
    $reply = ""
    if ($sb.Length -gt $before) { $reply = $sb.ToString($before, $sb.Length - $before) }
    $replyOneLine = ($reply -replace "[`r`n]+", " | ")

    if ($proc.HasExited) {
        $result = "MESIN-KELUAR"
        $repro += "$($c.Id) QEMU keluar tanpa penanda crash (kemungkinan fault tingkat tiga)"
        $outcome = "proses berhenti"
        Sum "[PANIK] $($c.Id) '$($c.Label)': proses QEMU keluar - lihat balasan: $(Show-Text $replyOneLine 300)"
        break
    } elseif ($crash) {
        $result = "PANIK"
        $repro += "$($c.Id) memicu '$crash'"
        $outcome = "penanda crash '$crash'"
        Sum "[PANIK] $($c.Id) '$($c.Label)' -> $crash"
        Sum "        balasan: $(Show-Text $replyOneLine 500)"
        break
    } elseif (-not $settled) {
        $result = "GANTUNG"
        $repro += "$($c.Id) shell tidak kembali ke prompt dalam $toSec s"
        $outcome = "prompt tidak kembali"
        Sum "[GANTUNG] $($c.Id) '$($c.Label)' tidak mendapat prompt kembali dalam $toSec s"
        Sum "          balasan: $(Show-Text $replyOneLine 500)"
    } else {
        $outcome = "prompt kembali"
    }

    $rows.Add(("{0}`t{1}`t{2}`t{3}`t{4}`t{5}`t{6}`t{7}" -f `
        $c.Id, $c.Label, $result, $c.Kind, $sentBytes, $elapsedMs, (Show-Text $sentText 120), (Show-Text $replyOneLine 240)))
    Sum ("  {0,-4} [{1,-8}] {2,-46} kirim {3,5} byte dalam {4,6} ms -> {5} :: {6}" -f `
        $c.Id, $result, $c.Label, $sentBytes, $elapsedMs, $outcome, (Show-Text $replyOneLine 150))
    Start-Sleep -Milliseconds 120
}

# ------------------------------------------------------------- penutup alat
Sum ""
Sum "=== Keadaan akhir sesi ==="
Sum "byte masuk (serial->alat)  : $bytesIn"
Sum "byte keluar (alat->serial) : $bytesOut"
$errBytes = if (Test-Path $StderrLog) { (Get-Item $StderrLog).Length } else { 0 }
$stderrOther = $false
Sum "stderr QEMU                : $errBytes byte"
if ($errBytes -gt 0) {
    $errTxt = (Get-Content $StderrLog -Raw -ErrorAction SilentlyContinue)
    if ($errTxt -match 'info: QEMU waiting for connection') {
        Sum "  isi stderr = pesan 'info: QEMU waiting for connection' dari chardev socket"
        Sum "  (konsekuensi wait=on agar tidak ada byte boot yang hilang). Bukan galat mesin."
        Sum "  Gerbang 'stderr 0 byte' alat G9 tetap berlaku untuk alat itu; di sesi ini"
        Sum "  jumlah byte ini dilaporkan terang, bukan disembunyikan."
    } else {
        Sum "  [MERAH] stderr berisi teks lain - lihat $StderrLog (prasyarat alat tidak terpenuhi)."
        $stderrOther = $true
    }
}
Sum "serial log                 : $SerialLog ($((Get-Item $SerialLog).Length) byte)"

if ($caseNo -lt $seq.Count) {
    Sum "CATATAN: sesi berhenti di kasus $($c.Id) - $(($seq.Count - $caseNo)) kasus berikutnya TIDAK dijalankan,"
    Sum "         termasuk pembersihan direktori uji. Setelah perbaikan, jalankan ulang dari awal."
}

if ($repro.Count -gt 0) {
    Sum "HASIL: REPRODUKSI POSITIF - $($repro.Count) kasus:"
    foreach ($r in $repro) { Sum "  - $r" }
    if ($DemoCommand -ne "") {
        Sum "E8-STATUS: MODE VERIFIKASI token '$DemoCommand' - crash terlihat, ambil teks penanganannya."
    } else {
        Sum "E8-STATUS: TERREPRODUKSI oleh alat. Ambil potongan panic di serial log sebagai akar bukti."
    }
} elseif ($DemoCommand -ne "") {
    Sum "HASIL: token '$DemoCommand' selesai tanpa penanda crash."
    Sum "E8-STATUS: MODE VERIFIKASI - jalur ini kini kembali ke prompt (diharapkan setelah perbaikan)."
} else {
    Sum "HASIL: TIDAK ADA kasus yang memicu panic/hang pada sesi ini."
    Sum "E8-STATUS: BELUM-TERREPRODUKSI lewat alat. Teks panic dari pemilik tetap diperlukan untuk menutup E8."
}
if ($stderrOther) { Sum "E8-STATUS: ditambah catatan MERAH pada stderr QEMU (bukan pesan chardev yang diperkirakan)." }

# simpan tabel kasus lalu tutup mesin
[System.IO.File]::WriteAllText($CasesTsv, ($rows -join "`r`n") + "`r`n", [System.Text.Encoding]::ASCII)
Sum "tabel kasus disimpan     : $CasesTsv ($($rows.Count) baris)"

try { if (-not $proc.HasExited) { $stream.Close() } } catch { }
try { $fs.Close() } catch { }
try { $client.Close() } catch { }
if (-not $proc.HasExited) {
    Stop-Process -Id $proc.Id -Force
    Sum "jendela QEMU ditutup alat (sesi selesai)."
}
Remove-Item $PidFile -Force -ErrorAction SilentlyContinue

if ($repro.Count -gt 0) { exit 2 }
if ($stderrOther) { exit 1 }
exit 0
