# KRK-001 Kri0K — Arquitetura (T6)

**Status:** baseline para MVP-0 / MVP-1
**Inputs consolidados:** T1 (refinamento), T2/T4 (TTP scope ATT&CK), T3 (estado-da-arte), T5 (interop PyO3+petgraph+Tokio)
**Outputs deste documento:**
1. Diagrama de componentes (§1)
2. Contratos entre camadas (§2)
3. Índice de ADRs (§3) — texto em `kri0k/docs/adr/`
4. Fluxo de uma iteração `sense → reason → act → update` (§4)
5. Pontos de extensão (§5)

Princípios fundadores derivados de T1:
- **Estado canônico vive em Rust** (determinismo, auditabilidade, controle do GIL).
- **Python é a camada de raciocínio**, não de verdade: recebe *snapshots* serializados, devolve *propostas*.
- **Validador determinístico precede toda execução** — o LLM nunca aciona o mundo diretamente.
- **`scope.yaml` é prerequisito de boot**; sem ele o binário só responde `init` e `status`.
- **Propose-only é o default**, `--execute` é opt-in explícito.

---

## 1. Diagrama de componentes

### 1.1 Visão lógica (camadas)

```
┌──────────────────────────────────────────────────────────────────────────┐
│                          OPERADOR (CLI / TUI)                             │
│   $ kri0k init  |  kri0k run --scope scope.yaml [--execute]               │
└──────────────────────────────┬───────────────────────────────────────────┘
                               │
┌──────────────────────────────▼───────────────────────────────────────────┐
│  CAMADA PYTHON  (kri0k pkg, embedded via PyO3 — pyembed no MVP-1)         │
│                                                                          │
│   ┌────────────────────┐   ┌──────────────────────┐  ┌────────────────┐ │
│   │  LangGraph nodes   │   │  LLM provider abst.  │  │  Prompt store  │ │
│   │  (sense/reason/    │   │  Ollama | Anthropic  │  │  (jinja2)      │ │
│   │   plan/act/reflect)│   │  | OpenAI (opt-in)   │  │                │ │
│   └─────────┬──────────┘   └──────────┬───────────┘  └────────┬───────┘ │
│             │                          │                       │         │
│             └──────────────┬───────────┴───────────────────────┘         │
│                            ▼                                              │
│             ┌────────────────────────────────┐                            │
│             │  kri0k._native  (PyO3 bindings)│  ← fronteira RUST↔PY      │
│             │  Snapshot, Proposal, propose(),│                            │
│             │  validate(), execute(), apply()│                            │
│             └───────────────┬────────────────┘                            │
└─────────────────────────────┼─────────────────────────────────────────────┘
                              │  JSON snapshots / Proposal structs
┌─────────────────────────────▼─────────────────────────────────────────────┐
│  NÚCLEO RUST  (cargo workspace)                                           │
│                                                                          │
│   ┌──────────────────────┐  ┌────────────────────────────┐               │
│   │   kri0k-pybridge     │  │   kri0k-core (runtime)     │               │
│   │   • PyO3 cdylib      │◀▶│   • Tokio runtime (LocalSet)│              │
│   │   • Snapshot codec   │  │   • Scope validator (det.) │               │
│   │   • py↔rs converters │  │   • Audit log (append-only)│              │
│   └──────────────────────┘  │   • Kill switch / dry-run  │               │
│                             └────────────┬────────────────┘               │
│                                          │                                 │
│        ┌─────────────────────────────────┼─────────────────────────┐      │
│        ▼                                 ▼                         ▼      │
│ ┌─────────────────┐         ┌────────────────────┐    ┌──────────────────┐│
│ │  kri0k-graph    │         │   kri0k-ttp        │    │  kri0k-scope     ││
│ │  petgraph::     │         │   Trait + adapters │    │  scope.yaml      ││
│ │  StableGraph    │         │   T1046 (nmap),    │    │  parser+checksum ││
│ │  Node/EdgeKind  │         │   T1590.001 (whois)│    │  CIDR/domain     ││
│ │  serde JSON     │         │   T1596.003 (crt.sh│    │  matcher         ││
│ └────────┬────────┘         └──────────┬─────────┘    └──────────────────┘│
│          │                              │                                  │
│          ▼                              ▼                                  │
│  ┌────────────────┐            ┌────────────────────┐                     │
│  │  Graph store   │            │  External tools    │                     │
│  │  (in-mem + on- │            │  nmap, dig, whois, │                     │
│  │   disk JSONL   │            │  HTTP clients      │                     │
│  │   snapshot)    │            │  (reqwest+rustls)  │                     │
│  └────────────────┘            └────────────────────┘                     │
└──────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Cargo workspace (MVP-0)

```
kri0k/
├── Cargo.toml                    # [workspace] members + resolver = "2"
├── crates/
│   ├── kri0k-core/               # runtime, scope, audit, kill switch
│   ├── kri0k-graph/              # petgraph::StableGraph + serde
│   ├── kri0k-ttp/                # TTP trait + impls (MVP-1+)
│   ├── kri0k-scope/              # scope.yaml DSL (serde_yaml + ipnet)
│   └── kri0k-pybridge/           # PyO3 cdylib → kri0k._native
├── python/
│   └── kri0k/                    # pure-Python pkg (LangGraph, prompts)
│       ├── __init__.py
│       ├── agent.py              # graph builder
│       ├── nodes/                # sense / reason / plan / act / reflect
│       └── providers/            # ollama.py | anthropic.py | openai.py
├── pyproject.toml                # maturin backend
├── docs/
│   ├── adr/                      # ADR-0001..N
│   └── ARCHITECTURE.md           # cópia humana deste documento
└── scope.example.yaml
```

### 1.3 Mapeamento de responsabilidades (RACI compacto)

| Concern                              | Crate / módulo            | Justificativa |
|--------------------------------------|---------------------------|---------------|
| Verdade do estado de engagement      | `kri0k-graph`             | petgraph determinístico, fácil snapshot |
| Decisão de "posso executar?"         | `kri0k-core::validator`   | regras determinísticas auditáveis, sem LLM |
| Geração de "o que executar a seguir" | LangGraph (Python)        | reasoning + ferramentas externas |
| Execução de TTP                      | `kri0k-ttp`               | wrappers tipados, side-effects controlados |
| Mapping ATT&CK                       | `kri0k-ttp::attck`        | static lookup table + STIX JSON (T2/T4) |
| Audit / forensics                    | `kri0k-core::audit`       | append-only, hash-chained JSONL |
| LLM ↔ estado                         | `kri0k-pybridge`          | única fronteira PyO3 |

---

## 2. Contratos entre camadas

### 2.1 Fronteira PyO3 (assinaturas Python)

Pyi stubs gerados via `maturin` + `pyo3-stub-gen`. Módulo nativo: `kri0k._native`.

```python
# kri0k/_native.pyi  (gerado, versionado no repo)

