# Architecture

**Analysis Date:** 2026-05-14

## System Overview

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                          OPERATOR (CLI / TUI)                                │
│   $ kri0k init  |  kri0k run --scope scope.yaml [--execute]                  │
└──────────────────────────────┬──────────────────────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────────────────────┐
│  PYTHON LAYER  (kri0k pkg, embedded via PyO3)                                │
│                                                                              │
│   ┌────────────────────┐   ┌──────────────────────┐  ┌────────────────────┐ │
│   │  LangGraph nodes   │   │  LLM provider abst.  │  │   Prompt store     │ │
│   │  (sense/reason/    │   │  Ollama | Anthropic  │  │   (jinja2)         │ │
│   │   plan/act/reflect)│   │  | OpenAI (opt-in)   │  │                    │ │
│   └─────────┬──────────┘   └──────────┬───────────┘  └────────┬───────────┘ │
│             │                          │                       │             │
│             └──────────────┬───────────┴───────────────────────┘             │
│                            ▼                                                  │
│             ┌────────────────────────────────┐                                │
│             │  kri0k._native (PyO3 bindings) │  <- Rust<->Python boundary    │
│             │  Snapshot, Proposal, validate, │                                │
│             │  execute(), apply()            │                                │
│             └───────────────┬────────────────┘                                │
└─────────────────────────────┼────────────────────────────────────────────────┘
                              │  JSON snapshots / Proposal structs
┌─────────────────────────────▼────────────────────────────────────────────────┐
│  RUST CORE  (cargo workspace)                                                 │
│                                                                              │
│   ┌──────────────────────┐  ┌────────────────────────────┐                   │
│   │   kri0k-pybridge     │  │   kri0k-core (runtime)     │                   │
│   │   • PyO3 cdylib      │<>│   • Tokio runtime          │                   │
│   │   • Snapshot codec   │  │   • Scope validator        │                   │
│   │   • py<->rs convert  │  │   • Audit log (stub)       │                   │
│   └──────────────────────┘  │   • Kill switch / dry-run  │                   │
│                             │   • Safeguards config      │                   │
│                             └────────────┬───────────────┘                   │
│                                          │                                    │
│        ┌─────────────────────────────────┼─────────────────────────┐         │
│        ▼                                 ▼                         ▼         │
│ ┌─────────────────┐         ┌────────────────────┐    ┌──────────────────┐  │
│ │  kri0k-graph    │         │   kri0k-ttp        │    │  kri0k-scope     │  │
│ │  petgraph::     │         │   (planned MVP-1+) │    │  (planned)       │  │
│ │  StableGraph    │         │   Trait + adapters │    │  scope.yaml      │  │
│ │  Node/EdgeKind  │         │   T1046 (nmap),    │    │  parser+checksum │  │
│ │  serde JSON     │         │   T1590.001 (whois)│    │  CIDR/domain     │  │
│ └────────┬────────┘         └──────────┬─────────┘    └──────────────────┘  │
│          │                              │                                     │
│          ▼                              ▼                                     │
│  ┌────────────────┐            ┌────────────────────┐                        │
│  │  Graph store   │            │  External tools    │                        │
│  │  (in-mem + on- │            │  nmap, dig, whois, │                        │
│  │   disk JSONL   │            │  HTTP clients      │                        │
│  │   snapshot)    │            │  (reqwest+rustls)  │                        │
│  └────────────────┘            └────────────────────┘                        │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Component Responsibilities

| Component | Responsibility | File |
|-----------|----------------|------|
| kri0k-core | Error types, NodeId/EdgeId (ULID), audit trait, scope validation, safeguards, TTP trait | `crates/kri0k-core/src/lib.rs` |
| kri0k-graph | petgraph StableGraph wrapper, Node/Edge/NodeKind/EdgeKind types, JSON serialization | `crates/kri0k-graph/src/lib.rs` |
| kri0k-pybridge | PyO3 cdylib bindings, Tokio runtime init, Python module `_native` | `crates/kri0k-pybridge/src/lib.rs` |
| kri0k Python pkg | Python entry point, re-exports native bindings | `python/kri0k/__init__.py` |
| audit module | AuditSink trait, event types (TtpExecution, ScopeViolation, Engagement) | `crates/kri0k-core/src/audit.rs` |
| scope module | Scope struct, validate_target function (stubs) | `crates/kri0k-core/src/scope.rs` |
| safeguards module | SafeguardsConfig, propose_only/kill_switch flags | `crates/kri0k-core/src/safeguards.rs` |
| ttp module | Ttp trait, RateLimits, ExecutionPlan, RiskLevel, ExecutionResult | `crates/kri0k-core/src/ttp.rs` |

