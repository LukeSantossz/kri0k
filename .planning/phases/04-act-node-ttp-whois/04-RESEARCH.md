# Phase 4 Research: Act Node + TTP Whois

**Researched:** 2026-05-18
**Domain:** Cross-language (Rust + Python via PyO3) subprocess orchestration with structured graph mutation
**Confidence:** HIGH for stack/versions; HIGH for Tokio/PyO3 patterns; MEDIUM for whois output parsing (registry redaction reality)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

29 design decisions are locked (D-34..D-64) across 12 categories. The planner MUST honor every one of them. The full text lives in `04-CONTEXT.md`; this section enumerates the constraint surface so the planner cannot miss any. The research below explores **how to implement** these decisions, not whether to reconsider them.

**A. State & Engagement (D-34..D-38):**
- D-34: `Engagement` pyclass owns `{graph: Mutex<Graph>, scope: ScopeConfig, audit: Box<dyn AuditSink>, registry: HashMap<String, Box<dyn Ttp>>, cancel: CancellationToken}`.
- D-35: Façade API: `snapshot()`, `execute_proposal(proposal: dict)`, `scope_hash()`. Nothing else exposed to Python.
- D-36: Python helper `kri0k.engagement.create(scope_dict, objective, propose_only=True)` injects the instance into `AgentState.engagement_context["engagement"]` before `graph.invoke()`.
- D-37: Lock strategy = `Mutex<Graph>` (not RwLock).
- D-38: `Box<dyn AuditSink>` slot reserved now with `NoopAuditSink` default; `execute_proposal` calls `audit.log_ttp_execution(...)` already.

**B. Graph Data Model (D-39..D-43):**
- D-39: `EdgeKind` gains `RegisteredBy` (Domain -> Organization) and `HasNameserver` (Domain -> Nameserver). Existing `RelatesTo{relation}` stays.
- D-40: `NodeKind` gains `Domain { name: String }`, `Organization { name: String }`, `Nameserver { hostname: String }`. Dates go in `Node.metadata` with keys `created_at`, `updated_at`, `expires_at` (ISO 8601).
- D-41: Heuristic key:value parser; non-ICANN TLDs degrade gracefully with `metadata["raw_block_<n>"]`.
- D-42: Only Registrant Organization becomes an `Organization` node. Admin/Tech/Billing contacts are ignored.
- D-43: Idempotency via `HashMap<(NodeKindTag, String), NodeId>` keyed on natural name. Edge dedupe by `(src, dst, kind_tag)`. Re-execution updates `metadata["last_whois_at"]` but does not grow the graph.

**C. Execution Stack (D-44..D-47):**
- D-44: `tokio::process::Command` async; trait `Ttp` becomes async (`#[async_trait]`).
- D-45: TTP-local rate limit via `Mutex<Instant>` field on the TTP struct.
- D-46: pyclass methods stay synchronous; internally call `runtime().block_on(async { ... })` like `get_dummy_graph`. Python side wraps with `await asyncio.to_thread(engagement.execute_proposal, ...)`.
- D-47: `execute_proposal` returns rich outcome dict: `status, result, error, graph_delta, audit_id`.

**D. Gates & Safeguards (D-48..D-51):**
- D-48: Scope check Phase 4 = allowlist exact-match domain. `Scope::from_yaml(path)` parses `targets:`; `validate_target(t)` is `targets.contains(t)`. CIDR/wildcards deferred to Phase 7.
- D-49: Propose-only flag at `engagement_context["propose_only"]` (default `True`). `act.py` decides; if `True` returns `{status: "proposed", proposal, would_execute}` without touching Rust.
- D-50: `Engagement::new()` calls `which::which("whois")` and fails fast with `Error::MissingDependency("whois")` if absent. Python helper translates to `RuntimeError` with install hint.
- D-51: 30s default timeout via new trait method `Ttp::default_timeout() -> Duration`.

**E. TTP Registry (D-52):**
- D-52: `HashMap<String, Box<dyn Ttp>>` hardcoded in `Engagement::new()`. No auto-discovery yet.

**F. Error Taxonomy (D-53):**
- D-53: Outcome `status` strings: `executed | scope_violation | rate_limited | timeout | error | proposed`. Rust `Error` enum gains variants `ScopeViolation{target, reason}`, `RateLimitExceeded{ttp_id, retry_in_ms}`, `SubprocessTimeout{ttp_id, timeout_ms}`, `ParseError{source, detail}`, `MissingDependency{binary}`, `UnknownTtp{ttp_id}`, `Io(#[from] std::io::Error)`, plus `Cancelled` (implied by D-62).

**G. Testing (D-54):**
- D-54: `Subprocess` trait abstraction (Real + Mock). `WhoisTtp::new(subprocess: Arc<dyn Subprocess>)`. Mock reads fixture path from constructor. Integration tests with real binary gated by `#[cfg(feature = "integration")]`.

**H. Observability & History (D-55, D-56):**
- D-55: `tracing = "0.1"` introduced. `#[tracing::instrument(skip(self, subprocess))]` on `Engagement::execute_proposal`, `WhoisTtp::execute`, `parse_whois_output`. No subscriber bundled (Phase 12 CLI configures).
- D-56: History entry shape: `{iteration, ttp_id, target, status, summary, graph_delta, audit_id}`.

**I. CI/CD (D-57):**
- D-57: Integration tests feature-gated; main CI runs only unit tests with mocked subprocess. `CONTRIBUTING.md` documents `cargo test --features integration` for local repro. Optional `nightly-integration.yml` workflow with `whois` installed.

**J. scope.yaml Schema (D-58):**
- D-58: Full lookahead schema (all v1 fields declared with `#[serde(default)]` stubs). Phase 4 actually consumes `version`, `targets`, `safeguards.propose_only`. Mandatory `version: 1`; unknown version rejected.

**K. Task & Commit Workflow (D-59):**
- D-59: TASK-015 major aggregated in `.claude/tasks.md`. Branch `feat/phase-4-act-ttp-whois`. Conventional Commits, one line, no body, no `Co-authored-by`. PR to `master` after `/gsd-verify-work`.

**L. Documentation (D-60):**
- D-60: README quickstart "Running the whois TTP", CONTRIBUTING "Adding a new TTP", CHANGELOG 0.2.0, ADR-0013-ttp-trait-subprocess-abstraction, optional `.planning/codebase/ARCHITECTURE.md` re-render.

**M. Threat Model (D-61):**
- D-61: Explicit M-XX mapping table in CONTEXT.md + inline `// M-XX` annotations in code at the relevant call sites.

**N. Definition of Done (D-64):**
- D-64: Seven explicit criteria including ROADMAP 1-4, `cargo clippy --workspace --all-targets`, `ruff + mypy strict`, `pytest + cargo test --features integration`, docs delivered, threat table populated, end-to-end `whois example.com` smoke test.

**O. Security: Command Injection (D-63):**
- D-63: Three-layer defense — (1) allowlist exact match, (2) regex domain validation, (3) `Command::arg()` no shell.

**R. Graceful Shutdown (D-62):**
- D-62: `CancellationToken` (`tokio-util = "0.7"`) inside `Engagement`. `tokio::select!` over `cancel.cancelled()` vs `child.wait_with_output()`. Python surface: `Engagement.kill()` calls `cancel.cancel()`. Adds `Error::Cancelled` variant.

### Claude's Discretion

- Exact `HashMap<(NodeKindTag, String), NodeId>` type — could use `IndexMap` if order matters; otherwise `HashMap`.
- Layout of new Rust modules (`ttp/mod.rs` re-exports vs flat) — pick whatever matches `STRUCTURE.md`.
- Exact domain regex — D-63 supplies the base pattern; tweaks for punycode/IPv4-PTR are at the planner's discretion.
- Human-readable strings inside `Error::*` variants.
- CHANGELOG.md layout (keep-a-changelog vs simpler).

### Deferred Ideas (OUT OF SCOPE)

- Admin/Tech/Billing contact organizations as separate `Organization` nodes (v2).
- CIDR + wildcard scope validation (Phase 7).
- TTP auto-discovery via `inventory` crate (deferred until >= 5 TTPs).
- `pyo3-asyncio` for native async pyclass methods (deferred — `asyncio.to_thread` is enough).
- Real JSONL hash-chained audit log (Phase 8).
- Interactive TUI approval keybinding (Phase 11).
- `kri0k doctor` health check (Phase 12).
- `RwLock<Graph>` (deferred until snapshot becomes hot).
- Non-ICANN TLD parsing (.br, .uk) (v2).
- Provider switching at runtime (v2).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AGENT-05 | Nó ACT envia Proposal para Rust, recebe resultado da execução | Pattern 2 (Engagement pyclass), Pattern 3 (async dispatch), `act.py` rewrite in File Manifest |
| TTP-01 | TTP T1590.001 (whois) implementa trait Ttp em Rust | Pattern 3 (`#[async_trait]` + `Box<dyn Ttp>`), `ttp/whois.rs` in File Manifest |
| TTP-02 | TTP whois executa comando whois via std::process::Command | D-44 overrides this — async `tokio::process::Command` (D-44 documented divergence; planner must align). Pattern 1 covers timeout + cancellation. |
| TTP-03 | TTP whois parseia output para campos estruturados (registrant, nameservers, dates) | Pattern 8 (heuristic key:value parser). Pitfall #4 documents GDPR redaction reality. |
| TTP-04 | TTP whois adiciona nós ao grafo (Domain, Organization, Nameserver) | D-40/D-43 (NodeKind variants + idempotency cache), graph mutation flow in Pattern 2 |
| TTP-05 | TTP whois respeita rate limits configurados (default 1 req/sec) | D-45 (TTP-local `Mutex<Instant>`), Validation Architecture maps to test |
</phase_requirements>

## Summary

Phase 4 fecha o loop de execução do agente. Entrega três blocos cross-language: (1) o pyclass `Engagement` no `kri0k-pybridge` como container canônico de estado por engagement (grafo + scope + audit sink + cancel token + TTP registry), (2) refactor do trait `Ttp` para `async` mais a primeira TTP concreta — `WhoisTtp` invocando `whois.exe` Sysinternals via `tokio::process::Command` com rate limit, timeout e cancellation, e (3) o re-write do `act.py` que faz o gate `propose_only` e dispara `execute_proposal` via `asyncio.to_thread`. A camada de scope/audit é stubada para Phase 7/8, mas o slot existe.

Os riscos técnicos mais agudos que o planner precisa estruturar como tarefas explícitas: (a) **redação GDPR no whois ICANN** — desde 2018 a maioria dos registries devolve só registrar info; o flag `-v` da Sysinternals dispara consulta thick (registrar), que de fato traz `Registrant Organization` (verificado experimentalmente com google.com nesta pesquisa); (b) **EULA prompt interativo na primeira execução** do whois.exe — sem `-accepteula` o subprocess hangs indefinido (verificado); (c) **dyn-compatibility de `Box<dyn Ttp>` com `async fn`** — exige `#[async_trait]` (não é nativo no Rust 1.85), com custo de uma allocation por chamada; (d) **GIL + Tokio runtime** dentro de `#[pymethods]` — o padrão certo é `py.allow_threads(|| runtime().block_on(async { ... }))` para evitar deadlock; (e) **`serde_yaml` deprecated** (oficial deprecation 2024-03-25) e `serde_yml` tem RUSTSEC advisory — usar `serde_yaml_ng` em vez disso.

