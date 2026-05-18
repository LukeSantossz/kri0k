# Roadmap: kri0k v1

**Created:** 2026-05-14
**Milestone:** MVP com loop de execucao, seguranca, e TUI operador

## Overview

| # | Phase | Goal | Requirements | Success Criteria |
|---|-------|------|--------------|------------------|
| 1 | LangGraph Structure | Grafo de nos conectados | AGENT-01 | 3 |
| 2 | Sense + Ollama | Snapshot para LLM | AGENT-02, LLM-01-04 | 4 |
| 3 | Reason + Plan | LLM propoe acao | AGENT-03, AGENT-04 | 3 |
| 4 | Act + TTP Whois | Executa whois | AGENT-05, TTP-01-05 | 4 |
| 5 | Reflect | Avalia e decide | AGENT-06 | 3 |
| 6 | Loop Integration | Ciclo completo | AGENT-07, INT-01-04 | 4 |
| 7 | Scope Validation | CIDR + dominios | SCOPE-01-05 | 4 |
| 8 | Audit Logging | JSONL hash chain | AUDIT-01-06 | 4 |
| 9 | TUI Base | Ratatui setup | TUI-01, TUI-02 | 3 |
| 10 | TUI Interaction | Proposals + log | TUI-03-06 | 4 |
| 11 | TUI Control | Pause/kill | TUI-07, TUI-08 | 3 |
| 12 | CLI Commands | init/run/status | CLI-01-05 | 4 |

**Total:** 12 phases | 42 requirements | 43 success criteria

---

## Milestone 1: MVP Execution Loop (Phases 1-6)

### Phase 1: LangGraph Structure (COMPLETE)
**Goal:** Criar estrutura base do grafo LangGraph com nos placeholder

**Requirements:** AGENT-01 (complete)

**Plans:** 1/1 complete

Plans:
- [x] 01-01-PLAN.md - Create StateGraph with placeholder nodes and test coverage

**Success Criteria:**
1. [x] `python/kri0k/agent/graph.py` existe com StateGraph definido
2. [x] Nos sense, reason, plan, act, reflect registrados no grafo
3. [x] Edges conectam nos na sequencia correta
4. [x] `pytest tests/test_agent_graph.py` passa (12 tests)

**Completed:** 2026-05-15

**Depends on:** None

---

### Phase 2: Sense Node + Ollama Provider (COMPLETE)
**Goal:** Sense obtem snapshot e Ollama provider funciona

**Requirements:** AGENT-02, LLM-01, LLM-02, LLM-03, LLM-04 (complete)

**Plans:** 1/1 complete

Plans:
- [x] 02-01-PLAN.md - Implement LLM module with Ollama provider and sense node

**Success Criteria:**
1. [x] Sense node chama `_native.get_dummy_graph()` e recebe dict
2. [x] Provider Ollama conecta via httpx a `localhost:11434`
3. [x] Prompt template renderiza snapshot + contexto
4. [x] Rate limiting previne mais de 10 req/min ao LLM

**Completed:** 2026-05-15

**Depends on:** Phase 1

---

### Phase 3: Reason + Plan Nodes (COMPLETE)
**Goal:** LLM analisa estado e propoe proxima acao

**Requirements:** AGENT-03, AGENT-04, LLM-03 (complete)

**Plans:** 1/1 complete

Plans:
- [x] 03-01-PLAN.md - Implement reason/plan nodes with LLM and parser

**Success Criteria:**
1. [x] Reason node retorna analise estruturada (JSON schema validado)
2. [x] Plan node retorna Proposal com campos target, ttp_id, params
3. [x] Proposal e tipo Python dataclass compativel com Rust struct

**Completed:** 2026-05-16

**Depends on:** Phase 2

---

### Phase 4: Act Node + TTP Whois
**Goal:** Act executa TTP whois e atualiza grafo

**Requirements:** AGENT-05, TTP-01, TTP-02, TTP-03, TTP-04, TTP-05

**Plans:** 5 plans (3 waves)

Plans:
**Wave 1**
- [x] 04-01-PLAN.md - Graph data model: NodeKind Domain/Organization/Nameserver + EdgeKind RegisteredBy/HasNameserver (Wave 1)
- [x] 04-02-PLAN.md - Error taxonomy + Cargo deps (tokio-util, which, async-trait, tracing, serde_yaml_ng) + NoopAuditSink rename (Wave 1)

**Wave 2** *(blocked on Wave 1 completion)*
- [x] 04-03-PLAN.md - TTP module promotion to async + Subprocess abstraction + WhoisTtp + parser + fixtures (Wave 2)
- [x] 04-04-PLAN.md - ScopeConfig parser (lookahead v1 schema) + allowlist exact-match + SHA-256 hash + scope.example.yaml (Wave 2)

**Wave 3** *(blocked on Wave 2 completion)*
- [ ] 04-05-PLAN.md - Engagement pyclass + Python wiring (engagement.py, act.py, sense.py) + tests + docs (Wave 3)

