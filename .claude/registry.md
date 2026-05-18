# Registro de Projeto — Estado e Histórico

> Atualizado pelo agente ao final de cada implementação. Regras de atualização no `CLAUDE.md`.

---

## Informações do Projeto

- **Nome:** kri0k
- **Stack:** Rust (kri0k-core/graph/pybridge via PyO3) + Python 3.11+ (LangGraph, httpx, jinja2, pyyaml). LLM: Ollama local-first.
- **Repositório:** LukeSantossz/kri0k
- **Estrutura:** `crates/` (Rust workspace), `python/kri0k/` (binding + agent + llm), `tests/` (pytest), `config/` (scope.yaml), `docs/` (ADRs + threat model), `.planning/` (GSD).

## Histórico de Implementações

| # | Data | Task | Complexidade | Escopo Alterado | Resultado | Observações |
|---|------|------|--------------|-----------------|-----------|-------------|
| 01 | 2026-05-15 | Phase 2 (TASK-001..012) | major (agregada) | `python/kri0k/llm/*` (8 módulos novos), `python/kri0k/llm/prompts/*.jinja2` (6 templates), `python/kri0k/agent/nodes/sense.py` (wiring), `config/scope.example.yaml` (bloco `llm:`), `tests/test_llm_*.py` (7 arquivos), `tests/test_sense_node.py`, `tests/test_agent_graph.py` (1 teste renomeado), `.planning/phases/02-sense-node-ollama-provider/02-VERIFICATION.md` | aprovado | Sense + OllamaProvider + ping_ollama. 70 testes verdes, ruff/mypy clean. Fix-up necessário para tagged-dict node kind do `_native`. Verificação live falhou (Ollama não rodando local) — modo de falha capturado, sucesso documentado em forma esquemática. |
| 02 | 2026-05-15 | TASK-013 | patch | `tests/test_*.py` (10 arquivos) | aprovado | Adicionados pytest markers (`unit`, `integration`, `graph`) para CI. 51 unit, 7 integration, 12 graph. |
| 03 | 2026-05-16 | Phase 3 (não decomposta em TASK-NNN) | major (agregada) | `python/kri0k/llm/parser.py` (novo), `python/kri0k/llm/prompts/{reason,plan}.jinja2` (2 templates), `python/kri0k/agent/nodes/{reason,plan}.py` (wiring LLM), `python/kri0k/llm/__init__.py` (exports), `tests/test_llm_parser.py`, `tests/test_reason_node.py` (4), `tests/test_plan_node.py` (5) | aprovado | Reason + Plan nodes integrados ao LLM. `<think>` tag stripping resolvido (D-33). 33 testes novos → 103 totais. Commits: `a202c35`, `3ea70d1`, `4659683`, `0be9f0c`. Merged em master via PR #2 junto da Phase 2. **Lacuna documental:** artefatos GSD `03-CONTEXT.md`/`03-DISCUSSION-LOG.md`/`03-PLAN.md` não foram persistidos; apenas `03-VERIFICATION.md` retroativo gerado em 2026-05-18. |
| 04 | 2026-05-18 | TASK-014 | patch | `.gitignore` (+`AGENTS.md`), `.claude/tasks.md`, `.claude/registry.md`, `.planning/phases/03-reason-plan-nodes/03-VERIFICATION.md` (novo retroativo) | aprovado | Sanitização de meta-estado pré-Phase 4. Ignora `AGENTS.md` (espelho local do CLAUDE.md). Registry reconciliado com `master` pós-merge PR #2. VERIFICATION da Phase 3 reconstruído a partir de git+código. |
| 05 | 2026-05-18 | TASK-015 | major | Plan 04-05 completo: `crates/kri0k-pybridge/{Cargo.toml,src/lib.rs,tests/engagement_missing_whois.rs}` (Engagement pyclass + 10 Rust tests), `python/kri0k/{_native.pyi,engagement.py,agent/nodes/{act,sense}.py}` (Python wiring), `tests/{test_act_node,test_engagement_smoke}.py` (16 Python tests), `README.md`+`CONTRIBUTING.md`+`CHANGELOG.md`+`docs/adr/ADR-0013-*.md` (docs pack), `.planning/phases/04-act-node-ttp-whois/{04-05-SUMMARY.md,04-VALIDATION.md}`+`.planning/{STATE,ROADMAP}.md` (meta-state) | aprovado | Phase 4 completa (5/5 plans). 5 commits atômicos (`8fc8877`, `c281f77`, `81e5e05`, `7e7bd96`, `0b35479`, `5949170`). Validações: cargo test workspace 65/65 ✅, clippy strict zero issues, ruff+mypy strict zero issues, pytest unit 104 pass, pytest integration 8/8 pass com whois real (67s). D-63 defense-in-depth L1+L2+L3 covered. AGENT-05 + TTP-04 cumpridos. Alinhamento 100% com 04-05-PLAN.md acordado upfront (helpers renomeados, dedupe unificado, regex const module-level, audit timestamp real). |