**Primary recommendation:** Estruturar Phase 4 em **5 plans, 2-3 waves**. Wave 0 paralela: (P1) graph data model (NodeKind/EdgeKind expansion) — independent, sem deps de outros plans. (P2) ttp/subprocess refactor (Ttp async + Subprocess trait + Cargo.toml deps) — paralelizável com P1. Wave 1 sequencial: (P3) WhoisTtp implementation + parser + fixtures — depende de P1+P2. (P4) Engagement pyclass + scope parser + `_native.pyi` stubs — depende de P2 (mas pode arrancar com mocks para o Ttp registry). Wave 2: (P5) Python wiring (act.py rewrite + engagement.create helper + sense.py snapshot fallback) + docs (README/CONTRIBUTING/CHANGELOG/ADR-0013).

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Subprocess execution + timeout + cancel | Rust (`kri0k-core::ttp`) | — | ADR-0001 / ADR-0012: Rust holds runtime control; LLM never executes |
| Whois output parsing | Rust (`kri0k-core::ttp::whois`) | — | Determinístico, testável com fixtures; Python só consome resultado tipado |
| Graph mutation (add Domain/Org/Nameserver nodes) | Rust (`kri0k-graph` + `Engagement`) | — | ADR-0001: canonical state in Rust; Python recebe snapshot read-only |
| Scope validation (allowlist exact-match) | Rust (`kri0k-core::scope`) | — | ADR-0005: deterministic validator pre-execution, fail-closed |
| Propose-only gate (decide if execute happens) | Python (`agent/nodes/act.py`) | Rust (defense-in-depth via `safeguards.propose_only`) | D-49: gate é boolean no contexto LangGraph; Rust não precisa enforçar enquanto a flag for honrada por act.py |
| Rate limit enforcement (1 req/sec) | Rust (TTP-local `Mutex<Instant>`) | — | D-45: cada TTP guarda seu próprio bucket; Python não interfere |
| Cross-language bridging (snapshot, execute_proposal) | Rust (`kri0k-pybridge` pyclass) | Python (`engagement.py` helper) | PyO3 sync methods + `runtime().block_on`; Python wrappa com `asyncio.to_thread` |
| Engagement bootstrap | Python (`kri0k.engagement.create`) | Rust (`_native.Engagement.new`) | D-36: helper Python orquestra; chama factory Rust |
| Outcome consumption (status, history append) | Python (`act.py`) | — | D-56: history shape é Python-native; reflect/reason consomem |
| LangGraph topology | Python (`agent/graph.py`) | — | Phase 1 ownership; Phase 4 não toca |
| Audit logging (TTP execution event) | Rust (`Box<dyn AuditSink>` no-op em Phase 4) | — | D-38: slot reservado; real impl em Phase 8 |
| Tracing instrumentation | Rust (`#[tracing::instrument]`) | Python (consumer via tracing-subscriber em Phase 12) | D-55: span infra no Rust; subscriber vem depois |

## Standard Stack

### Core (workspace already pinned)

| Library | Version | Purpose | Source |
|---------|---------|---------|--------|
| `tokio` | 1.40 (already pinned) | Async runtime; add `process`, `time`, `sync` features | `Cargo.toml:19` [VERIFIED: workspace] |
| `pyo3` | 0.24 (already pinned) | PyO3 bindings | `Cargo.toml:20` [VERIFIED: workspace] |
| `petgraph` | 0.6 (already pinned) | Graph backend | `Cargo.toml:21` [VERIFIED: workspace] |
| `serde` / `serde_json` | 1.0 (already pinned) | Cross-boundary serialization | `Cargo.toml:16,17` [VERIFIED: workspace] |
| `thiserror` | 1.0 (already pinned) | Error derive | `Cargo.toml:18` [VERIFIED: workspace] |
| `ulid` | 1.1 (already pinned) | NodeId/EdgeId | `Cargo.toml:22` [VERIFIED: workspace] |

### New Crates (to add for Phase 4)

| Crate | Version | Feature Flags | Purpose | MSRV-Compatible (1.85)? | Source |
|-------|---------|---------------|---------|------------------------|--------|
| `tokio-util` | `0.7.18` | `["rt"]` (pulls in CancellationToken) | D-62 graceful shutdown via `CancellationToken` | YES — MSRV 1.71 | [VERIFIED: crates.io 2026-01-04 release] |
| `which` | `8.0.2` | (default) | D-50 fail-fast `whois` binary detection | YES — MSRV 1.70 | [VERIFIED: crates.io] |
| `async-trait` | `0.1.89` | (default) | Make `Ttp` trait dyn-compatible with async | YES — MSRV 1.56 | [VERIFIED: crates.io 2025-08-14 release] |
| `tracing` | `0.1.44` | (default) | D-55 structured spans on hot paths | YES — MSRV 1.65 | [VERIFIED: crates.io 2025-12-18 release] |
| `serde_yaml_ng` | `0.10.0` | (default) | D-58 scope.yaml parsing — actively maintained fork | YES (no MSRV bump observed) | [VERIFIED: crates.io 2024-05-26 release] |
| **Add to `kri0k-core` only:** | | | | | |
| `arc-swap` (optional) | — | — | NOT NEEDED for Phase 4 (Mutex suffices per D-37) | — | — |

### Crates Considered and Rejected

| Crate | Why rejected |
|-------|---------|
| `serde_yaml` (dtolnay) | DEPRECATED 2024-03-25 — version 0.9.34+deprecated explicitly; archived [VERIFIED: docs.rs/crate/serde_yaml] |
| `serde_yml` (Sebastien Rousseau) | RUSTSEC-2025-0068 — unsoundness advisory; archived [VERIFIED: rustsec.org/advisories/RUSTSEC-2025-0068.html] |
| `yaml-rust2` | Lower-level parser, no serde integration out of box; serde_yaml_ng is more direct drop-in |
| `pyo3-asyncio` | D-46 explicitly defers; `asyncio.to_thread` is enough |
| `inventory` (TTP registry) | D-52 explicitly defers until >= 5 TTPs |
| `whois-rust` / `parse-whois` | D-41 picks heuristic parser; external crate would force schema we don't need |
| `regex` for domain validation | Possibly add, but D-63 base pattern is simple enough for `once_cell::Regex::new` if needed. Planner can choose; if added, use `regex = "1"` workspace dep |

### Installation Plan

Add to `[workspace.dependencies]` in `Cargo.toml`:

```toml
tokio-util = { version = "0.7", features = ["rt"] }
which = "8"
async-trait = "0.1"
tracing = "0.1"
serde_yaml_ng = "0.10"
```

Add to `crates/kri0k-core/Cargo.toml`:

```toml
[dependencies]
serde.workspace = true
serde_json.workspace = true
serde_yaml_ng.workspace = true   # NEW (D-58)
thiserror.workspace = true
ulid.workspace = true
tokio = { workspace = true, features = ["process", "time", "sync", "macros"] }  # NEW features
tokio-util.workspace = true       # NEW (D-62)
which.workspace = true            # NEW (D-50)
async-trait.workspace = true      # NEW (D-44)
tracing.workspace = true          # NEW (D-55)

[features]
default = []
integration = []                  # NEW (D-54): gates real subprocess tests

[dev-dependencies]
tokio = { workspace = true, features = ["macros", "rt-multi-thread"] }
```

Add to `crates/kri0k-graph/Cargo.toml`: no new deps (only NodeKind/EdgeKind enum expansion).

Add to `crates/kri0k-pybridge/Cargo.toml`:

```toml
[dependencies]
kri0k-core = { path = "../kri0k-core" }
kri0k-graph = { path = "../kri0k-graph" }
pyo3.workspace = true
tokio.workspace = true
tokio-util.workspace = true       # NEW (CancellationToken in Engagement)
serde_json.workspace = true
tracing.workspace = true          # NEW (instrument on pyclass methods)
```

**Version verification (run before adding):**

```bash
# Confirm latest versions before pinning
cargo search tokio-util --limit 1
cargo search which --limit 1
cargo search async-trait --limit 1
cargo search tracing --limit 1
cargo search serde_yaml_ng --limit 1
```

## Implementation Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Python LangGraph Agent                                                  │
│                                                                          │
│  engagement.create(scope_dict)  ──► _native.Engagement.new()             │
│         │                                                                │
│         ▼                                                                │
│  AgentState.engagement_context["engagement"] = <Engagement instance>     │
│  AgentState.engagement_context["propose_only"] = True (default)          │
│         │                                                                │
│         ▼                                                                │
│  ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐    │
│  │ sense   │──►│ reason  │──►│  plan   │──►│  act    │──►│ reflect │    │
│  └────┬────┘   └─────────┘   └────┬────┘   └────┬────┘   └────┬────┘    │
│       │ snapshot()                │ Proposal     │ propose_only?         │
│       │                           │              │                       │
│       ▼                           ▼              ▼ (if False)            │
│  engagement.snapshot()      proposal dict   await asyncio.to_thread(     │
│                                                 engagement.execute_      │
│                                                 proposal, proposal)      │
│                                                  │                       │
└──────────────────────────────────────────────────┼───────────────────────┘
                                                   │ JSON dict cross PyO3
┌──────────────────────────────────────────────────┼───────────────────────┐
│  Rust pybridge cdylib (_native)                  │                       │
│                                                  ▼                       │
│  #[pyclass] Engagement {                                                 │
│      graph: Mutex<Graph>,                                                │
│      scope: ScopeConfig,                                                 │
│      audit: Box<dyn AuditSink>,                                          │
│      registry: HashMap<String, Box<dyn Ttp>>,  // {"T1590.001": Box::    │
│      cancel: CancellationToken,                //  new(WhoisTtp::new)}   │
│      dedupe: Mutex<HashMap<(KindTag, String), NodeId>>,                  │
│  }                                                                       │
│      │                                                                   │
│      ▼ execute_proposal(proposal_dict)                                   │
│  runtime().block_on(async {                                              │
│      1. validate_target(scope, proposal.target) ──► ScopeViolation?      │
│      2. registry.get(proposal.ttp_id) ──► UnknownTtp?                    │
│      3. tokio::select! {                                                 │
│           _ = cancel.cancelled() => Err(Cancelled)                       │
│           r = ttp.execute(target, cancel.clone()) => r                   │
│         }                                                                │
│      4. Apply graph mutations via dedupe cache                           │
│      5. audit.log_ttp_execution(event)                                   │
│      6. Build outcome dict                                               │
│  })                                                                      │
└──────────┬───────────────────────────────────────────────────────────────┘
           │
           ▼ via Subprocess trait                                          
┌──────────────────────────────────────────────────────────────────────────┐
│  TTP: WhoisTtp                                                           │
│                                                                          │
│  WhoisTtp {                                                              │
│      subprocess: Arc<dyn Subprocess>,  // Real or Mock                   │
│      last_call: Mutex<Instant>,        // D-45 rate limit                │
│  }                                                                       │
│      │                                                                   │
│      ▼ execute(target, cancel)                                           │
│  1. wait for rate limit (now - last_call >= 1s)                          │
│  2. subprocess.run("whois", &["-v", "-nobanner", target], 30s)           │
│       ──► RealSubprocess uses tokio::process::Command with               │
│           tokio::select!(timeout, cancel, child.wait_with_output)        │
│  3. parse_whois_output(stdout) ──► WhoisOutput                           │
│  4. update last_call = now                                               │
└──────────────────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure

```
crates/kri0k-core/src/
├── lib.rs            # Extend Error enum (+ 7 new variants per D-53 + Cancelled)
├── audit.rs          # Add NoopAuditSink impl (already partial; rename NoOpAuditSink -> NoopAuditSink per D-38)
├── safeguards.rs     # NO CHANGE
├── scope.rs          # Replace stub: ScopeConfig struct + from_yaml + validate_target (D-48, D-58)
└── ttp/              # PROMOTE single ttp.rs to module per D-44, D-52
    ├── mod.rs        # Trait Ttp (async), RateLimits, ExecutionPlan, RiskLevel, re-exports
    ├── subprocess.rs # NEW: Subprocess trait + RealSubprocess + MockSubprocess (D-54)
    └── whois.rs      # NEW: WhoisTtp impl + WhoisOutput struct + parse_whois_output

crates/kri0k-graph/src/
└── lib.rs            # Extend NodeKind (+ Domain, Organization, Nameserver) + EdgeKind (+ RegisteredBy, HasNameserver) per D-39, D-40

crates/kri0k-pybridge/src/
└── lib.rs            # Add #[pyclass] Engagement + #[pymethods] impl

python/kri0k/
├── __init__.py       # Re-export Engagement (optional)
├── _native.pyi       # Add class Engagement stubs (D-34, D-35)
├── engagement.py     # NEW: kri0k.engagement.create helper (D-36)
└── agent/
    └── nodes/
        ├── act.py    # REWRITE: propose_only gate + execute_proposal (D-49, D-56)
        └── sense.py  # MODIFY: prefer engagement.snapshot() when context has it (backward-compat)

tests/
├── test_act_node.py              # NEW: act node with mocked engagement
├── test_engagement_smoke.py      # NEW: integration test with real Engagement
└── fixtures/
    └── whois_example_com.txt     # NEW: real whois output sample for parser tests
```

