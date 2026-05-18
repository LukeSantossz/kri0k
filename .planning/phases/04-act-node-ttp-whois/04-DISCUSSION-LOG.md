# Phase 4: Act Node + TTP Whois - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-18
**Phase:** 04-act-node-ttp-whois
**Areas discussed:** A. Estado do grafo, B. Modelo whois, C. Stack execução, D. Gates, E. TTP registry, F. Error taxonomy, G. Testing, H. Observability & history, I. CI/CD, J. scope.yaml schema, K. Task/commit workflow, L. Documentation, M. Threat model mapping, N. Verification criteria, O. Command injection, R. Graceful shutdown
**Mode:** default (interactive, 1 question per turn)
**Total decisions captured:** 29 (D-34..D-64; numbering continues from Phase 3's D-33)

---

## A. Estado do grafo entre iterações

### A.1 Onde mora o grafo persistente?

| Option | Description | Selected |
|--------|-------------|----------|
| Engagement pyclass | Expor `Engagement` ao Python com grafo dentro. Mutex<Graph>. Alinha com ADR-0001. Comporta multi-engagement. | ✓ |
| Singleton Rust | `OnceLock<Mutex<Graph>>` no pybridge, process-wide. Simples mas reimplementa em Phase 5/6/8. | |
| Sem estado / update structures | TTP devolve `{nodes_added, edges_added}`; Python merge em snapshot. Viola ADR-0001. | |

### A.2 Surface API do Engagement?

| Option | Selected |
|--------|----------|
| Façade mínimo: `snapshot()` + `execute_proposal(p)` + `scope_hash()` | ✓ |
| API granular: `execute_ttp`, `add_node`, `add_edge`, `snapshot` | |
| Engagement só container; funções livres `_native.execute_ttp(engagement, ...)` | |

### A.3 Bootstrap do Engagement?

| Option | Selected |
|--------|----------|
| Helper Python `kri0k.engagement.create(...)` → injeta em `engagement_context` antes de `graph.invoke()` | ✓ |
| Lazy no primeiro node (sense detecta ausência e cria) | |
| Top-level runner `kri0k.engagement.run(scope_path, objective)` | |

### A.4 Lock strategy?

| Option | Selected |
|--------|----------|
| `Mutex<Graph>` simples | ✓ |
| `RwLock<Graph>` (snapshot reads paralelos) | |
| `Arc<RwLock<Graph>>` + clone do snapshot | |

### A.4b Audit slot agora ou Phase 8?

| Option | Selected |
|--------|----------|
| Reservar agora com `Box<dyn AuditSink>` stub no-op (NoopAuditSink) | ✓ |
| Adicionar só em Phase 8 (YAGNI) | |

---

## B. Modelo de dados whois

### B.1 EdgeKind?

| Option | Selected |
|--------|----------|
| Variantes específicas: `RegisteredBy`, `HasNameserver` | ✓ |
| `RelatesTo{relation}` genérico com string | |
| Híbrido | |

### B.2 Parser strategy?

| Option | Selected |
|--------|----------|
| Heurístico key:value | ✓ |
| Crate externa (whois-rust ou similar) | |
| Regex por TLD com fallback heurístico | |

### B.3 Múltiplos contatos?

| Option | Selected |
|--------|----------|
| Só Registrant Organization | ✓ |
| Os 4 contatos (Registrant, Admin, Tech, Billing) | |
| Registrant como Organization; demais em metadata | |

### B.4 Idempotência?

| Option | Selected |
|--------|----------|
| Idempotente por chave natural (Domain.name, Organization.name, Nameserver.hostname) | ✓ |
| Sempre adicionar (sem dedup) | |
| Rejeitar re-execução com `AlreadyExecuted` | |

---

## C. Stack de execução cross-language

### C.1 Subprocess strategy?

| Option | Selected |
|--------|----------|
| `tokio::process::Command` async + trait `Ttp` vira async | ✓ |
| `std::process::Command` sync + trait sync | |
| `tokio::task::spawn_blocking` wrapping sync (híbrido) | |

### C.2 Rate limit location?

| Option | Selected |
|--------|----------|
| TTP-local: `WhoisTtp` mantém `Mutex<Instant>` | ✓ |
| Engagement-level `HashMap<TtpId, TokenBucket>` | |
| Rate limit em Python no act.py | |

### C.3 Python async ↔ Rust async wiring?

| Option | Selected |
|--------|----------|
| Sync pyclass method + `asyncio.to_thread` no Python | ✓ |
| Async pyclass via `pyo3-asyncio` | |
| Sync pyclass + sync act node (quebra D-06) | |

### C.4 Outcome shape?

| Option | Selected |
|--------|----------|
| Outcome rico: `{status, result, error, graph_delta, audit_id}` | ✓ |
| Minimal `{success: bool, error: str|None}` | |
| Outcome rico + snapshot embutido | |

---

## D. Gates Phase 4 vs deferir

### D.1 Scope check level?

| Option | Selected |
|--------|----------|
| Allowlist exact-match domain (parse mínimo de scope.yaml) | ✓ |
| Stub permissivo: `validate_target()` Ok para qualquer target | |
| Bypass total: não chamar `validate_target` em Phase 4 | |

### D.2 Propose-only gate (ADR-0006)?

| Option | Selected |
|--------|----------|
| Phase 4 implementa flag boolean `engagement_context["propose_only"]` (default True) | ✓ |
| Phase 5 reflect implementa o gate | |
| Deferir totalmente para Phase 11 TUI | |

### D.3 Whois.exe pré-requisito?

| Option | Selected |
|--------|----------|
| Fail-fast no startup do Engagement via `which::which("whois")` | ✓ |
| Lazy fail no primeiro execute | |
| Health-check separado `kri0k doctor` (Phase 12) | |

### D.4 Timeout subprocess?

| Option | Selected |
|--------|----------|
| 30s default configurável via `Ttp::default_timeout() -> Duration` | ✓ |
| 60s hardcoded uniforme | |
| Sem timeout | |

---

## E. TTP registry & dispatch

| Option | Selected |
|--------|----------|
| HashMap hardcoded no `Engagement::new()` | ✓ |
| `inventory` crate (auto-discovery) | |
| Match expression hardcoded | |

---

## F. Error taxonomy & status codes

| Option | Selected |
|--------|----------|
| Tagged enum dual: `Status` enum no outcome + `Error` variants expandidos | ✓ |
| Binário `{success: bool, error_msg: str}` | |
| Status enum + Error Rust genérico (só `Generic`/`Json`) | |

---

## G. Testing strategy

| Option | Selected |
|--------|----------|
| Trait abstraction `Subprocess` com impl real + impl mock | ✓ |
| Sem mocking; todos tests usam whois real, integration `#[ignore]` | |
| Env var `KRI0K_TEST_WHOIS_BINARY` para mock script | |

---

## H. Observability & history

### H.1 Tracing crate?

| Option | Selected |
|--------|----------|
| Introduzir `tracing` agora com `#[instrument]` em hot paths | ✓ |
| Deferir para Phase 8 | |
| Só `eprintln!` em failure paths | |

### H.2 History entry shape?

| Option | Selected |
|--------|----------|
| Entry estruturada `{iteration, ttp_id, target, status, summary, graph_delta, audit_id}` | ✓ |
| Append do outcome cru | |
| Deixar para Phase 5 reflect definir; Phase 4 não popula | |

---

## I. CI/CD impact

| Option | Selected |
|--------|----------|
| Integration tests gated por feature flag, rodam local/nightly | ✓ |
| CI matrix Linux + Windows com whois instalado | |
| Sem integration tests automáticos; validação manual | |

---

## J. scope.yaml minimal schema

| Option | Selected |
|--------|----------|
| Schema mínimo (só `targets: [string]` exact-match) | |
| Schema completo já em Phase 4 (lookahead com stubs) | ✓ |
| Sem scope.yaml; allowlist hardcoded em `Engagement::new()` | |

---

## K. Task & commit workflow (framework local)

| Option | Selected |
|--------|----------|
| TASK-015 major agregada + commits atômicos por sub-escopo + branch + PR | ✓ |
| Decomposta em TASK-015..025 (uma por módulo) | |
| Sem tasks granulares — apenas TASK-015 high-level | |

---

## L. Documentation deliverables

| Option | Selected |
|--------|----------|
| Pacote mínimo (README + CONTRIBUTING + CHANGELOG) | |
| Pacote completo (+ ADR-0013 + ARCHITECTURE.md re-render) | ✓ |
| Mínimo absoluto (só VERIFICATION + commit messages) | |

---

## M. Threat model explicit mapping

| Option | Selected |
|--------|----------|
| Tabela em CONTEXT.md + inline `// M-XX:` annotations no código | ✓ |
| Só tabela centralizada (sem inline) | |
| Só inline annotations (sem tabela) | |

---

## N. Verification criteria (Definition of Done)

| Option | Selected |
|--------|----------|
| ROADMAP 4 + lint clean + tests pass + docs delivered + mitigations marked | ✓ |
| ROADMAP 4 apenas | |
| ROADMAP 4 + mitigations marked (sem lint/docs gate) | |

---

## O. Security: command injection no `target`

| Option | Selected |
|--------|----------|
| Defense in depth: allowlist + regex domain + `Command::arg` sem shell | ✓ |
| Só `Command::arg` sem shell (confia que whois rejeita garbage) | |
| Confiar só no scope check (D.1) | |

---

## R. Graceful shutdown & kill switch (M-36)

| Option | Selected |
|--------|----------|
| `CancellationToken` no Engagement (`tokio-util` crate) | ✓ |
| Apenas timeout (D.4) cobre | |
| Kill switch só entre iterações; subprocess corrente termina | |

---

## Claude's Discretion

Áreas onde o planner tem flexibilidade (capturadas em CONTEXT.md §Decisions › Claude's Discretion):
- Tipo exato do dedupe cache (`HashMap` vs `IndexMap` vs equivalente).
- Layout interno de `crates/kri0k-core/src/ttp/` (mod.rs re-exports vs flat).
- Ajustes finos no regex de domain validation (punycode `xn--`, IPv4-in-PTR).
- Strings exatas de mensagens de erro.
- Format exato do CHANGELOG.md (keep-a-changelog vs simples).

---

## Deferred Ideas

Capturadas em CONTEXT.md §Deferred Ideas. Resumo:
- Admin/Tech/Billing contact organizations como nós próprios → v2.
- CIDR + wildcard scope validation → Phase 7.
- Auto-discovery de TTPs via `inventory` crate → quando registry ≥ 5 TTPs.
- `pyo3-asyncio` para async pyclass methods → quando `to_thread` for limitante.
- Audit log JSONL real com hash chain → Phase 8.
- TUI interactive approval → Phase 11.
- `kri0k doctor` health-check command → Phase 12.
- Refactor para `RwLock<Graph>` → quando snapshot virar hot path.
- TLDs não-ICANN parsing (.br, .uk) → futura TTP enrichment.

---

*Discussion log generated by /gsd-discuss-phase 4 in default mode (single-question turns).*
