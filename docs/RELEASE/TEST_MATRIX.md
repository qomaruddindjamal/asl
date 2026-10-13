# ASL CORE KERNEL: ACCEPTANCE & TEST MATRIX
==================================================
**Release Target:** v1.0.0-rc.1 / v1.0.0 Reference Baseline  
**Verification Date:** 2026-10-13  

| Test ID | Requirement / Function | Test Command / Execution Method | Target | Expected Result | Actual Result | Status | Evidence Location |
| :--- | :--- | :--- | :--- | :--- | :--- | :---: | :--- |
| **TEST-01** | Workspace Host Logic & Memory Protection | `cargo test --workspace --target x86_64-pc-windows-msvc` | Host MSVC | 31 tests pass, 0 fail | 31 passed, 0 failed | **PASS** | Crate test runners |
| **TEST-02** | Bare-Metal ELF64 Kernel Compilation | `cargo build --package asl-kernel-bin --target x86_64-unknown-none` | `x86_64-unknown-none` | Clean ELF64 compilation | Exit code 0, 0 warnings | **PASS** | `asl-kernel/target/x86_64-unknown-none/debug/asl-kernel` |
| **TEST-03** | Hybrid Bootable ISO Mastering | `xorriso` + `limine bios-install` | Hybrid El Torito | 5.2 MB bootable ISO | Size: 5,216,256 B | **PASS** | `asl-kernel/target/asl-core-kernel.iso` |
| **TEST-04** | Hypervisor Boot Telemetry M1..M3 | VBox headless boot (`VBoxManage startvm ASL`) | VirtualBox VM `ASL` | M1, M2, M3 in COM1 log | M1, M2, M3 logged | **PASS** | `Analysis/reports/asl_vm_serial.log` |
| **TEST-05** | Microkernel Allocator Telemetry M4 | VBox headless boot (`VBoxManage startvm ASL`) | VirtualBox VM `ASL` | M4 logged via COM1 UART | M4 logged | **PASS** | `Analysis/reports/asl_vm_serial.log` |
| **TEST-06** | ARINC 653 Partitions & Scheduler M5 | VBox headless boot (`VBoxManage startvm ASL`) | VirtualBox VM `ASL` | M5 logged via COM1 UART | M5 logged | **PASS** | `Analysis/reports/asl_vm_serial.log` |
| **TEST-07** | Deterministic Idle Loop Execution M6 | VBox headless boot (`VBoxManage startvm ASL`) | VirtualBox VM `ASL` | M6 logged via COM1 UART | M6 logged | **PASS** | `Analysis/reports/asl_vm_serial.log` |
| **TEST-08** | Local Workspace Hygiene & Governance | `python tools/Sub_Assist.py --hygiene` | Local Host | 100% clean status | 100% clean status | **PASS** | stdout |
| **TEST-09** | Architecture Audit & Gate Locking | `python tools/Gemini_Assist.py --audit` | Local Host | 25/25 Gates LOCKED | 25/25 Gates LOCKED | **PASS** | stdout / `docs/ROADMAP/CHECKLIST_STATE.json` |
