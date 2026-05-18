---
status: complete
phase: 04-act-node-ttp-whois
source:
  - 04-01-SUMMARY.md
  - 04-02-SUMMARY.md
  - 04-03-SUMMARY.md
  - 04-04-SUMMARY.md
  - 04-05-SUMMARY.md
started: 2026-05-18
updated: 2026-05-18
completed: 2026-05-18
mvp_mode: false
---

## Current Test

[testing complete]

## Tests

### 1. Engagement Construction & Empty Snapshot
expected: |
  Constructing `Engagement(scope_dict)` from a minimal v1 scope succeeds. Initial
  snapshot returns `{"nodes": [], "edges": []}`. Verified by
  `test_engagement_construct_with_minimal_scope`.
result: pass

### 2. Whois TTP Real Execution Grows Graph
expected: |
  `execute_proposal({"ttp_id": "T1590.001", "target": "example.com"})` returns
  outcome with `status="executed"` and `graph_delta.nodes_added >= 1`. Subsequent
  `snapshot()` exposes at least one `domain` node. Validates ROADMAP Phase 4
  criteria 1-3 + TTP-04. Verified by `test_whois_grows_graph` (integration,
  passed against Sysinternals whois.exe in 67s smoke run).
result: pass

### 3. Scope Violation Fail-Closed (M-02)
expected: |
  `execute_proposal({"ttp_id": "T1590.001", "target": "evil.com"})` against a scope
  whose allowlist contains only `example.com` returns `status="scope_violation"`
  and the error message mentions `evil.com`. Subprocess is NEVER invoked.
  Verified by `test_scope_violation_short_circuits`.
result: pass

### 4. Idempotency on Repeated Execution (D-43)
expected: |
  Calling `execute_proposal` twice in a row against the same target produces
  `graph_delta = {nodes_added: 0, edges_added: 0}` on the second call. No
  duplicate Domain/Org/NS nodes are added. Verified by
  `test_whois_idempotent_second_call_zero_delta`.
result: pass

### 5. Defense in Depth: D-63 Layer 2 Regex
expected: |
  `execute_proposal({"ttp_id": "T1590.001", "target": "invalid..domain"})`
  returns `status="error"` whose message contains "parse_error" (or "not a valid
  domain"). The regex check fires BEFORE scope.validate_target, blocking
  malformed targets even when an attacker would otherwise be allowlisted.
  Verified by `test_invalid_target_rejected_by_regex` +
  `test_domain_regex_accepts_valid_domains`.
result: pass

### 6. Kill-Switch / Missing Dependency (D-50, M-36)
expected: |
  When the `whois` binary is absent from `PATH`, `Engagement::new` fails fast with
  `RuntimeError: missing dependency: binary "whois" not found in PATH`. The
  message includes the install hint (winget / apt). Verified by
  `cargo test --package kri0k-pybridge --test engagement_missing_whois` (2/2 pass
  via `temp_env::with_var`).
result: pass

### 7. Propose-Only Gate Skips Rust (D-49, M-05)
expected: |
  When `engagement_context["propose_only"] = True` (the default from
  `engagement.create()`), the `act` node returns `decision.status="proposed"` and
  the history entry shows `"propose-only:"` in the summary. The engagement's
  `execute_proposal` is NEVER called. Verified by `test_act_propose_only_skips_rust`
  + `test_act_propose_only_appends_history_d56`.
result: pass

### 8. Sense Node Backward-Compat
expected: |
  When `engagement_context.engagement` is absent (legacy Phase 1/2 flow), the
  `sense` node falls back to `_native.get_dummy_graph()` without raising.
  Existing Phase 2 sense tests (`tests/test_sense_node.py`) keep passing. When an
  Engagement IS present, `sense` calls `engagement.snapshot()` instead. Verified
  by the 4/4 regression in `test_sense_node.py` plus the act/sense diff
  reviewed in `python/kri0k/agent/nodes/sense.py`.
result: pass

### 9. Documentation Pack Delivered (D-60)
expected: |
  - `README.md` contains the section `## Running the whois TTP` with winget/apt
    install hint and a Python end-to-end example using `kri0k.engagement.create`.
  - `CONTRIBUTING.md` contains `## Adding a new TTP` with the 5-step guide
    referencing `MockSubprocess::from_fixture`.
  - `CHANGELOG.md` exists with a `## [0.2.0]` entry listing Engagement, WhoisTtp,
    ScopeConfig, NoopAuditSink rename, D-63 L1+L2+L3.
  - `docs/adr/ADR-0013-ttp-trait-subprocess-abstraction.md` exists with the
    Decisão section referencing `async-trait`, `Subprocess` Real/Mock impls, and
    the 4 alternatives considered.
result: pass

### 10. Quality Gates Green Across Workspace
expected: |
  - `cargo test --workspace`: 65/65 pass (36 core + 13 graph + 8 pybridge lib + 2
    pybridge integration + others).
  - `cargo clippy --workspace --all-targets -- -D warnings`: zero issues.
  - `pytest -m "not integration"`: 104 pass / 15 deselected.
  - `pytest -m integration` (with whois): 8/8 pass in ~67s.
  - `ruff check python/ tests/` + `mypy python/kri0k --strict` +
    `mypy tests/ --strict`: zero issues.
  Verifies D-64 Definition of Done items 2-4.
result: pass

## Summary

total: 10
passed: 10
issues: 0
pending: 0
skipped: 0

## Gaps

[none yet]
