# Codebase Structure

**Analysis Date:** 2026-05-15

## Directory Layout

```
kri0k/
├── .github/                    # GitHub configuration
│   ├── workflows/              # CI/CD workflows
│   │   └── ci.yaml             # Main CI pipeline
│   └── PULL_REQUEST_TEMPLATE.md
├── .planning/                  # GSD planning documents
│   └── codebase/               # Architecture analysis docs
├── config/                     # Configuration examples
│   └── scope.example.yaml      # Scope configuration template
├── crates/                     # Rust workspace crates
│   ├── kri0k-core/             # Core types, errors, traits
│   ├── kri0k-graph/            # petgraph wrapper, graph types
│   └── kri0k-pybridge/         # PyO3 bindings (cdylib)
├── docs/                       # Documentation
│   ├── adr/                    # Architecture Decision Records
│   ├── runs/                   # Execution run logs
│   └── security/               # Security documentation
│       └── THREAT_MODEL.md     # Threat model document
├── python/                     # Python source
│   └── kri0k/                  # Main Python package
│       ├── __init__.py         # Package entry, re-exports _native
│       ├── _native.pyi         # Type stubs for Rust bindings
│       └── agent/              # LangGraph agent module
│           ├── __init__.py     # Exports AgentState, get_graph
│           ├── state.py        # AgentState TypedDict definition
│           ├── graph.py        # StateGraph builder, routing
│           └── nodes/          # 5 engagement loop nodes
│               ├── __init__.py # Re-exports all node functions
│               ├── sense.py    # Observe graph state
│               ├── reason.py   # Analyze observations
│               ├── plan.py     # Generate proposals
│               ├── act.py      # Execute actions
│               └── reflect.py  # Evaluate and loop control
├── tests/                      # Python tests
│   ├── test_smoke.py           # Native bindings smoke tests
│   └── test_agent_graph.py     # LangGraph agent tests
├── Cargo.toml                  # Workspace manifest
├── Cargo.lock                  # Dependency lockfile
├── pyproject.toml              # Python/maturin config
├── clippy.toml                 # Clippy linting config
├── rustfmt.toml                # Rust formatting config
├── .pre-commit-config.yaml     # Pre-commit hooks
└── README.md                   # Project overview
```

## Directory Purposes

**`.github/`:**
- Purpose: GitHub-specific configuration
- Contains: CI workflow (`ci.yaml`), PR template
- Key files: `.github/workflows/ci.yaml`

**`config/`:**
- Purpose: Configuration file examples and templates
- Contains: Example scope.yaml for engagements
- Key files: `config/scope.example.yaml`

**`crates/`:**
- Purpose: Rust cargo workspace members
- Contains: All Rust library crates
- Key files: Individual `Cargo.toml` in each subcrate

**`crates/kri0k-core/`:**
- Purpose: Core types, error handling, traits (audit, scope, ttp, safeguards)
- Contains: Foundation types shared across crates
- Key files: `src/lib.rs`, `src/audit.rs`, `src/scope.rs`, `src/safeguards.rs`, `src/ttp.rs`

**`crates/kri0k-graph/`:**
- Purpose: Attack state graph implementation using petgraph
- Contains: Graph wrapper, Node/Edge types, JSON serialization
- Key files: `src/lib.rs`

**`crates/kri0k-pybridge/`:**
- Purpose: PyO3 Python extension module
- Contains: `_native` module, Tokio runtime, Python bindings
- Key files: `src/lib.rs`, `Cargo.toml` (defines cdylib)

**`docs/`:**
- Purpose: Project documentation
- Contains: ADRs, architecture docs, security docs, run logs
- Key files: `ARCHITECTURE.md`, `adr/*.md`, `security/THREAT_MODEL.md`

**`docs/adr/`:**
- Purpose: Architecture Decision Records
- Contains: Numbered ADR documents (ADR-0001 through ADR-0012)
- Key files: `ADR-0001-canonical-state-in-rust.md`, `README.md`

**`python/kri0k/`:**
- Purpose: Python package source
- Contains: Package init, type stubs, agent module
- Key files: `__init__.py`, `_native.pyi`, `agent/`

**`python/kri0k/agent/`:**
- Purpose: LangGraph-based autonomous agent
- Contains: State schema, graph builder, node functions
- Key files: `state.py`, `graph.py`, `nodes/`

**`python/kri0k/agent/nodes/`:**
- Purpose: Individual engagement loop node implementations
- Contains: 5 async node functions (sense, reason, plan, act, reflect)
- Key files: `sense.py`, `reason.py`, `plan.py`, `act.py`, `reflect.py`

**`tests/`:**
- Purpose: Python test suite
- Contains: Smoke tests, agent graph tests
- Key files: `test_smoke.py`, `test_agent_graph.py`

