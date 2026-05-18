---
phase: "04"
plan: "02"
subsystem: kri0k-core
tags: [rust, cargo, deps, error-handling, audit, tdd]
dependency_graph:
  requires: [04-01]
  provides: [Error enum 10 variants, NoopAuditSink, workspace deps tokio-util/which/async-trait/tracing/serde_yaml_ng/regex]
  affects: [04-03, 04-04, 04-05]
tech_stack:
  added:
    - tokio-util 0.7.18 (CancellationToken — D-62)
    - which 8.0.2 (binary detection — D-50)
    - async-trait 0.1.89 (dyn-compat async trait — D-44)
    - tracing 0.1.44 (instrumentation — D-55)
    - serde_yaml_ng 0.10.0 (YAML parser — D-58; NOT serde_yaml/serde_yml)
    - regex 1.x (D-63 Layer 2 domain validation — consumed by Plan 04-05)
    - sha2 0.10.9 (SHA-256 for ScopeConfig hash — D-58/Plan 04-04)
  patterns:
    - thiserror Error enum with structured named-field variants
    - TDD RED/GREEN cycle (compile-error RED confirmed before implementation)
    - Clippy strict compliance in test code (assert_sync helper hoisted out of test fn)
key_files:
  created: []
  modified:
    - Cargo.toml
    - crates/kri0k-core/Cargo.toml
    - crates/kri0k-core/src/lib.rs
    - crates/kri0k-core/src/audit.rs
decisions:
  - "ParseError field renamed from 'source' to 'origin' because thiserror treats fields named 'source' as error source chains (StdError bound), not display strings. Display format is 'parse error in {origin}: {detail}'. Semantically equivalent; consumers in Plan 04-04 must use the field name 'origin'."
  - "assert_sync helper function hoisted to module level in audit test block to satisfy clippy::items_after_statements — no behavioral change."
  - "NoopAuditSink::default() calls removed in test code in favor of plain NoopAuditSink (unit struct) to satisfy clippy::default_constructed_unit_structs."
metrics:
  duration: "~10 min"
  completed: "2026-05-18"
---

# Phase 4 Plan 02: Error Taxonomy + Cargo Deps + NoopAuditSink Rename Summary

**One-liner:** Expanded kri0k-core Error enum from 2 to 10 variants (D-53/D-62), added 7 workspace deps (tokio-util/which/async-trait/tracing/serde_yaml_ng/regex/sha2), and renamed NoOpAuditSink to NoopAuditSink (D-38) via TDD RED/GREEN cycle with clippy strict green.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Adicionar deps no workspace Cargo.toml e no kri0k-core/Cargo.toml | 994bcb5 | Cargo.toml, crates/kri0k-core/Cargo.toml, Cargo.lock |
| 2 (RED) | Testes de Error enum + NoopAuditSink (falham antes da impl) | 14f97b9 | crates/kri0k-core/src/lib.rs |
| 2 (GREEN) | Expandir Error enum + renomear NoOpAuditSink -> NoopAuditSink | 51daf22 | crates/kri0k-core/src/lib.rs, crates/kri0k-core/src/audit.rs |

## Dependencies Added

### Workspace (`Cargo.toml`)

| Crate | Version | Purpose |
|-------|---------|---------|
| tokio-util | 0.7 (features: rt) | CancellationToken para D-62 kill switch |
| which | 8 | Fail-fast binary detection D-50 |
| async-trait | 0.1 | dyn-compatible async Ttp trait D-44 |
| tracing | 0.1 | #[instrument] instrumentation D-55 |
| serde_yaml_ng | 0.10 | YAML parser (NOT serde_yaml / serde_yml) D-58 |
| regex | 1 | Layer 2 domain validation D-63 (consumed by Plan 04-05) |

### kri0k-core (`crates/kri0k-core/Cargo.toml`)

- All workspace deps above inherited via `.workspace = true`
- `sha2 = "0.10"` — SHA-256 for ScopeConfig::compute_hash (D-58, used by Plan 04-04)
- `tokio` with features: `process`, `time`, `sync`, `macros`
- Feature flag `integration = []` for gating tests requiring real `whois` binary (D-54, D-57)
- Dev-deps: `tokio` (macros, rt-multi-thread) + `tracing-subscriber = "0.3"`

## Error Enum Expansion

Before: 2 variants (`Json`, `Generic`)  
After: **10 variants** (2 existing + 8 new)

| Variant | Display | Purpose |
|---------|---------|---------|
| `Json(#[from] serde_json::Error)` | "JSON error: {0}" | existing |
| `Generic(String)` | "{0}" | existing |
| `Io(#[from] std::io::Error)` | "I/O error: {0}" | OS I/O errors D-53 |
| `ScopeViolation { target, reason }` | "scope violation for target {target:?}: {reason}" | M-02 |
| `RateLimitExceeded { ttp_id, retry_in_ms }` | "rate limit exceeded for TTP {ttp_id}: retry in {retry_in_ms}ms" | M-34 |
| `SubprocessTimeout { ttp_id, timeout_ms }` | "TTP {ttp_id} subprocess timeout after {timeout_ms}ms" | M-34, D-51 |
| `ParseError { origin, detail }` | "parse error in {origin}: {detail}" | scope.yaml + whois parse |
| `MissingDependency { binary }` | "missing dependency: binary {binary:?} not found in PATH. Install with: ..." | D-50 |
| `UnknownTtp { ttp_id }` | "unknown TTP id: {ttp_id:?}" | D-52 |
| `Cancelled` | "operation cancelled" | D-62, M-36 kill switch |

