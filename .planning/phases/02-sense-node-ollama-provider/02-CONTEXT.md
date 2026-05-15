# Phase 2: Sense Node + Ollama Provider - Context

**Gathered:** 2026-05-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Tornar o nó `sense` funcional (obtém snapshot do grafo Rust via `_native.get_dummy_graph()` e formata para consumo do LLM) e entregar o primeiro provider LLM (Ollama via httpx, com rate limiting, retry e prompt templating Jinja2). A integração `sense → LLM` no graph loop não é entregue aqui — Phase 2 prova o provider via módulo de health-check separado. Structured output / parsing fica para Phase 3.

Requisitos cobertos: AGENT-02, LLM-01, LLM-02, LLM-03, LLM-04.

</domain>

<decisions>
## Implementation Decisions

### Modelo Ollama
- **D-18:** Modelo default = `deepseek-r1:32b` (referenciado em PROJECT.md). Heads-up: emite tags `<think>...</think>` no output — handling fica para Phase 3 (reason/plan)
- **D-19:** Override de modelo via `scope.yaml` campo `llm.model` apenas. Sem fallback para env var (uma fonte de verdade, alinhado com ADR-0011 scope.yaml + checksum)
- **D-20:** Configuração lida em `LLMConfig` dataclass em `python/kri0k/llm/config.py`. Instanciada uma vez no bootstrap, dependency-injected no provider

### Sense node
- **D-21:** `sense` formata o snapshot e armazena resultado. `AgentState.snapshot` permanece `dict[str, Any]` (tipo de Phase 1 não muda) com forma:
  ```python
  {
      "raw": <dict retornado por _native.get_dummy_graph()>,
      "formatted": <str pronta para injetar no prompt do LLM>
  }
  ```
- **D-22:** Estratégia de formatação = **híbrida** (resumo textual no topo + JSON pretty-printed anexo). Helper `format_snapshot_hybrid(raw: dict) -> str` em `python/kri0k/llm/formatters.py`
- **D-23:** Sense permanece puro — não chama LLM em Phase 2. Loop sense→reason só conecta em Phase 3

### Prompt templates
- **D-24:** Templates por nó em `python/kri0k/llm/prompts/`:
  - `sense.jinja2` (Phase 2: entregue funcional)
  - `reason.jinja2`, `plan.jinja2`, `act.jinja2`, `reflect.jinja2` (Phase 2: stubs vazios para fixar estrutura)
- **D-25:** Loader Jinja2 com `FileSystemLoader` apontando para o diretório `prompts/`. Templates carregados sob demanda, não no import
- **D-26:** Variáveis do template `sense`: `formatted_snapshot`, `scope`, `objective`, `iteration_count`, `history_summary`

### Prova de funcionamento do provider (sem ligar no graph loop)
- **D-27:** Módulo `python/kri0k/llm/healthcheck.py` com `async def ping_ollama(config: LLMConfig) -> PingResult`. Renderiza um template trivial (`healthcheck.jinja2`) e faz uma chamada real. Servirá de base para futuro comando CLI `kri0k doctor`

### Rate limit + retry (Claude's discretion — LLM-04)
- **D-28:** Token bucket assíncrono em `python/kri0k/llm/rate_limit.py`. Capacidade 10, refill 10/60s. Estado mantido na instância de `OllamaProvider` (uma por engagement)
- **D-29:** Backoff exponencial: base 1s, max 30s, max 5 retries, jitter ±20%
- **D-30:** Retry para: status 429, 5xx, `httpx.ConnectError`, `httpx.ReadTimeout`. Propaga (sem retry): 4xx ≠ 429, erros de parse JSON, erros de template

### Organização do módulo (Claude's discretion)
- **D-31:** Layout `python/kri0k/llm/`:
  ```
  llm/
  ├── __init__.py        # exporta LLMProvider, OllamaProvider, LLMConfig, build_provider
  ├── config.py          # LLMConfig dataclass
  ├── protocol.py        # LLMProvider Protocol (async chat method)
  ├── ollama.py          # OllamaProvider httpx implementation
  ├── rate_limit.py      # TokenBucket async
  ├── healthcheck.py     # ping_ollama
  ├── formatters.py      # format_snapshot_hybrid
  └── prompts/
      ├── sense.jinja2
      ├── reason.jinja2 (stub)
      ├── plan.jinja2 (stub)
      ├── act.jinja2 (stub)
      ├── reflect.jinja2 (stub)
      └── healthcheck.jinja2
  ```
- **D-32:** Factory `build_provider(config: LLMConfig) -> LLMProvider` em `llm/__init__.py`. Chamada uma vez no bootstrap do engagement; instância armazenada em `engagement_context["llm_provider"]` (coerente com D-03 de Phase 1)

