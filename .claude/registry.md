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

## Estado da Codebase

- **Última atualização:** 2026-05-18
- **Último responsável:** kri0k-agent (Claude Opus 4.7)
- **Branch ativa:** `master` (Phases 1-3 merged via PR #1 + PR #2; também `feat/phase-2-sense-ollama` mergeada em `50991b9`)
- **Dependências alteradas recentemente:** PyO3 upgraded para 0.24 (compat Python 3.14 CI, commit `3b4d4f5`)
- **Testes passando:** sim — 60/60 unit em 1.10s; 103 totais coletados (51 unit + 7 integration + 12 graph + 33 Phase-3 não-markados)
- **Lint/types:** ✅ cargo clippy strict clean; ruff all checks passed; mypy strict 0 issues em 20 files
- **Divergências externas pendentes:** nenhuma
- **Última task concluída:** TASK-014 (sanitização de meta-estado 2026-05-18)

## Pendências Conhecidas

- **Phase 4 ambiente Windows:** `whois` CLI não existe nativamente. Decisão tomada: usar `whois.exe` da Sysinternals (fiel ao ADR-0012). Documentar pré-requisito no README ao planejar Phase 4.
- **`requirements.md` declara Windows como Out of Scope** mas o desenvolvimento está acontecendo em Windows 11. Revisar essa declaração antes da Phase 4 ou aceitar divergência documentada.
- **Phase 3 sem artefatos GSD upstream:** apenas `03-VERIFICATION.md` retroativo existe. Decisões originais da Phase 3 perdidas — recuperáveis parcialmente via `git log` e prompts em `python/kri0k/llm/prompts/`.
- **Hooks Git ainda não instalados** (`git config core.hooksPath` não definido). Próximas sessões podem rodar `bash .claude/setup-hooks.sh` para ativar enforcement local.
- Verificação live de sucesso do `ping_ollama` (Phase 2) requer Ollama rodando + `deepseek-r1:32b` puxado. Documentado em `02-VERIFICATION.md` §4.

## Decisões Técnicas Relevantes

- **Tagged-dict node kind:** `_native.get_dummy_graph()` retorna `kind` como dict `{"type": "host", ...}`, não como string. O formatter (`_node_kind_label`) extrai `kind.type` quando dict, com fallback para string ou `"unknown"`. Importante para Phase 3+ ao consumir snapshots.
- **`get_dummy_graph()` é não-determinístico:** gera ULIDs frescos por chamada. Testes não comparam dois retornos diretamente; comparam contagens e shape.
- **Provider lifecycle:** `OllamaProvider` rastreia `_owns_client`. Quando construído sem `client=` injetado, fecha o `httpx.AsyncClient` no `aclose()`. Caller-injected clients permanecem responsabilidade do caller. `ping_ollama` exemplifica o padrão.
- **`<think>` tags do `deepseek-r1:32b`:** Phase 2 retorna texto cru (D-33). Stripping/parsing fica para Phase 3 (reason node). Documentado em `protocol.py` e `healthcheck.py`.

## Padrões Recorrentes Observados

| Padrão | Frequência | Impacto | Ação Corretiva |
|--------|------------|---------|----------------|
| —      | —          | —       | —              |

---

## Notas de Sessão

- **Framework atualizado para 2.0.0.** Compressão: 11 regras consolidadas em CLAUDE.md único. Regras 00-09 inline. Regra 10 (modelo/checkpoint) removida. quick-ref.md eliminado. Guias e modos condicionais movidos para `.claude/skills/`. PRD virou skill de bootstrap. Hooks e templates inalterados.
- **2026-05-15 — Reconciliação framework `.claude/` x GSD.** Decisão do usuário: arquivos sob `.planning/` e `.claude/` (artefatos de framework / metaframework) são tratados como **fora da trava de segurança** do CLAUDE.md. Justificativa: GSD opera por design escrevendo CONTEXT.md/PLAN.md/STATE.md/etc. sem necessariamente abrir tasks individuais em `tasks.md`. Código do projeto (Rust em `crates/`, Python em `python/`, configs em `config/`, docs em `docs/`, testes em `tests/`) **continua sob a trava** e exige task registrada. Risco assumido: a trava perde cobertura sobre artefatos de planejamento, mas eles são reversíveis e auditáveis via git.
- **2026-05-18 — Sessão de mapeamento + sanitização (TASK-014).** Reconhecimento da codebase executado em modo Review. Validações funcionais re-executadas: cargo check/clippy ✅, ruff ✅, mypy strict ✅, pytest unit 60/60 ✅. Divergências encontradas: (a) registry desatualizado sobre Phase 3 e branch ativa; (b) `AGENTS.md` untracked na raiz (espelho idêntico do CLAUDE.md, presumivelmente gerado por tooling externo agentic-md); (c) `.planning/phases/03-reason-plan-nodes/` vazia. Decisões do usuário: (a) sanitizar primeiro; (b) ignorar AGENTS.md no `.gitignore`; (c) próxima phase será Phase 4 (Act + TTP Whois) usando `whois.exe` Sysinternals. Único arquivo do projeto tocado: `.gitignore` (TASK-014 patch). Resto é meta-framework dentro da exceção declarada em 2026-05-15.
