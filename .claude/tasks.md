# TASKS.md — Registro de Tasks

> Nenhum agente pode modificar a codebase sem task registrada aqui.

---

## Mini-Template (Patch)

```markdown
### TASK-[NNN] | patch
- **Status:** pendente | em andamento | concluída
- **Objetivo:** [uma frase]
- **Arquivo(s):** [listar]
```

## Template Completo (Minor / Major)

```markdown
### TASK-[NNN]
- **Status:** pendente | em andamento | concluída | descartada | revertida
- **Modo:** desenvolvimento | review | tutor
- **Complexidade:** minor | major
- **Data de criação:** [YYYY-MM-DD]

#### Objetivo (!obrigatório)
[Uma frase clara. Teste: se alguém ler apenas esta linha, entende o que será entregue?]

#### Contexto (!obrigatório)
[Por que essa mudança é necessária? Link de issue/PR/card se houver.]

#### Escopo Técnico (!obrigatório)
- **Arquivos/módulos envolvidos:** [listar]
- **Dependências necessárias:** [novas ou "nenhuma"]
- **Impacto em funcionalidades existentes:** [descrever ou "nenhum"]

#### Critérios de Aceite (!obrigatório)
- [ ] [Critério verificável 1]
- [ ] [Critério verificável 2]
- [ ] [Critério verificável 3]

#### Restrições (opcional)
[Limitações técnicas, de tempo, de escopo.]

#### Referências (opcional)
[Links de documentação, PRs, issues, artigos.]

#### Log de Andamento (atualizado pelo agente)
| Data | Sessão | Ação Realizada | Status ao Final |
|------|--------|----------------|-----------------|
| —    | —      | —              | —               |

#### Resultado (preenchido ao concluir)
- **Data de conclusão:** [YYYY-MM-DD]
- **Branch:** [nome]
- **Commit(s):** [hash ou mensagem]
- **Avaliação pós-implementação:** [aprovado / com ressalvas / reprovado]
- **Observações:** [notas para futuras tasks]
```

### Classificação de Complexidade

| Nível | Quando usar | Exemplos |
|-------|-------------|----------|
| **patch** | Mudança trivial, sem risco de efeito colateral | Renomear variável, corrigir typo, ajustar espaçamento |
| **minor** | Mudança localizada, risco baixo | Função isolada, bug em um arquivo, adicionar teste |
| **major** | Mudança estrutural, múltiplos arquivos, risco de cascata | Nova feature multi-módulo, refatoração arquitetural |

---

## Tasks Ativas

> A primeira task listada é a ativa. O agente trabalha nela até conclusão, descarte ou bloqueio.

_(nenhuma task ativa — última conclusão: TASK-015 em 2026-05-18)_

## Tasks Concluídas

> Movidas para cá após conclusão e atualização do `registry.md`. Nunca remova entradas.

### TASK-015 | major (concluída 2026-05-18) — Phase 4: Act Node + TTP Whois

- **Objetivo:** Implementar Phase 4 (T1590.001 whois) e expor `Engagement` pyclass como container canônico de estado, fechando o loop sense→reason→plan→act→reflect.
- **Branch:** `feat/phase-4-act-ttp-whois` (6 commits ahead de master, aguardando PR per D-59)
- **Plans GSD executados (5/5):** 04-01 NodeKind/EdgeKind · 04-02 Error+deps · 04-03 WhoisTtp+Subprocess · 04-04 ScopeConfig · 04-05 Engagement+wiring+docs
- **Commits do plan 04-05:** `8fc8877` (Task 1 pyclass) · `c281f77` (Task 2 stubs+bootstrap) · `81e5e05` (Task 3 act/sense) · `7e7bd96` (Task 4 tests) · `0b35479` (Task 5 docs) · `5949170` (meta-state wrap-up)
- **Avaliação pós-implementação:** aprovado — cargo test workspace 65/65 ✅, clippy strict zero issues, ruff+mypy strict zero issues, pytest unit 104 pass, pytest integration 8/8 pass com whois real (67s)
- **Requisitos completados:** AGENT-05, TTP-04. D-63 defense-in-depth L1+L2+L3 covered. M-02/M-03/M-05/M-15/M-34/M-36/AB-03 todos covered.
- **Lições:**
  1. Trabalho draft pré-existente em `lib.rs` foi refatorado 100% para o plano em vez de adaptar — escolha do usuário que evitou divergência documentada futura.
  2. Plan typo `Error::ParseError { source }` corrigido em runtime para `origin` (campo real do `thiserror` enum).
  3. `maturin develop` é mandatório após mudanças em cdylib antes de qualquer pytest que importe `_native` (smoke tests inicialmente falhavam por `.pyd` antigo sem a class `Engagement`).
