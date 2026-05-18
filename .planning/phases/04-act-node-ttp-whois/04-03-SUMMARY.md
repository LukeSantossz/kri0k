---
phase: 04-act-node-ttp-whois
plan: "03"
subsystem: kri0k-core::ttp
tags:
  - rust
  - async-trait
  - subprocess
  - whois
  - ttp
  - tokio
  - cancellation
  - rate-limit
dependency_graph:
  requires:
    - "04-02 (Error enum variants SubprocessTimeout, Cancelled, Io, ParseError)"
  provides:
    - "Async Ttp trait (Box<dyn Ttp> compatible via #[async_trait])"
    - "Subprocess trait + RealSubprocess + MockSubprocess"
    - "WhoisTtp (T1590.001) with rate limit, timeout, cancellation"
    - "parse_whois_output (heuristic key:value parser)"
    - "3 whois fixtures (google_com verbose, example_com redacted, invalid)"
  affects:
    - "04-04 (scope parser — no overlap, parallel plan)"
    - "04-05 (Engagement registry consumes Box<dyn Ttp> + WhoisTtp)"
tech_stack:
  added:
    - "async-trait = 0.1 (already in workspace from 04-02)"
    - "tokio features: process, time, sync, macros (already from 04-02)"
    - "tokio-util CancellationToken (already from 04-02)"
    - "tracing::instrument attribute on execute"
  patterns:
    - "tokio::select! biased; (cancel > timeout > output)"
    - "kill_on_drop(true) belt-and-suspenders (Pitfall 4)"
    - "std::sync::Mutex for non-async rate-limit lock (D-45)"
    - "String::from_utf8_lossy (Pitfall 3 — Windows locale)"
    - "u64::try_from(...).unwrap_or(u64::MAX) (cast_possible_truncation clippy)"
key_files:
  created:
    - "crates/kri0k-core/src/ttp/mod.rs (async Ttp trait + TtpOutput + RateLimits + RiskLevel)"
    - "crates/kri0k-core/src/ttp/subprocess.rs (Subprocess trait + RealSubprocess + MockSubprocess)"
    - "crates/kri0k-core/src/ttp/whois.rs (WhoisTtp + parse_whois_output + 8 tests)"
    - "crates/kri0k-core/tests/fixtures/whois_google_com.txt (real capture, verbose, Registrant Organization present)"
    - "crates/kri0k-core/tests/fixtures/whois_example_com.txt (real capture, redacted IANA)"
    - "crates/kri0k-core/tests/fixtures/whois_invalid.txt (synthetic malformed)"
    - "tests/fixtures/whois_google_com.txt (mirror for Python tests)"
    - "tests/fixtures/whois_example_com.txt (mirror)"
    - "tests/fixtures/whois_invalid.txt (mirror)"
  deleted:
    - "crates/kri0k-core/src/ttp.rs (promoted to ttp/mod.rs)"
  modified:
    - "crates/kri0k-core/src/ttp/mod.rs (updated trait signature to &'static str)"
decisions:
  - "D-44: async Ttp trait via #[async_trait] — required for Box<dyn Ttp> dyn-compat"
  - "D-45: TTP-local rate limit via std::sync::Mutex<Option<Instant>> — lock never held across await"
  - "D-51: default_timeout = 30s on trait; subprocess-level timeout via tokio::select!"
  - "D-54: Subprocess trait abstraction with RealSubprocess (tokio::process) + MockSubprocess (fixture/hanging)"
  - "D-62: CancellationToken propagated to subprocess; biased select! ensures cancel polled first"
  - "Pitfall 1+2+8: args literal &[\"-v\", \"-nobanner\", \"-accepteula\", target] — mandatory flags"
  - "Pitfall 3: String::from_utf8_lossy for Windows locale stderr safety"
  - "Pitfall 4: kill_on_drop(true) + RealOutcome enum to avoid child borrow-after-move"
  - "Pitfall 6: #[async_trait] mandatory — native async fn in trait not dyn-compatible in Rust 1.85"
  - "Example.com fixture required ISO-8859-1 -> UTF-8 conversion (Windows locale characters in connection error messages)"
