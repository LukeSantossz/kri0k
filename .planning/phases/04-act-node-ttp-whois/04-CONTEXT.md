# Phase 4: Act Node + TTP Whois - Context

**Gathered:** 2026-05-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Fechar o loop de execução do agente. Python `act` node envia `Proposal` validada para o Rust core através do PyO3 bridge; Rust executa a primeira TTP concreta (T1590.001 whois) via subprocess externo (`whois.exe` Sysinternals com `-accepteula -nobanner`), respeita rate limit de 1 req/sec, atualiza o grafo canônico com nós `Domain` / `Organization` / `Nameserver` e edges `RegisteredBy` / `HasNameserver`, e retorna outcome estruturado para `act` consumir.

Phase 4 também entrega:
- `Engagement` pyclass como container canônico do estado por engagement (grafo + scope + audit sink + cancellation token).
- Parser parcial de `scope.yaml` com schema versionado.
- Refactor cirúrgico do trait `Ttp` para `async`.
- Trait `Subprocess` abstraction para testabilidade.
- Tracing instrumentation + history entries estruturadas.

**Fora desta phase (deferido):**
- Scope check completo com CIDR + wildcards (Phase 7).
- Gate humano interativo via TUI (Phase 11).
- Audit log JSONL real com hash chain (Phase 8 — Phase 4 apenas reserva o slot).
- TTPs adicionais: nmap, dig, crt.sh (v2 + futuras phases).
- Reflect logic completo (Phase 5 — Phase 4 apenas popula `history`).
- CLI commands (Phase 12).

**Requisitos cobertos:** AGENT-05, TTP-01, TTP-02, TTP-03, TTP-04, TTP-05.

</domain>

<decisions>
## Implementation Decisions

### State & Engagement
- **D-34:** Estado persistente do grafo entre iterações vive num **`Engagement` pyclass** exposto pelo `kri0k-pybridge` ao Python. Container: `Engagement { graph: Mutex<Graph>, scope: ScopeConfig, audit: Box<dyn AuditSink>, registry: HashMap<String, Box<dyn Ttp>>, cancel: CancellationToken }`. Alinha com ADR-0001 (Rust source of truth) e suporta evolução para multi-engagement sem refactor.
- **D-35:** Surface API mínima do `Engagement`: `snapshot() -> dict`, `execute_proposal(proposal: dict) -> dict`, `scope_hash() -> str`. Todo o resto (add_node, add_edge, etc.) é interno ao Rust — Python não muta grafo diretamente.
- **D-36:** Bootstrap do `Engagement` via helper Python `kri0k.engagement.create(scope_dict, objective, propose_only=True) -> Engagement`. Helper chama `_native.Engagement.new(...)` e injeta a instância em `engagement_context["engagement"]` antes de `graph.invoke()`. Futuro CLI (Phase 12) reusa este helper. Mantém D-03 (engagement_context na invoke) intacto.
- **D-37:** Sincronização interna: **`Mutex<Graph>`** simples. Rate limit 1 req/sec + GIL Python tornam contenção irrelevante no MVP single-engagement. Evolução para `RwLock` deferida.
- **D-38:** `Engagement` reserva slot **`Box<dyn AuditSink>`** já em Phase 4 com implementação default `NoopAuditSink`. `execute_proposal` chama `audit.log_ttp_execution(...)` após cada execução — vira no-op em Phase 4, vira real em Phase 8 sem mudar call sites.

