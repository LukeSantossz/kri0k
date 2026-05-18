---
phase: "04"
plan: "05"
subsystem: kri0k-pybridge + python/kri0k.agent + docs
tags: [engagement, pyclass, act-node, sense-node, defense-in-depth, docs]
dependency_graph:
  requires:
    - "04-01 (NodeKind Domain/Organization/Nameserver + EdgeKind RegisteredBy/HasNameserver)"
    - "04-02 (Error variants + tokio-util/which/regex/tracing deps + NoopAuditSink rename)"
    - "04-03 (Ttp trait async, WhoisTtp, Subprocess abstraction)"
    - "04-04 (ScopeConfig::{from_dict_value, validate_target, compute_hash})"
  provides:
    - "kri0k._native.Engagement pyclass (5 methods exposed to Python)"
    - "kri0k.engagement.create() bootstrap helper (D-36)"
    - "kri0k.agent.nodes.act with propose_only gate + asyncio.to_thread dispatch"
    - "kri0k.agent.nodes.sense backward-compat (Engagement.snapshot fallback to _native.get_dummy_graph)"
  affects:
    - "Closes AGENT-05 + TTP-04; Phase 4 complete"
    - "Phase 5+ consumers build on Engagement facade (no add_node/add_edge exposed)"
tech_stack:
  added:
    - "tokio-util.workspace = true (CancellationToken in Engagement)"
    - "which.workspace = true (D-50 fail-fast in #[new])"
    - "regex.workspace = true (D-63 Layer 2 DOMAIN_REGEX)"
    - "tracing.workspace = true (#[instrument] on snapshot + execute_proposal)"
    - "temp-env = 0.3 (dev-dep for engagement_missing_whois Rust integration test)"
  patterns:
    - "Arc<EngagementInner> wrapper for cheap shared state (D-34)"
    - "Mutex<Box<dyn AuditSink>> for Sync requirement (Pitfall 5+12)"
    - "py.allow_threads wrapping #[new] body (Pitfall 7 — incl. constructor I/O)"
    - "Single dedupe HashMap<(kind_tag, natural_key), NodeId> with edge invariant (D-43)"
    - "DOMAIN_REGEX module-level const (D-63 Layer 2 defense in depth)"
    - "error_to_outcome / build_executed_outcome helpers for D-47 outcome shape"
key_files:
  created:
    - "python/kri0k/engagement.py (bootstrap helper, 39 LOC)"
    - "tests/test_act_node.py (8 unit tests with MockEngagement)"
    - "tests/test_engagement_smoke.py (8 integration tests, real whois)"
    - "crates/kri0k-pybridge/tests/engagement_missing_whois.rs (2 Rust integration tests)"
    - "CHANGELOG.md (0.2.0 entry)"
    - "docs/adr/ADR-0013-ttp-trait-subprocess-abstraction.md"
  modified:
    - "crates/kri0k-pybridge/Cargo.toml (4 workspace deps + temp-env dev-dep)"
    - "crates/kri0k-pybridge/src/lib.rs (635 LOC final; +547 from baseline)"
    - "python/kri0k/_native.pyi (Engagement class stubs added)"
    - "python/kri0k/agent/nodes/act.py (rewritten — propose_only gate + asyncio.to_thread + D-56 history)"
    - "python/kri0k/agent/nodes/sense.py (engagement.snapshot preferred; _native.get_dummy_graph fallback)"
    - "README.md (Running the whois TTP section)"
    - "CONTRIBUTING.md (Adding a new TTP section)"
    - ".planning/phases/04-act-node-ttp-whois/04-VALIDATION.md (frontmatter: nyquist_compliant=true, wave_0_complete=true)"