## Estado da Codebase

- **Última atualização:** 2026-05-18
- **Último responsável:** kri0k-agent (Claude Opus 4.7)
- **Branch ativa:** `feat/phase-4-act-ttp-whois` (Phase 4 completa, 6 commits ahead de master, aguardando `/gsd-verify-work 4` + PR per D-59)
- **Dependências alteradas recentemente:** Phase 4 add `tokio-util`/`which`/`regex`/`tracing` workspace deps consumidos em pybridge; dev-dep `temp-env = "0.3"` em pybridge
- **Testes passando:** sim — `cargo test --workspace` 65/65 (36 core + 13 graph + 8 pybridge lib + 2 pybridge integration + outras); `pytest -m "not integration"` 104 pass / 15 deselected; `pytest -m integration` 8/8 pass com whois real em 67s
- **Lint/types:** ✅ cargo clippy --workspace strict clean; ruff all checks passed em python/+tests/; mypy strict zero issues
- **Divergências externas pendentes:** nenhuma
- **Última task concluída:** TASK-015 (Phase 4 Plan 04-05 completo — Engagement pyclass + Python wiring + docs)

## Pendências Conhecidas

- **Phase 4 follow-up:** rodar `/gsd-verify-work 4` para verificação final, depois abrir PR `feat/phase-4-act-ttp-whois → master` per D-59. Em `04-05-SUMMARY.md` "Próximo passo".
- **`requirements.md` declara Windows como Out of Scope** mas o desenvolvimento está acontecendo em Windows 11 e Phase 4 foi entregue com `whois.exe` Sysinternals. Aceita divergência documentada (CHANGELOG 0.2.0 e README cobrem o caso Windows + Linux).
- **Phase 3 sem artefatos GSD upstream:** apenas `03-VERIFICATION.md` retroativo existe. Decisões originais da Phase 3 perdidas — recuperáveis parcialmente via `git log` e prompts em `python/kri0k/llm/prompts/`.
- **Hooks Git ainda não instalados** (`git config core.hooksPath` não definido). Próximas sessões podem rodar `bash .claude/setup-hooks.sh` para ativar enforcement local.
- Verificação live de sucesso do `ping_ollama` (Phase 2) requer Ollama rodando + `deepseek-r1:32b` puxado. Documentado em `02-VERIFICATION.md` §4.

## Decisões Técnicas Relevantes

- **Tagged-dict node kind:** `_native.get_dummy_graph()` retorna `kind` como dict `{"type": "host", ...}`, não como string. O formatter (`_node_kind_label`) extrai `kind.type` quando dict, com fallback para string ou `"unknown"`. Importante para Phase 3+ ao consumir snapshots.
- **`get_dummy_graph()` é não-determinístico:** gera ULIDs frescos por chamada. Testes não comparam dois retornos diretamente; comparam contagens e shape.
- **Provider lifecycle:** `OllamaProvider` rastreia `_owns_client`. Quando construído sem `client=` injetado, fecha o `httpx.AsyncClient` no `aclose()`. Caller-injected clients permanecem responsabilidade do caller. `ping_ollama` exemplifica o padrão.
- **`<think>` tags do `deepseek-r1:32b`:** Phase 2 retorna texto cru (D-33). Stripping/parsing fica para Phase 3 (reason node). Documentado em `protocol.py` e `healthcheck.py`.
- **Engagement facade (Phase 4):** `kri0k._native.Engagement` expõe exatamente 5 métodos (`__new__`, `snapshot`, `execute_proposal`, `scope_hash`, `kill`). NÃO há `add_node`/`add_edge` em Python — toda mutação de grafo passa por `execute_proposal` (M-15). Construtor faz fail-fast em `which::which("whois")`. Toda I/O bloqueante (incl. `#[new]`) é wrapped em `py.allow_threads` (Pitfall 7 estendido per CONTEXT.md D-50 update 2026-05-18).
- **D-63 defense in depth (Phase 4):** `execute_proposal` aplica 3 camadas na ordem: Layer 2 (`DOMAIN_REGEX.is_match`) → Layer 1 (`scope.validate_target`) → Layer 3 (`Command::arg` sem shell via `RealSubprocess`). Target malformado nunca chega no subprocess.
- **`Error::ParseError` field name:** usa `origin` (NÃO `source` — evita conflito com `thiserror` `source` magic). Plan 04-05 tinha typo escrevendo `source`; código correto usa `origin`.
- **Dedupe shape do Engagement:** `Mutex<HashMap<(kind_tag: String, natural_key: String), NodeId>>`. Lock order constante: dedupe antes de graph (deadlock-free). Edge invariant: edge Domain↔Org adicionada se ≥1 endpoint novo; edge Domain→NS só se NS é nova.
- **act/sense backward-compat (Phase 4):** `act.py` levanta `RuntimeError("engagement missing...")` se `propose_only=False` AND engagement ausente — never silent. `sense.py` prefere `engagement.snapshot()` mas mantém fallback `_native.get_dummy_graph()` para testes Phase 1/2 não quebrarem.

