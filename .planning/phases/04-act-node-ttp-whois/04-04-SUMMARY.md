---
phase: "04"
plan: "04"
subsystem: kri0k-core/scope
tags: [scope, allowlist, sha256, serde_yaml_ng, safeguards, security]
dependency_graph:
  requires:
    - "04-02 (Error::ParseError origin field, serde_yaml_ng dep, sha2 dep)"
  provides:
    - "ScopeConfig::from_yaml — used by Engagement::new (04-05)"
    - "ScopeConfig::from_dict_value — used by Engagement::new via Python dict (04-05)"
    - "ScopeConfig::validate_target — used by execute_proposal pre-subprocess gate (04-05, M-02)"
    - "ScopeConfig::compute_hash — scope_hash embedded in snapshots (04-05, M-03)"
  affects:
    - "04-05 (Engagement pyclass depends on all four ScopeConfig methods)"
tech_stack:
  added:
    - "sha2 = 0.10 (already in kri0k-core/Cargo.toml from 04-02)"
    - "serde_yaml_ng = 0.10 (workspace dep, used for from_yaml + from_dict_value raw_yaml)"
  patterns:
    - "Manual impl Default for SafeguardsSection (Pitfall 11 mitigation — propose_only=true)"
    - "serde(default = 'default_true') per-field override for propose_only"
    - "serde(skip) on raw_yaml to exclude from serialization while retaining for hash"
    - "ScopeConfig::from_dict_value serializes JSON to YAML before hashing (M-03 consistency)"
key_files:
  created: []
  modified:
    - "crates/kri0k-core/src/scope.rs (stub replaced, 397 LOC final)"
    - "config/scope.example.yaml (v1 schema, all lookahead fields documented)"
decisions:
  - "D-48 exact-match implemented as targets.iter().any(|t| t == target) — NEVER contains/starts_with (T-04-04-04 mitigated)"
  - "D-49 SafeguardsSection::default() returns propose_only=true via manual impl (Pitfall 11 mitigated, T-04-04-02)"
  - "D-58 lookahead schema v1: targets_cidr + targets_wildcards parsed but unused until Phase 7"
  - "raw_yaml field (serde skip) stores original bytes for deterministic SHA-256 hash (M-03)"
  - "from_dict_value serializes JSON to YAML via serde_yaml_ng::to_string for hash consistency"
  - "Backwards-compat free fn validate_target(scope, target) retained for old API references"
  - "Legacy Scope struct retained with #[deprecated] annotation — new consumers use ScopeConfig"
metrics:
  duration: "~10 min"
  completed: "2026-05-18T14:43:32Z"
  tasks_completed: 2
  files_modified: 2
---

# Phase 4 Plan 04: ScopeConfig v1 Parser + Allowlist + Hash Summary

ScopeConfig full lookahead schema v1 parser with SHA-256 hash and exact-match allowlist, replacing the `todo!()` stub; SafeguardsSection manual Default forces `propose_only=true` (Pitfall 11 mitigation).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Rewrite scope.rs with ScopeConfig + SafeguardsSection + parser + validator + hash | f34a2e6 | crates/kri0k-core/src/scope.rs |
| 2 | Update config/scope.example.yaml to v1 schema | 86dabc3 | config/scope.example.yaml |

## Tests (13 passing)

All tests in `scope::tests` module:

| Test | Covers |
|------|--------|
| `parses_full_v1_schema` | D-58 full YAML with all fields including Phase 7/8 stubs |
| `unknown_version_rejected` | D-58 version gate — `version: 2` → ParseError "unsupported" |
| `missing_version_rejected` | serde missing field → ParseError |
| `validate_target_accepts_exact_match` | D-48 happy path — "example.com" in targets |
| `validate_target_rejects_non_match` | D-48 reject — "evil.com" → ScopeViolation |
| `validate_target_substring_not_accepted` | T-04-04-04 — "xexample.com", "example.com.evil", "" all rejected |
| `compute_hash_deterministic` | M-03 — same YAML → same 64-char hex; valid hex digits |
| `compute_hash_differs_for_different_yaml` | M-03 — different targets → different hash |
| `default_safeguards_is_propose_only_true` | Pitfall 11 / D-49 / T-04-04-02 — missing safeguards block → propose_only=true |
| `from_dict_value_parses_json` | from_dict_value: JSON → ScopeConfig, propose_only=true default, raw_yaml populated |
| `from_dict_value_rejects_version_2` | from_dict_value: version gate identical to from_yaml |
| `parses_committed_example_yaml` | scope.example.yaml loads, version=1, targets includes example.com, propose_only=true |
| `test_error_scope_violation_display` | (pre-existing in lib.rs tests, still green) |

## Implementation Details

### ScopeConfig (397 LOC)

Structs implemented:
- `ScopeConfig` — full v1 schema with `#[serde(default)]` stubs for Phase 7/8 fields
- `SafeguardsSection` — **manual `impl Default`** (`propose_only=true`) to avoid Pitfall 11
- `RateLimitsSection` — `#[derive(Default)]` is safe (only Option<u32> fields)
- `default_true() -> bool` const fn for `#[serde(default = "default_true")]`