## NoopAuditSink Rename (D-38)

3 call sites updated in `crates/kri0k-core/src/audit.rs`:
- Line ~87: `pub struct NoOpAuditSink` → `pub struct NoopAuditSink`
- Line ~89: `impl AuditSink for NoOpAuditSink` → `impl AuditSink for NoopAuditSink`
- Line ~118: `Ok(Box::new(NoOpAuditSink))` → `Ok(Box::new(NoopAuditSink))`

Verified: `grep -r "NoOpAuditSink" crates/ --include="*.rs" | wc -l` == 0

## Test Coverage

**Before plan:** 8 tests (kri0k-core: 3 NodeId/EdgeId + 2 safeguards + 1 scope + 2 from other existing)  
**After plan:** **13 tests** (5 new Error tests + 2 new AuditSink tests)

New tests:
- `tests::test_error_io_from_conversion` — `std::io::Error` converts to `Error::Io(_)`
- `tests::test_error_scope_violation_display` — Display contains "evil.com"
- `tests::test_error_missing_dependency_display` — Display contains "whois" and "PATH"
- `tests::test_error_cancelled_display` — exact string "operation cancelled"
- `tests::test_error_json_still_works` — regression: Json variant still works
- `audit::tests::test_noop_audit_sink_is_boxable` — `Box<dyn AuditSink>` type-erases correctly
- `audit::tests::test_noop_audit_sink_is_mutex_boxable_sync` — compile-time: `Mutex<Box<dyn AuditSink + Send>>: Sync` (PyO3 Pitfall 5+12)

## Verification Results

- `cargo check --workspace --exclude kri0k-pybridge`: **PASS**
- `cargo check --package kri0k-core --features integration`: **PASS**
- `cargo test --package kri0k-core`: **PASS** (13/13)
- `cargo clippy --package kri0k-core --all-targets -- -D warnings`: **PASS**
- `grep -r "NoOpAuditSink" crates/ --include "*.rs" | wc -l` == 0: **PASS**
- `grep -E "^serde_yaml = " Cargo.toml | wc -l` == 0: **PASS**
- `grep -c "serde_yml" Cargo.toml` == 0: **PASS**

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `thiserror` field-named-source conflict in ParseError variant**
- **Found during:** Task 2 GREEN phase (cargo compile error)
- **Issue:** The plan specified `ParseError { source: String, detail: String }` with display `"parse error in {source}: {detail}"`. The `thiserror` crate treats any field named `source` as an `std::error::Error` chain source (requires `StdError` bound on `String`), causing a compile error.
- **Fix:** Renamed field from `source` to `origin`. Display string is now `"parse error in {origin}: {detail}"`. Semantically identical — consumers in Plans 04-04/04-05 must use field name `origin` when constructing `Error::ParseError { origin: "scope.yaml".into(), detail: "...".into() }`.
- **Files modified:** `crates/kri0k-core/src/lib.rs`
- **Commit:** 51daf22

**2. [Rule 1 - Bug] Clippy strict violations in test code**
- **Found during:** Task 2 clippy verification
- **Issues (4 total):**
  1. `NoopAuditSink::default()` on unit struct → use `NoopAuditSink` directly (clippy::default_constructed_unit_structs)
  2. `fn assert_sync<T: Sync>()` declared inside test fn → hoist to module level (clippy::items_after_statements)
  3. `unwrap_err()` → use `expect_err("reason")` (clippy::unwrap_used)
- **Fix:** Applied all 3 clippy suggestions; tests remain semantically identical.
- **Files modified:** `crates/kri0k-core/src/audit.rs`, `crates/kri0k-core/src/lib.rs`
- **Commit:** 51daf22

## Threat Surface Scan

No new network endpoints, auth paths, or file access patterns introduced. This plan is pure type declarations and dependency additions. Supply chain threat T-04-02-01 (serde_yaml deprecated) mitigated — only `serde_yaml_ng = "0.10"` in Cargo.toml.

## Requirement Coverage

- **TTP-01** (partial pre-requisite): Error vocabulary for Phase 4 complete. Plan 04-03 consumes `ScopeViolation`, `SubprocessTimeout`, `Cancelled`, `Io`, `RateLimitExceeded`, `ParseError`, `MissingDependency`, `UnknownTtp`.

## Self-Check: PASSED

| Item | Status |
|------|--------|
| `crates/kri0k-core/src/lib.rs` exists | FOUND |
| `crates/kri0k-core/src/audit.rs` exists | FOUND |
| `Cargo.toml` exists | FOUND |
| `crates/kri0k-core/Cargo.toml` exists | FOUND |
| Commit 994bcb5 (deps) | FOUND |
| Commit 14f97b9 (RED tests) | FOUND |
| Commit 51daf22 (GREEN implementation) | FOUND |
