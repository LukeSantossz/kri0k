# Phase 1: LangGraph Structure - Pattern Map

**Mapped:** 2026-05-14
**Files analyzed:** 10 (new files to be created)
**Analogs found:** 3 / 10

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `python/kri0k/agent/graph.py` | graph-builder | request-response | None (LangGraph-specific) | no-analog |
| `python/kri0k/agent/state.py` | model/type | data-transform | `python/kri0k/_native.pyi` | partial |
| `python/kri0k/agent/nodes/sense.py` | node | async-transform | None (new pattern) | no-analog |
| `python/kri0k/agent/nodes/reason.py` | node | async-transform | None (new pattern) | no-analog |
| `python/kri0k/agent/nodes/plan.py` | node | async-transform | None (new pattern) | no-analog |
| `python/kri0k/agent/nodes/act.py` | node | async-transform | None (new pattern) | no-analog |
| `python/kri0k/agent/nodes/reflect.py` | node | async-transform | None (new pattern) | no-analog |
| `python/kri0k/agent/__init__.py` | package-init | export | `python/kri0k/__init__.py` | exact |
| `python/kri0k/agent/nodes/__init__.py` | package-init | export | `python/kri0k/__init__.py` | exact |
| `tests/test_agent_graph.py` | test | validation | `tests/test_smoke.py` | exact |

## Pattern Assignments

### `python/kri0k/agent/__init__.py` (package-init, export)

**Analog:** `python/kri0k/__init__.py`

**Full pattern** (lines 1-9):
```python
"""Kri0k — AI-driven reconnaissance orchestrator.

This package provides Python bindings to the kri0k Rust core.
"""

from kri0k._native import get_dummy_graph, hello

__version__ = "0.1.0"
__all__ = ["get_dummy_graph", "hello"]
```

**Adapt for agent package:**
- Docstring describes agent module purpose
- Import `get_graph` from graph.py
- Export `AgentState` from state.py
- Define `__all__` with public exports

---

### `python/kri0k/agent/nodes/__init__.py` (package-init, export)

**Analog:** `python/kri0k/__init__.py`

**Pattern:**
- Docstring for nodes package
- Import all node functions: `sense`, `reason`, `plan`, `act`, `reflect`
- Export via `__all__`

---

### `python/kri0k/agent/state.py` (model/type, data-transform)

**Analog:** `python/kri0k/_native.pyi` (partial match for typing style)

**Typing pattern** (lines 1-6):
```python
"""Type stubs for kri0k._native (Rust extension module).

Generated from Rust PyO3 bindings.
"""

from typing import Any
```

**Adapt for AgentState TypedDict:**
- Module-level docstring explaining state structure
- Import from `typing`: `TypedDict`, `Any`
- Define `AgentState(TypedDict)` with 7 fields per D-02:
  - `snapshot: dict[str, Any]`
  - `analysis: dict[str, Any]`
  - `proposal: dict[str, Any]`
  - `decision: dict[str, Any]`
  - `iteration_count: int`
  - `history: list[dict[str, Any]]`
  - `engagement_context: dict[str, Any]`

---

### `tests/test_agent_graph.py` (test, validation)

**Analog:** `tests/test_smoke.py`

**Docstring pattern** (line 1):
```python
"""Smoke tests for kri0k Python package."""
```

**Import pattern** (line 3):
```python
import kri0k
```

**Test function pattern** (lines 6-10):
```python
def test_hello() -> None:
    """Test hello() function from Rust core."""
    result = kri0k.hello()
    assert isinstance(result, str)
    assert "kri0k" in result.lower()
```

**Structure test pattern** (lines 14-44):
```python
def test_get_dummy_graph_structure() -> None:
    """Test get_dummy_graph() returns valid structure."""
    graph = kri0k.get_dummy_graph()

    # Verify top-level structure
    assert isinstance(graph, dict)
    assert "nodes" in graph
    assert "edges" in graph

    # Verify nodes structure
    nodes = graph["nodes"]
    assert isinstance(nodes, list)
    assert len(nodes) > 0
```

**Adapt for agent graph tests:**
- Test `get_graph()` returns compiled StateGraph
- Test graph has all 5 nodes (sense, reason, plan, act, reflect)
- Test graph topology (START -> sense, reflect -> conditional -> sense or END)
- Test placeholder nodes return empty dict
- Test iteration_count increments
- Use `pytest-asyncio` for async tests

---

### `python/kri0k/agent/graph.py` (graph-builder, request-response)

**Analog:** None (LangGraph-specific)

**Reference patterns from LangGraph standard:**

**Imports (LangGraph convention):**
```python
"""LangGraph agent graph builder."""

from typing import Any

from langgraph.graph import END, START, StateGraph

from kri0k.agent.nodes import act, plan, reason, reflect, sense
from kri0k.agent.state import AgentState
```

