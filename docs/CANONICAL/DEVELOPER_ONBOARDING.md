# ASL DEVELOPER ONBOARDING & STANDARD OPERATING PROCEDURES
=============================================================
**Target Audience:** Core Kernel Developers, Verification Engineers, Systems Auditors  
**Standard:** DO-178C-aligned, ladder DAL-C->A (lihat ADR-002) / ARINC 653 IMA  

---

## 1. WORKSPACE PREREQUISITES

The development workstation requires:
1. **Rust Toolchain:** `rustc 1.80+` with targets:
   - `x86_64-pc-windows-msvc` (Host compiler & unit tests)
   - `x86_64-unknown-none` (Bare-metal higher-half launcher target)
2. **ISO Mastering Tools:**
   - `xorriso` (v1.5.0+)
   - `limine` (v8.x binary utilities)
3. **Hardware Emulation / Hypervisor:**
   - Oracle VirtualBox with VM configured as `ASL` (PIIX4 IDE, UART COM1 redirected to file).
4. **Python 3.10+** for governance automation scripts.

> **Workspace Cargo Configuration Map (verified 2026-10-20, 6 config files total):**
> | Config | Purpose | Effect on `cargo test` host |
> | :--- | :--- | :--- |
> | `C:\asl\.cargo\config.toml` (repo root) | `build.target = x86_64-pc-windows-msvc` + Windows `rustc-proxy.wrapper` (MSVC env init) | Host-native — helps tests |
> | `asl-kernel\.cargo\config.toml` (workspace) | Per-target rustflags/linker only (`x86_64-unknown-none`, `aarch64-unknown-none-softfloat`); **no `[build] target` default by policy** | No interference — tests run on host by default |
> | `asl-kernel\asl-kernel-bin\.cargo\config.toml` | Bare-metal flags scoped to the launcher package dir (`x86_64-unknown-none`, `i686-unknown-none`) | Applies only when cwd = `asl-kernel-bin` |
> | `asl-kernel\tools\ci\.cargo\config.toml` | Empty placeholder | None |
> | `Python\pythoncore-3.15-64\...\{i686,x86_64}-pc-windows-msvc.json` | Bundled **rustup toolchain metadata descriptors**, NOT cargo configs (name collision only) | None |
>
> **Conflict check result: NONE.** No file sets global `[build] rustflags`; every bare-metal flag lives under an explicit `[target.<triple>]` table, so no build path can silently poison another. **Policy: the workspace `.cargo/config.toml` must never reintroduce `[build] target`** (this would re-break host testing — see DEEP_AUDIT risk DA-01, closed by FIX-1).

---

## 2. REPOSITORY & GOVERNANCE STRUCTURE

- `asl-kernel/`: The unified Cargo workspace containing all 31 microkernel crates (30 library crates + 1 binary launcher), per `cargo metadata --no-deps --format-version 1` (verified 2026-10-20).
- `asl-kernel/asl-kernel-bin`: The bare-metal Higher-Half ELF launcher for Limine.
- `asl-kernel/asl-kernel-core`: The pure `#![no_std]` Integrated Modular Avionics microkernel.
- `docs/CANONICAL/`: Authoritative system specifications and architectural models.
- `docs/ROADMAP/`: Formal gate tracking, handoff state, and accountability logs.
- `tools/`: Local governance automation tools (Must remain local-only per `.gitignore`).

---

## 3. CORE COMMAND WORKFLOWS

### A. Running Host Regression Tests (All Crates)
```powershell
cd C:\asl\asl-kernel
cargo test --workspace --lib
```
*Expected Result:* 128 passed, 0 failed (host `x86_64-pc-windows-msvc`, verified 2026-10-20).

> **Official Command Policy:** `.cargo/config.toml` intentionally carries **no `[build] target` default**, so `cargo test --workspace --lib` runs on the host automatically. Every bare-metal build (kernel ELF / ISO / QEMU) MUST pass an explicit `--target x86_64-unknown-none` (or `aarch64-unknown-none-softfloat`); build scripts and the Makefile already do so.

### B. Building Bare-Metal Kernel Binary
```powershell
cd C:\asl\asl-kernel
cargo build --package asl-kernel-bin --target x86_64-unknown-none --release
```
*Output Artifact:* `target\x86_64-unknown-none\release\asl-kernel` (ELF64 executable).

### C. Rebuilding Bootable Avionics ISO
```powershell
# Copy kernel binary to staging root
Copy-Item C:\asl\asl-kernel\target\x86_64-unknown-none\release\asl-kernel C:\asl\asl-kernel\target\iso_root\boot\asl-kernel.elf -Force

# Master ISO image
xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin -no-emul-boot -boot-load-size 4 -boot-info-table --efi-boot boot/limine/limine-uefi-cd.bin -efi-boot-part --efi-boot-image --protective-msdos-label /cygdrive/c/asl/asl-kernel/target/iso_root -o /cygdrive/c/asl/asl-kernel/target/asl-core-kernel.iso

# Install Limine BIOS stages
C:\asl\asl-kernel\target\limine_v8_binary\limine.exe bios-install C:\asl\asl-kernel\target\asl-core-kernel.iso
```

### D. Running Verification on VirtualBox VM "ASL"
```powershell
& 'C:\Program Files\Oracle\VirtualBox\VBoxManage.exe' startvm 'ASL' --type headless
```
*Verification Check:* Inspect serial log (`asl_vm_serial_release.log`) for milestones M1..M6 and inspect screen output (`asl_vm_screen.png`).

### E. Workspace Hygiene Audit
```powershell
python C:\asl\tools\Sub_Assist.py --hygiene
```
*Requirement:* Must return `ROOT WORKSPACE BERSIH 100%`.
