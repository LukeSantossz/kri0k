# Phase 2: Sense Node + Ollama Provider — Discussion Log

**Session date:** 2026-05-15
**Mode:** discuss (default)
**Workflow:** /gsd-discuss-phase 2

> Human-reference log only. Downstream agents (researcher, planner) consume `02-CONTEXT.md`, not this file.

---

## Gray areas presented

1. ☐ Snapshot + prompt template
2. ☐ Rate limit + retry policy *(claude's discretion — see CONTEXT.md D-28..D-30)*
3. ☐ Organização e fronteira do LLM *(claude's discretion — see CONTEXT.md D-31..D-33)*
4. ☐ Configuração do modelo

## Areas selected by user

- Configuração do modelo
- Snapshot + prompt template

---

## Area: Configuração do modelo

| Question | Options presented | User selection |
|----------|-------------------|----------------|
| Modelo Ollama default | qwen3:32b / deepseek-r1:32b / modelo menor para dev | **deepseek-r1:32b** |
| Override em runtime | env var apenas / scope.yaml apenas / scope.yaml com fallback env | **scope.yaml apenas (campo llm.model)** |
| Onde lê config | LLMConfig dataclass em llm/config.py / provider lê direto / dentro de engagement_context | **LLMConfig dataclass em llm/config.py** |

**Heads-up emitido:** deepseek-r1:32b emite tags `<think>...</think>` no output — handling fica para Phase 3.

---

## Area: Snapshot + prompt template

| Question | Options presented | User selection |
|----------|-------------------|----------------|
| Formato do snapshot para o LLM | JSON cru / resumo textual / híbrido | **Híbrido (resumo + JSON anexo)** |
| Dono da formatação | sense armazena bruto / sense formata e armazena / helper format_snapshot_for_llm | **sense.py formata e armazena já formatado** |
| Estrutura do prompt | system+user separados / single template com seções / templates por nó | **Templates por nó** |
| Localização dos templates | python/kri0k/llm/prompts/*.jinja2 / strings inline / YAML versionado | **python/kri0k/llm/prompts/*.jinja2** |

### Follow-up: prova E2E do provider

Ambiguidade detectada: critério 3 do ROADMAP exige "prompt template renderiza snapshot + contexto" mas sense não chama LLM.

| Question | Options presented | User selection |
|----------|-------------------|----------------|
| Como Phase 2 prova o provider end-to-end | Provider standalone + testes / sense chama LLM com prompt de observação / módulo health-check | **Módulo health-check** |

### Follow-up: conflito de tipo

Conflito detectado: `AgentState.snapshot: dict[str, Any]` (Phase 1 D-02) vs escolha de "sense armazena já formatado".

| Question | Options presented | User selection |
|----------|-------------------|----------------|
| Como resolver | dict com keys raw/formatted / trocar snapshot para str + adicionar snapshot_raw / reverter para helper externo | **snapshot vira dict com chaves 'raw' e 'formatted'** |

Resultado: `AgentState.snapshot` permanece tipado como `dict[str, Any]`, mantendo o contrato de Phase 1; convenção interna para `{"raw": dict, "formatted": str}`.

---

## Claude's discretion (áreas não selecionadas)

Resolvidas conforme padrões do projeto:

- **Rate limit:** token bucket async, 10/min, estado por instância de provider (D-28)
- **Backoff:** exponencial 1s→30s, max 5 retries, jitter ±20% (D-29)
- **Retry list:** 429, 5xx, ConnectError, ReadTimeout (D-30)
- **Layout do módulo:** `python/kri0k/llm/{config,protocol,ollama,rate_limit,healthcheck,formatters}.py` + `prompts/` (D-31)
- **Factory:** `build_provider(config)` em `llm/__init__.py`, instância em `engagement_context["llm_provider"]` (D-32)
- **Parsing:** Phase 2 retorna texto cru — sem JSON mode/Pydantic (D-33)

---

## Scope creep redirected

Nenhum item de scope creep emergiu durante a discussão.

## Cross-framework decision

Conflito entre a trava de segurança em `.claude/CLAUDE.md` e a escrita de artefatos GSD em `.planning/`. Decisão do usuário: **`.planning/` e `.claude/` (arquivos de framework) tratados como fora da trava**. Decisão a ser registrada em `.claude/registry.md`.

---

*Discussion completed: 2026-05-15*
