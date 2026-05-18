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

### TASK-015
- **Status:** em andamento
- **Modo:** desenvolvimento
- **Complexidade:** major
- **Data de criação:** 2026-05-18

#### Objetivo (!obrigatório)
Implementar Phase 4 (Act Node + TTP Whois): primeiro TTP concreto (T1590.001 whois) executável via subprocess, expondo `Engagement` pyclass como container canônico de estado, fechando o loop sense→reason→plan→act→reflect do agente LangGraph.

#### Contexto (!obrigatório)
Phase 4 do roadmap GSD (MVP Execution Loop, Milestone 1). Implementa AGENT-05 + TTP-01..05. Pré-requisitos: Phases 1-3 já estão em master. Branch dedicada `feat/phase-4-act-ttp-whois` criada. Pré-req runtime: `whois.exe` (Sysinternals v1.21) instalado via winget. Plans GSD: `.planning/phases/04-act-node-ttp-whois/04-{01..05}-PLAN.md` (29 decisões D-34..D-64 lockadas em CONTEXT.md; RESEARCH.md cobre 16 áreas técnicas com 13 pitfalls verificados).

#### Escopo Técnico (!obrigatório)
- **Arquivos/módulos envolvidos:**
  - Rust: `crates/kri0k-graph/src/lib.rs` (NodeKind+EdgeKind), `crates/kri0k-core/{Cargo.toml,src/{lib,audit,scope}.rs}`, `crates/kri0k-core/src/ttp/{mod,subprocess,whois}.rs` (novos), `crates/kri0k-pybridge/{Cargo.toml,src/lib.rs,tests/engagement_missing_whois.rs}`
  - Python: `python/kri0k/{_native.pyi, engagement.py, agent/nodes/{act,sense}.py}`
  - Tests: `tests/{test_act_node, test_engagement_smoke}.py`, `tests/fixtures/whois_{google_com,example_com,invalid}.txt`, fixtures espelhadas em `crates/kri0k-core/tests/fixtures/`
  - Docs: `README.md` (Running whois TTP), `CONTRIBUTING.md` (Adding a new TTP), `CHANGELOG.md` (0.2.0), `docs/adr/ADR-0013-ttp-trait-subprocess-abstraction.md`
  - Config: `config/scope.example.yaml` (v1 schema completo), `Cargo.toml` workspace (deps), `.claude/tasks.md` (esta task)
- **Dependências necessárias:** Rust — `tokio` (process, time, sync), `tokio-util = "0.7"`, `which = "8"`, `async-trait = "0.1"`, `tracing = "0.1"`, `serde_yaml_ng = "0.10"`, `regex = "1"`, dev: `temp-env = "0.3"`. Python — nenhuma nova (já tem ollama, httpx, pyyaml, jinja2).
- **Impacto em funcionalidades existentes:** Refactor do trait `Ttp` síncrono → async (cirúrgico — métodos hoje são `todo!()`). Sense node ganha branch backward-compat para usar `engagement.snapshot()` quando presente em `engagement_context`. Demais funcionalidades de Phases 1-3 (LLM, parser, reason/plan) permanecem intactas.

#### Critérios de Aceite (!obrigatório)
- [ ] ROADMAP Phase 4 critério 1: `crates/kri0k-core/src/ttp/whois.rs` existe com `impl Ttp for WhoisTtp`
- [ ] ROADMAP Phase 4 critério 2: `whois example.com` retorna `WhoisOutput { registrar, nameservers, dates, ... }` populado
- [ ] ROADMAP Phase 4 critério 3: `Engagement::execute_proposal` adiciona ≥1 Domain + ≥1 Organization + N Nameserver nodes ao grafo
- [ ] ROADMAP Phase 4 critério 4: 2 chamadas consecutivas de `WhoisTtp::execute` levam ≥1.0s wall-clock (rate limit)
- [ ] CONTEXT D-64.2: `cargo clippy --workspace --all-targets -- -D warnings` zero issues
- [ ] CONTEXT D-64.3: `ruff check python/ tests/` + `mypy python/kri0k` strict zero issues
- [ ] CONTEXT D-64.4: `cargo test --workspace --features integration` + `pytest tests/` 100% pass
- [ ] CONTEXT D-64.5: README/CONTRIBUTING/CHANGELOG/ADR-0013 entregues
- [ ] CONTEXT D-64.6: Tabela M-XX em CONTEXT.md/VERIFICATION.md marcada com Covered/Partial/N/A
- [ ] Plans 04-01 a 04-05 todos com `<verify>` automated commands passando

#### Restrições (opcional)
- Plataforma: Windows 11 com `whois.exe` Sysinternals (não nativo Linux). Args list obrigatório: `["-v", "-nobanner", "-accepteula", target]` (Pitfalls 1+2 verificados).
- Commits: Conventional Commits, uma linha, sem body, sem `Co-authored-by` (CLAUDE.md raiz).
- Branch: `feat/phase-4-act-ttp-whois` (já criada e ativa).
- Pitfalls críticos (RESEARCH.md): #1+2 (whois flags), #5 (Mutex<Box<dyn AuditSink>>), #6 (#[async_trait]), #7 (py.allow_threads no #[new]), #9 (serde_yaml_ng).

#### Referências (opcional)
- `.planning/phases/04-act-node-ttp-whois/04-CONTEXT.md` — 29 decisões locked
- `.planning/phases/04-act-node-ttp-whois/04-RESEARCH.md` — patterns + crates + pitfalls
- `.planning/phases/04-act-node-ttp-whois/04-PATTERNS.md` — analog map por arquivo
- `.planning/phases/04-act-node-ttp-whois/04-VALIDATION.md` — per-task verification map
- `.planning/phases/04-act-node-ttp-whois/04-{01..05}-PLAN.md` — planos detalhados
- Branch: `feat/phase-4-act-ttp-whois`
- Pré-req: `whois.exe` v1.21 (Sysinternals) instalado via winget

#### Log de Andamento (atualizado pelo agente)
| Data | Sessão | Ação Realizada | Status ao Final |
|------|--------|----------------|-----------------|
| 2026-05-18 | init | TASK-015 aberta; pré-reqs validados (whois.exe instalado, branch criada, plans verificados); /gsd-execute-phase 4 iniciado | em andamento |

## Tasks Concluídas

> Movidas para cá após conclusão e atualização do `registry.md`. Nunca remova entradas.

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
