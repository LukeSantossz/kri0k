# Phase 1: LangGraph Structure - Context

**Gathered:** 2026-05-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Create the structural foundation of the LangGraph agent with StateGraph, placeholder nodes (sense/reason/plan/act/reflect), edges connecting them in the correct engagement loop sequence, and conditional routing for iteration control. No real node logic — this phase proves the graph topology works end-to-end.

</domain>

<decisions>
## Implementation Decisions

### Graph State Structure
- **D-01:** Use TypedDict for state — `class AgentState(TypedDict)` enforced by mypy
- **D-02:** Include 7 fields: `snapshot`, `analysis`, `proposal`, `decision`, `iteration_count`, `history`, `engagement_context`
- **D-03:** `engagement_context` (scope, objective, operator) passed at `graph.invoke()` call, not read from files
- **D-04:** History list structure is Claude's discretion — implementation will determine best format

### Node Signatures
- **D-05:** Typed input, generic return: `async def sense(state: AgentState) -> dict[str, Any]`
- **D-06:** All nodes are async functions (`async def`)
- **D-07:** Placeholder nodes return empty dict (`return {}`) — no-op pass-through
- **D-08:** Full state passed to all nodes — nodes decide what to use

### Edge Topology
- **D-09:** Full engagement loop: sense→reason→plan→act→reflect→[conditional: sense if continue, END if done/blocked]
- **D-10:** Router is a named function with typed signature, not inline lambda
- **D-11:** Entry point is START → sense (no init node)
- **D-12:** Lazy compilation via `get_graph()` function, not module-level eager compile
- **D-13:** Max iterations guard in router: `iteration_count < MAX_ITERATIONS`
- **D-14:** MAX_ITERATIONS = 10 hardcoded default

### File Organization
- **D-15:** Separate files from start, not single file
- **D-16:** One file per node: `sense.py`, `reason.py`, `plan.py`, `act.py`, `reflect.py`
- **D-17:** Router logic lives in `graph.py` alongside graph builder

### Claude's Discretion
- History list entry structure (D-04) — implementation detail
- Any internal helper functions needed for the graph builder
- Test organization within `tests/test_agent_graph.py`

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Architecture
- `docs/adr/ADR-0001-canonical-state-in-rust.md` — Rust is source of truth; Python receives snapshots
- `.planning/codebase/ARCHITECTURE.md` — System overview and data flow
- `.planning/codebase/STRUCTURE.md` — File organization conventions

### Python/Rust Bridge
- `python/kri0k/__init__.py` — Current Python package exports
- `python/kri0k/_native.pyi` — Type stubs for Rust bindings
- `crates/kri0k-pybridge/src/lib.rs` — PyO3 module implementation

### Project Configuration
- `pyproject.toml` — Python/maturin config, mypy/ruff settings

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `kri0k._native.get_dummy_graph()` — Returns dummy graph dict; sense node will call this in Phase 2
- `kri0k._native` module — PyO3 bridge already initialized with Tokio runtime

### Established Patterns
- **Typing:** mypy strict mode enforced — all new code must be fully typed
- **Async:** Future LLM/Rust calls will be async — nodes defined as `async def` now
- **Snake case:** Python functions use `snake_case` per project conventions
- **Serde JSON:** Cross-boundary data is serialized JSON dicts, not live references

### Integration Points
- `python/kri0k/agent/` — New directory for LangGraph code
- `python/kri0k/__init__.py` — Will need to export graph entry point eventually
- `tests/` — Python tests for the new agent graph

</code_context>

<specifics>
## Specific Ideas

No specific requirements — standard LangGraph patterns with project typing conventions.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 01-langgraph-structure*
*Context gathered: 2026-05-14*
