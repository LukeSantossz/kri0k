# Roadmap: kri0k v1

**Created:** 2026-05-14
**Milestone:** MVP com loop de execução, segurança, e TUI operador

## Overview

| # | Phase | Goal | Requirements | Success Criteria |
|---|-------|------|--------------|------------------|
| 1 | LangGraph Structure | Grafo de nós conectados | AGENT-01 | 3 |
| 2 | Sense + Ollama | Snapshot para LLM | AGENT-02, LLM-01-04 | 4 |
| 3 | Reason + Plan | LLM propõe ação | AGENT-03, AGENT-04 | 3 |
| 4 | Act + TTP Whois | Executa whois | AGENT-05, TTP-01-05 | 4 |
| 5 | Reflect | Avalia e decide | AGENT-06 | 3 |
| 6 | Loop Integration | Ciclo completo | AGENT-07, INT-01-04 | 4 |
| 7 | Scope Validation | CIDR + domínios | SCOPE-01-05 | 4 |
| 8 | Audit Logging | JSONL hash chain | AUDIT-01-06 | 4 |
| 9 | TUI Base | Ratatui setup | TUI-01, TUI-02 | 3 |
| 10 | TUI Interaction | Proposals + log | TUI-03-06 | 4 |
| 11 | TUI Control | Pause/kill | TUI-07, TUI-08 | 3 |
| 12 | CLI Commands | init/run/status | CLI-01-05 | 4 |

**Total:** 12 phases | 42 requirements | 43 success criteria

---

## Milestone 1: MVP Execution Loop (Phases 1-6)

### Phase 1: LangGraph Structure
**Goal:** Criar estrutura base do grafo LangGraph com nós placeholder

**Requirements:** AGENT-01

**Success Criteria:**
1. `python/kri0k/agent/graph.py` existe com StateGraph definido
2. Nós sense, reason, plan, act, reflect registrados no grafo
3. Edges conectam nós na sequência correta
4. `pytest tests/test_agent_graph.py` passa

**Depends on:** None

---

### Phase 2: Sense Node + Ollama Provider
**Goal:** Sense obtém snapshot e Ollama provider funciona

**Requirements:** AGENT-02, LLM-01, LLM-02, LLM-03, LLM-04

**Success Criteria:**
1. Sense node chama `_native.get_dummy_graph()` e recebe dict
2. Provider Ollama conecta via httpx a `localhost:11434`
3. Prompt template renderiza snapshot + contexto
4. Rate limiting previne mais de 10 req/min ao LLM

**Depends on:** Phase 1

---

### Phase 3: Reason + Plan Nodes
**Goal:** LLM analisa estado e propõe próxima ação

**Requirements:** AGENT-03, AGENT-04

**Success Criteria:**
1. Reason node retorna análise estruturada (JSON schema validado)
2. Plan node retorna Proposal com campos target, ttp_id, params
3. Proposal é tipo Python dataclass compatível com Rust struct

**Depends on:** Phase 2

---

### Phase 4: Act Node + TTP Whois
**Goal:** Act executa TTP whois e atualiza grafo

**Requirements:** AGENT-05, TTP-01, TTP-02, TTP-03, TTP-04, TTP-05

**Success Criteria:**
1. TTP whois implementa trait `Ttp` em `crates/kri0k-core/src/ttp/whois.rs`
2. `whois example.com` retorna struct com registrant, nameservers, dates
3. Grafo recebe nós Domain, Organization, Nameserver após execução
4. Rate limit de 1 req/sec é respeitado

**Depends on:** Phase 3

---

### Phase 5: Reflect Node
**Goal:** Reflect avalia resultado e decide continuação

**Requirements:** AGENT-06

**Success Criteria:**
1. Reflect recebe resultado da execução e estado do grafo
2. Decide entre: next_iteration, goal_reached, blocked
3. Retorna decisão estruturada com razão

**Depends on:** Phase 4

---

### Phase 6: Loop Integration
**Goal:** Ciclo completo funciona end-to-end

**Requirements:** AGENT-07, INT-01, INT-02, INT-03, INT-04

**Success Criteria:**
1. Loop executa 3+ iterações sem intervenção humana
2. Grafo acumula nós de múltiplas execuções whois
3. Snapshot na iteração N reflete mudanças da iteração N-1
4. `kri0k run --scope scope.yaml` executa loop em modo propose-only

**Depends on:** Phase 5

---

## Milestone 2: Security Foundation (Phases 7-8)

### Phase 7: Scope Validation
**Goal:** Validação completa de CIDR e domínios

**Requirements:** SCOPE-01, SCOPE-02, SCOPE-03, SCOPE-04, SCOPE-05

**Success Criteria:**
1. `scope.yaml` parser carrega targets como `Vec<Target>`
2. IP `192.168.1.50` validado contra CIDR `192.168.1.0/24` retorna Ok
3. Domínio `sub.example.com` validado contra `*.example.com` retorna Ok
4. Target fora do escopo retorna `Err(ScopeViolation)` antes de execute

**Depends on:** Phase 6

---

### Phase 8: Audit Logging
**Goal:** Log append-only com hash chain

**Requirements:** AUDIT-01, AUDIT-02, AUDIT-03, AUDIT-04, AUDIT-05, AUDIT-06

**Success Criteria:**
1. `engagement.audit.jsonl` criado no diretório do engagement
2. Cada linha é JSON com `timestamp`, `event_type`, `payload`, `prev_hash`
3. SHA-256 do evento N-1 está em `prev_hash` do evento N
4. TtpExecution e ScopeViolation são eventos distintos no log

**Depends on:** Phase 7

---

## Milestone 3: CLI Operational (Phases 9-12)

### Phase 9: TUI Base
**Goal:** TUI básica com ratatui renderizando grafo

**Requirements:** TUI-01, TUI-02

**Success Criteria:**
1. `kri0k tui` abre terminal alternativo com ratatui
2. Grafo renderizado como ASCII/box drawing com nós e edges
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
4. Status bar mostra scope, objetivo, contagem de nós

**Depends on:** Phase 9

---

### Phase 11: TUI Control
**Goal:** Operador controla execução via TUI

**Requirements:** TUI-07, TUI-08

**Success Criteria:**
1. Keybinding `p` pausa loop do agente
2. Keybinding `r` resume loop pausado
3. Keybinding `k` aciona kill switch (encerra imediatamente)

**Depends on:** Phase 10

---

### Phase 12: CLI Commands
**Goal:** CLI completa para operação

**Requirements:** CLI-01, CLI-02, CLI-03, CLI-04, CLI-05

**Success Criteria:**
1. `kri0k init --scope scope.yaml` cria diretório de engagement
2. `kri0k run` executa em propose-only (default seguro)
3. `kri0k run --execute` habilita execução real
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
