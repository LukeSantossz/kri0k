# Changelog

Todas as mudanças notáveis ao projeto kri0k. Formato baseado em [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.2.0] - 2026-05-18 (Phase 4: Act + TTP Whois)

### Added
- `kri0k_core::ttp::whois::WhoisTtp` — primeira TTP concreta (MITRE T1590.001).
- `kri0k_core::ttp::subprocess::{Subprocess, RealSubprocess, MockSubprocess}` — abstração para subprocess injection + testabilidade (ADR-0013).
- `kri0k_core::scope::ScopeConfig` — parser de scope.yaml com schema v1 (lookahead) + allowlist exact-match (D-48).
- `kri0k_core::audit::NoopAuditSink` — slot reservado para Phase 8 (rename de `NoOpAuditSink`).
- `kri0k_pybridge::Engagement` — pyclass stateful expondo grafo + scope + TTP registry para Python (D-34).
- `kri0k.engagement.create()` — Python bootstrap helper (D-36).
- `kri0k_core::Error` variantes: `ScopeViolation`, `RateLimitExceeded`, `SubprocessTimeout`, `ParseError`, `MissingDependency`, `UnknownTtp`, `Io`, `Cancelled`.
- `NodeKind`: `Domain`, `Organization`, `Nameserver`. `EdgeKind`: `RegisteredBy`, `HasNameserver`.
- D-63 defense in depth: Layer 1 (scope allowlist), Layer 2 (regex domain validation pre-subprocess), Layer 3 (`Command::arg` sem shell).
- Tracing instrumentation via `tracing` crate em hot paths.
- Feature flag `integration` no `kri0k-core` para gating de tests com binary real.
- Fixtures whois capturadas em `tests/fixtures/`.

### Changed
- `kri0k_core::ttp::Ttp` trait agora é async (via `async-trait`). Trichotomia propose/dry-run/execute substituída por execute único + propose_only gate em Python.
- `kri0k_core::audit::NoOpAuditSink` renomeado para `NoopAuditSink` (D-38).
- `python/kri0k/agent/nodes/act.py` reescrito: propose_only gate (D-49) + `asyncio.to_thread` dispatch (D-46) + history entry D-56.
- `python/kri0k/agent/nodes/sense.py` modificado: prefere `engagement.snapshot()` quando disponível; mantém fallback `_native.get_dummy_graph()` (backward-compat Phase 1/2).
- `config/scope.example.yaml` reescrito para schema v1.

### Deferred (futuras phases)
- Scope validation completa com CIDR + wildcards (Phase 7).
- Audit log JSONL real com hash chain (Phase 8).
- TUI interactive approval keybinding (Phase 11).
- CLI commands (`kri0k init/run/status`) (Phase 12).
- Auto-discovery de TTPs via `inventory` crate (quando >= 5 TTPs).
- `pyo3-asyncio` para async pyclass methods (`asyncio.to_thread` suficiente).

### Security
- M-02 (scope check pre-execution): partial — exact-match allowlist em Phase 4; CIDR + wildcards em Phase 7.
- M-05 (propose-only default): covered — boolean flag default True.
- M-15 (LLM não executa direto): covered — facade `Engagement` minimal.
- M-34 (TTP rate limit): covered — `Mutex<Instant>` por TTP.
- M-36 (kill switch): covered — `CancellationToken` + fail-fast `which::which` em startup (wrapped em `py.allow_threads` per Pitfall 7).
- AB-03 (prompt injection out-of-scope): covered — defense in depth D-63 (Layer 1 allowlist + Layer 2 regex domain validation + Layer 3 `Command::arg` sem shell).

### Known Limitations
- **Dedupe cache unbounded (T-04-05-10):** O `HashMap<(kind_tag, natural_key), NodeId>` interno do `Engagement` cresce indefinidamente enquanto o engagement estiver vivo. Aceitável em Phase 4 (MVP single-engagement, short-lived; bounded por `|targets × kinds|`). TTL / eviction policy ficam para Phase 5+ se engagements long-running forem necessários. Documentado em `.planning/phases/04-act-node-ttp-whois/04-SECURITY.md` (Accepted Risks Log).

## [0.1.0] - 2026-05-16 (Phases 1-3)
- Phase 1: LangGraph structure base.
- Phase 2: Sense node + Ollama provider.
- Phase 3: Reason + Plan nodes.