### Graph Data Model (whois domain)
- **D-39:** `kri0k-graph::EdgeKind` ganha variantes específicas: **`RegisteredBy`** (Domain → Organization) e **`HasNameserver`** (Domain → Nameserver). Type-safe e auto-documentado. `RelatesTo{relation}` permanece para casos exploratórios.
- **D-40:** `kri0k-graph::NodeKind` ganha variantes novas: **`Domain { name: String }`**, **`Organization { name: String }`**, **`Nameserver { hostname: String }`**. Datas (created/updated/expires) vão em `Node.metadata: HashMap<String, String>` com chaves `created_at`, `updated_at`, `expires_at` no formato ISO 8601.
- **D-41:** Parser whois usa **heurístico key:value** linha-a-linha. Lida bem com TLDs ICANN-style (.com/.net/.org). TLDs com formato diferente (.br, .uk, alguns ccTLDs) entregam parse parcial e o restante vai em `metadata["raw_block_<n>"]` para inspeção. Limitação documentada no README.
- **D-42:** Apenas **Registrant Organization** vira nó `Organization` em Phase 4. Admin/Tech/Billing contacts são ruído para reconnaissance MVP — deferidos para v2.
- **D-43:** **Idempotência por chave natural**. `Engagement` mantém `HashMap<(NodeKindTag, String), NodeId>` (chave: Domain.name, Organization.name, Nameserver.hostname). `execute_proposal` faz lookup antes de inserir; reusa o `NodeId` existente. Edges deduplicadas por `(src, dst, kind_tag)`. Re-execução do mesmo TTP no mesmo target atualiza `metadata["last_whois_at"]` mas não cresce o grafo.

### Execution Stack (cross-language)
- **D-44:** Subprocess via **`tokio::process::Command`** (async). Alinha com `TOKIO_RUNTIME` já existente em `crates/kri0k-pybridge/src/lib.rs:9-22`. Cancellation natural via `tokio::time::timeout` e `CancellationToken`. Refactor: trait `Ttp` vira `async` — cirúrgico porque métodos hoje são `todo!()`.
- **D-45:** **Rate limit TTP-local**: `WhoisTtp { last_call: Mutex<Instant>, ... }`. `execute()` aguarda `tokio::time::sleep(remaining)` se `now - last_call < 1s`. Coeso com `RateLimits` declarado no trait (`crates/kri0k-core/src/ttp.rs:76-92`). Cada futura TTP guarda seu próprio bucket.
- **D-46:** Bridge mantém pyclass methods **sync** (não usa `pyo3-asyncio`). Rust internamente faz `runtime().block_on(async { ... })` no padrão de `get_dummy_graph`. Python async (`act.py`) chama via `await asyncio.to_thread(engagement.execute_proposal, proposal_dict)` — não bloqueia event loop, sem dependência extra.
- **D-47:** `execute_proposal` retorna **outcome rico** como dict:
  ```python
  {
      "status": "executed" | "scope_violation" | "rate_limited" | "timeout" | "error" | "proposed",
      "result": <WhoisOutput tipado> | None,
      "error": <str> | None,
      "graph_delta": {"nodes_added": int, "edges_added": int},
      "audit_id": <str> | None,
  }
  ```
  Phase 5 reflect consome `status` + `graph_delta` para decidir continuação.

### Gates & Safeguards
- **D-48:** **Scope check Phase 4 = allowlist exact-match domain.** `Scope::from_yaml(path)` parseia o campo `targets:` (lista de strings); `validate_target(t)` faz `targets.contains(&t)`. CIDR + wildcards (`*.example.com`) ficam para Phase 7. Fail-closed respeitado (ADR-0005, M-02). `validate_target` é chamado dentro de `execute_proposal` antes de qualquer subprocess.
- **D-49:** **Propose-only gate (ADR-0006)** implementado via flag boolean `engagement_context["propose_only"]` (default `True`). `act.py` checa antes de chamar `execute_proposal`:
  - Se `True`: act retorna `{status: "proposed", proposal: {...}, would_execute: ttp_id}` sem chamar Rust.
  - Se `False`: act chama `await asyncio.to_thread(engagement.execute_proposal, proposal)`.
  CLI Phase 12 e TUI Phase 11 controlarão a flag.
- **D-50:** **Whois.exe pré-requisito fail-fast no startup.** `Engagement::new()` chama `which::which("whois")` (crate `which = "6"`). Retorna `Error::MissingDependency("whois")` se ausente. Helper Python propaga a exceção como `RuntimeError` claro. Cobre M-36 indiretamente (sistema não inicia sem dependência crítica).
- **D-51:** **Timeout subprocess** = 30s default, configurável via novo trait method `Ttp::default_timeout() -> Duration { Duration::from_secs(30) }`. Implementação no `execute()` via `tokio::time::timeout(ttp.default_timeout(), ...)`. Se exceder: kill subprocess (drop do `Child`), retorna `status: "timeout"` no outcome.

