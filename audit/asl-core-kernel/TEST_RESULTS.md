# TEST RESULTS: ASL CORE KERNEL AUDIT
**Date:** 2026-10-13  
**Auditor:** Senior Software Auditor & Quality Assurance Engineer  

---

## 1. TEST EXECUTION ENVIRONMENT
* **Platform:** Windows x86_64
* **Compiler:** `rustc 1.99.0 (b940084d7 2026-09-28)`
* **Build Manager:** `cargo 1.99.0 (5f94df478 2026-08-27)`
* **Profile:** Development (`unoptimized + debuginfo`)

---

## 2. STATIC ANALYSIS & CHECK OUTCOMES

### Command:
```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
cargo check --workspace
```
### Result:
* **Exit Code:** 0 (SUCCESS)
* **Status:** All 25 crates compiled successfully through static check analysis.
* **Compiler Warnings (15 warnings identified):**
  1. `asl-fleet-regression`: Unused import `asl_bootloader::TargetArch` (Line 9).
  2. `asl-power10-loongson`: Non-camel case enum variants (`X86_Legacy`, `Arm_Legacy`, `Mips_Legacy`).
  3. `asl-ont-gpon`: Non-camel case enum variants (`O1_InitialState`, `O2_StandbyState`, etc.).
  4. `asl-do178c-cert`: Non-camel case enum variants (`DalA_Catastrophic`, `DalB_Hazardous`, etc.).
  5. `asl-kernel-core`: Unused import `AfdxFrame` (Line 22).

---

## 3. UNIT & INTEGRATION TEST OUTCOMES

### Command:
```powershell
cargo test --workspace
```
### Result:
* **Exit Code:** 0 (SUCCESS)
* **Status:** 26 automated unit tests executed and passed across the workspace.
* **Test Breakdown by Crate:**
  1. `asl-security-vault`: 5 passed (SEC-01 integer overflow, zero size, boundary checks, partition matrix).
  2. `asl-bootloader`: 7 passed (SEC-02 physical address range limit 52-bit, integer overflow, memory map validation).
  3. `asl-afdx-ipc`: 4 passed (SEC-03 SPSC lock-free circular ring buffer, BAG rate limiter, capacity wraparound).
  4. `asl-dsp-rpc`: 3 passed (SEC-03 SPSC lock-free shared memory RPC mailbox, FIFO ordering, capacity wraparound).
  5. `asl-pmd`: 3 passed (SEC-03 SPSC descriptor flags, ownership handoff, burst receive, congestion return).
  6. `asl-ont-gpon`: 2 passed (SEC-04 mock HNAT acceleration behavior and optical signal state machine).
  7. `asl-mips-fastpath`: 2 passed (SEC-05 TLB refill fastpath and big/little endian packet normalization).
* **Summary:**
  * Total Passed: 26
  * Failed: 0
  * Ignored: 0
  * Filtered out: 0

---

## 4. TESTS & CHECKS NOT PERFORMED & REASONS
1. **Dynamic Cross-Target Bare-Metal Hardware Execution:** Not executed on physical silicon. The host execution environment is `x86_64-pc-windows-msvc`. Target-specific memory-mapped I/O (MMIO), DMA bus snooping, hardware cache flush instructions (e.g., ARM `dc civac`, MIPS `cache`), and hardware interrupts require actual silicon or configured hardware emulators (e.g. QEMU).
2. **Code Coverage Measurement (`cargo-tarpaulin` / `llvm-cov`):** Tarpaulin is Linux-specific; native Windows LLVM source-based coverage requires additional LLVM toolchain configuration.
3. **Formal Mathematical Verification Tools (Kani, Creusote, Frama-C):** Formal proof claims in architectural documentation reflect intended design properties and target standards (DO-178C DAL-A) rather than machine-checked mathematical proof artifacts in this repository.