- **Próximo passo (fora desta task):** `/gsd-verify-work 4` + PR `feat/phase-4-act-ttp-whois → master` per D-59.

### Phase 2 — Sense Node + Ollama Provider (concluída 2026-05-15)

Branch: `feat/phase-2-sense-ollama` · Commits: 13 (12 task commits + 1 fix-up).

| Task | Tipo | Commit | Resultado |
|------|------|--------|-----------|
| TASK-001 | minor | `chore(llm): scaffold kri0k.llm package` | aprovado |
| TASK-002 | minor | `feat(llm): add LLMConfig dataclass with scope-only model override` | aprovado |
| TASK-003 | minor | `feat(llm): define LLMProvider protocol returning raw text` | aprovado |
| TASK-004 | minor | `feat(llm): add hybrid snapshot formatter with M-16 sanitization` | aprovado (com fix-up sobre tagged-dict node kind) |
| TASK-005 | minor | `feat(llm): add async TokenBucket for rate limiting` | aprovado |
| TASK-006 | major | `feat(llm): add OllamaProvider with retry, backoff, and rate limiting` | aprovado |
| TASK-007 | minor | `feat(llm): add jinja2 prompt templates and on-demand loader` | aprovado |
| TASK-008 | minor | `feat(llm): add ping_ollama health-check module` | aprovado |
| TASK-009 | minor | `feat(agent): wire sense node to format Rust snapshot` | aprovado |
| TASK-010 | patch | `docs(config): document llm.model override in scope.example.yaml` | aprovado |
| TASK-011 | minor | `feat(llm): expose build_provider factory and public exports` | aprovado |
| TASK-012 | patch | `docs(phase-2): record live ollama verification` | aprovado |

Fix-up: `fix(llm): handle tagged-dict node kind in formatter histogram` —
descoberto durante execução: o `_native.get_dummy_graph()` emite `kind` como
dict tagged-union (`{"type": "host", ...}`), não como string.

Lint pass: `style(llm): satisfy ruff strict and mypy strict on phase-2 surface`.

### TASK-013 | patch (concluída 2026-05-15)
- **Objetivo:** Adicionar pytest markers para CI
- **Arquivos:** `tests/test_*.py` (10 arquivos)
- **Commit:** `51a1ec8` `test(ci): add pytest markers for CI test execution`
- **Resultado:** aprovado — 51 unit, 7 integration, 12 graph

### TASK-014 | patch (concluída 2026-05-18)
- **Objetivo:** Ignorar `AGENTS.md` (espelho local do CLAUDE.md, gerado por tooling externo) no controle de versão.
- **Arquivos:** `.gitignore`
- **Resultado:** aprovado — `git status` confirma que `AGENTS.md` deixou de aparecer como untracked.
- **Acompanhamento (meta-framework, fora da trava):** sincronizado `.claude/registry.md` com Phase 3 + branch `master`, gerado `.planning/phases/03-reason-plan-nodes/03-VERIFICATION.md` retroativo a partir de git + código.

## Tasks Descartadas

[nenhuma task descartada]

---

## Regras de Preenchimento

1. Objetivo cabe em uma frase. Se não cabe, quebre em subtasks.
2. Uma task por sessão. Se afeta mais de 10 arquivos, decompor.
3. Critérios de Aceite verificáveis — "funcionar corretamente" não conta.
4. Escopo Técnico lista arquivos concretos.
5. Uma task por implementação. Necessidade fora do escopo → nova task.
6. Tasks não são retroativas. Código sem task → Modo Review.
7. Na dúvida sobre complexidade, classifique para cima.
8. Log de Andamento obrigatório para minor/major.
9. Tasks revertidas não são deletadas — recebem status `revertida`.

## Política de Arquivamento

Quando "Tasks Concluídas" ultrapassar 20 entradas: mover as mais antigas (manter 10 recentes) para `.claude/tasks-archive.md`. Arquivo cumulativo, nunca editado após inserção.