### TTP Registry & Dispatch
- **D-52:** **HashMap hardcoded no `Engagement::new()`**. `let mut registry: HashMap<String, Box<dyn Ttp>> = HashMap::new(); registry.insert("T1590.001".to_string(), Box::new(WhoisTtp::new(subprocess_impl)));`. Phase 5+ adicionam linhas. Refactor para auto-discovery (`inventory` crate, INTEGRATIONS.md §Planned) quando >= 5 TTPs. Resolução: `registry.get(&proposal.ttp_id).ok_or(Error::UnknownTtp(...))`.

### Error Taxonomy & Status Codes
- **D-53:** **Tagged enum dual** (status visível ao Python + Error Rust expandido):
  - `Outcome.status`: `"executed" | "scope_violation" | "rate_limited" | "timeout" | "error" | "proposed"` (string em Python).
  - `kri0k_core::Error` enum ganha variantes: `ScopeViolation { target: String, reason: String }`, `RateLimitExceeded { ttp_id: String, retry_in_ms: u64 }`, `SubprocessTimeout { ttp_id: String, timeout_ms: u64 }`, `ParseError { source: String, detail: String }`, `MissingDependency { binary: String }`, `UnknownTtp { ttp_id: String }`, `Io(#[from] std::io::Error)`. Existing `Json` e `Generic` permanecem.
  - Reflect (Phase 5) faz pattern match no `status` string; audit (Phase 8) discrimina por error variant nativa.

### Testing Strategy
- **D-54:** **Trait abstraction sobre subprocess** para testabilidade total. Definir em `crates/kri0k-core/src/ttp/subprocess.rs`:
  ```rust
  #[async_trait]
  pub trait Subprocess: Send + Sync {
      async fn run(&self, cmd: &str, args: &[&str], timeout: Duration) -> Result<SubprocessOutput>;
  }
  ```
  Impls: `RealSubprocess` (usa `tokio::process::Command`) e `MockSubprocess` (lê fixture path do constructor). `WhoisTtp::new(subprocess: Arc<dyn Subprocess>)`. Unit tests injetam `MockSubprocess` apontando para `tests/fixtures/whois_example_com.txt`. Integration tests com binary real gated por `#[cfg(feature = "integration")]` em `crates/kri0k-core/Cargo.toml`, rodam via `cargo test --features integration` (local/nightly only).

### Observability & History
- **D-55:** **`tracing` crate introduzido em Phase 4.** Adicionar `tracing = "0.1"` ao workspace; `#[tracing::instrument(skip(self, subprocess))]` em `Engagement::execute_proposal`, `WhoisTtp::execute`, `parse_whois_output`. Subscriber não vem incluído (CLI Phase 12 configura). Env var `RUST_LOG=kri0k=debug` para debug ad-hoc. Base para Phase 8 audit (auditor consome events via `tracing-subscriber` custom layer) e Phase 11 TUI logs.
- **D-56:** **History entry estruturada** no `AgentState.history` após execução. Shape produzido por `act.py`:
  ```python
  {
      "iteration": int,
      "ttp_id": str,
      "target": str,
      "status": str,            # mesmo do outcome
      "summary": str,           # humano: "whois example.com → +1 Domain +1 Organization +4 Nameservers"
      "graph_delta": {"nodes_added": int, "edges_added": int},
      "audit_id": str | None,
  }
  ```
  Format estável desde Phase 4. Reflect (Phase 5) e LLM REASON (próxima iteração) consomem direto.

### CI/CD
- **D-57:** **Integration tests do whois gated por feature flag**, rodam apenas localmente ou em CI nightly dedicado. CI principal (`.github/workflows/ci.yml`) roda só unit (mock subprocess) — rápido, sem rede, sem flake. `CONTRIBUTING.md` documenta `cargo test --features integration` para reprodução local. Adição opcional: workflow `nightly-integration.yml` com `whois` instalado.

### scope.yaml Schema
- **D-58:** **Schema completo já em Phase 4 (lookahead com stubs).** `ScopeConfig` struct declara todos os campos planejados para v1 (`version`, `objective`, `operator`, `targets`, `targets_cidr`, `targets_wildcards`, `safeguards { propose_only, kill_switch }`, `rate_limits`, `audit_path`). Phase 4 **usa** apenas `version`, `targets`, `safeguards.propose_only`. Demais são parseados (com defaults) mas não consumidos. Backward-compat garantida: YAML escrito em Phase 4 funciona em Phase 7+. Versionamento: `version: 1` mandatório; parser rejeita versão desconhecida.

