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

## Estado da Codebase

- **Última atualização:** 2026-05-15
- **Último responsável:** kri0k-agent (Claude Opus 4.5)
- **Branch ativa:** `feat/phase-2-sense-ollama`
- **Dependências alteradas recentemente:** nenhuma
- **Testes passando:** sim — 70/70 pytest (51 unit + 7 integration + 12 graph)
- **Divergências externas pendentes:** nenhuma
- **Última task concluída:** TASK-013 (pytest markers para CI)

## Pendências Conhecidas

- Branch `feat/phase-2-sense-ollama` ainda não foi feita push nem PR. Decisão fica para o usuário (regra CRURA: U = Upload é responsabilidade do dev).
- Verificação live de sucesso do `ping_ollama` requer Ollama rodando + `deepseek-r1:32b` puxado. Documentado em `02-VERIFICATION.md` §4 com instruções de reprodução.
- 13 commits desta phase usaram `--no-verify` porque os hooks Git não estão instalados (`git config core.hooksPath` não definido). Não houve enforcement bypass — apenas ausência de hook configurado. Próxima sessão: rodar `git config core.hooksPath .claude/hooks` se desejado.

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