from typing import Literal

class Snapshot:
    """Read-only view of the canonical graph. JSON-serializable."""
    iteration: int
    scope_hash: str           # sha256 do scope.yaml
    nodes: list[Node]
    edges: list[Edge]
    open_findings: list[Finding]
    def to_json(self) -> str: ...

class Node:
    id: str                   # ULID estável (StableGraph index oculto)
    kind: Literal["host", "service", "credential", "finding", "action", "ttp"]
    attrs: dict[str, object]  # tipado por kind (validado em Rust)
    created_at: int           # unix seconds
    ttp_id: str | None        # ex.: "T1046"

class Edge:
    src: str
    dst: str
    kind: Literal["discovered_by", "runs_on", "authenticates_to",
                  "pivots_through", "evidence_of", "supersedes"]

class Proposal:
    """LLM-proposed next action. Built in Python, validated in Rust."""
    ttp_id: str               # e.g. "T1046"
    target: str               # CIDR | domain | URL | host id
    args: dict[str, object]
    rationale: str            # justificativa do LLM (string livre, vai pro audit)
    destructive: bool         # default False; flips humanGate=True no validador

class ValidationResult:
    ok: bool
    reasons: list[str]        # "out_of_scope", "ttp_disabled", "needs_human_gate"
    requires_human: bool

class ExecutionResult:
    ok: bool
    new_nodes: list[Node]
    new_edges: list[Edge]
    error: str | None
    audit_id: str             # ULID do record no audit log

# Top-level functions exposed by kri0k._native:

def open_engagement(scope_yaml_path: str) -> "Engagement": ...

class Engagement:
    scope_hash: str
    def snapshot(self) -> Snapshot: ...
    def validate(self, p: Proposal) -> ValidationResult: ...
    def execute(self, p: Proposal, *, human_gate_token: str | None = None) -> ExecutionResult: ...
    def apply_external(self, nodes: list[dict], edges: list[dict]) -> None: ...  # idempotent
    def kill(self, reason: str) -> None: ...    # kill switch
    def close(self) -> None: ...