### Task & Commit Workflow (framework local)
- **D-59:** Phase 4 entra em `.claude/tasks.md` como **TASK-015 major agregada** (consistente com Phase 2 e Phase 3 no registry). Branch dedicada: **`feat/phase-4-act-ttp-whois`**. Commits atômicos por sub-escopo seguindo Conventional Commits, e.g.:
  - `feat(graph): add Domain/Organization/Nameserver NodeKind variants + RegisteredBy/HasNameserver EdgeKind`
  - `feat(core): add Engagement struct with Mutex<Graph> and AuditSink slot`
  - `feat(ttp): refactor Ttp trait to async; add Subprocess abstraction`
  - `feat(ttp): implement WhoisTtp with rate limit and timeout`
  - `feat(ttp): add whois output parser (heuristic key:value)`
  - `feat(core): add Scope::from_yaml with v1 schema (lookahead stubs)`
  - `feat(bridge): expose Engagement pyclass with snapshot/execute_proposal`
  - `feat(agent): wire act node to engagement with propose_only gate`
  - `feat(python): add kri0k.engagement.create bootstrap helper`
  - `feat(observability): instrument execute paths with tracing`
  - `test(ttp): add fixtures and unit tests for WhoisTtp`
  - `docs(...): README quickstart + CONTRIBUTING TTP guide + ADR-0013 + CHANGELOG`
  
  PR final para `master` após `/gsd-verify-work`. Registry.md ganha 1 entrada agregada referenciando TASK-015.

### Documentation
- **D-60:** Pacote completo de docs em Phase 4:
  - **README.md**: nova seção "Running the whois TTP" com pré-req (`whois.exe` instalado), exemplo `scope.yaml` mínimo, comando para rodar uma iteração end-to-end.
  - **CONTRIBUTING.md**: nova seção "Adding a new TTP" descrevendo pattern (Trait `Ttp` + `Subprocess` abstraction + registry entry + tests com fixture).
  - **CHANGELOG.md** (criar se ausente): entry `## 0.2.0 — Phase 4 (Act + TTP Whois)` listando deltas.
  - **`docs/adr/ADR-0013-ttp-trait-subprocess-abstraction.md`**: documenta o pattern Trait + `Subprocess` para futuros TTPs.
  - **`.planning/codebase/ARCHITECTURE.md`**: re-render via `/gsd-map-codebase` opcional ao final.

### Threat Model Coverage
- **D-61:** **Mapping explícito** em CONTEXT.md + inline annotations `// M-XX:` no código. Tabela:

  | Mitigation | Threat | Phase 4 decision/code | Status |
  |---|---|---|---|
  | M-02 | Scope violation | D-48 allowlist + `validate_target` chamado em `execute_proposal` | Partial (exact-match; Phase 7 completa) |
  | M-05 | LLM triggers TTPs sem aprovação | D-49 propose-only flag default `True` | Covered (Phase 4 boolean; Phase 11 interactive) |
  | M-15 | LLM executa direto | D-34 Engagement pyclass + D-35 façade mínimo (Python nunca muta grafo direto) | Covered |
  | M-21 | TTP destrutivo | Whois é `RiskLevel::Safe`; não aplica em Phase 4 | N/A |
  | M-34 | TTP sem rate limit | D-45 TTP-local `Mutex<Instant>` + `Ttp::rate_limits()` | Covered |
  | M-36 | Kill switch | D-50 fail-fast `which()` + D-62 `CancellationToken` no Engagement | Covered |
  | AB-03 | Prompt injection causa out-of-scope | D-63 defense in depth (allowlist + regex + `Command::arg` no shell) | Covered |

