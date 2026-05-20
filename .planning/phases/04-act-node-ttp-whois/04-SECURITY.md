---
phase: 4
phase_name: Act Node + TTP Whois
audit_date: 2026-05-18
auditor: gsd-security-auditor
register_authored_at_plan_time: true
threats_total: 24
threats_closed: 24
threats_open: 0
asvs_level: not_configured
block_on: any_open_critical_or_high
verdict: SECURED
---

# Phase 4 — Security Audit

Verification of every threat declared in the `<threat_model>` blocks of the five Phase 4 PLAN.md files. Mode: **verify mitigations exist** (register authored at plan time; auditor does not scan for new threats).

Implementation files were NOT modified during this audit.

## Threat Register (24 / 24 closed)

### Plan 04-01 — Graph data model (1 / 1)

| ID | Category | Disposition | Status | Evidence |
|---|---|---|---|---|
| T-04-01-01 | Tampering (JSON ambiguity) | accept | CLOSED | `crates/kri0k-graph/src/lib.rs:9-10` — `#[serde(tag = "type", rename_all = "snake_case")]` on `NodeKind` + tests `test_node_kind_*_serialization` lines 226-271. Tagged union prevents ambiguous parse. Rationale holds: no cross-language input at this layer in Phase 4. |

### Plan 04-02 — Error taxonomy + deps (2 / 2)

| ID | Category | Disposition | Status | Evidence |
|---|---|---|---|---|
| T-04-02-01 | Tampering / Supply Chain (YAML parser) | mitigate | CLOSED | `Cargo.toml:27` — `serde_yaml_ng = "0.10"` (active, no advisories). Workspace grep confirms zero references to deprecated `serde_yaml` or `serde_yml` (RUSTSEC-2025-0068). |
| T-04-02-02 | Information Disclosure (error leakage) | accept | CLOSED | `crates/kri0k-core/src/lib.rs:17-88` — `Error` enum uses typed controlled fields (`target`, `ttp_id`, `binary`, `timeout_ms`). No auto stack traces; no panics surface to logs. Rationale: operator-controlled inputs only. |

### Plan 04-03 — WhoisTtp + Subprocess (6 / 6)

| ID | Category | Disposition | Status | Evidence |
|---|---|---|---|---|
| T-04-03-01 | Tampering (shell injection) | mitigate | CLOSED | `crates/kri0k-core/src/ttp/whois.rs:148-156` — args passed as literal slice `&["-v", "-nobanner", "-accepteula", target]`; `crates/kri0k-core/src/ttp/subprocess.rs:91-97` — `tokio::process::Command::new(cmd).args(args)` (no shell). |
| T-04-03-02 | Denial of Service (subprocess hang) | mitigate | CLOSED | `crates/kri0k-core/src/ttp/subprocess.rs:99-111` — `tokio::select! { biased; cancel / sleep(timeout) / wait_with_output }`; `whois.rs:110-112` 30s default; `subprocess.rs:95` `kill_on_drop(true)`. Test `test_mock_subprocess_hanging_times_out`. |
| T-04-03-03 | Denial of Service (TTP flood) | mitigate | CLOSED | `crates/kri0k-core/src/ttp/whois.rs:61` `last_call: Mutex<Option<Instant>>`; `whois.rs:134-143` lock + `tokio::time::sleep(wait)` before subprocess. Test `rate_limit_enforced` `whois.rs:376-388` asserts >=950ms between two calls. |
| T-04-03-04 | DoS / Repudiation (kill switch) | mitigate | CLOSED | `crates/kri0k-core/src/ttp/subprocess.rs:99-104` — `biased` select polls `cancel.cancelled()` first; `whois.rs:154` propagates `cancel.clone()`; `kill_on_drop(true)` ensures process termination. Test `cancellation_returns_cancelled` `whois.rs:392-404`. |
| T-04-03-05 | Tampering / Parsing (malformed output) | accept | CLOSED | `crates/kri0k-core/src/ttp/whois.rs:186-245` — `parse_whois_output` is total: empty/missing values + unrecognised lines → `raw_unparsed`; no `unwrap`/`expect`. Test `parser_handles_invalid_input` covers the failure shape. |
| T-04-03-06 | Information Disclosure (locale leak) | accept | CLOSED | `crates/kri0k-core/src/ttp/subprocess.rs:129-130` — `String::from_utf8_lossy` for both stdout/stderr; never panics on non-UTF-8 (e.g., ISO-8859-1 Windows locale OS errors). Rationale: raw_unparsed may contain OS error text, no PII. |

### Plan 04-04 — ScopeConfig (5 / 5)