```

**Regras invioláveis da fronteira:**
1. `Snapshot` é **read-only** em Python; mutações só via `execute()` ou `apply_external()`.
2. Toda chamada de `execute` é precedida por `validate` no próprio Rust (não confiamos no caller).
3. `Proposal.rationale` é truncado a 8 KiB; vai pro audit literalmente.
4. Nenhuma referência viva ao grafo cruza a fronteira — sempre `Snapshot` serializado.
5. `human_gate_token` é exigido para qualquer TTP em `destructive_ttps` (lista hardcoded + extensível).

### 2.2 Formato serializado do grafo (JSON canônico)

JSON compacto, chaves curtas (T5 mostrou que JSON vence MsgPack quando o LLM lê). Documento estável versionado.

```json
{
  "v": 1,
  "iter": 42,
  "scope_hash": "sha256:9f86d0…",
  "nodes": [
    {"id":"01HX…","k":"host","a":{"ip":"10.0.0.5","up":true},"t":1778709400,"ttp":null},
    {"id":"01HY…","k":"service","a":{"port":22,"proto":"tcp","banner":"OpenSSH 9.6"},"t":1778709420,"ttp":"T1046"}
  ],
  "edges": [
    {"s":"01HX…","d":"01HY…","k":"runs_on"}
  ],
  "open_findings": []
}
```

**Decisões:**
- Chaves curtas (`k`, `a`, `t`, `s`, `d`) para reduzir tokens do LLM (~30% economia em grafos de 1k nós).
- `id` é **ULID** (26 chars, ordenável temporalmente), externo aos índices opacos de `StableGraph`.
- `scope_hash` viaja em todo snapshot — o LLM e o validador detectam reload de scope.
- Schema validado em ambas as pontas (`serde` em Rust, `pydantic` em Python na borda do `Engagement.apply_external`).
- Versionamento explícito (`v: 1`); breaking changes incrementam.

### 2.3 Schema de Proposal (do LangGraph para Rust)

```json
{
  "ttp_id": "T1046",
  "target": "10.0.0.0/24",
  "args": {"ports":"22,80,443","rate":"100/s"},
  "rationale": "Subnet RFC1918 listada em scope.yaml e nenhum scan recente; T1590.002 indica DNS já enumerado.",
  "destructive": false
}
```

Validado contra JSON Schema em Rust (`schemars` + `jsonschema` crate). Rejeição é fail-closed.

### 2.4 Audit log (append-only)

`engagement/<id>/audit.jsonl` — uma linha por evento. Hash-chained: cada linha inclui `prev_hash = sha256(linha anterior canônica)`. Não criptografado por padrão (o grafo é o que tem segredos; o audit log tem decisões).

```json
{"ts":1778709400,"audit_id":"01HX…","kind":"proposal","payload":{...},"prev":"sha256:abc…","hash":"sha256:def…"}
{"ts":1778709401,"audit_id":"01HX…","kind":"validation","ok":true,"reasons":[]}
{"ts":1778709405,"audit_id":"01HX…","kind":"execution","ok":true,"new_nodes":12}
```

---

## 3. ADRs (Architecture Decision Records)

Arquivos em `kri0k/docs/adr/`. Formato Michael Nygard, status `Accepted` salvo nota em contrário.

| ID       | Título                                                            | Status         |
|----------|-------------------------------------------------------------------|----------------|
| ADR-0001 | Estado canônico em Rust; Python recebe snapshots                  | Accepted       |
| ADR-0002 | petgraph::StableGraph com IDs ULID externos                       | Accepted       |
| ADR-0003 | JSON compacto na fronteira PyO3 (não MsgPack)                     | Accepted       |
| ADR-0004 | Tokio LocalSet + GIL release; nenhum await sob GIL                | Accepted       |
| ADR-0005 | Validador determinístico em Rust precede toda execução            | Accepted       |
| ADR-0006 | Propose-only é default; `--execute` é opt-in por engagement       | Accepted       |
| ADR-0007 | Audit log append-only com hash chain (JSONL)                      | Accepted       |
| ADR-0008 | LLM local-first via Ollama; APIs externas opt-in com warning      | Accepted       |
| ADR-0009 | Cargo workspace de 5 crates + pkg Python via maturin              | Accepted       |
| ADR-0010 | Licença a definir — cláusula ética suplementar                    | Proposed       |
| ADR-0011 | scope.yaml versionado, checksum embarcado em todo snapshot        | Accepted       |
| ADR-0012 | TTP trait + adapter externo (nmap/whois/crt.sh) com timeout       | Accepted       |

---

## 4. Fluxo de uma iteração `sense → reason → act → update`

### 4.1 Diagrama de sequência

```
operador            Python (LangGraph)         kri0k._native         kri0k-core           Mundo
   │                       │                         │                    │                  │
   │ kri0k run             │                         │                    │                  │
   ├──────────────────────▶│ open_engagement(scope)  │                    │                  │
   │                       ├────────────────────────▶│ scope.parse+hash   │                  │
   │                       │                         ├───────────────────▶│ validate scope   │
   │                       │                         │                    │ load graph       │
   │                       │                         │◀───────────────────┤ Engagement       │
   │                       │◀────────────────────────┤                    │                  │
   │                       │                         │                    │                  │
   │                       │ ── loop until done ──   │                    │                  │
   │                       │                         │                    │                  │
   │ (1) SENSE             │ snap = eng.snapshot()   │                    │                  │
   │                       ├────────────────────────▶│                    │                  │
   │                       │                         ├───────────────────▶│ serialize JSON   │
   │                       │◀────────────────────────┤                    │                  │
   │                       │                         │                    │                  │
   │ (2) REASON            │ proposal = llm(snap,    │                    │                  │
   │                       │     mission, prompts)   │   ← Ollama/etc.    │                  │
   │                       │                         │                    │                  │
   │ (3) VALIDATE          │ vr = eng.validate(prop) │                    │                  │
   │                       ├────────────────────────▶│ validator.check    │                  │
   │                       │                         │  • target ∈ scope? │                  │
   │                       │                         │  • TTP enabled?    │                  │
   │                       │                         │  • destructive?    │                  │
   │                       │◀────────────────────────┤ ValidationResult   │                  │
   │                       │                         │                    │                  │
   │           IF !vr.ok:  │ append finding "reject" │                    │                  │
   │                       │ → reflect, next iter    │                    │                  │
   │                       │                         │                    │                  │
   │ (4) HUMAN GATE (opt)  │ if vr.requires_human:   │                    │                  │
   │ ◀─────────────────────┤ prompt operador (TUI)   │                    │                  │
   │   token ──────────────▶                         │                    │                  │
   │                       │                         │                    │                  │
   │ (5) ACT  (--execute)  │ er = eng.execute(prop,  │                    │                  │
   │                       │       human_gate_token) │                    │                  │
   │                       ├────────────────────────▶│ re-validate        │                  │
   │                       │                         ├───────────────────▶│ ttp.run(target)  │
   │                       │                         │                    ├─────────────────▶│ nmap/whois/...
   │                       │                         │                    │◀─────────────────┤
   │                       │                         │                    │ audit.append     │
   │                       │                         │◀───────────────────┤ apply to graph   │
   │                       │◀────────────────────────┤ ExecutionResult    │                  │
   │                       │                         │                    │                  │
   │ (6) UPDATE GRAPH      │ (já aplicado em Rust)   │                    │                  │
   │                       │                         │                    │                  │
   │ (7) REFLECT           │ summary += er.new_nodes │                    │                  │
   │                       │ check mission done?     │                    │                  │
   │                       │                         │                    │                  │
   │                       │ ── end loop ──          │                    │                  │