### Verification Criteria (Definition of Done)
- **D-64:** Phase 4 está done quando **todos** os critérios abaixo passam:
  1. ROADMAP critérios 1-4 verificáveis (trait Ttp, struct whois output, grafo recebe nós, rate limit respeitado).
  2. `cargo clippy --workspace --all-targets` strict (zero warnings).
  3. `ruff check python/ tests/` + `mypy python/kri0k` strict (zero issues).
  4. `pytest tests/` 100% pass; `cargo test --features integration` 100% pass.
  5. README + CONTRIBUTING + CHANGELOG + ADR-0013 entregues.
  6. Tabela M-XX em CONTEXT.md/VERIFICATION.md marcada com status `Covered`/`Partial`/`N/A` para cada threat aplicável.
  7. `whois example.com` rodando localmente produz outcome `status: "executed"` com `result.registrant` populado e grafo cresce em ≥ 1 Domain + 1 Organization + ≥ 1 Nameserver.

### Security: Command Injection
- **D-63:** **Defense in depth para `target` (untrusted input do LLM):**
  - **Layer 1**: allowlist (D-48) rejeita targets não listados antes de qualquer subprocess.
  - **Layer 2**: regex domain validation no parser de `Proposal` — `^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?(?:\.[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?)+$` (case-insensitive). Reject `Error::ParseError` se inválido.
  - **Layer 3**: `Command::arg(target)` sem shell (Rust default seguro). Cobre AB-03 (prompt injection) explicitamente.

### Graceful Shutdown & Kill Switch
- **D-62:** **`CancellationToken` no Engagement.** `Engagement { ..., cancel: CancellationToken }` (crate `tokio-util = "0.7"`). `WhoisTtp::execute(target, cancel: CancellationToken)` usa:
  ```rust
  tokio::select! {
      _ = cancel.cancelled() => {
          child.kill().await?;
          Err(Error::Cancelled)
      }
      result = child.wait_with_output() => result,
  }
  ```
  Surface Python: `Engagement.kill()` chama `cancel.cancel()`. TUI Phase 11 e Ctrl-C handler usarão. Cobre M-36 explicitamente.

### Claude's Discretion
- Tamanho exato do `HashMap<(NodeKindTag, String), NodeId>` dedupe cache — planner pode usar `IndexMap` ou tipo equivalente.
- Layout interno dos módulos novos no Rust (e.g., `ttp/mod.rs` re-exports vs flat layout) — planner decide consistente com STRUCTURE.md.
- Exact regex para domain validation — D-63 dá a base; pequenos ajustes (suporte a punycode `xn--`, IPv4-em-PTR) ficam para planner avaliar caso a caso.
- Mensagens de erro humanas exatas (strings dentro de cada `Error::*` variant).
- Layout exato do CHANGELOG.md (keep-a-changelog format vs simples).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Architecture & ADRs
- `docs/adr/ADR-0001-canonical-state-in-rust.md` — Rust é source of truth; Python recebe snapshots imutáveis
- `docs/adr/ADR-0005-deterministic-rust-validator.md` — Scope validation fail-closed antes de execução
- `docs/adr/ADR-0006-propose-only-default.md` — Propose-only é default operacional (D-49)
- `docs/adr/ADR-0007-append-only-audit-log.md` — AuditSink slot reservado em Phase 4 (D-38)
- `docs/adr/ADR-0011-scope-yaml-checksum.md` — scope.yaml com checksum (D-58 implementa schema parcial)
- `docs/adr/ADR-0012-ttp-trait-adapters.md` — TTPs como subprocess (D-44 alinha)
- `docs/security/THREAT_MODEL.md` — M-02, M-05, M-15, M-21, M-34, M-36, AB-03 (mapeados em D-61)

### Codebase maps
- `.planning/codebase/STACK.md` — versões, lints, conventions
- `.planning/codebase/ARCHITECTURE.md` — componentes, layers, data flow
- `.planning/codebase/INTEGRATIONS.md` — TTP adapters planejados §Planned Integrations
- `.planning/codebase/STRUCTURE.md` — layout de módulos
- `.planning/codebase/TESTING.md` — convenções de testes

### Prior phases
- `.planning/phases/01-langgraph-structure/01-CONTEXT.md` — D-01..D-17 (TypedDict, async nodes, engagement_context na invoke)
- `.planning/phases/02-sense-node-ollama-provider/02-CONTEXT.md` — D-18..D-33 (snapshot shape `{raw, formatted}`, LLM provider layout)
- `.planning/phases/03-reason-plan-nodes/03-VERIFICATION.md` — Proposal dataclass shape, parser do LLM