## Padrões Recorrentes Observados

| Padrão | Frequência | Impacto | Ação Corretiva |
|--------|------------|---------|----------------|
| —      | —          | —       | —              |

---

## Notas de Sessão

- **Framework atualizado para 2.0.0.** Compressão: 11 regras consolidadas em CLAUDE.md único. Regras 00-09 inline. Regra 10 (modelo/checkpoint) removida. quick-ref.md eliminado. Guias e modos condicionais movidos para `.claude/skills/`. PRD virou skill de bootstrap. Hooks e templates inalterados.
- **2026-05-15 — Reconciliação framework `.claude/` x GSD.** Decisão do usuário: arquivos sob `.planning/` e `.claude/` (artefatos de framework / metaframework) são tratados como **fora da trava de segurança** do CLAUDE.md. Justificativa: GSD opera por design escrevendo CONTEXT.md/PLAN.md/STATE.md/etc. sem necessariamente abrir tasks individuais em `tasks.md`. Código do projeto (Rust em `crates/`, Python em `python/`, configs em `config/`, docs em `docs/`, testes em `tests/`) **continua sob a trava** e exige task registrada. Risco assumido: a trava perde cobertura sobre artefatos de planejamento, mas eles são reversíveis e auditáveis via git.
- **2026-05-18 — Sessão de mapeamento + sanitização (TASK-014).** Reconhecimento da codebase executado em modo Review. Validações funcionais re-executadas: cargo check/clippy ✅, ruff ✅, mypy strict ✅, pytest unit 60/60 ✅. Divergências encontradas: (a) registry desatualizado sobre Phase 3 e branch ativa; (b) `AGENTS.md` untracked na raiz (espelho idêntico do CLAUDE.md, presumivelmente gerado por tooling externo agentic-md); (c) `.planning/phases/03-reason-plan-nodes/` vazia. Decisões do usuário: (a) sanitizar primeiro; (b) ignorar AGENTS.md no `.gitignore`; (c) próxima phase será Phase 4 (Act + TTP Whois) usando `whois.exe` Sysinternals. Único arquivo do projeto tocado: `.gitignore` (TASK-014 patch). Resto é meta-framework dentro da exceção declarada em 2026-05-15.
- **2026-05-18 — Sessão Phase 4 Plan 04-05 (TASK-015 conclusão).** Estado inicial: 4/5 plans Phase 4 commitados; `crates/kri0k-pybridge/{Cargo.toml,src/lib.rs}` com working draft (~80% Task 1) e divergências do plano. Decisão do usuário no início da sessão: "alinhar 100% com o plano, commit task 1 primeiro" + "prossiga" para cada task seguinte. Execução: 5 commits atômicos sequenciais (Task 1→5) cada um validado por suite específica antes do commit. Refatoração da Task 1 corrigiu: assinatura `#[new]` para só `scope_dict` (objective/propose_only movidos para Python); dedupe HashMap unificado com edge invariant correto; helpers `error_to_outcome`+`build_executed_outcome`; `DOMAIN_REGEX` const module-level; `#[instrument]` em snapshot/execute_proposal; audit timestamp `unix-ms:N` real com `?` propagation; 8 testes inline `#[cfg(test)] mod tests` adicionados. Clippy strict exigiu 5 micro-fixes (doc backticks, `&Error` allow, `'x'` char pattern, `map_or_else` para SystemTime, allow em apply_whois_output para legibilidade). Maturin develop necessário para o `_native.pyd` exportar a nova `Engagement` class — sem isso, smoke tests falhavam em import. Integration suite passou em 67s contra whois.exe real (Sysinternals v1.21). Phase 4 totalmente fechada: AGENT-05 + TTP-04 cumpridos, D-63 L1+L2+L3 covered, M-02/M-03/M-05/M-15/M-34/M-36/AB-03 todos covered. Branch `feat/phase-4-act-ttp-whois` agora 6 commits ahead de master, pronta para `/gsd-verify-work 4` + PR per D-59.