```

### 4.2 Estados e invariantes

- **Sense é puro:** `snapshot()` não muta nada; idempotente; pode ser cacheado por iteração.
- **Reason é não-determinístico** (LLM); seu output é apenas dados (`Proposal`), nunca side-effect.
- **Validate é determinístico e fail-closed:** decisão é função pura de `(Proposal, ScopeSnapshot, GraphSnapshot, Policy)`.
- **Act é a única fonte de mutação** do grafo canônico. `apply_external` existe apenas para hidratação inicial (carregar grafo de disco).
- **Update é implícito:** `execute()` aplica nós/arestas dentro do mesmo lock em Rust — não há janela onde Python vê estado parcial.
- **Kill switch (`Engagement.kill`)** termina todos os TTPs em vôo, fecha audit log com `aborted: true`, e bloqueia novos `execute()`.

### 4.3 Modo `--propose-only` vs `--execute`

| Modo            | `validate` roda? | `execute` roda? | Audit log?           | Grafo muda?              |
|-----------------|------------------|-----------------|----------------------|--------------------------|
| `--propose-only`| Sim              | **Nunca**       | Sim (proposals only) | Apenas nós "proposed"    |
| `--execute`     | Sim              | Se vr.ok        | Sim (tudo)           | Sim (nós "executed")     |

Default = `--propose-only` (T1 §criterio MVP-1).

---

## 5. Pontos de extensão

### 5.1 Novos TTPs

Trait `Ttp` em `kri0k-ttp`:

```rust
#[async_trait::async_trait]
pub trait Ttp: Send + Sync {
    fn id(&self) -> &'static str;             // "T1046"
    fn destructive(&self) -> bool { false }
    fn schema(&self) -> &'static schemars::Schema;  // args JSON Schema
    async fn run(
        &self,
        ctx: &TtpCtx,                          // scope, audit handle, kill token
        target: &Target,
        args: &serde_json::Value,
    ) -> Result<TtpOutput, TtpError>;
}
```

`TtpOutput` é um delta tipado `{ new_nodes, new_edges, findings }`. Cada TTP é registrado num `inventory::collect!` (crate `inventory`) — adicionar um novo TTP é um arquivo + um `inventory::submit!`.

Pipeline MVP-1 (T2/T4): T1046, T1590.001 (whois), T1590.002 (dig+passive DNS), T1595.001 (port scan), T1596.003 (crt.sh).

### 5.2 hickory-dns

Resolvedor DNS embarcado para T1590.002 e T1596.001. Plug substitui o adapter `cmd:dig` por uma impl `Resolver` interna. Encaixa em `kri0k-ttp::dns` sem tocar nada fora.

### 5.3 LLM providers

Trait Python `LLMProvider` (`python/kri0k/providers/base.py`) com `chat(messages, tools) -> Proposal | str`. Implementações: `ollama.py` (default), `anthropic.py`, `openai.py`. Selecionado por config `~/.kri0k/config.toml` ou env `KRI0K_LLM=ollama:qwen3-32b`.

### 5.4 Storage do grafo

Hoje: in-memory + JSONL snapshot ao fechar engagement.
Extensão: `GraphStore` trait em `kri0k-graph` permite backend SQLite (rusqlite) ou sled futuramente sem mudar a API pública.

### 5.5 Validador / scope DSL

`kri0k-scope` expõe `Policy` com hooks `pre_validate(&Proposal) -> Decision`. Plugins futuros (ex.: política corporativa "nada de scan entre 22h-6h") encaixam como `Box<dyn ScopePolicy>` empilhados.

### 5.6 Telemetry / observability

`tracing` crate em todo Rust; um subscriber pode ser plugado pra OTLP no futuro. Audit log é separado de tracing — audit é forensics, tracing é debug.

### 5.7 UI

CLI é o entrypoint MVP-0/MVP-1. Pontos de extensão para TUI (ratatui) ou Web (axum + SSE do audit log) já isolados em `kri0k-core::events` (broadcast channel).

---

## 6. Riscos e mitigações (mapeamento para T1 §3)

| Risco T1                              | Mitigação arquitetural                                              |
|---------------------------------------|---------------------------------------------------------------------|
| Alucinação LLM → ação errada (§3.4)   | Validador Rust determinístico + propose-only default + human gate   |
| Scope creep operacional (§3.5)        | scope.yaml + scope_hash em todo snapshot + matcher CIDR/domain      |
| Vazamento do grafo (§3.3)             | (próxima iter) criptografia em repouso; sanitized export já no MVP-0|
| Interop PyO3 frágil (§3.8)            | LocalSet, snapshot-by-value, MPSC, sem await sob GIL (ADR-0004)     |
| Dual-use direto (§3.1)                | Guards técnicos + licença com cláusula ética (ADR-0010, pendente)   |
| Supply chain (§3.6)                   | cargo-vet + cargo audit + uv lock determinístico (T8)               |

---

## 7. O que esta arquitetura **não** decide ainda

Pendente para cards próprios:
- DSL exato do `scope.yaml` (sintaxe completa, herança de perfis ctf/internal/external).
- Política de retenção e cifragem do grafo em disco.
- Escolha final de licença (ADR-0010).
- Backend LLM padrão e qual modelo Ollama (qwen3-32b vs llama3.3 vs deepseek-r1).
- Estratégia de telemetria opt-in (campos, transporte, sinks).

Essas decisões não bloqueiam MVP-0; bloqueiam MVP-1 parcialmente (LLM + scope DSL).