decisions:
  - "D-34 Engagement is canonical state container; D-35 facade minimal (5 methods, no graph mutators)"
  - "D-36 engagement_context dict with engagement/objective/propose_only/scope_hash"
  - "D-38 NoopAuditSink slot wired; audit.log_ttp_execution called per execute (Phase 8 will populate hash chain)"
  - "D-43 idempotent dedupe — edge added only if >=1 endpoint is new (Domain<->Org); NS edge only if NS is new"
  - "D-46 asyncio.to_thread bridges Python event loop to Rust block_on (avoids stalling LangGraph)"
  - "D-47 outcome dict: {status, result, error, graph_delta, audit_id} — 6 status values"
  - "D-49 propose_only default True; act node honors via gate (no Rust call in propose mode)"
  - "D-50 which::which('whois') fail-fast in #[new] — wrapped in py.allow_threads (Pitfall 7)"
  - "D-52 registry HashMap hardcoded for Phase 4 (single TTP T1590.001); inventory crate deferred"
  - "D-56 history entry shape: 7 keys (iteration/ttp_id/target/status/summary/graph_delta/audit_id)"
  - "D-62 CancellationToken stored in Engagement, exposed via kill() pymethod"
  - "D-63 defense in depth: L1 scope allowlist + L2 regex DOMAIN_REGEX + L3 Command::arg no shell"
  - "Pitfall 7 mitigation extends to #[new] constructor: ALL blocking init runs inside allow_threads"
  - "Plan typo correction: Error::ParseError uses `origin` field (not `source` as plan text wrote)"
metrics:
  duration: "~2h"
  completed: "2026-05-18"
  tasks_completed: 5
  files_created: 6
  files_modified: 9
  commits: 5
---

# Phase 4 Plan 05: Engagement pyclass + Act/Sense Wiring + Docs Summary

Fechamento end-to-end da Phase 4: `Engagement` pyclass expõe 5 métodos (`__new__`, `snapshot`, `execute_proposal`, `scope_hash`, `kill`) que orquestram grafo + scope + TTP registry + cancellation com defense-in-depth completo (D-63 L1+L2+L3). Python ganha `engagement.create()` bootstrap, `act.py` reescrito (propose_only gate D-49 + `asyncio.to_thread` D-46 + history D-56), `sense.py` modificado com backward-compat. Pacote de docs entregue (README quickstart, CONTRIBUTING TTP guide, CHANGELOG 0.2.0, ADR-0013).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Engagement pyclass + 8 Rust unit tests | `8fc8877` | `crates/kri0k-pybridge/{Cargo.toml,src/lib.rs}` |
| 2 | _native.pyi stubs + engagement.create() | `c281f77` | `python/kri0k/{_native.pyi,engagement.py}` |
| 3 | act.py rewrite + sense.py backward-compat | `81e5e05` | `python/kri0k/agent/nodes/{act,sense}.py` |
| 4 | 8 act unit + 8 smoke + 2 Rust integration tests | `7e7bd96` | `tests/{test_act_node,test_engagement_smoke}.py` + `crates/kri0k-pybridge/tests/engagement_missing_whois.rs` |
| 5 | README + CONTRIBUTING + CHANGELOG + ADR-0013 | `0b35479` | `README.md`, `CONTRIBUTING.md`, `CHANGELOG.md`, `docs/adr/ADR-0013-*.md` |

## Tests

### Rust (10 new)

| Suite | Tests | Notes |
|-------|-------|-------|
| `kri0k-pybridge` lib (inline `#[cfg(test)]`) | 8 | apply_whois_output (3) + error_to_outcome (4) + DOMAIN_REGEX (1) |
| `kri0k-pybridge` `tests/engagement_missing_whois.rs` | 2 | D-50/M-36 via `temp_env::with_var` PATH clear |

### Python (16 new)

| Suite | Tests | Marker | Notes |
|-------|-------|--------|-------|
| `tests/test_act_node.py` | 8 | `unit + asyncio` | MockEngagement; propose_only gate, asyncio.to_thread dispatch, D-56 history shape, RuntimeError on missing engagement |
| `tests/test_engagement_smoke.py` | 8 | `integration + ttp` | Real whois.exe; scope construct, hash, grow graph, idempotent (D-43), scope_violation, L2 regex, unknown_ttp |

### Verification Results