**Success Criteria:**
1. TTP whois implementa trait `Ttp` em `crates/kri0k-core/src/ttp/whois.rs`
2. `whois example.com` retorna struct com registrant, nameservers, dates
3. Grafo recebe nos Domain, Organization, Nameserver apos execucao
4. Rate limit de 1 req/sec e respeitado

**Depends on:** Phase 3

---

### Phase 5: Reflect Node
**Goal:** Reflect avalia resultado e decide continuacao

**Requirements:** AGENT-06

**Success Criteria:**
1. Reflect recebe resultado da execucao e estado do grafo
2. Decide entre: next_iteration, goal_reached, blocked
3. Retorna decisao estruturada com razao

**Depends on:** Phase 4

---

### Phase 6: Loop Integration
**Goal:** Ciclo completo funciona end-to-end

**Requirements:** AGENT-07, INT-01, INT-02, INT-03, INT-04

**Success Criteria:**
1. Loop executa 3+ iteracoes sem intervencao humana
2. Grafo acumula nos de multiplas execucoes whois
3. Snapshot na iteracao N reflete mudancas da iteracao N-1
4. `kri0k run --scope scope.yaml` executa loop em modo propose-only

**Depends on:** Phase 5

---

## Milestone 2: Security Foundation (Phases 7-8)

### Phase 7: Scope Validation
**Goal:** Validacao completa de CIDR e dominios

**Requirements:** SCOPE-01, SCOPE-02, SCOPE-03, SCOPE-04, SCOPE-05

**Success Criteria:**
1. `scope.yaml` parser carrega targets como `Vec<Target>`
2. IP `192.168.1.50` validado contra CIDR `192.168.1.0/24` retorna Ok
3. Dominio `sub.example.com` validado contra `*.example.com` retorna Ok
4. Target fora do escopo retorna `Err(ScopeViolation)` antes de execute

**Depends on:** Phase 6

---

### Phase 8: Audit Logging
**Goal:** Log append-only com hash chain

**Requirements:** AUDIT-01, AUDIT-02, AUDIT-03, AUDIT-04, AUDIT-05, AUDIT-06

**Success Criteria:**
1. `engagement.audit.jsonl` criado no diretorio do engagement
2. Cada linha e JSON com `timestamp`, `event_type`, `payload`, `prev_hash`
3. SHA-256 do evento N-1 esta em `prev_hash` do evento N
4. TtpExecution e ScopeViolation sao eventos distintos no log

**Depends on:** Phase 7

---

## Milestone 3: CLI Operational (Phases 9-12)

### Phase 9: TUI Base
**Goal:** TUI basica com ratatui renderizando grafo

**Requirements:** TUI-01, TUI-02

**Success Criteria:**
1. `kri0k tui` abre terminal alternativo com ratatui
2. Grafo renderizado como ASCII/box drawing com nos e edges
3. Esc/q encerra TUI limpamente

**Depends on:** Phase 8

---

### Phase 10: TUI Interaction
**Goal:** TUI interativa com log e proposals

**Requirements:** TUI-03, TUI-04, TUI-05, TUI-06

**Success Criteria:**
1. Painel de log scrollable mostra eventos em tempo real
2. Proposal pendente exibido com target, ttp_id, params
3. Keybinding `y`/`n` aprova/rejeita proposal
4. Status bar mostra scope, objetivo, contagem de nos

**Depends on:** Phase 9

---

### Phase 11: TUI Control
**Goal:** Operador controla execucao via TUI

**Requirements:** TUI-07, TUI-08

**Success Criteria:**
1. Keybinding `p` pausa loop do agente
2. Keybinding `r` resume loop pausado
3. Keybinding `k` aciona kill switch (encerra imediatamente)

**Depends on:** Phase 10

---

### Phase 12: CLI Commands
**Goal:** CLI completa para operacao

**Requirements:** CLI-01, CLI-02, CLI-03, CLI-04, CLI-05

**Success Criteria:**
1. `kri0k init --scope scope.yaml` cria diretorio de engagement
2. `kri0k run` executa em propose-only (default seguro)
3. `kri0k run --execute` habilita execucao real
4. `kri0k status` exibe estado JSON do engagement

**Depends on:** Phase 11

---

## Requirement Coverage Matrix

| Category | Total | Mapped | Coverage |
|----------|-------|--------|----------|
| AGENT | 7 | 7 | 100% |
| LLM | 4 | 4 | 100% |
| TTP | 5 | 5 | 100% |
| SCOPE | 5 | 5 | 100% |
| AUDIT | 6 | 6 | 100% |
| TUI | 8 | 8 | 100% |
| CLI | 5 | 5 | 100% |
| INT | 4 | 4 | 100% |
| **Total** | **44** | **44** | **100%** |

---

*Roadmap created: 2026-05-14*