| ID | Category | Disposition | Status | Evidence |
|---|---|---|---|---|
| T-04-04-01 | Tampering (scope.yaml edited mid-engagement) | mitigate (partial) | CLOSED | `crates/kri0k-core/src/scope.rs:155-159` `compute_hash()` SHA-256 of `raw_yaml`; `scope.rs:42-45` `raw_yaml` field populated in both `from_yaml` (line 116) and `from_dict_value` (line 146); `crates/kri0k-pybridge/src/lib.rs:416-418` `scope_hash()` exposes digest. Full disk re-check deferred to Phase 7 as planned. |
| T-04-04-02 | EoP (default propose_only=false silent) | mitigate | CLOSED | `crates/kri0k-core/src/scope.rs:65-73` — manual `impl Default for SafeguardsSection { propose_only: true, kill_switch: false }`; `scope.rs:58` `#[serde(default = "default_true")]`. Test `default_safeguards_is_propose_only_true` `scope.rs:341-353`. |
| T-04-04-03 | Supply chain (vulnerable YAML parser) | mitigate | CLOSED | Same root as T-04-02-01: `Cargo.toml:27` `serde_yaml_ng = "0.10"`. |
| T-04-04-04 | Tampering (substring/prefix bypass) | mitigate | CLOSED | `crates/kri0k-core/src/scope.rs:171` — `self.targets.iter().any(\|t\| t == target)` strict equality. NEVER uses `contains`/`starts_with`/`ends_with`. Test `validate_target_substring_not_accepted` `scope.rs:303-319` asserts `xexample.com`, `example.com.evil`, and empty string are all rejected. |
| T-04-04-05 | Information Disclosure (YAML detail in errors) | accept | CLOSED | `crates/kri0k-core/src/scope.rs:101-106, 138-144` — `Error::ParseError { origin, detail }` may include serde snippets; rationale: file is operator-controlled, no PII enforcement required at this layer. |

### Plan 04-05 — Engagement + wiring (10 / 10)

| ID | Category | Disposition | Status | Evidence |
|---|---|---|---|---|
| T-04-05-01 | EoP (LLM fires TTP without approval) | mitigate | CLOSED | `python/kri0k/engagement.py:15` `propose_only: bool = True` default; `python/kri0k/agent/nodes/act.py:44-62` early-return with `status: "proposed"` — never reaches `engagement.execute_proposal`. |
| T-04-05-02 | Information Disclosure (out-of-scope target) | mitigate | CLOSED | `crates/kri0k-pybridge/src/lib.rs:365` — `inner.scope.validate_target(&target)?` runs BEFORE registry lookup (line 368) and BEFORE subprocess (`ttp.execute` line 376). |
| T-04-05-03 | EoP (bypass facade via add_node/add_edge) | mitigate | CLOSED | `crates/kri0k-pybridge/src/lib.rs:254-427` — `#[pymethods] impl Engagement` exposes exactly five methods: `new` (263), `snapshot` (305), `execute_proposal` (330), `scope_hash` (416), `kill` (424). No `add_node`/`add_edge` exported to Python. |
| T-04-05-04 | DoS (operator loses control) | mitigate | CLOSED | `crates/kri0k-pybridge/src/lib.rs:274-276` `which::which("whois")` inside `py.allow_threads` block starting line 272 (Pitfall 7); `lib.rs:424-426` `kill()` signals `CancellationToken`; integration test `crates/kri0k-pybridge/tests/engagement_missing_whois.rs:22-31` asserts the missing-binary path via `temp_env::with_var`. |
| T-04-05-05 | Tampering (mid-engagement edit) | mitigate | CLOSED | `crates/kri0k-pybridge/src/lib.rs:416-418` `scope_hash()` exposed via `compute_hash()`. Phase 5+ scheduled to re-check vs disk. Matches plan disposition. |
| T-04-05-06 | Prompt injection / shell metachar (AB-03, D-63) | mitigate | CLOSED | **Layer 2** regex: `crates/kri0k-pybridge/src/lib.rs:30-31` `DOMAIN_REGEX` + check at lines 352-362 (runs FIRST). **Layer 1** scope: `lib.rs:365` `validate_target`. **Layer 3** no-shell: `crates/kri0k-core/src/ttp/subprocess.rs:91-97` `Command::arg`. Tests: `test_domain_regex_accepts_valid_domains` `pybridge/src/lib.rs:620-634` plus Python smoke `test_invalid_target_rejected_by_regex`. |
| T-04-05-07 | Repudiation (no audit log) | mitigate (Phase 4 slot) | CLOSED | `crates/kri0k-pybridge/src/lib.rs:388-401` builds `TtpExecutionEvent` + calls `audit_lock.log_ttp_execution(event)?`; `crates/kri0k-core/src/audit.rs:89-93` `NoopAuditSink::log_ttp_execution` returns Ok. Hash-chained writer lands Phase 8 per plan. |
| T-04-05-08 | Race / deadlock (lock order) | mitigate | CLOSED | `crates/kri0k-pybridge/src/lib.rs:158-166` — `dedupe.lock()` acquired BEFORE `graph.lock()`. Comment at line 158 makes the invariant explicit. Single global lock order. |
| T-04-05-09 | Race (GIL + Tokio deadlock) | mitigate | CLOSED | `py.allow_threads(...)` wrappers found at all three blocking entry points: `crates/kri0k-pybridge/src/lib.rs:272` (`#[new]`), `lib.rs:307` (`snapshot`), `lib.rs:348` (`execute_proposal`). |
| T-04-05-10 | Tampering / DoS (unbounded dedupe HashMap) | accept | CLOSED | `crates/kri0k-pybridge/src/lib.rs:137` `dedupe: Mutex<HashMap<(String, String), NodeId>>`. Plan disposition is `accept` — bounded by \|targets x kinds\| for Phase 4 single-engagement runs. **NOTE:** plan mitigation text claimed documentation in CHANGELOG/CONTRIBUTING; that documentation is NOT present in those files. Recorded explicitly in the Accepted Risks Log below to satisfy the `accept` disposition. |