## Pattern Overview

**Overall:** Layered architecture with Rust as source of truth, Python as reasoning layer

**Key Characteristics:**
- Canonical state lives in Rust (determinism, auditability, GIL control)
- Python receives read-only snapshots, returns typed Proposals
- Deterministic validator precedes all execution (LLM never triggers TTPs directly)
- `scope.yaml` is boot prerequisite; without it only `init`/`status` respond
- Propose-only is default; `--execute` is explicit opt-in

## Layers

**Operator Layer (CLI/TUI):**
- Purpose: User interface for engagement commands
- Location: `kri0k` CLI (entry point defined in `pyproject.toml`)
- Contains: Command parsing, TUI (planned: ratatui)
- Depends on: Python layer
- Used by: Human operator

**Python Layer (Reasoning):**
- Purpose: LLM integration, LangGraph orchestration, prompt management
- Location: `python/kri0k/`
- Contains: LangGraph nodes (sense/reason/plan/act/reflect), LLM providers, Jinja2 templates
- Depends on: `kri0k._native` (PyO3 bindings)
- Used by: Operator layer

**PyO3 Bridge:**
- Purpose: Cross-language boundary between Python and Rust
- Location: `crates/kri0k-pybridge/src/lib.rs`
- Contains: `_native` module, Tokio runtime init, graph serialization to Python dicts
- Depends on: kri0k-core, kri0k-graph
- Used by: Python layer

**Rust Core:**
- Purpose: Canonical state, validation, audit, safety enforcement
- Location: `crates/kri0k-core/`
- Contains: Error types, IDs (ULID), Scope, Audit, Safeguards, TTP trait
- Depends on: serde, thiserror, ulid
- Used by: kri0k-graph, kri0k-pybridge

**Graph Layer:**
- Purpose: Attack state graph using petgraph StableGraph
- Location: `crates/kri0k-graph/src/lib.rs`
- Contains: Graph, Node, Edge, NodeKind, EdgeKind
- Depends on: kri0k-core, petgraph
- Used by: kri0k-pybridge

## Data Flow

### Primary Request Path (Engagement Loop)

1. **SENSE** - Python calls `eng.snapshot()` -> Rust serializes graph to JSON (`crates/kri0k-pybridge/src/lib.rs:34-78`)
2. **REASON** - LangGraph feeds snapshot to LLM, receives Proposal (Python layer, planned)
3. **VALIDATE** - Python calls `eng.validate(proposal)` -> Rust validator checks scope (stub: `crates/kri0k-core/src/scope.rs:56-64`)
4. **HUMAN GATE** - If `requires_human: true`, operator confirms (planned TUI)
5. **ACT** - Python calls `eng.execute(proposal)` -> Rust re-validates, runs TTP (stub: `crates/kri0k-core/src/ttp.rs:68-73`)
6. **UPDATE** - Rust applies new nodes/edges to graph atomically
7. **REFLECT** - Python evaluates results, decides next iteration

### Cross-Language Data Flow

1. Python requests graph snapshot via `_native.get_dummy_graph()` (`crates/kri0k-pybridge/src/lib.rs:34`)
2. Rust releases GIL with `py.allow_threads()`, builds graph (`crates/kri0k-pybridge/src/lib.rs:36`)
3. Rust serializes to `serde_json::Value` (`crates/kri0k-graph/src/lib.rs:160-190`)
4. PyO3 converts JSON to Python dict via `json.loads()` (`crates/kri0k-pybridge/src/lib.rs:73-77`)

**State Management:**
- All canonical state in Rust (Graph struct with petgraph StableGraph)
- Python receives immutable snapshots (JSON dicts)
- Mutations only via `execute()` or `apply_external()` (planned)
- No live references cross PyO3 boundary

## Key Abstractions

