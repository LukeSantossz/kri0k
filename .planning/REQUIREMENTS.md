# Requirements: kri0k

**Defined:** 2026-05-14
**Core Value:** Execução segura e auditável de técnicas ofensivas — validação determinística em Rust antes de qualquer ação na rede.

## v1 Requirements

Requirements para o MVP. Cada um mapeia a fases do roadmap.

### LangGraph Agent

- [x] **AGENT-01**: Sistema inicializa grafo LangGraph com nós sense/reason/plan/act/reflect
- [ ] **AGENT-02**: Nó SENSE obtém snapshot JSON do grafo Rust e formata para LLM
- [ ] **AGENT-03**: Nó REASON recebe snapshot e retorna análise estruturada do estado atual
- [ ] **AGENT-04**: Nó PLAN propõe próxima ação como Proposal tipado (target, ttp_id, params)
- [ ] **AGENT-05**: Nó ACT envia Proposal para Rust, recebe resultado da execução
- [ ] **AGENT-06**: Nó REFLECT avalia resultado e decide próxima iteração ou término
- [ ] **AGENT-07**: Loop completo executa ciclo sense→reason→plan→act→reflect sem intervenção

### LLM Provider

- [ ] **LLM-01**: Provider Ollama conecta a instância local via HTTP
- [ ] **LLM-02**: Provider formata prompts com contexto do engagement (scope, objetivo)
- [ ] **LLM-03**: Provider parseia resposta estruturada do LLM para tipos Python
- [ ] **LLM-04**: Provider gerencia rate limiting e retry com backoff exponencial

### TTP Implementation

- [ ] **TTP-01**: TTP T1590.001 (whois) implementa trait Ttp em Rust
- [ ] **TTP-02**: TTP whois executa comando whois via std::process::Command
- [ ] **TTP-03**: TTP whois parseia output para campos estruturados (registrant, nameservers, dates)
- [ ] **TTP-04**: TTP whois adiciona nós ao grafo (Domain, Organization, Nameserver)
- [ ] **TTP-05**: TTP whois respeita rate limits configurados (default 1 req/sec)

### Scope Validation

- [ ] **SCOPE-01**: Parser scope.yaml carrega configuração com targets permitidos
- [ ] **SCOPE-02**: Validador verifica IP contra ranges CIDR permitidos
- [ ] **SCOPE-03**: Validador verifica domínio contra wildcards permitidos (*.example.com)
- [ ] **SCOPE-04**: Validador rejeita targets fora do escopo com erro estruturado
- [ ] **SCOPE-05**: Validação é chamada antes de toda execução de TTP (fail-closed)

### Audit Logging

- [ ] **AUDIT-01**: AuditSink implementação escreve eventos em arquivo JSONL
- [ ] **AUDIT-02**: Cada evento inclui timestamp, tipo, payload, hash do evento anterior
- [ ] **AUDIT-03**: Hash chain usa SHA-256 para integridade forense
- [ ] **AUDIT-04**: Log registra TtpExecution (ttp_id, target, result, duration)
- [ ] **AUDIT-05**: Log registra ScopeViolation (target, reason, proposal)
- [ ] **AUDIT-06**: flush() é chamado após cada operação crítica

### TUI Operator

- [ ] **TUI-01**: TUI inicializa com ratatui e crossterm backend
- [ ] **TUI-02**: TUI exibe grafo de estado atual com nós e edges
- [ ] **TUI-03**: TUI mostra log de eventos em tempo real (scrollable)
- [ ] **TUI-04**: TUI exibe Proposal pendente para confirmação do operador
- [ ] **TUI-05**: Operador pode aprovar/rejeitar Proposal via keybinding
- [ ] **TUI-06**: TUI mostra status do engagement (scope, objetivo, progresso)
- [ ] **TUI-07**: Operador pode pausar/resumir loop do agente
- [ ] **TUI-08**: Operador pode acionar kill switch (encerra todas operações)

### CLI Commands

- [ ] **CLI-01**: Comando `kri0k init` cria engagement com scope.yaml
- [ ] **CLI-02**: Comando `kri0k run --scope scope.yaml` inicia engagement em modo propose-only
- [ ] **CLI-03**: Comando `kri0k run --execute` habilita execução real de TTPs
- [ ] **CLI-04**: Comando `kri0k status` exibe estado atual do engagement
- [ ] **CLI-05**: Flag `--dry-run` em qualquer comando simula sem efeitos