### Fronteira de parsing
- **D-33:** Phase 2 retorna **texto cru** do LLM (campo `message.content` do Ollama). Sem JSON mode, sem Pydantic, sem validação de schema. Parsing estruturado é responsabilidade de Phase 3 (reason/plan)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Architecture
- `docs/adr/ADR-0001-canonical-state-in-rust.md` — Rust é source of truth
- `docs/adr/ADR-0008-llm-local-first-ollama.md` — Justifica Ollama local
- `docs/adr/ADR-0011-scope-yaml-checksum.md` — scope.yaml como configuração canônica do engagement
- `docs/ARCHITECTURE.md` — visão geral do sistema
- `.planning/codebase/STACK.md` — dependências disponíveis (httpx, jinja2, ollama, pyyaml)
- `.planning/codebase/STRUCTURE.md` — convenções de organização
- `.planning/phases/01-langgraph-structure/01-CONTEXT.md` — decisões D-01 a D-17 aplicáveis

### Código existente relevante
- `python/kri0k/agent/state.py` — `AgentState` TypedDict (snapshot: `dict[str, Any]`)
- `python/kri0k/agent/nodes/sense.py` — placeholder atual a ser substituído
- `python/kri0k/_native.pyi` — assinatura de `get_dummy_graph() -> dict[str, Any]`
- `python/kri0k/__init__.py` — exports atuais
- `config/scope.example.yaml` — adicionar seção `llm.model` aqui
- `pyproject.toml` — confirmar httpx/jinja2/pyyaml em [project.dependencies]; ollama é opcional

### Testes existentes
- `tests/test_agent_graph.py` — 12 tests do schema de `AgentState`. Confirmar que mudar `snapshot` para `{"raw": ..., "formatted": ...}` não quebra nenhum teste (D-21 mantém o tipo `dict[str, Any]`, mas pode haver assertion sobre o conteúdo)
- `tests/test_smoke.py` — pode receber smoke test do `ping_ollama` (gated por env var)

</canonical_refs>

<code_context>
## Existing Code Insights

### Assets reusáveis
- `kri0k._native.get_dummy_graph()` — retorna dict pronto, sense só consome
- `AgentState` TypedDict (Phase 1) — sense vai escrever em `snapshot` e ler `engagement_context`
- httpx já está nas dependências (`pyproject.toml`)
- jinja2 já está nas dependências
- pyyaml já está nas dependências (para scope.yaml)
- Tokio runtime do PyO3 não é usado em Phase 2 (LLM calls são puro Python async)

### Padrões estabelecidos
- **Async first:** todos os nodes e operações de I/O são `async def`
- **mypy strict:** type hints obrigatórios em toda função pública. Protocols/dataclasses para contratos
- **ruff strict:** ANN, S, B habilitados. Sem `# noqa` exceto justificado
- **Snake case** em Python; PascalCase em classes e TypedDicts
- **Sem unsafe / sem unwrap:** lints estritos do projeto. Em Python, equivalente é tratar exceções de forma útil (propagar tipadas, não engolir)

### Integration points
- `python/kri0k/agent/nodes/sense.py` — substituir placeholder (linha 12, return {})
- `python/kri0k/__init__.py` — pode precisar exportar `LLMConfig` e `build_provider`
- `config/scope.example.yaml` — adicionar bloco `llm:` exemplo
- Bootstrap do engagement (futuro Phase 12 CLI) chamará `build_provider` — Phase 2 só precisa que a factory exista e seja testada

### Não tocar
- ROADMAP.md e REQUIREMENTS.md (atualizados por /gsd-transition após a fase)
- Crates Rust (Phase 2 é Python puro)

</code_context>

<specifics>
## Specific Ideas

- O comportamento de `deepseek-r1:32b` com tags `<think>` deve ser documentado num comment do `healthcheck.py` (next fase precisa saber lidar)
- Health-check é base do futuro `kri0k doctor` (CLI Phase 12)
- Rate limit "10 req/min" do critério 4 é interpretado como **token bucket** com capacidade 10 e refill linear de 10 tokens por 60s — não como "exatamente 1 req a cada 6s"

</specifics>

<deferred>
## Deferred Ideas

- **Structured output (JSON mode / Pydantic):** Phase 3 (reason/parses) — necessário para AGENT-03/04
- **Handling de tags `<think>` do deepseek-r1:** Phase 3 (responsabilidade do reason node)
- **CLI `kri0k doctor`:** Phase 12 (CLI Commands) — reutilizará `ping_ollama`
- **Provider switching runtime (Anthropic/OpenAI):** v2 (LLM-V2-03), fora do escopo do milestone 1
- **Prompt versioning (A/B testing):** sem demanda atual, não introduzir overhead em Phase 2

</deferred>

---

*Phase: 02-sense-node-ollama-provider*
*Context gathered: 2026-05-15*