## Accepted Risks Log

The following threats have disposition `accept`. Rationale recorded here per workflow:

| ID | Risk | Rationale | Owner | Re-evaluate |
|---|---|---|---|---|
| T-04-01-01 | NodeKind JSON ambiguity | Internal Rust-only at this layer; tagged enum (`#[serde(tag = "type")]`) prevents ambiguous parse. No cross-language input flows through NodeKind in Phase 4. | core | Phase 5 (when Python emits NodeKind dicts) |
| T-04-02-02 | Error message detail leakage | Variants carry only typed operator-controlled fields. No auto stack traces escape into outcomes. | core | Phase 8 (audit-log review) |
| T-04-03-05 | Malformed whois output | Parser is total (`parse_whois_output`); unknown lines preserved in `raw_unparsed`, no panics. Acceptable downgrade. | ttp | Phase 5 (when new TTP parsers land) |
| T-04-03-06 | Locale text in raw_unparsed | `String::from_utf8_lossy` prevents crash; OS-locale error strings carry no PII. | ttp | When localized OS strings begin including secrets |
| T-04-04-05 | YAML detail in `ParseError.detail` | scope.yaml is operator-controlled; serde detail aids debugging. No PII enforcement at parse layer. | scope | Phase 8 (sanitised-export mode) |
| T-04-05-10 | Unbounded dedupe HashMap | Single-engagement, short-lived Phase 4 process. Bounded by `\|targets\| x \|kinds\|` (~thousands of entries max in practice). TTL/eviction deferred to Phase 5+. **Gap:** CHANGELOG/CONTRIBUTING do not contain the limitation note the plan promised. The acceptance is recorded here instead — not a blocker because the disposition itself is `accept` and the bound argument is valid for Phase 4 scope. Recommend adding a one-line note to CHANGELOG `### Known Limitations` in Phase 5. | pybridge | Phase 5 (multi-engagement / long-lived runs) |

## Unregistered Flags

None.

SUMMARY files inspected:
- `04-01-SUMMARY.md` line 111-113 — "## Threat Flags: None"
- `04-02-SUMMARY.md` — "## Threat Surface Scan: No new network endpoints..."
- `04-03-SUMMARY.md` line 244-246 — "No new threat surface beyond what was declared in the plan's <threat_model>"
- `04-04-SUMMARY.md` — no flags section (data-model only)
- `04-05-SUMMARY.md` — no flags section (D-61 mitigation coverage table present, all mapped)

No attack-surface element appeared in implementation without a corresponding declared threat ID.

## Verification Method

- Each `mitigate` threat: located the file:line of the mitigation pattern declared in the plan, confirmed it executes on the documented entry path.
- Each `accept` threat: verified the code shape matches the accept rationale and recorded explicitly in this document's Accepted Risks Log.
- No `transfer` dispositions in Phase 4.
- Tests cited next to each threat were inspected for presence (not re-executed — Phase summaries confirm 65/65 Rust + 104 Python unit + 8 integration pass).

## Audit Trail

| Date | Auditor | Action | Result |
|---|---|---|---|
| 2026-05-18 | gsd-security-auditor (Opus 4.7) | Initial verification of Plan 04-01..04-05 register | 24/24 CLOSED |

## Findings / Recommendations

- **Documentation gap (non-blocker):** T-04-05-10 mitigation plan referenced a CHANGELOG/CONTRIBUTING limitation note that is absent. Acceptance recorded in this SECURITY.md. Suggest a one-line `### Known Limitations` entry in CHANGELOG.md during Phase 5 (no code change required).
- **No code modifications recommended.** Phase 4 mitigations are complete and verifiable.

## Verdict

**SECURED — threats_open: 0.**

Phase 4 may advance. Recommended next step: `/gsd-validate-phase 4` then `/gsd-verify-work 4`.