**Graph builder pattern:**
```python
MAX_ITERATIONS: int = 10


def route_after_reflect(state: AgentState) -> str:
    """Route after reflect node: continue loop or end."""
    if state["iteration_count"] >= MAX_ITERATIONS:
        return END
    # Placeholder: always continue until iteration limit
    return "sense"


def get_graph() -> StateGraph:
    """Build and compile the agent graph.

    Returns:
        Compiled StateGraph ready for invocation.
    """
    graph = StateGraph(AgentState)

    # Add nodes
    graph.add_node("sense", sense)
    graph.add_node("reason", reason)
    graph.add_node("plan", plan)
    graph.add_node("act", act)
    graph.add_node("reflect", reflect)

    # Add edges
    graph.add_edge(START, "sense")
    graph.add_edge("sense", "reason")
    graph.add_edge("reason", "plan")
    graph.add_edge("plan", "act")
    graph.add_edge("act", "reflect")
    graph.add_conditional_edges("reflect", route_after_reflect)

    return graph.compile()
```

---

### `python/kri0k/agent/nodes/*.py` (node, async-transform)

**Analog:** None (new pattern for this project)

**Node signature per D-05, D-06, D-07:**
```python
"""[Node name] node for LangGraph agent."""

from typing import Any

from kri0k.agent.state import AgentState


async def sense(state: AgentState) -> dict[str, Any]:
    """Sense node: observe current state.

    Placeholder implementation returns empty dict (no-op).

    Args:
        state: Current agent state.

    Returns:
        State updates (empty for placeholder).
    """
    return {}
```

**Pattern per node file:**
| File | Function | Docstring Focus |
|------|----------|-----------------|
| `sense.py` | `sense` | Observe current state from Rust snapshot |
| `reason.py` | `reason` | Analyze observations, identify patterns |
| `plan.py` | `plan` | Generate action proposals |
| `act.py` | `act` | Execute approved actions |
| `reflect.py` | `reflect` | Evaluate results, update iteration_count |

**Reflect node special case (increments iteration_count):**
```python
async def reflect(state: AgentState) -> dict[str, Any]:
    """Reflect node: evaluate results and update iteration count.

    Placeholder implementation increments iteration_count only.

    Args:
        state: Current agent state.

    Returns:
        State updates with incremented iteration_count.
    """
    return {"iteration_count": state["iteration_count"] + 1}
```

---

## Shared Patterns

### Module Docstrings
**Source:** `python/kri0k/__init__.py` line 1-4
**Apply to:** All new Python files
```python
"""[Brief description of module purpose].

[Optional longer description if needed.]
"""
```

### Type Annotations (mypy strict)
**Source:** `pyproject.toml` lines 170-182
**Apply to:** All new Python files
- All functions must have full type hints
- Return type required (`-> None`, `-> dict[str, Any]`, etc.)
- Use `dict[str, Any]` not `Dict[str, Any]` (Python 3.11+)

### Import Organization (ruff isort)
**Source:** `pyproject.toml` lines 150-152
**Apply to:** All new Python files
```python
# Standard library
from typing import Any

# Third-party
from langgraph.graph import StateGraph

# First-party
from kri0k.agent.state import AgentState
```

### Test Function Pattern
**Source:** `tests/test_smoke.py` lines 6-10
**Apply to:** `tests/test_agent_graph.py`
```python
def test_function_name() -> None:
    """Test description using Google docstring convention."""
    # Arrange
    # Act
    result = function_under_test()
    # Assert
    assert condition
```

### Google Docstrings
**Source:** `pyproject.toml` line 155 (`convention = "google"`)
**Apply to:** All new Python files
```python
def function(arg: ArgType) -> ReturnType:
    """Brief description.

    Args:
        arg: Description of argument.

    Returns:
        Description of return value.
    """
```

---

## No Analog Found

Files with no close match in the codebase (use reference patterns above):

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `python/kri0k/agent/graph.py` | graph-builder | request-response | No LangGraph code exists yet |
| `python/kri0k/agent/nodes/sense.py` | node | async-transform | First async node in project |
| `python/kri0k/agent/nodes/reason.py` | node | async-transform | First async node in project |
| `python/kri0k/agent/nodes/plan.py` | node | async-transform | First async node in project |
| `python/kri0k/agent/nodes/act.py` | node | async-transform | First async node in project |
| `python/kri0k/agent/nodes/reflect.py` | node | async-transform | First async node in project |

**Guidance:** Use the reference patterns in "Pattern Assignments" section above. These are derived from:
1. Project conventions (pyproject.toml settings)
2. LangGraph standard patterns
3. CONTEXT.md decisions (D-01 through D-17)

---

## Metadata

**Analog search scope:** `python/kri0k/`, `tests/`
**Files scanned:** 2 Python files, 1 pyi stub
**Pattern extraction date:** 2026-05-14

---

*Phase: 01-langgraph-structure*
*Pattern mapping: 2026-05-14*