### Integration

- [ ] **INT-01**: Python chama Rust via _native.validate(proposal) antes de execute
- [ ] **INT-02**: Rust retorna resultado tipado para Python (Success/Error/ScopeViolation)
- [ ] **INT-03**: Grafo Rust é atualizado atomicamente após TTP execution
- [ ] **INT-04**: Snapshot reflete estado atualizado na próxima iteração SENSE

## v2 Requirements

Deferred para release futura. Não no roadmap atual.

### Additional TTPs

- **TTP-V2-01**: T1046 Port scan via nmap
- **TTP-V2-02**: T1018 Remote system discovery
- **TTP-V2-03**: T1016 System network configuration discovery

### Additional Providers

- **LLM-V2-01**: Provider Anthropic Claude
- **LLM-V2-02**: Provider OpenAI GPT
- **LLM-V2-03**: Provider switching runtime

### Advanced Features

- **ADV-01**: Multi-engagement simultâneo
- **ADV-02**: Engagement persistence (save/restore)
- **ADV-03**: Report generation (markdown, HTML)
- **ADV-04**: Collaborative mode (multiple operators)

## Out of Scope

Explicitamente excluídos. Documentado para prevenir scope creep.

| Feature | Reason |
|---------|--------|
| TTPs destrutivos (T1485, T1486) | Requer gates de segurança adicionais, fora do MVP |
| Web interface | CLI/TUI suficiente para operadores técnicos |
| Windows support | Linux-first per mypy config, Windows later |
| Cloud deployment | Local-first per ADR-0008, air-gapped scenarios |
| Real-time collaboration | Single operator no MVP |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| AGENT-01 | Phase 1 | Complete |
| AGENT-02 | Phase 2 | Pending |
| AGENT-03 | Phase 3 | Pending |
| AGENT-04 | Phase 3 | Pending |
| AGENT-05 | Phase 4 | Pending |
| AGENT-06 | Phase 5 | Pending |
| AGENT-07 | Phase 6 | Pending |
| LLM-01 | Phase 2 | Pending |
| LLM-02 | Phase 2 | Pending |
| LLM-03 | Phase 2 | Pending |
| LLM-04 | Phase 2 | Pending |
| TTP-01 | Phase 4 | Pending |
| TTP-02 | Phase 4 | Pending |
| TTP-03 | Phase 4 | Pending |
| TTP-04 | Phase 4 | Pending |
| TTP-05 | Phase 4 | Pending |
| SCOPE-01 | Phase 7 | Pending |
| SCOPE-02 | Phase 7 | Pending |
| SCOPE-03 | Phase 7 | Pending |
| SCOPE-04 | Phase 7 | Pending |
| SCOPE-05 | Phase 7 | Pending |
| AUDIT-01 | Phase 8 | Pending |
| AUDIT-02 | Phase 8 | Pending |
| AUDIT-03 | Phase 8 | Pending |
| AUDIT-04 | Phase 8 | Pending |
| AUDIT-05 | Phase 8 | Pending |
| AUDIT-06 | Phase 8 | Pending |
| TUI-01 | Phase 9 | Pending |
| TUI-02 | Phase 9 | Pending |
| TUI-03 | Phase 10 | Pending |
| TUI-04 | Phase 10 | Pending |
| TUI-05 | Phase 10 | Pending |
| TUI-06 | Phase 10 | Pending |
| TUI-07 | Phase 11 | Pending |
| TUI-08 | Phase 11 | Pending |
| CLI-01 | Phase 12 | Pending |
| CLI-02 | Phase 12 | Pending |
| CLI-03 | Phase 12 | Pending |
| CLI-04 | Phase 12 | Pending |
| CLI-05 | Phase 12 | Pending |
| INT-01 | Phase 6 | Pending |
| INT-02 | Phase 6 | Pending |
| INT-03 | Phase 6 | Pending |
| INT-04 | Phase 6 | Pending |

**Coverage:**
- v1 requirements: 42 total
- Mapped to phases: 42
- Unmapped: 0 ✓

---
*Requirements defined: 2026-05-14*
*Last updated: 2026-05-14 after initial definition*