## Key File Locations

**Entry Points:**
- `python/kri0k/__init__.py`: Python package entry point
- `python/kri0k/agent/__init__.py`: Agent module entry, exports `AgentState`, `get_graph`
- `python/kri0k/agent/graph.py:36`: `get_graph()` function builds compiled StateGraph
- `crates/kri0k-pybridge/src/lib.rs`: Native module init (`#[pymodule] fn _native`)
- `pyproject.toml:60`: CLI entry point definition (`kri0k = "kri0k.cli:main"`)

**Configuration:**
- `Cargo.toml`: Workspace config, shared dependencies, lints
- `pyproject.toml`: Python project config, maturin settings, ruff/mypy/pytest config
- `config/scope.example.yaml`: Scope template for engagements
- `clippy.toml`: Clippy lint overrides
- `rustfmt.toml`: Rust formatting rules

**Core Logic:**
- `crates/kri0k-core/src/lib.rs`: Error types, NodeId, EdgeId
- `crates/kri0k-core/src/audit.rs`: AuditSink trait, event types
- `crates/kri0k-core/src/scope.rs`: Scope validation (stub)
- `crates/kri0k-core/src/safeguards.rs`: SafeguardsConfig (propose-only, kill switch)
- `crates/kri0k-core/src/ttp.rs`: Ttp trait, ExecutionPlan, RateLimits
- `crates/kri0k-graph/src/lib.rs`: Graph, Node, Edge, NodeKind, EdgeKind

**LangGraph Agent:**
- `python/kri0k/agent/state.py`: `AgentState` TypedDict (7 fields)
- `python/kri0k/agent/graph.py`: `get_graph()`, `route_after_reflect()`, `MAX_ITERATIONS`
- `python/kri0k/agent/nodes/sense.py`: Observe graph state from Rust
- `python/kri0k/agent/nodes/reason.py`: Analyze observations
- `python/kri0k/agent/nodes/plan.py`: Generate action proposals
- `python/kri0k/agent/nodes/act.py`: Execute approved actions
- `python/kri0k/agent/nodes/reflect.py`: Evaluate results, increment iteration

**Testing:**
- `tests/test_smoke.py`: Python smoke tests for native bindings
- `tests/test_agent_graph.py`: LangGraph agent structure and node tests (14 tests)

**Documentation:**
- `docs/ARCHITECTURE.md`: System architecture (Portuguese, detailed)
- `docs/security/THREAT_MODEL.md`: Threat model and mitigations
- `docs/adr/`: Individual ADR files

## Naming Conventions

**Files:**
- Rust: `snake_case.rs` (e.g., `lib.rs`, `audit.rs`)
- Python: `snake_case.py` (e.g., `__init__.py`, `state.py`, `sense.py`)
- Docs: `UPPERCASE.md` for key docs (e.g., `ARCHITECTURE.md`, `THREAT_MODEL.md`)
- ADRs: `ADR-NNNN-kebab-case-title.md` (e.g., `ADR-0001-canonical-state-in-rust.md`)
- Tests: `test_{feature}.py` (e.g., `test_smoke.py`, `test_agent_graph.py`)

**Directories:**
- Rust crates: `kri0k-{name}` prefix (e.g., `kri0k-core`, `kri0k-graph`)
- Python packages: `snake_case` (e.g., `kri0k/`, `agent/`, `nodes/`)
- Config: `lowercase` (e.g., `config/`)

**Types/Structs:**
- Rust: `PascalCase` (e.g., `NodeId`, `AuditSink`, `SafeguardsConfig`)
- Python TypedDict: `PascalCase` (e.g., `AgentState`)
- Python classes: `PascalCase`

**Functions:**
- Rust: `snake_case` (e.g., `validate_target`, `add_node`)
- Python: `snake_case` (e.g., `get_graph`, `route_after_reflect`)
- Node functions: `snake_case` verb (e.g., `sense`, `reason`, `plan`, `act`, `reflect`)

**Constants:**
- Rust: `SCREAMING_SNAKE_CASE` (e.g., `TOKIO_RUNTIME`)
- Python: `SCREAMING_SNAKE_CASE` (e.g., `MAX_ITERATIONS`)

## Where to Add New Code

**New Rust Feature:**
- Core type/trait: `crates/kri0k-core/src/{feature}.rs`, export in `lib.rs`
- Graph-related: `crates/kri0k-graph/src/lib.rs` or new module
- Python binding: `crates/kri0k-pybridge/src/lib.rs`, add `#[pyfunction]`

**New TTP Implementation:**
- Implementation: `crates/kri0k-core/src/ttp.rs` (or future `kri0k-ttp` crate)
- Pattern: Implement `Ttp` trait, register via `inventory::submit!` (planned)

