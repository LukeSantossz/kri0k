# kri0k

## What This Is

Framework de engajamento de segurança ofensiva com agente LLM autônomo. Combina um core Rust determinístico (grafo de estado, validação, auditoria) com orquestração Python via LangGraph. O operador define o escopo, o agente propõe ações, o Rust valida e executa TTPs mapeados ao MITRE ATT&CK.

## Core Value

**Execução segura e auditável de técnicas ofensivas** — o LLM nunca executa diretamente; toda ação passa por validação de escopo determinística em Rust antes de tocar a rede.

## Requirements

### Validated

<!-- Shipped and confirmed valuable. -->

- ✓ Tipos core Rust (NodeId, EdgeId com ULID, Error handling) — existing
- ✓ Grafo de estado petgraph (Graph, Node, Edge, NodeKind, EdgeKind) — existing
- ✓ Bridge PyO3 funcional (snapshot JSON, Tokio runtime 2 workers) — existing
- ✓ Trait Ttp definido (propose, execute_dry_run, execute) — existing
- ✓ Trait AuditSink definido (log_ttp_execution, log_scope_violation, flush) — existing
- ✓ Struct Scope e validate_target stub — existing
- ✓ SafeguardsConfig (propose_only default, kill_switch) — existing
- ✓ Serialização JSON cross-boundary via serde — existing

### Active

<!-- Current scope. Building toward these. -->

- [ ] Loop LangGraph completo (sense/reason/plan/act/reflect)
- [ ] Implementação TTP T1590.001 (whois)
- [ ] Provider LLM Ollama funcional
- [ ] Validação de escopo completa (CIDR + domínios)
- [ ] Audit log append-only JSONL com hash chain
- [ ] TUI operador com ratatui

### Out of Scope

<!-- Explicit boundaries. Includes reasoning to prevent re-adding. -->

- TTPs destrutivos (T1485, T1486) — fora do MVP, requer gates adicionais
- Providers OpenAI/Anthropic — Ollama local-first por agora (ADR-0008)
- Mobile/web interface — CLI/TUI suficiente para operadores
- Multi-engagement simultâneo — single engagement no MVP

## Context

**Codebase existente:** Rust workspace com 3 crates (kri0k-core, kri0k-graph, kri0k-pybridge) + Python package. Arquitetura documentada em `.planning/codebase/`.

**Decisões arquiteturais:** 12 ADRs documentados em `docs/adr/`. Key decisions:
- ADR-0001: Estado canônico em Rust
- ADR-0008: Local-first LLM (Ollama)
- ADR-0012: TTPs como ferramentas externas (nmap, dig, whois)

**Threat model:** Documentado em `docs/security/THREAT_MODEL.md`. Principais ameaças mitigadas:
- AB-03: Prompt injection → validação determinística em Rust
- Mutable state cross-boundary → snapshots JSON imutáveis

**Build system:** Maturin para PyO3, Cargo workspace, pytest + Rust inline tests.

## Constraints

- **Tech stack**: Rust 1.75+ / Python 3.11+ / PyO3 / LangGraph — já estabelecido
- **LLM**: Ollama local (qwen3:32b ou deepseek-r1:32b recomendados)
- **Platform**: Linux primário (mypy configured for linux)
- **External tools**: whois CLI disponível no sistema para TTP T1590.001
- **Security**: Clippy strict, no unsafe, no unwrap/panic em código não-test

## Key Decisions

<!-- Decisions that constrain future work. Add throughout project lifecycle. -->

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| TTP whois primeiro | Mais simples que nmap, prova o loop sem complexidade de parsing | — Pending |
| Ollama local-first | Air-gapped deployment, custo zero, ADR-0008 | — Pending |
| TUI ratatui | Operador precisa de visibilidade em tempo real do grafo | — Pending |
| Scope CIDR + domínios | Cobertura mínima para engajamentos reais | — Pending |
| Audit JSONL hash chain | Integridade forense desde o início | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-05-14 after initialization*
