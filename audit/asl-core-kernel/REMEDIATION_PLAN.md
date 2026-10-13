# REMEDIATION PLAN: ASL CORE KERNEL
**Date:** 2026-10-13  
**Auditor Lead:** Principal Software Architect & Lead Security Auditor  

---

## 1. REMEDIATION ROADMAP OVERVIEW

Remediation is partitioned into four priority tiers:
* **P0: Immediate Blockers & Build Fixes** (Panic handler collision, integer overflow hardening).
* **P1: High-Priority Security & Concurrency Enhancements** (Checked arithmetic, atomic indices).
* **P2: Medium-Priority Quality & Verification Coverage** (Comprehensive unit test suite).
* **P3: Low-Priority & Long-Term Modernization** (Naming conventions, volatile register wrappers, strongly typed error enums).

---

## 2. DETAILED ACTIONABLE TASKS

### P0: Immediate Response & Critical Fixes

#### Task P0.1: Fix `asl-kernel-core` Test Compilation Breakage
* **Affected Component:** `c:\asl\asl-kernel\asl-kernel-core\src\lib.rs`
* **Proposed Change:** Annotate the bare-metal `panic_handler` with `#[cfg(not(test))]` to prevent duplicate symbol collisions during `cargo test`.
* **Validation Method:** Run `cargo test -p asl-kernel-core`.
* **Completion Criteria:** `cargo test` executes without `E0152 duplicate lang item` error.

#### Task P0.2: Harden Memory Boundary Check Against Integer Overflow (SEC-01)
* **Affected Component:** `c:\asl\asl-kernel\asl-security-vault\src\lib.rs`
* **Proposed Change:** Replace raw addition with `checked_add` in `MemoryWindow::contains`.
* **Validation Method:** Write targeted unit tests verifying that boundary queries with `u64::MAX` cleanly return `false` without panicking or overflowing.
* **Completion Criteria:** Mathematical overflow immunity verified.

---

### P1: High-Priority Security & Logic Hardening

#### Task P1.1: Physical Base & Size Overflow Validation in Handover (SEC-02)
* **Affected Component:** `c:\asl\asl-kernel\asl-bootloader\src\lib.rs`
* **Proposed Change:** In `AslBootHandover::validate()`, verify `self.kernel_physical_base.checked_add(self.kernel_size_bytes).is_some()`.
* **Validation Method:** Unit test passing extreme `u64::MAX` addresses.
* **Completion Criteria:** Return `Err("ERR_KERNEL_ADDRESS_OVERFLOW")` when overflow occurs.

#### Task P1.2: Endianness Standardization for Network Packets (SEC-05)
* **Affected Component:** `c:\asl\asl-kernel\asl-mips-fastpath\src\lib.rs`
* **Proposed Change:** Convert `normalize_packet_header_u32` to use explicit `u32::from_be` conversions for network wire format.
* **Validation Method:** Unit test on little-endian and big-endian representations.

---

### P2: Medium-Priority Test Coverage & Verification

#### Task P2.1: Establish Comprehensive Unit Test Suites
* **Affected Components:** All 25 crates in `c:\asl\asl-kernel`.
* **Proposed Change:** Implement unit test modules (`mod tests`) in each crate covering:
  - Bootloader handover validation.
  - Buddy allocator page allocation and boundary free limits.
  - Spatial vault partition access rights matrix.
  - Scheduler deterministic major/minor frame tick switching.
  - AFDX traffic shaping (BAG violation and packet rate limiting).
  - TMR majority voting error correction and memory scrubber.
* **Validation Method:** Execute `cargo test --workspace`.
* **Completion Criteria:** 100% test pass rate across all crates.

---

### P3: Low-Priority Quality & Style Improvements

#### Task P3.1: Clean Up Compiler Warnings & Naming Conventions
* **Affected Components:** `asl-power10-loongson`, `asl-ont-gpon`, `asl-do178c-cert`, `asl-fleet-regression`.
* **Proposed Change:** Rename enum variants to UpperCamelCase and remove unused imports.
* **Validation Method:** Run `cargo check --workspace` and verify 0 warnings.
* **Completion Criteria:** Zero warnings output.
