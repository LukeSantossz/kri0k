# Technology Stack

**Analysis Date:** 2026-05-15

## Languages

**Primary:**
- Rust 1.75.0+ (MSRV) - Core logic, graph engine, validation, PyO3 bindings
- Python 3.11+ - LangGraph agent, LLM orchestration, CLI entrypoint

**Secondary:**
- YAML - Scope configuration (`config/scope.example.yaml`)
- TOML - Rust and Python package configuration

## Runtime

**Rust Environment:**
- Edition 2021
- Cargo workspace with resolver = "2"
- Multi-threaded Tokio runtime (2 worker threads, named `kri0k-tokio`)

**Python Environment:**
- Requires Python >= 3.11
- Supports Python 3.12
- Native extension via PyO3 + maturin build

**Package Managers:**
- Cargo (Rust) - Lockfile: `Cargo.lock` present
- uv (Python) - Lockfile: `uv.lock` present

## Frameworks

**Core:**
- petgraph 0.6 (serde-1 feature) - Graph data structure (`kri0k-graph`)
- pyo3 0.22 (extension-module feature) - Rust-Python bindings
- tokio 1.40 (rt, rt-multi-thread, macros) - Async runtime

**AI/Agent (Phase 1 Implemented):**
- langgraph >=0.2.0 - Agent orchestration (sense/reason/plan/act/reflect loop)
- langchain >=0.3.0 - LLM abstractions

**Serialization:**
- serde 1.0 (derive feature) - Rust serialization
- serde_json 1.0 - JSON codec
- pyyaml >=6.0 - YAML parsing in Python
- jinja2 >=3.1.0 - Prompt templates

**Testing:**
- pytest >=7.4 - Python test runner
- pytest-asyncio >=0.24 - Async test support
- pytest-cov >=4.1 - Coverage reporting
- Rust inline tests (`#[cfg(test)]` modules)

**Build/Dev:**
- maturin >=1.7,<2.0 - Build backend for PyO3 extension
- ruff >=0.8.0 - Python linting and formatting
- mypy >=1.11 - Python static type checking
- pre-commit >=3.8 - Git hooks
- rustfmt (stable) - Rust formatting
- clippy - Rust linting (strict config)

## Key Dependencies

**Critical (Rust):**
- `ulid 1.1` (serde feature) - Stable unique IDs for nodes/edges (temporally ordered)
- `thiserror 1.0` - Error handling derive macros
- `petgraph 0.6` - StableGraph for deterministic node indices

**Critical (Python):**
- `httpx >=0.27.0` - HTTP client for external APIs
- `ipaddress >=1.0.23` - CIDR/IP validation

**Optional LLM Providers:**
- `ollama >=0.3.0` - Local LLM (default, local-first per ADR-0008)
- `anthropic >=0.39.0` - Anthropic Claude API (opt-in)
- `openai >=1.54.0` - OpenAI API (opt-in)

## Configuration

**Rust Linting:**
- Clippy with strict lints: `correctness`, `suspicious`, `perf` = deny
- Security-critical denies: `unwrap_used`, `panic`, `unimplemented`
- Config files: `clippy.toml`, `Cargo.toml` workspace lints section

**Rust Formatting:**
- `rustfmt.toml` - Edition 2021, max_width 100, 4-space indentation
- Nightly options commented (available via `cargo +nightly fmt`)

**Python Linting:**
- ruff with extensive ruleset: E, W, F, I, N, UP, ANN, S, B, etc.
- Security rules enabled (S - bandit)
- Target version: py311, line length: 100

**Python Type Checking:**
- mypy strict mode with all warnings enabled
- Platform: linux
- Type stubs for `kri0k._native` native module (`python/kri0k/_native.pyi`)

**Environment:**
- Scope configuration: `config/scope.example.yaml`
- LLM selection: `KRI0K_LLM` env var or `--llm` flag
- User config: `~/.kri0k/config.toml` (documented, not present in repo)

**Build:**
- `pyproject.toml` - Maturin build backend
- `Cargo.toml` - Workspace root with 3 crates
- Release profile: LTO thin, codegen-units 1, strip symbols

## Workspace Structure

**Cargo Crates:**
```
crates/
├── kri0k-core/      # Runtime, scope, audit, types (serde, thiserror, ulid)
│   └── src/
│       ├── lib.rs       # NodeId, EdgeId, Error types
│       ├── audit.rs     # AuditSink trait, event types (stub)
│       ├── safeguards.rs # SafeguardsConfig (propose-only, kill switch)
│       ├── scope.rs     # Scope struct, validate_target (stub)
│       └── ttp.rs       # Ttp trait, RateLimits, ExecutionPlan (stub)
├── kri0k-graph/     # petgraph wrapper with typed nodes/edges
│   └── src/lib.rs   # Graph, Node, Edge, NodeKind, EdgeKind
└── kri0k-pybridge/  # PyO3 cdylib -> kri0k._native module
    └── src/lib.rs   # hello(), get_dummy_graph(), global Tokio runtime
```

**Python Package:**
```
python/kri0k/
├── __init__.py          # Exports from _native (hello, get_dummy_graph)
├── _native.pyi          # Type stubs for Rust bindings
└── agent/               # LangGraph agent module (Phase 1)
    ├── __init__.py      # Exports AgentState, get_graph
    ├── state.py         # AgentState TypedDict (7 fields)
    ├── graph.py         # StateGraph builder with 5 nodes, conditional routing
    └── nodes/           # Engagement loop node functions
        ├── __init__.py  # Exports all 5 nodes
        ├── sense.py     # Observe state from Rust snapshot (placeholder)
        ├── reason.py    # Analyze observations (placeholder)
        ├── plan.py      # Generate action proposals (placeholder)
        ├── act.py       # Execute approved actions (placeholder)
        └── reflect.py   # Evaluate results, increment iteration_count
```

## Agent Architecture (Phase 1)

**State Schema:**
- `AgentState` TypedDict with 7 fields: `snapshot`, `analysis`, `proposal`, `decision`, `iteration_count`, `history`, `engagement_context`
- Location: `python/kri0k/agent/state.py`

**Graph Structure:**
- 5 nodes: sense -> reason -> plan -> act -> reflect
- Conditional edge from reflect: continue to sense or END based on iteration_count
- MAX_ITERATIONS constant: 10
- Location: `python/kri0k/agent/graph.py`

**Node Functions:**
- All nodes are async functions returning `dict[str, Any]`
- Currently placeholder implementations (empty dict or iteration increment)
- Location: `python/kri0k/agent/nodes/`

## Platform Requirements

**Development:**
- Rust toolchain 1.75.0+
- Python 3.11+
- maturin for building native extension
- Ollama running locally for LLM features (recommended: qwen3:32b or deepseek-r1:32b)

**Production/Deployment:**
- Linux platform (mypy configured for linux)
- External tools for TTPs: nmap, dig, whois (planned per ADR-0012)
- Air-gapped deployment supported (Ollama local-first)

---

*Stack analysis: 2026-05-15*