**New Python Module:**
- Source: `python/kri0k/{module}.py`
- Export: Add to `python/kri0k/__init__.py`
- Type stubs: `python/kri0k/{module}.pyi` if needed

**New LangGraph Node:**
- Location: `python/kri0k/agent/nodes/{node_name}.py`
- Pattern: `async def {node_name}(state: AgentState) -> dict[str, Any]`
- Export: Add to `python/kri0k/agent/nodes/__init__.py`
- Register: Add `graph.add_node("{name}", {name})` in `graph.py`
- Connect: Add edges in `graph.py`

**Modify Agent State:**
- Edit: `python/kri0k/agent/state.py` AgentState TypedDict
- Update: All node return types that set new fields
- Test: Add field assertion in `tests/test_agent_graph.py:test_agent_state_has_required_fields`

**New Conditional Route:**
- Location: `python/kri0k/agent/graph.py`
- Pattern: `def route_{from_node}(state: AgentState) -> str`
- Register: `graph.add_conditional_edges("{from_node}", route_{from_node})`

**New LLM Provider (planned):**
- Location: `python/kri0k/providers/{provider}.py`
- Pattern: Implement `LLMProvider` protocol/ABC

**New Test:**
- Rust unit test: Add `#[cfg(test)] mod tests` in same file
- Python test: `tests/test_{feature}.py`
- Agent test: Add to `tests/test_agent_graph.py` for graph/node tests

**New ADR:**
- Location: `docs/adr/ADR-NNNN-title.md`
- Naming: Increment from highest existing (currently ADR-0012)
- Update: `docs/adr/README.md` index

**New Security Documentation:**
- Location: `docs/security/{doc}.md`
- Reference: Link from `THREAT_MODEL.md` if relevant

## Special Directories

**`target/`:**
- Purpose: Cargo build output
- Generated: Yes (by cargo)
- Committed: No (in `.gitignore`)

**`.git/`:**
- Purpose: Git repository data
- Generated: Yes (by git)
- Committed: No (git internal)

**`__pycache__/`:**
- Purpose: Python bytecode cache
- Generated: Yes (by Python)
- Committed: No (in `.gitignore`)

**`.mypy_cache/`:**
- Purpose: mypy type checking cache
- Generated: Yes (by mypy)
- Committed: No (in `.gitignore`)

**`.venv/` / `venv/`:**
- Purpose: Python virtual environment
- Generated: Yes (by venv/uv)
- Committed: No (in `.gitignore`)

## Crate Dependency Graph

```
kri0k-core (foundation)
    │
    ├── kri0k-graph (depends on kri0k-core)
    │       │
    │       └── kri0k-pybridge (depends on kri0k-core, kri0k-graph)
    │                │
    │                └── kri0k._native (Python extension module)
    │                         │
    │                         └── kri0k (Python package)
    │                                  │
    │                                  └── kri0k.agent (LangGraph agent)
    │                                           │
    │                                           ├── state.py (AgentState)
    │                                           ├── graph.py (StateGraph)
    │                                           └── nodes/ (5 node functions)
    │
    └── kri0k-ttp (planned, will depend on kri0k-core)
    └── kri0k-scope (planned, will depend on kri0k-core)
```

## Workspace Members

From `Cargo.toml`:
```toml
[workspace]
resolver = "2"
members = [
    "crates/kri0k-core",
    "crates/kri0k-graph",
    "crates/kri0k-pybridge",
]
```

Planned crates (from `docs/ARCHITECTURE.md`):
- `kri0k-ttp`: TTP trait implementations
- `kri0k-scope`: scope.yaml DSL parser

## Python Package Structure

From `pyproject.toml`:
```toml
[tool.maturin]
manifest-path = "crates/kri0k-pybridge/Cargo.toml"
module-name = "kri0k._native"
python-source = "python"
```

The native Rust code compiles to `kri0k._native` module, re-exported by `python/kri0k/__init__.py`.

**Agent Module Structure:**
```
python/kri0k/agent/
├── __init__.py     # Exports: AgentState, get_graph
├── state.py        # AgentState TypedDict (7 fields)
├── graph.py        # get_graph(), route_after_reflect(), MAX_ITERATIONS=10
└── nodes/
    ├── __init__.py # Exports: sense, reason, plan, act, reflect
    ├── sense.py    # async def sense(state) -> dict
    ├── reason.py   # async def reason(state) -> dict
    ├── plan.py     # async def plan(state) -> dict
    ├── act.py      # async def act(state) -> dict
    └── reflect.py  # async def reflect(state) -> dict (increments iteration_count)
```

---

*Structure analysis: 2026-05-15*