### Código existente relevante (Rust)
- `crates/kri0k-core/src/ttp.rs` — trait `Ttp` atual (síncrono, métodos `todo!()`); vira async em D-44
- `crates/kri0k-core/src/lib.rs` — `Error` enum atual (expandir per D-53)
- `crates/kri0k-core/src/scope.rs` — stub a substituir per D-48
- `crates/kri0k-core/src/safeguards.rs` — `SafeguardsConfig` (D-49 propose_only)
- `crates/kri0k-core/src/audit.rs` — `AuditSink` trait (slot per D-38)
- `crates/kri0k-graph/src/lib.rs` — `NodeKind`/`EdgeKind` enums; expandir per D-39/D-40
- `crates/kri0k-pybridge/src/lib.rs` — padrão de `get_dummy_graph()` e `runtime().block_on` para D-46; `Engagement` pyclass entra aqui

### Código existente relevante (Python)
- `python/kri0k/agent/state.py` — `AgentState` TypedDict (history shape per D-56)
- `python/kri0k/agent/nodes/act.py` — placeholder a substituir per D-49
- `python/kri0k/llm/parser.py` — `Proposal` dataclass que vira input do `execute_proposal`
- `python/kri0k/_native.pyi` — atualizar com signatures de `Engagement` per D-34

### Config / planejamento
- `.planning/ROADMAP.md` §Phase 4 — 4 success criteria (D-64 expande)
- `.planning/REQUIREMENTS.md` — AGENT-05, TTP-01..05
- `config/scope.example.yaml` — atualizar com schema v1 per D-58

### Project policy
- `CLAUDE.md` (root) — arquitetura e convenções gerais
- `.claude/CLAUDE.md` — framework local (D-59 task/commit)
- `clippy.toml` — strict lints (D-64.2 verification)
- `pyproject.toml` — ruff/mypy config (D-64.3 verification)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`TOKIO_RUNTIME` singleton** (`crates/kri0k-pybridge/src/lib.rs:9-22`) — D-46 reusa para `block_on` interno; tokio::process::Command precisa de runtime ativo.
- **`Graph` struct + `to_json()`** (`crates/kri0k-graph/src/lib.rs:101-191`) — `Engagement::snapshot()` chama `graph.to_json()` direto.
- **`get_dummy_graph()` pattern** — D-46 segue o mesmo (sync method PyO3 wrapping `runtime().block_on`).
- **`Proposal` dataclass** (`python/kri0k/llm/parser.py:21-71`) — input do `execute_proposal`; já existe Phase 3.
- **`AgentState.engagement_context`** (D-03) — slot pronto para receber `engagement` instance e `propose_only` flag.
- **`AgentState.history`** (Phase 1) — list pronta para D-56 entries.
- **`Ttp` trait + `RateLimits`/`ExecutionPlan`/`ExecutionResult`** (`crates/kri0k-core/src/ttp.rs`) — esqueleto a refatorar para async (D-44).

### Established Patterns
- **mypy strict + ruff strict** — todo código Python novo precisa passar (D-64.3).
- **clippy strict + `unwrap_used`/`panic`/`unimplemented` denied** (`clippy.toml`) — Rust novo precisa passar (D-64.2).
- **Async first** — D-44/D-46 mantêm padrão.
- **Serde derive + JSON cross-boundary** — `WhoisOutput` será `#[derive(Serialize)]` para serializar no outcome.
- **No `unsafe` workspace-wide** (`Cargo.toml:24`).
- **ULID NodeId/EdgeId** (`crates/kri0k-core/src/lib.rs:32-89`) — D-43 dedupe cache mantém o NodeId existente em re-execução.
- **Test markers pytest** (`unit`, `integration`, `graph`) — adicionar `act` marker se necessário para isolation, ou reusar `unit`/`integration`.