### Pattern 1: `tokio::process::Command` + timeout + CancellationToken

The canonical pattern for running an external command with both a deadline and a cancellation signal, killing the child cleanly on either.

```rust
// crates/kri0k-core/src/ttp/subprocess.rs
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;
use async_trait::async_trait;

#[async_trait]
pub trait Subprocess: Send + Sync {
    async fn run(
        &self,
        cmd: &str,
        args: &[&str],
        timeout: Duration,
        cancel: CancellationToken,
    ) -> Result<SubprocessOutput, crate::Error>;
}

#[derive(Debug, Clone)]
pub struct SubprocessOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Default)]
pub struct RealSubprocess;

#[async_trait]
impl Subprocess for RealSubprocess {
    async fn run(
        &self,
        cmd: &str,
        args: &[&str],
        timeout: Duration,
        cancel: CancellationToken,
    ) -> Result<SubprocessOutput, crate::Error> {
        // M-63 (D-63 layer 3): use .arg() not shell — Rust default is safe
        let mut child = Command::new(cmd)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)  // Belt-and-suspenders: ensure cleanup even on panic
            .spawn()
            .map_err(crate::Error::Io)?;

        // The triple-branch select: cancellation wins over timeout wins over completion
        let output = tokio::select! {
            biased;  // Check cancel first
            _ = cancel.cancelled() => {
                let _ = child.start_kill();  // start_kill is sync; sends SIGKILL on Unix, TerminateProcess on Windows
                let _ = child.wait().await;  // Reap to avoid zombie
                return Err(crate::Error::Cancelled);
            }
            r = tokio::time::timeout(timeout, child.wait_with_output()) => r,
        };

        match output {
            Ok(Ok(out)) => Ok(SubprocessOutput {
                stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
                exit_code: out.status.code(),
            }),
            Ok(Err(e)) => Err(crate::Error::Io(e)),
            Err(_elapsed) => Err(crate::Error::SubprocessTimeout {
                ttp_id: "subprocess".to_string(),
                timeout_ms: u64::try_from(timeout.as_millis()).unwrap_or(u64::MAX),
            }),
        }
    }
}
```

**Why these choices:**

- `kill_on_drop(true)` — extra safety net. If `select!` panics or the future is dropped from the outer scope, the child is still cleaned up. [VERIFIED: docs.rs/tokio/latest/tokio/process/struct.Child.html]
- `biased` in `select!` — Rust's `select!` is fair-shuffled by default; `biased` makes cancel-check happen first every poll, lowering kill latency. [CITED: tokio docs on select!]
- `start_kill()` (sync) + `wait()` rather than `kill().await` — `kill().await` is async-wait-on-signal; `start_kill()` is fire-and-forget signal followed by explicit wait, which lets us guarantee no zombie even if the kill races. [VERIFIED: docs.rs/tokio/latest/tokio/process/struct.Child.html]
- `String::from_utf8_lossy` — whois output sometimes contains non-UTF-8 control sequences (especially Sysinternals on Windows with locale-dependent connection error messages); lossy conversion prevents parse failures from invalid bytes.

### Pattern 2: PyO3 `#[pyclass] Engagement` with internal async

The skeleton showing how Engagement holds the mutex/audit/cancel/registry and bridges sync pyclass methods to async Rust via `runtime().block_on`.

```rust
// crates/kri0k-pybridge/src/lib.rs (add to existing file)
use kri0k_core::{
    audit::{AuditSink, NoopAuditSink},
    scope::ScopeConfig,
    ttp::{Ttp, whois::WhoisTtp, subprocess::RealSubprocess},
    Error,
};
use kri0k_graph::Graph;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyAny};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;
use tracing::instrument;

#[pyclass]
pub struct Engagement {
    graph: Mutex<Graph>,
    scope: ScopeConfig,
    audit: Mutex<Box<dyn AuditSink>>,            // Mutex because AuditSink methods take &mut self
    registry: HashMap<String, Box<dyn Ttp>>,
    cancel: CancellationToken,
    dedupe: Mutex<HashMap<(String, String), kri0k_core::NodeId>>,  // (KindTag, natural_key) -> NodeId
}

#[pymethods]
impl Engagement {
    /// Construct Engagement from a parsed scope dict.
    /// Fails fast if whois binary is missing (D-50).
    #[new]
    fn new(scope_dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        // 1. Parse scope_dict -> ScopeConfig (via serde_json round-trip from PyDict)
        let scope: ScopeConfig = pydict_to_serde(scope_dict)?;

        // 2. Fail-fast on whois binary (D-50, M-36)
        which::which("whois").map_err(|_| {
            pyo3::exceptions::PyRuntimeError::new_err(
                "whois binary not found in PATH. Install with: \
                 winget install Microsoft.Sysinternals.Whois (Windows) \
                 or apt install whois (Linux)"
            )
        })?;

        // 3. Build TTP registry (D-52: hardcoded map)
        let subprocess = Arc::new(RealSubprocess);
        let mut registry: HashMap<String, Box<dyn Ttp>> = HashMap::new();
        registry.insert(
            "T1590.001".to_string(),
            Box::new(WhoisTtp::new(subprocess)),
        );

        Ok(Self {
            graph: Mutex::new(Graph::new()),
            scope,
            audit: Mutex::new(Box::new(NoopAuditSink::default())),
            registry,
            cancel: CancellationToken::new(),
            dedupe: Mutex::new(HashMap::new()),
        })
    }

    /// D-35 façade: return graph snapshot as Python dict.
    #[instrument(skip(self, py))]
    fn snapshot(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let json_value = py.allow_threads(|| {
            self.graph.lock()
                .map_err(|e| Error::Generic(format!("graph mutex poisoned: {e}")))
                .and_then(|g| g.to_json())
        }).map_err(error_to_pyerr)?;

        json_value_to_pydict(py, &json_value)
    }

    /// D-35 façade: execute a Proposal.
    /// Returns rich outcome dict (D-47).
    #[instrument(skip(self, py, proposal))]
    fn execute_proposal(
        &self,
        py: Python<'_>,
        proposal: &Bound<'_, PyDict>,
    ) -> PyResult<Py<PyAny>> {
        // Extract proposal fields (must happen with GIL held)
        let target: String = proposal.get_item("target")?.ok_or_else(|| {
            pyo3::exceptions::PyKeyError::new_err("proposal.target missing")
        })?.extract()?;
        let ttp_id: String = proposal.get_item("ttp_id")?.ok_or_else(|| {
            pyo3::exceptions::PyKeyError::new_err("proposal.ttp_id missing")
        })?.extract()?;

        let cancel = self.cancel.clone();

        // Release GIL, run async work on the Tokio runtime
        let outcome_json = py.allow_threads(|| -> Result<serde_json::Value, Error> {
            runtime().block_on(async {
                // 1. Scope check (D-48, M-02)
                self.scope.validate_target(&target)?;

                // 2. TTP lookup (D-52)
                let ttp = self.registry.get(&ttp_id)
                    .ok_or_else(|| Error::UnknownTtp { ttp_id: ttp_id.clone() })?;

                // 3. Execute with cancel + timeout (D-44, D-51, D-62)
                let result = ttp.execute(&target, cancel).await?;

                // 4. Apply graph mutations idempotently (D-43)
                let delta = self.apply_whois_output(&target, &result)?;

                // 5. Audit (D-38 - currently no-op)
                let event = build_audit_event(&ttp_id, &target);
                self.audit.lock()
                    .map_err(|e| Error::Generic(format!("audit mutex: {e}")))?
                    .log_ttp_execution(event)?;

                // 6. Build outcome
                Ok(build_outcome("executed", Some(result), None, delta))
            })
        });

        let outcome = match outcome_json {
            Ok(v) => v,
            Err(e) => error_to_outcome(&e),  // map ScopeViolation -> status:scope_violation, etc.
        };

        json_value_to_pydict(py, &outcome)
    }

    /// D-35 façade: scope hash for embedding in snapshots.
    fn scope_hash(&self) -> PyResult<String> {
        Ok(self.scope.compute_hash())
    }

    /// D-62: signal kill switch.
    fn kill(&self) {
        self.cancel.cancel();
    }
}

// Helper: convert serde_json::Value -> Python dict via json.loads round-trip
// (same trick already in get_dummy_graph)
fn json_value_to_pydict(py: Python<'_>, v: &serde_json::Value) -> PyResult<Py<PyAny>> {
    let json_str = serde_json::to_string(v)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("json: {e}")))?;
    let json_module = py.import("json")?;
    let result = json_module.getattr("loads")?.call1((json_str,))?;
    Ok(result.into())
}
```

**Why these choices:**