metrics:
  duration: "~35 min"
  completed: "2026-05-18"
  tasks: 2
  files: 9
---

# Phase 4 Plan 03: TTP Module + WhoisTtp Implementation Summary

**One-liner:** Async `Ttp` trait via `#[async_trait]` + `WhoisTtp` (T1590.001) with `tokio::select! biased` cancellation, 1 req/sec `Mutex<Instant>` rate limit, heuristic key:value whois parser, and 3 captured fixtures — 26 tests pass including integration smoke.

## What Was Built

### Module Structure

`crates/kri0k-core/src/ttp.rs` was promoted to a module directory. The old file was deleted via `git rm` and replaced by three files:

| File | Role |
|------|------|
| `ttp/mod.rs` | Async `Ttp` trait + `TtpOutput` enum + `RateLimits` + `RiskLevel` |
| `ttp/subprocess.rs` | `Subprocess` trait + `RealSubprocess` (tokio) + `MockSubprocess` (fixture/hanging) |
| `ttp/whois.rs` | `WhoisTtp` struct + `impl Ttp` + `parse_whois_output` + inline tests |

`pub mod ttp;` in `lib.rs` was left unchanged — Rust resolves `ttp/mod.rs` automatically.

### Types Preserved (from old `ttp.rs`)

- `pub struct RateLimits { max_rps: Option<u32>, max_concurrent: Option<u32> }` (with `Default`)
- `pub enum RiskLevel { Safe, Low, Medium, High }`

### Types Deleted (per D-49 and plan spec)

- `ExecutionContext`, `ExecutionPlan`, `DryRunOutput`, `ExecutionResult`, `DestructiveTtp`
- Free fn `requires_human_gate` (gate moved to `act.py` Python side per D-49)
- Trait methods `propose`, `execute_dry_run`, sync `execute(&plan)` (trichotomy removed)

### Ttp Trait (D-44)

```rust
#[async_trait]
pub trait Ttp: Send + Sync {
    fn id(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn risk_level(&self) -> RiskLevel { RiskLevel::Safe }
    fn rate_limits(&self) -> RateLimits;
    fn default_timeout(&self) -> Duration { Duration::from_secs(30) }
    async fn execute(&self, target: &str, cancel: CancellationToken) -> Result<TtpOutput, Error>;
}
```

`#[async_trait]` is required because native `async fn in trait` is not dyn-compatible in Rust 1.85 (Pitfall 6). This enables `HashMap<String, Box<dyn Ttp>>` in Engagement (Plan 04-05).

### Subprocess Abstraction (D-54)

`RealSubprocess.run` uses `tokio::select! { biased; }` with three branches:
1. `() = cancel.cancelled()` → kill + `Error::Cancelled` (M-36, D-62)
2. `() = tokio::time::sleep(timeout)` → kill + `Error::SubprocessTimeout` (D-51)
3. `res = child.wait_with_output()` → decode stdout/stderr + `SubprocessOutput`

Technical note: `wait_with_output()` moves `child`, so branches 1 and 2 rely on `kill_on_drop(true)` for cleanup (the `RealOutcome` enum was used to avoid borrow-after-move — Pitfall 4).

### WhoisTtp (T1590.001)

- `id()` = `"T1590.001"`, `risk_level()` = `RiskLevel::Safe`
- `rate_limits()` = `{ max_rps: Some(1), max_concurrent: Some(1) }` (TTP-05)
- `default_timeout()` = `Duration::from_secs(30)` (D-51)
- Args: `&["-v", "-nobanner", "-accepteula", target]` — MANDATORY order (Pitfall 1+2+8)
- Rate limit: `std::sync::Mutex<Option<Instant>>` field; lock dropped before `await` (D-45)
- `last_call` updated BEFORE parse to ensure rate limit applies even on retry
- `#[tracing::instrument]` on `execute` (D-55)

### Parser (`parse_whois_output`)

Heuristic key:value scanner (D-41). Never panics. Fields:

| Key match (lowercase) | Maps to |
|---|---|
| `registrant organization` | `registrant` |
| `registrar` | `registrar` |
| `name server` / `nserver` | `nameservers` (lowercase, deduplicated) |
| `creation date` / `created` / `created on` | `created_at` |
| `updated date` / `last updated` / `modified` | `updated_at` |
| `registry expiry date` / `expiration date` / `expires on` / `registrar registration expiration date` | `expires_at` |
| anything else | `raw_unparsed` |

### Fixtures

| File | Source | Content |
|------|--------|---------|
| `whois_google_com.txt` | Real `whois -v -nobanner -accepteula google.com` | Verbose (MarkMonitor thick referral): `Registrant Organization: Google LLC`, 4 Name Servers, Creation/Updated/Expiry dates. 7434 bytes. |
| `whois_example_com.txt` | Real `whois -v -nobanner -accepteula example.com` | IANA registry only (GDPR-redacted): no Registrant Organization. Converted from ISO-8859-1 to UTF-8 (Windows connection error messages in Portuguese triggered encoding issue). 4068 bytes. |
| `whois_invalid.txt` | Hand-crafted (synthetic) | Malformed lines: `ERROR`, no colon, empty key, empty value. 64 bytes. |

Both `tests/fixtures/` and `crates/kri0k-core/tests/fixtures/` contain all 3 files (per Pitfall 13 — CARGO_MANIFEST_DIR resolution in Rust tests).

## Tests Added

### Subprocess tests (4)

| Test | Verifies |
|------|---------|
| `test_real_subprocess_command_not_found` | `Error::Io` when binary missing + `Box<dyn Subprocess>` dyn-compat smoke |
| `test_real_subprocess_cancel_before_spawn` | Pre-cancelled token returns `Error::Cancelled` or `Error::Io` |
| `test_mock_subprocess_from_fixture_returns_file_content` | Fixture content returned as stdout |
| `test_mock_subprocess_hanging_times_out` | Hanging mock returns `Error::SubprocessTimeout` in 50ms |

### WhoisTtp unit tests (7)

| Test | Requirement |
|------|-------------|
| `implements_trait` | TTP-01: `Box<dyn Ttp>` compiles; `id()` = `"T1590.001"` |
| `executes_via_mock_subprocess` | TTP-02: returns `TtpOutput::Whois(_)` via mock |
| `parses_google_fixture` | TTP-03: all 6 fields extracted from verbose fixture |
| `handles_redacted_output` | TTP-03: `registrant` is `None` for GDPR-redacted output |
| `parser_handles_invalid_input` | D-41: never panics; `raw_unparsed` populated |
| `rate_limit_enforced` | TTP-05: 2 calls take >= 950ms |
| `cancellation_returns_cancelled` | D-62: token.cancel() returns `Error::Cancelled` |
| `timeout_kills_child` | D-51 alias: subprocess-level timeout returns `Error::SubprocessTimeout` |

### Integration (1, feature-gated)

| Test | Requirement |
|------|-------------|
| `real_whois_smoke` | TTP-02: real whois binary, `example.com`, returns `TtpOutput::Whois(_)` |

**Total: 25 unit + 1 integration = 26 tests; all pass.**

## Requirements Covered

| ID | Status | Evidence |
|----|--------|----------|
| TTP-01 | Covered | `implements_trait` test; `Box<dyn Ttp>` compiles |
| TTP-02 | Covered | `executes_via_mock_subprocess` + `real_whois_smoke` integration |
| TTP-03 | Covered | `parses_google_fixture` + `handles_redacted_output` |
| TTP-05 | Covered | `rate_limit_enforced` >= 950ms elapsed |

## Pitfalls Blocked