**NodeId / EdgeId:**
- Purpose: Stable external identifiers using ULID
- Examples: `crates/kri0k-core/src/lib.rs:32-89`
- Pattern: Newtype wrapper around `ulid::Ulid`, serializable via serde

**NodeKind / EdgeKind:**
- Purpose: Tagged enum for node/edge classification
- Examples: `crates/kri0k-graph/src/lib.rs:9-49`
- Pattern: Serde tagged enum (`#[serde(tag = "type")]`)

**Ttp Trait:**
- Purpose: Interface for offensive techniques (MITRE ATT&CK mapped)
- Examples: `crates/kri0k-core/src/ttp.rs:19-74`
- Pattern: Trait with `propose()`, `execute_dry_run()`, `execute()` methods

**AuditSink Trait:**
- Purpose: Append-only audit logging interface
- Examples: `crates/kri0k-core/src/audit.rs:20-44`
- Pattern: Trait with `log_ttp_execution()`, `log_scope_violation()`, `flush()`

## Entry Points

**Python Package:**
- Location: `python/kri0k/__init__.py`
- Triggers: `import kri0k`
- Responsibilities: Re-exports `hello()`, `get_dummy_graph()` from `_native`

**CLI (planned):**
- Location: `kri0k.cli:main` (defined in `pyproject.toml:60`)
- Triggers: `kri0k` command
- Responsibilities: Parse args, manage engagements

**Native Module Init:**
- Location: `crates/kri0k-pybridge/src/lib.rs:81-90`
- Triggers: Python imports `kri0k._native`
- Responsibilities: Initialize Tokio runtime, register PyO3 functions

## Architectural Constraints

- **Threading:** Global Tokio runtime with 2 worker threads (`crates/kri0k-pybridge/src/lib.rs:14-22`); GIL released during Rust operations (`py.allow_threads()`)
- **Global state:** `TOKIO_RUNTIME` OnceLock singleton (`crates/kri0k-pybridge/src/lib.rs:9`)
- **Circular imports:** None detected; clean crate dependency tree (core -> graph -> pybridge)
- **No unsafe code:** Workspace lints warn on `unsafe_code` (`Cargo.toml:24`)
- **Fail-closed validation:** Scope validation returns error until fully implemented (`crates/kri0k-core/src/scope.rs:61-63`)

## Anti-Patterns

### Direct LLM Execution

**What happens:** LLM output directly triggers network operations
**Why it's wrong:** Prompt injection could cause out-of-scope attacks (see `docs/security/THREAT_MODEL.md` AB-03)
**Do this instead:** LLM produces typed `Proposal`, Rust validator checks scope before execution (`crates/kri0k-core/src/scope.rs`)

### Mutable State Across PyO3 Boundary

**What happens:** Python holds live references to Rust objects
**Why it's wrong:** GIL contention, potential memory unsafety, non-deterministic behavior
**Do this instead:** Serialize to JSON snapshots, pass by value (`crates/kri0k-pybridge/src/lib.rs:73-77`)

### Execution Without Scope Check

**What happens:** TTP runs without validating target is in scope
**Why it's wrong:** Legal liability, out-of-scope attacks
**Do this instead:** Call `validate_target()` before every `execute()` (`crates/kri0k-core/src/scope.rs:56`)

## Error Handling

**Strategy:** Typed errors via `thiserror`, Result<T, Error> throughout

**Patterns:**
- `kri0k_core::Error` enum with `Json` and `Generic` variants (`crates/kri0k-core/src/lib.rs:17-26`)
- `kri0k_core::Result<T>` type alias (`crates/kri0k-core/src/lib.rs:29`)
- No `unwrap()` or `panic!()` in non-test code (clippy denies `unwrap_used`, `panic`)

## Cross-Cutting Concerns

**Logging:** `tracing` crate (planned per `docs/ARCHITECTURE.md` section 5.6)

**Validation:** Deterministic Rust validator in `kri0k-core::scope` (stub), checks target in scope before execution

**Serialization:** Serde JSON throughout; compact format with short keys for LLM efficiency (`docs/ARCHITECTURE.md` section 2.2)

**Audit:** `AuditSink` trait in `kri0k-core::audit`, append-only JSONL with hash chain (stub implementation)

**Authentication:** scope.yaml `operator` field required; engagement boot logs operator identity

---

*Architecture analysis: 2026-05-14*