- `Mutex<Graph>` and `Mutex<Box<dyn AuditSink>>` are `std::sync::Mutex` (not `tokio::sync::Mutex`) because they are held only for short, non-async sections inside `block_on`. PyO3 0.24 requires `Sync`; `std::sync::Mutex` provides it. [CITED: pyo3.rs/v0.24.0/class/thread-safety]
- `py.allow_threads(|| ...)` wrapping `runtime().block_on(...)` — releases the GIL before the runtime blocks, preventing deadlocks when Tokio worker threads try to interact with Python. This is the exact pattern already used in `get_dummy_graph` at `crates/kri0k-pybridge/src/lib.rs:36`. [CITED: PyO3 GitHub discussion #3045]
- `Bound<'_, PyDict>` is the PyO3 0.24 idiomatic input type (replacing legacy `&PyDict`). [CITED: PyO3 0.24 migration guide]
- `CancellationToken::clone()` before passing into the async block — clones share cancellation state with the parent. [CITED: docs.rs/tokio-util CancellationToken docs]

### Pattern 3: `async-trait` + `Box<dyn Ttp>` dispatch

The trait declaration that makes `Box<dyn Ttp>` work with async methods.

```rust
// crates/kri0k-core/src/ttp/mod.rs
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use crate::ttp::subprocess::Subprocess;
use crate::ttp::whois::WhoisOutput;

#[async_trait]
pub trait Ttp: Send + Sync {
    /// Stable TTP identifier (e.g., "T1590.001").
    fn id(&self) -> &str;

    /// MITRE technique name.
    fn description(&self) -> &str;

    /// Risk classification (used by Phase 11 human gate).
    fn risk_level(&self) -> RiskLevel { RiskLevel::Safe }

    /// Per-TTP rate limit configuration.
    fn rate_limits(&self) -> RateLimits;

    /// Per-TTP default subprocess timeout (D-51).
    fn default_timeout(&self) -> Duration { Duration::from_secs(30) }

    /// Execute the TTP against `target`, honoring `cancel`.
    /// Returns `WhoisOutput` for whois; other TTPs will define their own output types.
    /// For Phase 4, Outcome is monomorphic on WhoisOutput; refactor to enum if v2 needs polymorphism.
    async fn execute(
        &self,
        target: &str,
        cancel: CancellationToken,
    ) -> Result<TtpOutput, crate::Error>;
}

/// Outcome of any TTP execution.
/// Phase 4 only populates `Whois(WhoisOutput)`.
/// Phase 5+ adds variants per TTP.
#[derive(Debug, Clone)]
pub enum TtpOutput {
    Whois(WhoisOutput),
    // Future: Nmap(NmapOutput), Dig(DigOutput), etc.
}
```

**Why `async-trait`:**

`Box<dyn Ttp>` requires the trait to be *dyn-compatible* (object-safe). Rust 1.85's native `async fn in trait` produces `impl Future` return types that DIFFER across implementors, breaking vtable construction. The `#[async_trait]` macro rewrites every `async fn` into `fn (...) -> Pin<Box<dyn Future + Send + 'async_trait>>`, giving every impl the *same* return type — which fixes dyn-compatibility at the cost of one heap allocation per call. [CITED: blog.rust-lang.org dyn-async-traits series; smallcultfollowing.com/babysteps/blog/2025/03/24]

For Phase 4 (one TTP, rate-limited to 1 req/sec) this allocation is negligible. When the registry crosses ~5 TTPs and hot dispatch matters, revisit (TAIT or other.).

### Pattern 4: `which::which("whois")` fail-fast

```rust
// Inside Engagement::new(), before building the registry:
which::which("whois").map_err(|e| match e {
    which::Error::CannotFindBinaryPath => Error::MissingDependency {
        binary: "whois".to_string(),
    },
    other => Error::Generic(format!("which error: {other}")),
})?;
```

**Notes verified:**

- `which::which` does ONLY a PATH lookup; it does NOT execute the binary or validate that it works. It returns `PathBuf` of the first match. [CITED: docs.rs/which]
- On Windows, `which::which("whois")` automatically tries `.exe`/`.cmd`/`.bat` extensions in PATHEXT order. So pass the bare name. [CITED: which crate docs cross-platform note]
- `which::Error` has two main variants: `CannotFindBinaryPath` (most common) and `BadAbsolutePath` (when given an absolute path that isn't executable). [VERIFIED: docs.rs/which/8.0.2/which/enum.Error.html]
- Python-side, the `RuntimeError` from `Engagement.new(...)` becomes a Python exception with the install hint message; `engagement.create()` helper re-raises with extra context if needed.

### Pattern 5: `tracing` instrument setup

```rust
use tracing::{instrument, info, warn};

#[instrument(skip(self, subprocess), fields(ttp_id = %self.id()))]
pub async fn execute(
    &self,
    target: &str,
    cancel: CancellationToken,
) -> Result<TtpOutput, crate::Error> {
    info!(target = %target, "starting whois execution");

    // ... work ...

    let elapsed_ms = start.elapsed().as_millis();
    info!(elapsed_ms, ns_count = output.nameservers.len(), "whois complete");
    Ok(TtpOutput::Whois(output))
}
```

**Key facts:**

- `#[tracing::instrument]` wraps the function body in a span. `skip(self)` prevents `self` from being recorded (it's huge); `skip(subprocess)` likewise omits the trait object. [CITED: docs.rs/tracing/latest/tracing/attr.instrument.html]
- `fields(...)` adds custom span fields evaluated once at entry.
- For ASYNC functions, `#[instrument]` is the right macro (not `#[instrument_async]` — that doesn't exist). It correctly handles `.await` points. [CITED: docs.rs/tracing instrument macro section "Async functions"]
- The crate emits events to the **global subscriber**. If no subscriber is installed, events are dropped silently — no performance cost beyond the span creation. The Python host doesn't need to do anything; the Phase 12 CLI will call `tracing_subscriber::fmt::init()` once. For Phase 4 development debugging, set `RUST_LOG=kri0k=debug` and run with a one-line `tracing_subscriber::fmt::init()` in a test harness. [CITED: tracing docs § "Using the macros"]

**Env var convention:** `RUST_LOG=kri0k=debug` (uses `tracing_subscriber`'s `EnvFilter`).

### Pattern 6: Test gating with `#[cfg(feature = "integration")]`

`crates/kri0k-core/Cargo.toml`:

```toml
[features]
default = []
integration = []
```

Then in test files:

```rust
// crates/kri0k-core/src/ttp/whois.rs (or tests/whois_integration.rs)

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ttp::subprocess::MockSubprocess;

    #[tokio::test]
    async fn parses_google_fixture() {
        // Unit test with mock — always runs
        let mock = Arc::new(MockSubprocess::from_fixture("tests/fixtures/whois_google_com.txt"));
        let ttp = WhoisTtp::new(mock);
        let result = ttp.execute("google.com", CancellationToken::new()).await.unwrap();
        // ... assertions
    }

    #[cfg(feature = "integration")]
    #[tokio::test]
    async fn real_whois_against_example_com() {
        // Integration test with real binary — only runs with --features integration
        let real = Arc::new(RealSubprocess::default());
        let ttp = WhoisTtp::new(real);
        let result = ttp.execute("example.com", CancellationToken::new()).await.unwrap();
        assert!(matches!(result, TtpOutput::Whois(_)));
    }
}
```

**Run commands:**

```bash
# Unit only (CI default)
cargo test --workspace

# Unit + integration (local, nightly CI)
cargo test --workspace --features integration

# Integration only
cargo test --workspace --features integration --test '*integration*'
```

[CITED: doc.rust-lang.org/cargo/reference/features.html § "Features and tests"]

**Alternative considered:** `#[ignore]` + `cargo test -- --ignored` works without features but loses discoverability (the test always shows as `ignored` in normal runs). Feature flag is clearer.

### Pattern 7: scope.yaml parser with lookahead schema

```rust
// crates/kri0k-core/src/scope.rs
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScopeConfig {
    pub version: u32,                              // Required; v1 only
    pub objective: Option<String>,                 // Phase 4: unused
    pub operator: Option<String>,                  // Phase 4: parsed but unused (M-42)

    #[serde(default)]
    pub targets: Vec<String>,                      // Phase 4: USED (D-48)

    #[serde(default)]
    pub targets_cidr: Vec<String>,                 // Phase 7: stub

    #[serde(default)]
    pub targets_wildcards: Vec<String>,            // Phase 7: stub

    #[serde(default)]
    pub safeguards: SafeguardsSection,             // Phase 4: USES propose_only only

    #[serde(default)]
    pub rate_limits: RateLimitsSection,            // Phase 4: parsed but unused (TTP-local rate-limit per D-45)

    #[serde(default)]
    pub audit_path: Option<String>,                // Phase 8: parsed but unused

    // Capture the raw YAML for hashing (D-58)
    #[serde(skip)]
    pub raw_yaml: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SafeguardsSection {
    #[serde(default = "default_true")]
    pub propose_only: bool,                        // D-49 default True
    #[serde(default)]
    pub kill_switch: bool,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RateLimitsSection {
    #[serde(default)]
    pub global_rps: Option<u32>,
    // Phase 7+ may add per-TTP overrides
}

const fn default_true() -> bool { true }

impl ScopeConfig {
    pub fn from_yaml(path: &Path) -> Result<Self, crate::Error> {
        let raw = std::fs::read_to_string(path).map_err(crate::Error::Io)?;
        let mut config: ScopeConfig = serde_yaml_ng::from_str(&raw).map_err(|e| {
            crate::Error::ParseError {
                source: "scope.yaml".to_string(),
                detail: e.to_string(),
            }
        })?;

        // D-58: reject unknown version
        if config.version != 1 {
            return Err(crate::Error::ParseError {
                source: "scope.yaml".to_string(),
                detail: format!("unsupported scope version: {} (expected 1)", config.version),
            });
        }

        config.raw_yaml = raw;
        Ok(config)
    }

    pub fn from_dict_value(value: serde_json::Value) -> Result<Self, crate::Error> {
        // Used by Engagement::new() when scope_dict comes from Python
        let raw = serde_yaml_ng::to_string(&value)
            .map_err(|e| crate::Error::ParseError {
                source: "scope_dict".to_string(),
                detail: e.to_string(),
            })?;
        let mut config: ScopeConfig = serde_json::from_value(value)
            .map_err(crate::Error::Json)?;
        if config.version != 1 {
            return Err(crate::Error::ParseError { /* ... */ });
        }
        config.raw_yaml = raw;
        Ok(config)
    }

    pub fn compute_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.raw_yaml.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// D-48: exact-match allowlist (Phase 4 scope check).
    /// Phase 7 will replace this with CIDR + wildcard expansion.
    pub fn validate_target(&self, target: &str) -> Result<(), crate::Error> {
        if self.targets.iter().any(|t| t == target) {
            Ok(())
        } else {
            Err(crate::Error::ScopeViolation {
                target: target.to_string(),
                reason: format!("target {target:?} not in scope.targets allowlist"),
            })
        }
    }
}
```

**Notes:**

- `serde_yaml_ng` API is identical to deprecated `serde_yaml` (drop-in replacement). [VERIFIED: github.com/acatton/serde-yaml-ng]
- `sha2 = "0.10"` is a tiny new dep (probably already transitively present via ulid; check `cargo tree`). If not, add it explicitly to `kri0k-core/Cargo.toml`.
- The `from_dict_value` constructor exists because `Engagement::new(scope_dict)` receives a Python dict, not a file path. Serialize the dict back to YAML for hashing consistency, then deserialize into `ScopeConfig`. (Alternative: hash the JSON form, but YAML hash matches what Phase 7 will hash from disk.)

### Pattern 8: Whois output heuristic parser

```rust
// crates/kri0k-core/src/ttp/whois.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WhoisOutput {
    pub registrant: Option<String>,
    pub registrar: Option<String>,
    pub nameservers: Vec<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub expires_at: Option<String>,
    pub raw_unparsed: Vec<String>,
}

/// Heuristic key:value parser for ICANN-style whois output (D-41).
/// Handles whitespace-leading lines, multiple Name Server lines.
/// Lines that don't match the key:value heuristic accumulate in raw_unparsed.
pub fn parse_whois_output(raw: &str) -> WhoisOutput {
    let mut out = WhoisOutput::default();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('%') || trimmed.starts_with('#') {
            continue;  // comment/banner
        }

        if let Some((key, value)) = trimmed.split_once(':') {
            let key_lower = key.trim().to_ascii_lowercase();
            let value = value.trim();
            if value.is_empty() { continue; }

            match key_lower.as_str() {
                // D-42: only Registrant Organization
                "registrant organization" => out.registrant = Some(value.to_string()),
                "registrar" => out.registrar = Some(value.to_string()),
                "name server" | "nserver" => {
                    let ns = value.to_ascii_lowercase();  // NS hostnames are case-insensitive
                    if !out.nameservers.contains(&ns) {
                        out.nameservers.push(ns);
                    }
                }
                "creation date" | "created" | "created on" => out.created_at = Some(value.to_string()),
                "updated date" | "last updated" | "modified" => out.updated_at = Some(value.to_string()),
                "registry expiry date" | "expiration date" | "expires on" => {
                    out.expires_at = Some(value.to_string())
                }
                _ => {
                    // Anything not in the recognized set
                    out.raw_unparsed.push(trimmed.to_string());
                }
            }
        } else {
            out.raw_unparsed.push(trimmed.to_string());
        }
    }

    out
}
```

**Verified against live Sysinternals output** (from this research session, 2026-05-18):

The exact `whois -v -nobanner google.com` output contains (from referral query to markmonitor.com):
- `Registrant Organization: Google LLC` -- matched by `registrant organization`
- `Registrar: MarkMonitor Inc.` -- matched by `registrar`
- `Name Server: NS1.GOOGLE.COM` (x4 separate lines) -- handled by deduplication
- `Creation Date: 1997-09-15T04:00:00Z` -- matched by `creation date`
- `Updated Date: 2019-09-09T15:39:04Z` -- matched by `updated date`
- `Registry Expiry Date: 2028-09-14T04:00:00Z` -- matched by `registry expiry date`

For `example.com` only the IANA registry layer responds (no Registrant Organization, no Registrar contact details), confirming the GDPR/RDAP redaction reality. Plan accordingly.

**Critical: The `-v` flag.** Without `-v`, the Sysinternals whois.exe queries only the registry (e.g., verisign for `.com`) which has been GDPR-redacted since 2018. With `-v`, it follows the referral to the registrar's thick whois server (e.g., markmonitor.com), which DOES return Registrant Organization. This research recommends `args = ["-v", "-nobanner", target]`.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | pytest 7.4+ (Python), cargo test (Rust) |
| Config file | `pyproject.toml` [tool.pytest.ini_options]; `Cargo.toml` workspace |
| Quick run command | `pytest tests/test_act_node.py -x` (Python) / `cargo test --package kri0k-core` (Rust) |
| Full suite command | `cargo test --workspace --features integration && pytest tests/` |
| Markers existing | `unit`, `integration`, `slow`, `ttp`, `graph`, `audit` (from `pyproject.toml`) |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AGENT-05 | `act` envia proposal para Rust, recebe outcome | unit | `pytest tests/test_act_node.py::test_act_calls_execute_proposal -x` | Wave 0 (new) |
| AGENT-05 | `act` honra `propose_only=True` (no Rust call) | unit | `pytest tests/test_act_node.py::test_act_propose_only_skips_rust -x` | Wave 0 (new) |
| TTP-01 | `WhoisTtp` implements `Ttp` trait | unit | `cargo test --package kri0k-core ttp::whois::tests::implements_trait` | Wave 0 (new) |
| TTP-02 | `WhoisTtp::execute` runs subprocess (with mock) | unit | `cargo test --package kri0k-core ttp::whois::tests::executes_via_mock_subprocess` | Wave 0 (new) |
| TTP-02 | `WhoisTtp::execute` runs real whois.exe | integration | `cargo test --features integration ttp::whois::tests::real_whois_smoke` | Wave 0 (new) |
| TTP-03 | Parser extracts registrar/NS/dates from fixture | unit | `cargo test --package kri0k-core ttp::whois::tests::parses_google_fixture` | Wave 0 (new) |
| TTP-03 | Parser handles redacted output (example.com) | unit | `cargo test --package kri0k-core ttp::whois::tests::handles_redacted_output` | Wave 0 (new) |
| TTP-04 | `Engagement::execute_proposal` adds 1 Domain + N Nameserver nodes | integration | `pytest tests/test_engagement_smoke.py::test_whois_grows_graph -m integration` | Wave 0 (new) |
| TTP-04 | Re-executing same target is idempotent (no duplicate nodes) | integration | `pytest tests/test_engagement_smoke.py::test_whois_idempotent -m integration` | Wave 0 (new) |
| TTP-05 | Rate limit: two consecutive calls take >= 1s | unit | `cargo test --package kri0k-core ttp::whois::tests::rate_limit_enforced` | Wave 0 (new) |
| D-50 | `Engagement::new` fails if whois absent | unit | `cargo test --package kri0k-core engagement::tests::fails_without_whois` (mock PATH) — OR document as manual smoke | Wave 0 (likely manual) |
| D-51 | 30s timeout kills subprocess | unit | `cargo test --package kri0k-core ttp::whois::tests::timeout_kills_child` (mock that sleeps 60s) | Wave 0 (new) |
| D-62 | `Engagement::kill` cancels in-flight execution | integration | `pytest tests/test_engagement_smoke.py::test_kill_cancels_execute -m integration` | Wave 0 (new) |
| D-48 | Scope violation rejected before subprocess | unit | `cargo test --package kri0k-core engagement::tests::scope_violation_short_circuits` | Wave 0 (new) |

### Sampling Rate
- **Per task commit:** `cargo test --package <touched crate>` + `pytest -k <touched test>`
- **Per wave merge:** `cargo test --workspace && pytest tests/ -m "not integration"`
- **Phase gate:** `cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --features integration && pytest tests/ && ruff check python/ tests/ && mypy python/kri0k`

### Wave 0 Gaps
- [ ] `tests/test_act_node.py` — covers AGENT-05 with mocked Engagement
- [ ] `tests/test_engagement_smoke.py` — covers TTP-04, D-62 (uses real Engagement, marker `integration`)
- [ ] `tests/fixtures/whois_google_com.txt` — captured `whois -v -nobanner google.com` output for parser fixtures
- [ ] `tests/fixtures/whois_example_com.txt` — captured `whois -v -nobanner example.com` output (redacted case)
- [ ] `tests/fixtures/whois_invalid.txt` — empty/malformed output to test parser robustness
- [ ] `crates/kri0k-core/tests/scope_yaml_integration.rs` (or inline mod) — scope.yaml parsing fixtures
- [ ] No new framework install needed — pytest, cargo test, and asyncio are already configured

## Known Pitfalls

### Pitfall 1: First-run EULA prompt on Sysinternals whois.exe (Windows)
**What goes wrong:** Without `-accepteula`, whois.exe prints the EULA to stdout and prompts for `Yes/No` on stdin. With a piped stdin (no terminal), it hangs forever waiting for input.
**Why it happens:** Sysinternals utilities check the registry key `HKCU\Software\Sysinternals\Whois\EulaAccepted` on launch. If absent, they expect interactive confirmation.
**How to avoid:** ALWAYS pass `-accepteula` as the first arg, or as a one-time bootstrap. Recommended Phase 4 approach: always include it in the args list — it's a no-op if already accepted.
**Warning signs:** Test process hangs for >5s with no output; first-time CI run on a fresh Windows machine never completes.
**Verified:** Tested in this research session — without `-accepteula`, the binary prints the full EULA license text to stdout and blocks.

### Pitfall 2: WHOIS data redaction (GDPR / ICANN sunset)
**What goes wrong:** Parser extracts only `Registrar` and `Name Server` fields; `registrant` is always `None`. Tests pass but the graph never gets `Organization` nodes.
**Why it happens:** Since GDPR (2018), gTLD registries (`.com`, `.net`, `.org`) return REDACTED whois data by default. The protocol was officially sunset on January 28, 2025 in favor of RDAP, but Sysinternals whois.exe still uses port 43 WHOIS. ICANN registry responses omit Registrant fields entirely.
**How to avoid:** USE THE `-v` FLAG. With `-v`, whois.exe follows the registry's referral to the *registrar's* whois server (e.g., markmonitor.com for google.com), which DOES return `Registrant Organization` for organizational registrants. Verified in this session: `whois -v -nobanner google.com` returns `Registrant Organization: Google LLC`; without `-v` it does NOT.
**Warning signs:** All integration tests against `.com` domains return `WhoisOutput { registrant: None, ... }`. Switch to `-v` and re-test.
**Trade-off:** `-v` doubles wall-clock time (two whois queries instead of one). With the 1 req/sec rate limit this is fine.

### Pitfall 3: Sysinternals whois.exe outputs OS-level error text in user's locale
**What goes wrong:** stderr (or stdout) contains a connection error in Portuguese / German / Chinese before the actual whois response, causing parser to either fail or produce noisy `raw_unparsed`.
**Why it happens:** The Windows WSA error messages are localized; Sysinternals dumps them prepended to whois server response. Verified in this session: `Uma tentativa de conexão falhou porque o componente conectado não respondeu corretamente após um período de tempo ou a conexão estabelecida falhou porque o host conectado não respondeu.` (Portuguese WSAETIMEDOUT-style message) appeared even on a successful run.
**How to avoid:** Parser already skips lines that don't match `key:` heuristic — they fall into `raw_unparsed`. Acceptable. Do NOT try to match error strings (locale-dependent). For tests, capture fixtures on an English-locale machine OR strip leading non-ASCII paragraphs from raw before parsing.
**Warning signs:** `raw_unparsed` grows unexpectedly large on Windows machines with non-English locales.

### Pitfall 4: `kill_on_drop(true)` + `tokio::select!` race on Windows
**What goes wrong:** Child process briefly orphans when cancel races with completion; on Windows, the `TerminateProcess` call from drop guard races with normal exit.
**Why it happens:** Tokio's `kill_on_drop` issues `TerminateProcess` from a destructor. If the child has already exited at the moment of drop, the call is a no-op (handle is invalid) — safe. If the child is mid-exit, the kill might succeed AFTER the wait_with_output completes — still safe but generates "process not found" log noise.
**How to avoid:** Pattern in Pattern 1 above uses `biased` select! plus explicit `start_kill()` + `wait().await` on the cancel branch, so the drop guard only runs as a safety net. Don't rely on it as the primary kill mechanism.
**Warning signs:** Intermittent Windows test failures showing "OS error 6: invalid handle" or "process not found" in stderr.

### Pitfall 5: `Box<dyn AuditSink>` is NOT `Sync` by default
**What goes wrong:** `Engagement` fails to compile as `#[pyclass]` — PyO3 requires `Sync`. `dyn AuditSink: Send` is given by the trait bound, but `Sync` is NOT.
**Why it happens:** The audit trait has `&mut self` methods (`log_ttp_execution(&mut self, event)`). `Box<dyn T>` is `Sync` only when `T: Sync`; trait with `&mut self` methods doesn't auto-imply Sync.
**How to avoid:** Wrap in `Mutex<Box<dyn AuditSink>>`. The Mutex provides interior mutability + Sync. Pattern 2 above shows this.
**Warning signs:** Compile error: `the trait bound 'dyn AuditSink: Sync' is not satisfied`.

### Pitfall 6: `async fn` in trait does NOT make `Box<dyn Trait>` work
**What goes wrong:** Migrating `trait Ttp` to native `async fn execute()` (Rust 1.85 stable native syntax) breaks `Box<dyn Ttp>` in the registry HashMap.
**Why it happens:** Native `async fn` returns `impl Future`, which has a different concrete type per implementor. The vtable can't store them. Rust 1.85 does NOT support `dyn` for async fn traits.
**How to avoid:** Use `#[async_trait]` macro from `async-trait` crate. It rewrites `async fn` -> `fn ... -> Pin<Box<dyn Future + Send>>`, which has a stable signature across impls.
**Warning signs:** `the trait Ttp cannot be made into an object`, `async fn in trait cannot be made into a trait object`.

### Pitfall 7: GIL deadlock when calling Python from Tokio worker thread
**What goes wrong:** Tokio task tries to acquire GIL via `Python::with_gil` while main thread holds GIL waiting for `block_on` to return -> deadlock.
**Why it happens:** PyO3 GIL acquisition is blocking. The Tokio runtime is multi-threaded (2 worker threads per pybridge config). If a worker thread needs the GIL while the main pyclass method holds it, it blocks.
**How to avoid:** Always wrap `runtime().block_on(...)` with `py.allow_threads(|| ...)`. Inside the async block, NEVER call back into Python. Pattern 2 above demonstrates the safe shape. If you must call Python from an async task, use `Python::with_gil` AFTER `allow_threads` releases it.
**Warning signs:** Test hangs indefinitely; process not killable except by SIGKILL.

### Pitfall 8: Sysinternals whois has limited args; passing `-accepteula` is undocumented
**What goes wrong:** Adding flags the binary doesn't recognize causes it to print usage to stderr and exit with non-zero.
**Why it happens:** Sysinternals whois.exe v1.21 official `/?` output ONLY documents `-v` and `-nobanner`. The `-accepteula` flag is a *Sysinternals-wide convention* (works on every Sysinternals tool) but not in this tool's help.
**How to avoid:** Use ONLY `["-v", "-nobanner", "-accepteula", target]` for Phase 4. Verified safe — tested in this session, the binary accepts `-accepteula` and proceeds normally. Treat any new flag as suspicious; check it against `whois /?` output before adding.
**Warning signs:** stderr shows `Whois v1.21 ... Usage: whois [-v] domainname [whois.server]` instead of WHOIS data.

### Pitfall 9: `serde_yaml` is deprecated; `serde_yml` has a security advisory
**What goes wrong:** Pulling in `serde_yaml` causes `cargo deny` warnings (or future failures); pulling in `serde_yml` triggers RUSTSEC-2025-0068.
**Why it happens:** David Tolnay archived `serde_yaml` on 2024-03-25; the version `0.9.34+deprecated` is the final release. The community fork `serde_yml` by Sebastien Rousseau has a documented unsoundness advisory (RUSTSEC-2025-0068).
**How to avoid:** Use `serde_yaml_ng = "0.10"` — drop-in API-compatible, actively maintained fork by Antoine Catton.
**Warning signs:** `cargo audit` / `cargo deny` reports advisories.

### Pitfall 10: `tracing` events are silently dropped without a subscriber
**What goes wrong:** Developer adds `#[instrument]` and `info!()` calls, runs tests, sees nothing in stdout — assumes tracing isn't working.
**Why it happens:** `tracing` is a tracing facade. Without a subscriber (`tracing_subscriber::fmt::init()`) registered, events go to `/dev/null`.
**How to avoid:** For Phase 4 dev/debug, add `tracing_subscriber` as `[dev-dependencies]` in `kri0k-core/Cargo.toml`. In tests that need to see output, call `let _ = tracing_subscriber::fmt::try_init();` in setup. The Phase 12 CLI will register a global subscriber. Document this in CONTRIBUTING.md.
**Warning signs:** `RUST_LOG=kri0k=debug cargo test ...` shows no output at all.

### Pitfall 11: `serde(default)` on a nested struct requires `Default` impl
**What goes wrong:** `ScopeConfig` declares `#[serde(default)] pub safeguards: SafeguardsSection` but `SafeguardsSection` doesn't have a sensible default.
**Why it happens:** `#[serde(default)]` on a field needs the type to implement `Default`. Phase 4 wants `propose_only: true` as default (D-49), which conflicts with `bool::default() == false`.
**How to avoid:** Implement `Default` manually with `propose_only: true`, OR use `#[serde(default = "default_safeguards")]` with a custom function. Pattern 7 above uses `#[serde(default = "default_true")]` on the field directly inside `SafeguardsSection`, then derives `Default` on the struct — which still gives `propose_only: false` if the WHOLE `safeguards:` section is absent. Either accept that footgun OR write a manual `Default` impl on `SafeguardsSection` that returns `propose_only: true`.
**Warning signs:** Integration tests with minimal scope.yaml unexpectedly execute TTPs (propose_only is false because YAML omitted the section).

### Pitfall 12: PyO3 0.24 `#[pyclass]` requires `Sync` (changed from earlier versions)
**What goes wrong:** Engagement holds `Cell<X>` or single-threaded primitives -> compile error.
**Why it happens:** PyO3 0.24 docs: "#[pyclass] types are now required to be Sync."
**How to avoid:** All fields must be `Send + Sync`. `Mutex<T>` where `T: Send` provides both. `Cell`/`RefCell` are NOT Sync — use `Mutex`. `Rc` is not Send — use `Arc`.
**Warning signs:** `'X' cannot be shared between threads safely` on the `#[pyclass]` line.

### Pitfall 13: Test working directory for fixture paths
**What goes wrong:** `MockSubprocess::from_fixture("tests/fixtures/whois_google_com.txt")` fails because cargo test runs from the workspace root, but the fixture path is relative to the crate root.
**Why it happens:** "The working directory when running each unit and integration test is set to the root directory of the package the test belongs to." [CITED: cargo book § cargo test working dir]
**How to avoid:** Use `concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/whois_google_com.txt")` for absolute paths, or put fixtures inside `crates/kri0k-core/tests/fixtures/`.
**Warning signs:** File-not-found errors in CI but not locally (or vice-versa, depending on which directory you ran from).

## File-by-File Implementation Manifest

### Rust changes

**`crates/kri0k-core/Cargo.toml`** -- ADD dependencies and feature flag:
- Add: `serde_yaml_ng.workspace = true`, `tokio = { workspace = true, features = ["process", "time", "sync", "macros"] }`, `tokio-util.workspace = true`, `which.workspace = true`, `async-trait.workspace = true`, `tracing.workspace = true`, `sha2 = "0.10"` (if not transitively present).
- Add `[features] integration = []` section.
- Add `[dev-dependencies] tokio = { workspace = true, features = ["macros", "rt-multi-thread"] }`, `tracing-subscriber = "0.3"`.

**`crates/kri0k-core/src/lib.rs`** -- EXTEND Error enum:
- Add variants per D-53: `ScopeViolation { target, reason }`, `RateLimitExceeded { ttp_id, retry_in_ms }`, `SubprocessTimeout { ttp_id, timeout_ms }`, `ParseError { source, detail }`, `MissingDependency { binary }`, `UnknownTtp { ttp_id }`, `Io(#[from] std::io::Error)`, `Cancelled` (D-62).
- Existing `Json` and `Generic` variants stay.
- Update `mod ttp;` to `pub mod ttp;` if not already; restructure as needed.

**`crates/kri0k-core/src/scope.rs`** -- REPLACE STUB with allowlist exact-match + YAML parser:
- New `ScopeConfig` struct per Pattern 7 (lookahead schema).
- `from_yaml(path: &Path) -> Result<Self, Error>`.
- `from_dict_value(value: serde_json::Value) -> Result<Self, Error>` for Python-dict input.
- `compute_hash(&self) -> String` (SHA-256 of raw YAML).
- `validate_target(&self, target: &str) -> Result<(), Error>` (Phase 4 = exact-match in `targets`).
- Delete the old `Scope` struct, `Scope::from_yaml`, `validate_target` free function.
- Tests: scope.yaml parses; missing `version` rejected; `version: 2` rejected; `targets: []` rejects every target; exact match accepts; non-match returns `ScopeViolation`.

**`crates/kri0k-core/src/audit.rs`** -- RENAME `NoOpAuditSink` to `NoopAuditSink` (D-38 spec). Otherwise no change in Phase 4 (real impl is Phase 8).

**`crates/kri0k-core/src/ttp.rs`** -- PROMOTE to `src/ttp/mod.rs`, REWRITE trait as async:
- Delete file; create `src/ttp/mod.rs` containing the async `Ttp` trait (Pattern 3).
- Keep `RateLimits`, `RiskLevel`, `ExecutionPlan`, `DryRunOutput` types if reused (some can be deleted as Phase 4 doesn't use them).
- Add `pub mod subprocess;` and `pub mod whois;`.
- Add `TtpOutput` enum per Pattern 3.

**`crates/kri0k-core/src/ttp/subprocess.rs`** -- NEW per D-54:
- `Subprocess` trait per Pattern 1.
- `RealSubprocess` impl using `tokio::process::Command` per Pattern 1.
- `MockSubprocess` impl that reads fixture path and returns `SubprocessOutput { stdout: fixture_contents, stderr: "", exit_code: Some(0) }`. Constructor `MockSubprocess::from_fixture(path: PathBuf) -> Self`. For timeout/cancel test paths, add `MockSubprocess::hanging()` that returns a future which never completes.

**`crates/kri0k-core/src/ttp/whois.rs`** -- NEW:
- `WhoisOutput` struct per Pattern 8.
- `WhoisTtp { subprocess: Arc<dyn Subprocess>, last_call: Mutex<Instant> }` struct.
- `impl WhoisTtp::new(subprocess: Arc<dyn Subprocess>) -> Self`.
- `#[async_trait] impl Ttp for WhoisTtp` with `id() = "T1590.001"`, `description() = "whois reconnaissance (MITRE T1590.001)"`, `rate_limits() = RateLimits { max_rps: Some(1), ... }`, `default_timeout() = Duration::from_secs(30)`, `execute(target, cancel)` that:
  1. Locks `last_call`, computes wait, sleeps if needed, updates `last_call`.
  2. Calls `subprocess.run("whois", &["-v", "-nobanner", "-accepteula", target], self.default_timeout(), cancel.clone())`.
  3. On success: `parse_whois_output(stdout)` -> `WhoisOutput`.
  4. Returns `Ok(TtpOutput::Whois(...))`.
- `parse_whois_output(raw: &str) -> WhoisOutput` per Pattern 8.
- Inline `#[cfg(test)] mod tests`:
  - `parses_google_fixture` (unit, with MockSubprocess + fixture)
  - `handles_redacted_output` (unit, example.com fixture)
  - `rate_limit_enforced` (unit, two calls with mock, assert >= 1s elapsed)
  - `timeout_kills_child` (unit, MockSubprocess::hanging, assert Err(SubprocessTimeout))
  - `cancellation_returns_cancelled` (unit, hang mock + cancel after 100ms)
  - `#[cfg(feature = "integration")] real_whois_smoke` -- live execute against example.com

### Graph changes

**`crates/kri0k-graph/src/lib.rs`** -- EXTEND enums per D-39, D-40:
- `NodeKind` add variants: `Domain { name: String }`, `Organization { name: String }`, `Nameserver { hostname: String }`.
- `EdgeKind` add variants: `RegisteredBy`, `HasNameserver`.
- Existing variants stay (no breaking change for Phase 1-3 tests).
- Update existing tests if any pattern-match exhaustively on the enum.
- No new tests strictly required; the graph changes are pure data shape — exercised by Engagement integration tests.

### Pybridge changes

**`crates/kri0k-pybridge/Cargo.toml`** -- ADD: `tokio-util.workspace = true`, `tracing.workspace = true`.

**`crates/kri0k-pybridge/src/lib.rs`** -- ADD Engagement pyclass per Pattern 2:
- Keep existing `hello()`, `get_dummy_graph()`, `TOKIO_RUNTIME`, `runtime()` unchanged.
- Add `#[pyclass] Engagement` struct (Pattern 2 fields).
- Add `#[pymethods] impl Engagement` with `#[new]`, `snapshot()`, `execute_proposal()`, `scope_hash()`, `kill()`.
- Helper: `apply_whois_output(&self, target: &str, output: &TtpOutput) -> Result<GraphDelta>` that:
  - Acquires `self.dedupe.lock()` and `self.graph.lock()`.
  - For each of (Domain target, Organization registrant, Nameserver each): check dedupe cache; if absent, add node + insert into cache; if present, update `metadata["last_whois_at"]`.
  - For each (Domain -> Organization, Domain -> Nameserver) edge: insert if not in dedupe-edge-cache.
  - Returns `GraphDelta { nodes_added: usize, edges_added: usize }`.
- Helper: `build_outcome(status: &str, result: Option<TtpOutput>, error: Option<&Error>, delta: GraphDelta) -> serde_json::Value` to build the D-47 dict shape.
- Helper: `error_to_outcome(e: &Error) -> serde_json::Value` that maps Error variants -> status strings (`Error::ScopeViolation` -> `"scope_violation"`, etc.).
- Update `#[pymodule] fn _native` to register `m.add_class::<Engagement>()?;`.

### Python changes

**`python/kri0k/_native.pyi`** -- ADD stubs for Engagement:
```python
class Engagement:
    def __new__(cls, scope_dict: dict[str, Any]) -> "Engagement": ...
    def snapshot(self) -> dict[str, Any]: ...
    def execute_proposal(self, proposal: dict[str, Any]) -> dict[str, Any]: ...
    def scope_hash(self) -> str: ...
    def kill(self) -> None: ...
```

**`python/kri0k/engagement.py`** -- NEW per D-36:
```python
"""Engagement bootstrap helper.

Creates an Engagement instance and injects it into AgentState.engagement_context
before graph.invoke().
"""
from typing import Any
from kri0k import _native

def create(
    scope_dict: dict[str, Any],
    objective: str,
    propose_only: bool = True,
) -> dict[str, Any]:
    """Create an Engagement and return the engagement_context dict.

    Args:
        scope_dict: Parsed scope.yaml as dict (must contain version, targets).
        objective: Engagement objective (free-form).
        propose_only: If True (default), act node returns proposals without execution.

    Returns:
        Dict suitable as engagement_context for AgentState.

    Raises:
        RuntimeError: If whois binary not found (D-50 fail-fast).
    """
    engagement = _native.Engagement(scope_dict)
    return {
        "engagement": engagement,
        "objective": objective,
        "propose_only": propose_only,
        "scope_hash": engagement.scope_hash(),
    }
```

**`python/kri0k/agent/nodes/act.py`** -- REWRITE per D-49, D-56:
```python
"""Act node: gate propose_only and execute via Engagement."""
import asyncio
from typing import Any
from kri0k.agent.state import AgentState


async def act(state: AgentState) -> dict[str, Any]:
    """Act node: execute approved proposal through Rust Engagement.

    If propose_only=True (default), returns a proposed status without
    invoking Rust. If False, dispatches to engagement.execute_proposal via
    asyncio.to_thread to avoid blocking the event loop.
    """
    proposal = state["proposal"]
    if not proposal:
        return {}

    context = state["engagement_context"]
    propose_only = context.get("propose_only", True)
    iteration = state["iteration_count"]

    if propose_only:
        history_entry = {
            "iteration": iteration,
            "ttp_id": proposal.get("ttp_id", ""),
            "target": proposal.get("target", ""),
            "status": "proposed",
            "summary": f"propose-only: would_execute {proposal.get('ttp_id', '')} on {proposal.get('target', '')}",
            "graph_delta": {"nodes_added": 0, "edges_added": 0},
            "audit_id": None,
        }
        return {
            "decision": {"status": "proposed", "would_execute": proposal.get("ttp_id", ""), "proposal": proposal},
            "history": state["history"] + [history_entry],
        }

    engagement = context.get("engagement")
    if engagement is None:
        raise RuntimeError("act: engagement missing from engagement_context (run kri0k.engagement.create first)")

    outcome = await asyncio.to_thread(engagement.execute_proposal, proposal)

    delta = outcome.get("graph_delta", {"nodes_added": 0, "edges_added": 0})
    summary = _format_summary(proposal, outcome, delta)
    history_entry = {
        "iteration": iteration,
        "ttp_id": proposal.get("ttp_id", ""),
        "target": proposal.get("target", ""),
        "status": outcome.get("status", "error"),
        "summary": summary,
        "graph_delta": delta,
        "audit_id": outcome.get("audit_id"),
    }
    return {
        "decision": outcome,
        "history": state["history"] + [history_entry],
    }


def _format_summary(proposal: dict[str, Any], outcome: dict[str, Any], delta: dict[str, int]) -> str:
    """Build human-readable summary per D-56."""
    status = outcome.get("status", "error")
    target = proposal.get("target", "<unknown>")
    ttp_id = proposal.get("ttp_id", "<unknown>")
    if status == "executed":
        nodes = delta.get("nodes_added", 0)
        if nodes == 0:
            return f"{ttp_id} {target} -> no new nodes (already known)"
        result = outcome.get("result") or {}
        # Count by kind from result.added_breakdown if Rust provides it; otherwise generic
        return f"{ttp_id} {target} -> +{nodes} nodes +{delta.get('edges_added', 0)} edges"
    return f"{ttp_id} {target} -> {status}: {outcome.get('error', '')}".strip()
```

**`python/kri0k/agent/nodes/sense.py`** -- MODIFY (backward-compat):
```python
async def sense(state: AgentState) -> dict[str, Any]:
    """Sense node: fetch and format the current graph snapshot.

    Prefers engagement.snapshot() when an Engagement is present in context
    (Phase 4); falls back to _native.get_dummy_graph() to preserve Phase 1/2
    test compatibility.
    """
    engagement = state.get("engagement_context", {}).get("engagement")
    if engagement is not None:
        raw = engagement.snapshot()
    else:
        raw = _native.get_dummy_graph()
    formatted = format_snapshot_hybrid(raw)
    return {"snapshot": {"raw": raw, "formatted": formatted}}
```

### Tests

**`tests/test_act_node.py`** -- NEW (8-12 tests):
- `test_act_returns_empty_when_no_proposal` -- empty proposal -> {}.
- `test_act_propose_only_skips_rust` -- propose_only=True -> decision.status=="proposed", no engagement.execute_proposal call (mock engagement).
- `test_act_propose_only_appends_history` -- history entry shape matches D-56.
- `test_act_calls_execute_proposal_when_not_propose_only` -- propose_only=False -> mock engagement called with proposal dict.
- `test_act_outcome_appended_to_history` -- history entry contains outcome.status, graph_delta, audit_id.
- `test_act_raises_when_engagement_missing` -- propose_only=False but no engagement in context -> RuntimeError.
- `test_act_summary_executed_format` -- summary matches expected string.
- `test_act_summary_idempotent_format` -- nodes_added=0 -> "no new nodes (already known)".

**`tests/test_engagement_smoke.py`** -- NEW (`@pytest.mark.integration`):
- `test_engagement_construct_with_minimal_scope` -- create Engagement with scope dict containing version+targets, snapshot() returns empty graph.
- `test_engagement_rejects_unknown_version` -- scope dict with version=2 -> RuntimeError.
- `test_engagement_fails_without_whois` -- skip if whois present; otherwise assert RuntimeError contains "whois binary not found".
- `test_whois_grows_graph` -- propose example.com (in scope), execute_proposal, snapshot shows Domain + Nameservers.
- `test_whois_idempotent` -- two consecutive execute_proposal calls -> second has nodes_added==0.
- `test_kill_cancels_execute` -- kick off execute_proposal in thread, call kill() within 100ms, expect status=="error" with cancellation message.
- `test_scope_violation_short_circuits` -- target outside scope -> status=="scope_violation", no subprocess invoked.

**`tests/fixtures/whois_google_com.txt`** -- NEW: capture from `whois -v -nobanner -accepteula google.com > tests/fixtures/whois_google_com.txt`. Inspect to verify contains Registrant Organization, Name Server x4, Creation Date, Updated Date, Registry Expiry Date.

**`tests/fixtures/whois_example_com.txt`** -- NEW: capture from `whois -v -nobanner -accepteula example.com > tests/fixtures/whois_example_com.txt`. Used to test redacted-output path (no Registrant Organization).

**`tests/fixtures/whois_invalid.txt`** -- NEW: hand-crafted empty/garbage content for parser robustness test.

### Documentation

**`README.md`** -- ADD section "Running the whois TTP":
- Pre-requisite: install Sysinternals whois.exe (`winget install Microsoft.Sysinternals.Whois`) or `apt install whois`.
- Minimal `scope.yaml`:
  ```yaml
  version: 1
  operator: you@example.com
  targets:
    - example.com
  safeguards:
    propose_only: false
  ```
- Example end-to-end snippet (Python):
  ```python
  from kri0k.engagement import create
  from kri0k.agent import get_graph
  context = create(scope_dict, objective="recon example.com", propose_only=False)
  graph = get_graph()
  state = {..., "engagement_context": context}
  result = await graph.ainvoke(state)
  ```

**`CONTRIBUTING.md`** -- ADD section "Adding a new TTP":
- Pattern: implement `Ttp` trait, accept `Subprocess` in constructor, register in `Engagement::new()`.
- Test pattern: fixture file in `tests/fixtures/`, MockSubprocess in unit tests, `#[cfg(feature = "integration")]` for real-binary tests.
- Document `cargo test --features integration` and `pytest -m integration`.

**`CHANGELOG.md`** -- NEW file, `## 0.2.0 -- Phase 4 (Act + TTP Whois)` entry listing:
- Added: `Engagement` pyclass, `WhoisTtp`, `ScopeConfig` (allowlist), `Subprocess` abstraction, `tracing` instrumentation.
- Changed: `Ttp` trait is now async (via `async-trait`); `NodeKind`/`EdgeKind` gained variants.
- Deferred: full scope validation (Phase 7), real audit log (Phase 8), interactive TUI approval (Phase 11).

**`docs/adr/ADR-0013-ttp-trait-subprocess-abstraction.md`** -- NEW: documents the Trait `Ttp` + `Subprocess` abstraction pattern, alternatives considered (direct std::process — rejected for testability; pyo3-asyncio — deferred), consequences.

**`config/scope.example.yaml`** -- UPDATE to v1 schema (Pattern 7):
```yaml
version: 1
operator: operator@example.com
objective: "recon for example domains"
targets:
  - example.com
targets_cidr: []        # Phase 7 stub
targets_wildcards: []   # Phase 7 stub
safeguards:
  propose_only: true
  kill_switch: false
rate_limits:
  global_rps: null
audit_path: null         # Phase 8 stub
metadata:
  engagement_id: ENGAGE-2026-001
  client: Example Corp
  authorized_by: security-team@example.com
llm:
  model: deepseek-r1:32b
```

### CI/CD

**`.github/workflows/ci.yml`** -- ENSURE: default `cargo test --workspace` (NO `--features integration`) and `pytest tests/ -m "not integration"`. The integration tests require `whois.exe` which may not be on the runner.

**`.github/workflows/nightly-integration.yml`** -- OPTIONAL NEW: scheduled nightly run with `whois` installed and `cargo test --features integration && pytest tests/`. Document in CONTRIBUTING.md as "if you add a new TTP integration test, verify it passes locally via `cargo test --features integration` before merge; the nightly CI run is a safety net".

### Task workflow artifact

**`.claude/tasks.md`** -- ADD TASK-015 entry per D-59 (major aggregated, branch `feat/phase-4-act-ttp-whois`).
**`.claude/registry.md`** -- ADD one aggregated entry after phase merge referencing TASK-015 with summary of changed crates and test counts.

## Open Questions for Planner

1. **Atomic commits vs feature-scoped commits across 3 crates.** The 12 commits listed in D-59 are feature-scoped (e.g., `feat(graph): add NodeKind variants`). The planner needs to decide ordering for clean compilation per commit. Recommended order: (a) graph enum expansion (compiles standalone), (b) core deps + Error enum + Subprocess trait, (c) Ttp async refactor + WhoisTtp + parser, (d) scope.yaml, (e) NoopAuditSink rename, (f) Engagement pyclass, (g) Python helpers + act.py rewrite + sense.py modify, (h) tests/fixtures, (i) tracing instrument, (j) docs. Each commit should pass `cargo check --workspace`.

2. **Test-first or impl-first per task?** This codebase doesn't have an explicit TDD norm. Recommendation: write the fixture files FIRST (`whois_google_com.txt`, `whois_example_com.txt`) capturing live output, then write parser tests, then implement parser. For Engagement, write a structural test ("can be constructed with minimal scope") before the full impl. For the cross-language smoke test, write impl + integration test together (otherwise the test can't even import the pyclass).

3. **Where does `MockSubprocess::hanging()` go?** A mock that returns a future which never completes is needed for timeout tests. Option A: separate `MockSubprocess` mode (constructor variant). Option B: separate `HangingSubprocess` struct. Planner picks; option A keeps the test surface narrow.

4. **Does `parse_whois_output` need to return `Result<WhoisOutput, Error>` or just `WhoisOutput`?** Pattern 8 returns `WhoisOutput` directly (parser is total — never panics, missing fields stay None). Planner can keep it total or wrap in Result for consistency with future fail-cases. Total is simpler and matches D-41 (heuristic, never fatal).

5. **Apply graph delta inside the `block_on` or outside?** Pattern 2 sketches it inside (inside the async block). Alternative: return `TtpOutput` from the async block and apply mutations after, on the sync side. Either works; inside-block is cleaner because the `outcome` map building can happen in one place.

6. **`engagement.create` arg shape — flat or kwargs?** D-36 says `create(scope_dict, objective, propose_only=True)`. Planner could prefer `create(scope_dict, *, objective, propose_only=True)` (kwargs-only after first positional) for clarity. Recommendation: match D-36 exactly.

7. **Does `Engagement::new` take a dict or a path?** D-36 implies dict (Python helper passes a parsed dict). Pattern 2 / Pattern 7 sketch supports BOTH (`from_yaml(path)` for future CLI use + `from_dict_value` for Engagement::new). Confirm dict-only for Phase 4; defer path entry to Phase 12 CLI.

8. **Cancellation semantics — does `kill()` reset the token or is the engagement single-shot after kill?** `CancellationToken::cancel()` is one-way; once cancelled, all `cancelled()` futures resolve immediately forever. After `kill()`, the next `execute_proposal` call would immediately return Cancelled. Planner decides: (a) accept single-shot engagement (kill = terminal), (b) replace the token on each execute. Recommendation: single-shot for MVP; document in `Engagement.kill()` docstring.

## Project Constraints (from CLAUDE.md)

**From `./CLAUDE.md` (project root):**
- Rust: clippy strict; `unwrap_used`, `panic`, `unimplemented` denied; no `unsafe`. `thiserror` for errors; `serde` for cross-boundary types.
- Python: ruff strict (E, W, F, I, N, UP, ANN, S, B); mypy strict; type hints required for public functions; `_native.pyi` stubs for Rust bindings.
- Build: `maturin develop` for dev, `maturin build --release` for release.
- Tests: `cargo test --workspace && pytest`.
- Scope validation: all targets must be in scope.yaml before execution (relevant: D-48).
- Audit logging: all TTP executions logged with hash chain (deferred: Phase 8; slot reserved in D-38).
- External deps: Ollama (not used in Phase 4), whois CLI (Phase 4 introduces).
- Linux is primary target, but Phase 4 development is happening on Windows — Pattern 1/Pitfalls 1-4 cover Windows-specific concerns.

**From `.claude/CLAUDE.md` (local framework):**
- **Trava de segurança incondicional:** No file creation/modification without (1) task registered in `tasks.md`, (2) declared mode (Desenvolvimento/Review/Tutor), (3) codebase recognized, (4) registry.md read. The planner MUST instruct the executor to register TASK-015 before any code change.
- **Commits:** strictly `type(scope): subject`, one line, no body, NO `Co-authored-by`, 10-100 chars. Conventional Commits types only.
- **Branches:** `type/TASK-NNN-short-desc`. Branch for Phase 4 is `feat/phase-4-act-ttp-whois` (per D-59).
- **Regra 2 (não invente APIs):** The planner must verify each crate exists at the documented version before specifying it in a task. This research did that verification; the version table above is authoritative.
- **Regra 3 (não toque fora do escopo):** Phase 4 explicitly defers Phase 7 (CIDR scope) and Phase 8 (audit) — the planner must not let tasks creep into those.
- **Regra 4 (não silencie erros):** The Error enum expansion (D-53) plus the Outcome status enum (D-47) together ensure errors are typed and visible. Don't `let _ = ...` returned Results.
- **Regra 8/9:** After each task: post-implementation evaluation + update registry.md. Plan must include a final "registry update" task.
- **Modes:** This phase will run in Desenvolvimento mode. The executor must declare mode at start.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `serde_yaml_ng` 0.10 is API-compatible with `serde_yaml` 0.9 | Stack table | Plan changes one line per use site; low risk |
| A2 | Sysinternals whois.exe accepts `-accepteula` as a flag even though `/?` doesn't list it | Pitfall 8 | **Verified in this session — confirmed accepted** |
| A3 | Sysinternals whois `-v` flag returns Registrant Organization for organizational gTLD registrants | Pattern 8 / Pitfall 2 | **Verified in this session — confirmed for google.com** |
| A4 | `winget install Microsoft.Sysinternals.Whois` provides the same `whois.exe` v1.21 | Stack / Pitfall 8 | **Verified in this session — the binary at user PATH is v1.21** |
| A5 | `kill_on_drop(true)` on Windows uses `TerminateProcess`, not graceful close | Pattern 1 | Low — both terminate the child; no data integrity concern for whois (read-only) |
| A6 | `async-trait`-rewritten methods are dyn-compatible with `Box<dyn Ttp>` | Pattern 3 | Documented behavior; well-attested in async-trait docs |
| A7 | PyO3 0.24 `#[pyclass]` Sync requirement is satisfied by `Mutex<T> where T: Send` | Pattern 2 / Pitfall 12 | HIGH confidence from PyO3 docs |
| A8 | `tracing_subscriber` doesn't need to be a Phase 4 runtime dep (only dev-dep) | Pitfall 10 | Low — silent drop is the documented no-subscriber behavior |
| A9 | `pytest-asyncio` strict mode plays well with `asyncio.to_thread` for mocked engagement | Test plan | Verified in Phase 1-3 tests already use `pytest.mark.asyncio` |
| A10 | `sha2 = "0.10"` does not already exist in the workspace dependency graph | Manifest | Low — `cargo tree | grep sha2` confirms presence/absence; if present transitively, no add needed |

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Build | YES | 1.95.0 (>= 1.85 MSRV) | — |
| Cargo | Build | YES | 1.95.0 | — |
| Python | Test/run | YES | 3.12.13 (>= 3.11) | — |
| maturin | Build PyO3 ext | EXPECTED (per pyproject.toml dev dep) | >= 1.7 | `pip install maturin` |
| `whois.exe` (Sysinternals) | TTP runtime | YES | v1.21 at `C:\Users\lucas\AppData\Local\Microsoft\WinGet\Links\whois.exe` | None — required (D-50 fail-fast) |
| Ollama | LLM (Phase 2) | UNVERIFIED (not needed for Phase 4 implementation) | — | n/a for Phase 4 |
| Internet access | Real whois queries | EXPECTED | — | MockSubprocess covers all unit tests; integration tests need it |

**Missing dependencies with no fallback:** None for Phase 4 implementation work. Whois.exe is required at runtime per D-50, but it IS installed locally.

**Missing dependencies with fallback:** None.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | NO | No user auth in Phase 4 (operator identity from scope.yaml `operator:` field, parsed but not enforced until Phase 12 CLI) |
| V3 Session Management | NO | No HTTP sessions |
| V4 Access Control | YES | scope.yaml allowlist (D-48); propose_only gate (D-49) |
| V5 Input Validation | YES | Domain regex (D-63 layer 2); scope.yaml schema validation (D-58); `Proposal` JSON shape check |
| V6 Cryptography | YES | SHA-256 for scope_hash (well-tested algorithm in `sha2` crate); no hand-rolled crypto |
| V7 Error Handling | YES | Typed `Error` enum with explicit variants (D-53); no `unwrap`/`panic` (clippy denies); errors propagate to outcome dict with status string |
| V8 Data Protection | PARTIAL | scope_hash detects tampering. Audit log encryption deferred to Phase 8. |
| V9 Communications | NO | Whois is unencrypted port 43 (protocol-level reality); not in MVP scope to wrap. |
| V12 Files & Resources | YES | `Command::arg()` no shell (D-63 layer 3); `tokio::process` no shell interpolation |
| V14 Configuration | YES | Strict scope.yaml schema with version field; no shell-style env interpolation |

### Known Threat Patterns for {Rust subprocess + Python LangGraph}

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Command injection via target string | Tampering | `Command::arg()` (Rust default, no shell expansion) — D-63 layer 3 |
| Out-of-scope execution | Information Disclosure | Allowlist exact-match before subprocess — D-48 (M-02) |
| LLM prompt injection causes harmful target | Tampering | Defense in depth — allowlist + domain regex + no-shell exec — D-63 (M-15, AB-03) |
| Unauthorized execution of TTP | Elevation of Privilege | propose_only default True — D-49 (M-05) |
| Subprocess never terminates (DoS) | Denial of Service | 30s timeout + kill_on_drop — D-51, Pattern 1 (M-34) |
| Operator cannot stop runaway agent | Denial of Service | CancellationToken via `Engagement.kill()` — D-62 (M-36) |
| Audit log tampering | Repudiation | Slot reserved with NoopAuditSink in Phase 4; real hash chain in Phase 8 — D-38 (M-22) |
| scope.yaml mid-engagement tampering | Tampering | scope_hash computed once at Engagement::new() and embedded in snapshot — Pattern 7 (M-03) |
| Subprocess output contains malicious unicode | Tampering | Parser uses `from_utf8_lossy`; non-key:value lines go to `raw_unparsed` (not interpreted) |
| Stale dependency (RUSTSEC) | Information Disclosure | Chose `serde_yaml_ng` over `serde_yml` (RUSTSEC-2025-0068) — Stack table |

### Phase 4 -> Threat Model Coverage Mapping

(Mirror of D-61 table for completeness — the planner can copy into VERIFICATION.md.)

| Mitigation | Threat | Phase 4 decision/code | Status |
|---|---|---|---|
| M-02 | Scope violation | D-48 allowlist + `validate_target` chamado em `execute_proposal` | Partial (exact-match; Phase 7 completa) |
| M-05 | LLM triggers TTPs sem aprovação | D-49 propose-only flag default `True` | Covered (Phase 4 boolean; Phase 11 interactive) |
| M-15 | LLM executa direto | D-34 Engagement pyclass + D-35 façade mínimo | Covered |
| M-21 | TTP destrutivo | Whois é Safe; não aplica | N/A |
| M-34 | TTP sem rate limit | D-45 TTP-local `Mutex<Instant>` | Covered |
| M-36 | Kill switch | D-50 fail-fast `which()` + D-62 `CancellationToken` | Covered |
| AB-03 | Prompt injection out-of-scope | D-63 defense in depth | Covered |

## Sources

### Primary (HIGH confidence)
- `crates.io` API for tokio-util, which, async-trait, tracing, serde_yaml_ng (version + MSRV verified 2026-05-18)
- `docs.rs/tokio-util` CancellationToken docs (select! pattern)
- `docs.rs/tokio/latest/tokio/process/struct.Child.html` (kill, wait_with_output, kill_on_drop semantics)
- `docs.rs/which` (API surface, error variants)
- `docs.rs/tracing/latest/tracing/attr.instrument.html` (instrument macro, async support, skip syntax)
- `pyo3.rs/v0.24.0/class/thread-safety` (Send/Sync requirements for pyclass)
- `pyo3.rs/v0.24.0` migration guide
- Microsoft Sysinternals whois docs: `learn.microsoft.com/en-us/sysinternals/downloads/whois`
- Live whois.exe v1.21 output captured experimentally in this research session (google.com and example.com)
- `doc.rust-lang.org/cargo/reference/features.html` (feature flags + cfg gating)
- `rustsec.org/advisories/RUSTSEC-2025-0068.html` (serde_yml unsoundness)
- ADR-0001, ADR-0005, ADR-0006, ADR-0012 (in-repo)
- `docs/security/THREAT_MODEL.md` (in-repo) — mitigations M-02, M-05, M-15, M-21, M-34, M-36, AB-03

### Secondary (MEDIUM confidence)
- `cybernetist.com/2024/04/19/rust-tokio-task-cancellation-patterns/` (tokio cancellation patterns blog)
- `smallcultfollowing.com/babysteps/blog/2025/03/24/box-box-box/` (dyn async traits roadmap)
- `users.rust-lang.org` forum threads on serde_yaml deprecation
- `github.com/PyO3/pyo3/discussions/3045` (GIL deadlock pattern with Tokio)

### Tertiary (LOW confidence — flagged for validation)
- Exact semantics of `tokio::process::Child::start_kill()` on Windows (TerminateProcess vs WM_CLOSE) — works in practice for our use case (whois is read-only and quick) but not relevant to data integrity
- ICANN RDAP sunset details — informational only; Phase 4 still uses WHOIS over port 43 via Sysinternals binary

## Metadata

**Confidence breakdown:**
- Standard stack (versions, MSRV, advisories): HIGH — crates.io and rustsec.org cross-verified
- Architecture patterns (pyclass, async-trait, CancellationToken, tracing): HIGH — official docs + verified in-repo by inspection of `get_dummy_graph` reference
- Whois output parsing (registrant fields, GDPR redaction): HIGH — experimentally verified against live `whois -v -nobanner google.com` in this session
- Pitfalls (EULA, kill_on_drop, GIL deadlock): HIGH — combined experimental verification + multi-source citation
- File-by-file manifest: HIGH — derived from CONTEXT.md decisions; no speculation
- Validation architecture: HIGH — maps each requirement to a concrete test command

**Research date:** 2026-05-18
**Valid until:** 2026-06-17 (30 days; revisit if Rust toolchain bump, PyO3 0.25 release, or serde_yaml_ng deprecation)

## RESEARCH COMPLETE

Phase 4 ready for planning. Key technical patterns documented in sections 1-8.
Crates to add: tokio-util 0.7, which 8, async-trait 0.1, tracing 0.1, serde_yaml_ng 0.10 (NOT serde_yaml or serde_yml — both deprecated/unsound).
Estimated plan count: 5 plans across 2-3 waves (graph data model + ttp/subprocess refactor parallel in Wave 0; WhoisTtp + Engagement pyclass in Wave 1; Python wiring + docs in Wave 2).