| Pitfall | How |
|---------|-----|
| P1 (`-accepteula` hang) | `"-accepteula"` literal in args slice; required for EULA acceptance |
| P2 (`-v` mandatory for Registrant) | `"-v"` first in args slice; triggers thick referral to MarkMonitor |
| P3 (Windows locale leak) | `String::from_utf8_lossy` in `RealSubprocess` output decoding |
| P4 (OS error 6 zombie) | `kill_on_drop(true)` belt-and-suspenders; `RealOutcome` enum avoids borrow-after-move |
| P6 (async fn not dyn-compat) | `#[async_trait]` on both trait and impl |
| P8 (unknown flags) | Args are a literal slice constant — no dynamic flag injection |
| P9 (serde_yaml deprecated) | Not used in this plan; `serde_yaml_ng` from 04-02 is available |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `tokio::fs` not available (no `fs` feature in Cargo.toml)**
- **Found during:** Task 1 build
- **Issue:** `MockSubprocess::from_fixture` used `tokio::fs::read_to_string` but the `fs` feature was not enabled
- **Fix:** Switched to `std::fs::read_to_string` (sync; acceptable for test-only mock with small fixtures)
- **Files modified:** `crates/kri0k-core/src/ttp/subprocess.rs`

**2. [Rule 1 - Bug] `wait_with_output()` moves `child`, causing borrow-after-move in `tokio::select!`**
- **Found during:** Task 1 build
- **Issue:** The `cancel` and `timeout` branches called `child.start_kill()` after `child` was moved by the `wait_with_output()` arm
- **Fix:** Introduced `RealOutcome` enum to carry the select result; `kill_on_drop(true)` handles cleanup for cancel/timeout branches. Child is not touched post-select in those arms.
- **Files modified:** `crates/kri0k-core/src/ttp/subprocess.rs`

**3. [Rule 1 - Bug] `whois_example_com.txt` fixture captured in ISO-8859-1 encoding**
- **Found during:** Task 2 test run (`handles_redacted_output` panicked on invalid UTF-8)
- **Issue:** Windows connection error messages in Portuguese ("`Uma tentativa de conexão falhou...`") were encoded in ISO-8859-1 by `whois.exe`
- **Fix:** Converted fixture to UTF-8 via `iconv -f ISO-8859-1 -t UTF-8`; updated both fixture copies
- **Files modified:** `crates/kri0k-core/tests/fixtures/whois_example_com.txt`, `tests/fixtures/whois_example_com.txt`

**4. [Rule 1 - Bug] Multiple clippy strict violations**
- **Found during:** Task 1 and Task 2 clippy runs
- **Issues:** `unnested_or_patterns`, `ignored_unit_patterns` (`_ = ` → `() = `), `must_use_candidate`, `missing_const_for_fn`, `unnecessary_literal_bound` (`&str` → `&'static str`), `used_underscore_binding`, `doc_markdown`
- **Fix:** Applied all clippy suggestions inline
- **Files modified:** `subprocess.rs`, `whois.rs`, `mod.rs`

## Known Stubs

None — all fields in `WhoisOutput` are properly wired to the parser. The fixture assertions verify end-to-end data flow.

## Threat Flags

No new threat surface beyond what was declared in the plan's `<threat_model>`. All mitigations from T-04-03-01 through T-04-03-06 are implemented and tested.

## Self-Check: PASSED

```
FOUND: crates/kri0k-core/src/ttp/mod.rs
FOUND: crates/kri0k-core/src/ttp/subprocess.rs
FOUND: crates/kri0k-core/src/ttp/whois.rs
FOUND: crates/kri0k-core/tests/fixtures/whois_google_com.txt
FOUND: crates/kri0k-core/tests/fixtures/whois_example_com.txt
FOUND: crates/kri0k-core/tests/fixtures/whois_invalid.txt
FOUND: tests/fixtures/whois_google_com.txt
FOUND: tests/fixtures/whois_example_com.txt
FOUND: tests/fixtures/whois_invalid.txt
MISSING: crates/kri0k-core/src/ttp.rs (expected — deleted by plan)
FOUND: commit 60da97b (refactor: ttp module promotion)
FOUND: commit 51d428e (feat: WhoisTtp implementation)
TESTS: 26 passed, 0 failed (including integration smoke)
CLIPPY: workspace --exclude kri0k-pybridge — 0 errors, 0 warnings
```