### Integration Points
- **`python/kri0k/agent/nodes/act.py`** — D-49 substitui placeholder no-op por gate `propose_only` + chamada para `engagement.execute_proposal`.
- **`python/kri0k/agent/nodes/sense.py`** — atualizar para usar `engagement.snapshot()` em vez de `_native.get_dummy_graph()` quando `engagement` presente no context (backward-compat: cai no get_dummy_graph se ausente, mantendo testes Phase 1/2).
- **`crates/kri0k-pybridge/src/lib.rs`** — adicionar `pyclass Engagement` + `#[pymethods] impl`, registrar no `_native` module.
- **`python/kri0k/_native.pyi`** — declarar stubs para `class Engagement`, mantendo mypy strict feliz.
- **`crates/kri0k-core/Cargo.toml`** — adicionar `tokio = { version = "1", features = ["process", "time", "sync"] }`, `tokio-util = { version = "0.7", features = ["rt"] }`, `which = "6"`, `async-trait = "0.1"`, `tracing = "0.1"`. Feature flag `integration` para gating de tests.
- **`crates/kri0k-graph/src/lib.rs`** — expandir `NodeKind` + `EdgeKind` enums (D-39/D-40). Migration: testes existentes do `Graph` continuam válidos.
- **`pyproject.toml`** — adicionar `pytest-asyncio` config para tests do act node (já existe).
- **`config/scope.example.yaml`** — atualizar com schema v1 completo (D-58 lookahead).

### Não tocar
- ROADMAP.md e REQUIREMENTS.md (atualizados por `/gsd-transition` após phase done).
- Crates kri0k-core/audit.rs além de adicionar `NoopAuditSink` impl (audit real fica Phase 8).
- LangGraph `StateGraph` topology (Phase 1) — Phase 4 só altera o conteúdo dos nodes.
- LLM provider/prompts (Phase 2) — Phase 4 não toca.

</code_context>

<specifics>
## Specific Ideas

- **Sysinternals whois binary** invocado com `-accepteula -nobanner` (registrado em TASK-014 da mesma sessão de discuss; primeira execução do binary requer `-accepteula` para evitar prompt do EULA).
- **WhoisOutput struct** (D-47 result):
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct WhoisOutput {
      pub registrant: Option<String>,
      pub registrar: Option<String>,
      pub nameservers: Vec<String>,
      pub created_at: Option<String>,
      pub updated_at: Option<String>,
      pub expires_at: Option<String>,
      pub raw_unparsed: Vec<String>, // linhas que o parser não cobriu, para inspeção
  }
  ```
- **History summary format** (D-56): `"whois <target> → +<N> Domain +<N> Organization +<N> Nameserver"` (somente kinds positivos no delta). Quando idempotente: `"whois <target> → no new nodes (already known)"`.
- **`propose_only` default `True`** alinha com ADR-0006 e cumpre M-05 sem dependência de TUI.
- **MissingDependency error** deve incluir hint: `"whois binary not found in PATH. Install with: winget install Microsoft.Sysinternals.Whois (Windows) or apt install whois (Linux)"`.

</specifics>

<deferred>
## Deferred Ideas

- **Admin/Tech/Billing contact organizations** como nós próprios — Phase v2 ou TTP-aware-enrichment. Whois retorna todos, Phase 4 ignora 3 dos 4.
- **CIDR + wildcard scope validation** (`*.example.com`, `192.168.0.0/24`) — Phase 7 (entrega completa do SCOPE-01..05).
- **Auto-discovery de TTPs via `inventory` crate** — quando registry tiver >= 5 TTPs.
- **`pyo3-asyncio` para async pyclass methods** — quando complexidade do `to_thread` wrapper for relevante.
- **Audit log JSONL real com hash chain** — Phase 8 (D-38 só reserva slot).
- **TUI interactive approval** (`y`/`n` keybinding) — Phase 11. Phase 4 entrega só o boolean gate.
- **`kri0k doctor` health-check command** — Phase 12 (CLI). Reusará `which::which("whois")` e `ping_ollama` (Phase 2).
- **Refactor para `RwLock<Graph>`** — quando snapshot virar hot path (Phase 9 TUI render).
- **TLDs com formato não-ICANN parsing (.br, .uk)** — futura phase de TTP enrichment ou TTP-aware-parser plugin.
- **Provider switching runtime (Anthropic/OpenAI)** — v2 (LLM-V2-03), fora milestone 1.

</deferred>

---

*Phase: 04-act-node-ttp-whois*
*Context gathered: 2026-05-18*