Methods:
- `ScopeConfig::from_yaml(path: &Path) -> Result<Self>` — reads file, parses via `serde_yaml_ng::from_str`, validates version==1, stores raw bytes
- `ScopeConfig::from_dict_value(value: serde_json::Value) -> Result<Self>` — serializes JSON to YAML first (hash consistency), then `serde_json::from_value`, same version gate
- `ScopeConfig::compute_hash(&self) -> String` — SHA-256 of `raw_yaml`, hex-encoded (64 chars)
- `ScopeConfig::validate_target(&self, target: &str) -> Result<()>` — exact-match via `iter().any(|t| t == target)`

### Pitfall 11 Mitigation (T-04-04-02, D-49)

`impl Default for SafeguardsSection` is written manually:
```rust
impl Default for SafeguardsSection {
    fn default() -> Self { Self { propose_only: true, kill_switch: false } }
}
```
`#[derive(Default)]` would set `propose_only = false` because `bool::default() == false`, which would silently allow TTP execution when the YAML omits the `safeguards:` block. Test `default_safeguards_is_propose_only_true` guards this.

### Pitfall 9 Mitigation (T-04-04-03, serde_yaml_ng)

Only `serde_yaml_ng::from_str` and `serde_yaml_ng::to_string` are used. No import of `serde_yaml` or `serde_yml`.

### Threat Mitigations Covered

| Mitigation | Status | Evidence |
|-----------|--------|----------|
| M-02 (D-48 exact-match allowlist) | Partial — Phase 4 only | `validate_target`: `iter().any(|t| t == target)`; Phase 7 adds CIDR+wildcards |
| M-03 (SHA-256 scope hash) | Complete | `compute_hash()`: SHA-256 of `raw_yaml`; stable across runs |
| M-15 / Pitfall 11 (propose_only=true default) | Complete | Manual `impl Default for SafeguardsSection`; test asserts |
| T-04-04-04 (substring bypass) | Complete | `iter().any(|t| t == target)` — never `contains`/`starts_with` |

### config/scope.example.yaml

Rewritten to v1 schema:
- `version: 1` (mandatory)
- `targets: [example.com]` — domain string, not CIDR
- `targets_cidr: []` + `targets_wildcards: []` — Phase 7 lookahead stubs
- `safeguards.propose_only: true` — ADR-0006 secure default
- `rate_limits.global_rps: null` — Phase 7/8 stub
- `audit_path: null` — Phase 8 stub
- `metadata` + `llm` blocks — parsed but ignored by serde (no `deny_unknown_fields`)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Clippy `use_self` in `impl ScopeConfig`**
- Found during: Task 1 verification (cargo clippy)
- Issue: `let mut config: ScopeConfig = ...` inside `impl ScopeConfig` triggered `use_self` lint (workspace `-D warnings`)
- Fix: replaced with `let mut config: Self = ...` in both `from_yaml` and `from_dict_value`
- Files modified: crates/kri0k-core/src/scope.rs
- Commit: f34a2e6 (fixed before commit)

**2. [Rule 1 - Bug] Clippy `needless_raw_string_hashes` in tests**
- Found during: Task 1 verification
- Issue: `r#"..."#` without embedded `#` characters in test constants is flagged
- Fix: changed `r#"..."#` to plain string literals with escaped quotes
- Files modified: crates/kri0k-core/src/scope.rs
- Commit: f34a2e6 (fixed before commit)

**3. [Rule 2 - Missing Critical] `#[allow(clippy::panic)]` in test module**
- Found during: Task 1 verification
- Issue: workspace denies `panic!` macro; test match arms used `panic!()` for exhaustive match reporting
- Fix: added `#[allow(clippy::panic)]` alongside existing `#[allow(clippy::expect_used)]` on the test module
- Files modified: crates/kri0k-core/src/scope.rs
- Commit: f34a2e6 (fixed before commit)

## Known Stubs

The following fields in `ScopeConfig` are parsed but not consumed in Phase 4:
- `targets_cidr: Vec<String>` — Phase 7 CIDR matching
- `targets_wildcards: Vec<String>` — Phase 7 wildcard matching
- `rate_limits: RateLimitsSection` — Phase 7/8 throttling
- `audit_path: Option<String>` — Phase 8 audit log path

These are intentional lookahead stubs per D-58. Plan 04-05 (Engagement) will consume `validate_target` and `compute_hash`. Phase 7 will extend `validate_target` with CIDR + wildcard logic.

## Self-Check: PASSED

| Item | Status |
|------|--------|
| crates/kri0k-core/src/scope.rs | FOUND |
| config/scope.example.yaml | FOUND |
| Commit f34a2e6 | FOUND |
| Commit 86dabc3 | FOUND |
| 13 scope tests pass | VERIFIED |
| clippy strict green | VERIFIED |
| No todo!() in scope.rs | VERIFIED |
| No serde_yaml (only serde_yaml_ng) | VERIFIED |
