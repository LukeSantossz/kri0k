# Codebase Structure

**Analysis Date:** 2026-05-14

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
├── tests/                      # Python tests
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
- Contains: Package init, type stubs for native module
- Key files: `__init__.py`, `_native.pyi`

**`tests/`:**
- Purpose: Python test suite
- Contains: Smoke tests, integration tests
- Key files: `test_smoke.py`

## Key File Locations

**Entry Points:**
- `python/kri0k/__init__.py`: Python package entry point
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

**Testing:**
- `tests/test_smoke.py`: Python smoke tests for native bindings

**Documentation:**
- `docs/ARCHITECTURE.md`: System architecture (Portuguese, detailed)
- `docs/security/THREAT_MODEL.md`: Threat model and mitigations
- `docs/adr/`: Individual ADR files

## Naming Conventions

**Files:**
- Rust: `snake_case.rs` (e.g., `lib.rs`, `audit.rs`)
- Python: `snake_case.py` (e.g., `__init__.py`, `test_smoke.py`)
- Docs: `UPPERCASE.md` for key docs (e.g., `ARCHITECTURE.md`, `THREAT_MODEL.md`)
- ADRs: `ADR-NNNN-kebab-case-title.md` (e.g., `ADR-0001-canonical-state-in-rust.md`)

**Directories:**
- Rust crates: `kri0k-{name}` prefix (e.g., `kri0k-core`, `kri0k-graph`)
- Python: `snake_case` (e.g., `kri0k/`)
- Config: `lowercase` (e.g., `config/`)

**Types/Structs:**
- Rust: `PascalCase` (e.g., `NodeId`, `AuditSink`, `SafeguardsConfig`)
- Python: `PascalCase` for classes, `snake_case` for functions

**Functions:**
- Rust: `snake_case` (e.g., `validate_target`, `add_node`)
- Python: `snake_case` (e.g., `get_dummy_graph`, `test_hello`)

**Constants:**
- Rust: `SCREAMING_SNAKE_CASE` (e.g., `TOKIO_RUNTIME`)

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

**New LangGraph Node (planned):**
- Location: `python/kri0k/nodes/{node_name}.py`
- Pattern: Follow sense/reason/plan/act/reflect structure

**New LLM Provider (planned):**
- Location: `python/kri0k/providers/{provider}.py`
- Pattern: Implement `LLMProvider` trait

**New Test:**
- Rust unit test: Add `#[cfg(test)] mod tests` in same file
- Python test: `tests/test_{feature}.py`

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

---

*Structure analysis: 2026-05-14*