- `cargo test --workspace`: **65/65 pass** (36 core + 13 graph + 8 pybridge lib + 2 pybridge integration + 0 doc + 6 phase-2..3 inline)
- `cargo clippy --workspace --all-targets -- -D warnings`: **zero issues**
- `pytest -m "not integration"`: **104 pass / 15 deselected**
- `pytest -m integration` (whois real): **8/8 pass in 67s**
- `ruff check python/ tests/`: **all checks passed**
- `mypy python/kri0k --strict` + `mypy tests/ --strict`: **zero issues**
- `maturin develop`: **cdylib built and installed**

## Mitigation Coverage (D-61 table — final)

| Mitigation | Status | Evidence |
|---|---|---|
| M-02 (scope check pre-execution) | **Covered (Phase 4 partial = exact-match)** | `execute_proposal` calls `scope.validate_target` before TTP dispatch; `test_scope_violation_short_circuits` verifies |
| M-03 (scope hash) | **Covered** | `scope_hash()` exposes SHA-256 of raw YAML; `test_engagement_scope_hash_is_hex_64` |
| M-05 (propose-only default) | **Covered** | `engagement.create(propose_only=True)` default + `act.py` honors flag; `test_act_propose_only_skips_rust` |
| M-15 (LLM not direct executor) | **Covered** | Engagement facade exposes 5 methods only; no add_node/add_edge surfaces |
| M-21 (destructive TTP) | **N/A** | Whois is `RiskLevel::Safe` — no destructive gate needed in Phase 4 |
| M-22 (audit slot reserved) | **Covered (Phase 4 = no-op)** | `audit.log_ttp_execution()` called per execute; NoopAuditSink — hash chain lands Phase 8 |
| M-34 (TTP rate limit) | **Covered** | `WhoisTtp` enforces 1 req/sec via `Mutex<Option<Instant>>` (Plan 04-03) |
| M-36 (kill switch) | **Covered** | D-50 fail-fast `which::which` + D-62 `CancellationToken` + `Engagement.kill()`; `engagement_missing_whois.rs` |
| AB-03 (prompt injection out-of-scope) | **Fully covered (L1+L2+L3)** | D-63 defense in depth: scope allowlist + DOMAIN_REGEX + `Command::arg` no shell; `test_invalid_target_rejected_by_regex` + `test_domain_regex_accepts_valid_domains` |

## D-64 Definition of Done

1. **ROADMAP Phase 4 criteria 1-4** — ✅ All verifiable: WhoisTtp implements Ttp; WhoisOutput populated; Engagement grows graph with Domain/Org/NS; rate limit 1 req/sec.
2. **cargo clippy --workspace -- -D warnings**: ✅ zero issues.
3. **ruff + mypy --strict zero issues**: ✅.
4. **cargo test --workspace + pytest 100% pass**: ✅ (65 Rust + 104 Python unit + 8 integration).
5. **README + CONTRIBUTING + CHANGELOG + ADR-0013 delivered**: ✅ (commit `0b35479`).
6. **M-XX table in CONTEXT/VERIFICATION updated**: ✅ (this SUMMARY documents final status).
7. **End-to-end smoke `whois example.com` produces executed outcome + graph grows**: ✅ verified via `test_whois_grows_graph` integration.

## Files Modified

```
modified:   .planning/phases/04-act-node-ttp-whois/04-VALIDATION.md
modified:   CONTRIBUTING.md
modified:   README.md
modified:   crates/kri0k-pybridge/Cargo.toml
modified:   crates/kri0k-pybridge/src/lib.rs
modified:   python/kri0k/_native.pyi
modified:   python/kri0k/agent/nodes/act.py
modified:   python/kri0k/agent/nodes/sense.py
created:    CHANGELOG.md
created:    crates/kri0k-pybridge/tests/engagement_missing_whois.rs
created:    docs/adr/ADR-0013-ttp-trait-subprocess-abstraction.md
created:    python/kri0k/engagement.py
created:    tests/test_act_node.py
created:    tests/test_engagement_smoke.py
```

## Requisitos Completados

- **AGENT-05**: act sends proposal → Engagement.execute_proposal → outcome → D-56 history entry.
- **TTP-04**: graph receives Domain + Organization + Nameserver nodes with idempotent dedupe.

## Próximo passo

`/gsd-verify-work 4` para verificação final + PR para `master` per D-59.
