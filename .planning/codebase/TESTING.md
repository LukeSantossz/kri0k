# Testing Patterns

**Analysis Date:** 2026-05-15

## Test Framework

### Rust Testing

**Runner:** Built-in `cargo test`

**Config:** Workspace-level in `Cargo.toml`

**Run Commands:**
```bash
cargo test --all-features              # All tests
cargo test --lib --all-features        # Unit tests only
cargo test --doc --all-features        # Documentation tests
cargo test -p kri0k-core               # Single crate
```

### Python Testing

**Runner:** pytest 7.4+

**Config:** `pyproject.toml [tool.pytest.ini_options]`

**Assertion Library:** Built-in `assert` (pytest native)

**Async Support:** pytest-asyncio 0.24+

**Run Commands:**
```bash
pytest                                 # All tests
pytest -m unit                         # Unit tests only
pytest -m integration                  # Integration tests
pytest -m graph                        # Graph fixture tests
pytest tests/test_agent_graph.py       # Single test file
pytest --cov=kri0k --cov-report=html   # With coverage
```

## Test File Organization

### Rust Test Location

**Pattern:** Inline `#[cfg(test)]` modules in source files

**Structure:**
```
crates/kri0k-core/src/
├── lib.rs          # Contains #[cfg(test)] mod tests
├── scope.rs        # Contains #[cfg(test)] mod tests
├── safeguards.rs   # Contains #[cfg(test)] mod tests
└── ...

crates/kri0k-graph/src/
└── lib.rs          # Contains #[cfg(test)] mod tests
```

**Naming:** Tests named `test_<description>` inside `mod tests` block

### Python Test Location

**Pattern:** Separate `tests/` directory at project root

**Structure:**
```
tests/
├── test_smoke.py         # Smoke tests (Rust-Python bridge)
├── test_agent_graph.py   # LangGraph agent tests (12 tests)
├── unit/                 # Unit tests (planned)
├── integration/          # Integration tests (planned)
├── graph/                # Graph fixture tests (planned)
├── ttp/                  # TTP tests (planned)
├── audit/                # Audit log tests (planned)
└── fixtures/             # Test fixtures (planned)
```

**Naming:** `test_*.py` or `*_test.py` files

**Config:**
```toml
[tool.pytest.ini_options]
python_files = ["test_*.py", "*_test.py"]
python_classes = ["Test*"]
python_functions = ["test_*"]
```

## Test Structure

### Rust Test Pattern

```rust
#[cfg(test)]
#[allow(clippy::expect_used)] // expect is ok in tests
mod tests {
    use super::*;

    #[test]
    fn test_node_id_uniqueness() {
        let id1 = NodeId::new();
        let id2 = NodeId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_node_id_serialization() {
        let id = NodeId::new();
        let json = serde_json::to_string(&id).expect("serialize");
        let deserialized: NodeId = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(id, deserialized);
    }
}
```

**Key Patterns:**
- `#[allow(clippy::expect_used)]` on test module (expect is OK in tests)
- Import from parent with `use super::*;`
- `#[test]` attribute on each test function
- Descriptive function names: `test_<noun>_<action>` or `test_<action>_<noun>`

### Python Sync Test Pattern

```python
"""Tests for the LangGraph agent structure and execution."""

from langgraph.graph import END
import pytest

from kri0k.agent import AgentState, get_graph
from kri0k.agent.graph import MAX_ITERATIONS, route_after_reflect


def test_get_graph_compiles() -> None:
    """Test that get_graph() returns a compiled graph."""
    graph = get_graph()
    assert graph is not None
    assert hasattr(graph, "invoke")


def test_graph_has_five_nodes() -> None:
    """Test that the graph contains all five engagement loop nodes."""
    graph = get_graph()
    node_names = set(graph.nodes.keys())
    expected_nodes = {"sense", "reason", "plan", "act", "reflect"}
    assert expected_nodes.issubset(node_names), f"Missing nodes: {expected_nodes - node_names}"
```

**Key Patterns:**
- Module docstring describing test file purpose
- Function docstring describing individual test
- Type annotation `-> None` on test functions
- Descriptive assertion messages using f-strings
- Import constants directly: `from kri0k.agent.graph import MAX_ITERATIONS`

### Python Async Test Pattern

```python
import pytest

from kri0k.agent.nodes import act, plan, reason, reflect, sense


def _minimal_state() -> AgentState:
    """Create a minimal AgentState with default values."""
    return AgentState(
        snapshot={},
        analysis={},
        proposal={},
        decision={},
        iteration_count=0,
        history=[],
        engagement_context={},
    )


@pytest.mark.asyncio
async def test_sense_node_returns_empty_dict() -> None:
    """Test that sense node placeholder returns empty dict."""
    state = _minimal_state()
    result = await sense(state)
    assert result == {}


@pytest.mark.asyncio
async def test_reflect_node_increments_iteration() -> None:
    """Test that reflect node increments iteration_count."""
    state = _minimal_state()
    state["iteration_count"] = 5
    result = await reflect(state)
    assert result == {"iteration_count": 6}
```

**Key Patterns:**
- Use `@pytest.mark.asyncio` decorator for async tests
- Use `await` when calling async node functions
- Create helper factory functions with leading underscore (`_minimal_state()`)
- Test state mutations through TypedDict key access

## Test Markers (Python)

**Defined in `pyproject.toml`:**

```toml
markers = [
    "unit: Unit tests (fast, no I/O)",
    "integration: Integration tests (cross-language, Rust <-> Python)",
    "slow: Slow tests (network, external tools)",
    "ttp: TTP-specific tests (requires external tools: nmap, dig, whois)",
    "graph: Graph fixture tests (petgraph state validation)",
    "audit: Audit log validation tests",
]
```

**Usage:**
```python
import pytest

@pytest.mark.unit
def test_something_fast():
    ...

@pytest.mark.integration
def test_rust_python_bridge():
    ...

@pytest.mark.asyncio
async def test_async_node():
    ...

@pytest.mark.ttp
@pytest.mark.skipif(not shutil.which("nmap"), reason="nmap not installed")
def test_nmap_scanner():
    ...
```

**Run by marker:**
```bash
pytest -m unit           # Only unit tests
pytest -m integration    # Only integration tests
pytest -m "slow or ttp"  # Slow tests
pytest -m "not slow"     # Exclude slow tests
```

## Test Categories

### 1. Rust Unit Tests

**Location:** Inline `#[cfg(test)]` modules
**Purpose:** Test individual Rust functions and modules in isolation
**Run:** `cargo test --lib --all-features`

**Examples in codebase:**
- `crates/kri0k-core/src/lib.rs`: `NodeId`/`EdgeId` uniqueness and serialization
- `crates/kri0k-core/src/scope.rs`: Scope validation (stub tests)
- `crates/kri0k-core/src/safeguards.rs`: Kill switch logic
- `crates/kri0k-graph/src/lib.rs`: Graph creation, node/edge operations

### 2. Python Smoke Tests

**Location:** `tests/test_smoke.py`
**Purpose:** Verify Rust - Python bridge works
**Run:** `pytest tests/test_smoke.py`

**Current tests (3 tests):**
- `test_hello()`: Verify `kri0k.hello()` returns string with "kri0k"
- `test_get_dummy_graph_structure()`: Verify graph dict structure
- `test_graph_node_kinds()`: Verify node type serialization

### 3. LangGraph Agent Tests

**Location:** `tests/test_agent_graph.py`
**Purpose:** Test LangGraph agent structure and node behavior
**Run:** `pytest tests/test_agent_graph.py`

**Current tests (12 tests):**

| Test | Purpose |
|------|---------|
| `test_get_graph_compiles` | Graph compiles and has invoke method |
| `test_graph_has_five_nodes` | All 5 engagement nodes present |
| `test_agent_state_has_required_fields` | TypedDict has 7 required fields |
| `test_sense_node_returns_empty_dict` | Sense placeholder returns {} |
| `test_reason_node_returns_empty_dict` | Reason placeholder returns {} |
| `test_plan_node_returns_empty_dict` | Plan placeholder returns {} |
| `test_act_node_returns_empty_dict` | Act placeholder returns {} |
| `test_reflect_node_increments_iteration` | Reflect increments iteration_count |
| `test_route_after_reflect_continues_under_max` | Router returns "sense" when < 10 |
| `test_route_after_reflect_ends_at_max` | Router returns END at 10 |
| `test_route_after_reflect_ends_above_max` | Router returns END above 10 |
| `test_max_iterations_is_ten` | MAX_ITERATIONS constant is 10 |

### 4. Integration Tests (Planned)

**Location:** `tests/integration/`
**Purpose:** Test Rust - Python interactions via PyO3 bridge
**Marker:** `@pytest.mark.integration`
**Run:** `pytest -m integration`

**Expected tests (from CONTRIBUTING.md):**
- `test_pybridge.py`: Snapshot codec, Proposal validation
- `test_engagement_lifecycle.py`: open -> snapshot -> validate -> execute -> close

### 5. Graph Fixture Tests (Planned)

**Location:** `tests/graph/`
**Purpose:** Test graph state transitions, invariant preservation
**Marker:** `@pytest.mark.graph`
**Run:** `pytest -m graph`

**Expected tests:**
- Node ID stability after add/remove
- Edge integrity after node removal
- Snapshot idempotency (serialize -> deserialize -> serialize)

### 6. TTP Tests (Planned)

**Location:** `tests/ttp/`
**Purpose:** Test TTP implementations with external tools
**Marker:** `@pytest.mark.ttp`
**Requirements:** External tools (nmap, dig, whois)
**Run:** `pytest -m ttp` (slow, requires network)

### 7. Audit Log Tests (Planned)

**Location:** `tests/audit/`
**Purpose:** Validate audit log integrity, hash chaining
**Marker:** `@pytest.mark.audit`
**Run:** `pytest -m audit`

## Mocking

### Rust Mocking

**Framework:** None observed (std test framework only)

**Pattern for stubs (current):**
```rust
/// No-op audit sink for testing.
#[derive(Debug, Default)]
pub struct NoOpAuditSink;

impl AuditSink for NoOpAuditSink {
    fn log_ttp_execution(&mut self, _event: TtpExecutionEvent) -> Result<(), crate::Error> {
        Ok(())
    }
    // ...
}
```

### Python Mocking

**Framework:** Not yet used in current minimal tests

**Expected:** pytest fixtures + unittest.mock for integration tests

## Fixtures

### Test Helper Functions

**Pattern (from `tests/test_agent_graph.py`):**
```python
def _minimal_state() -> AgentState:
    """Create a minimal AgentState with default values."""
    return AgentState(
        snapshot={},
        analysis={},
        proposal={},
        decision={},
        iteration_count=0,
        history=[],
        engagement_context={},
    )
```

**Rules:**
- Use leading underscore for non-test helper functions
- Include docstring explaining purpose
- Return fully-initialized objects with sensible defaults

### Planned Fixtures (from CONTRIBUTING.md)

**Graph Fixtures (`tests/fixtures/`):**
- `empty_graph.jsonl`: Empty engagement, scope only
- `single_host.jsonl`: One discovered host
- `multi_host_network.jsonl`: Small network topology
- `post_pivot.jsonl`: After credential discovery and lateral movement

**Scope Fixtures (`tests/fixtures/scope_*.yaml`):**
- `scope_single_cidr.yaml`: Single /24 network
- `scope_multi_domain.yaml`: Multiple domains + subdomains
- `scope_strict.yaml`: Minimal permissions, propose-only
- `scope_permissive.yaml`: Execute-enabled, broader permissions

### pytest Fixture Pattern (Expected)

```python
import pytest
from pathlib import Path

@pytest.fixture
def tmp_scope_yaml(tmp_path: Path) -> Path:
    """Create a temporary scope.yaml file."""
    scope_file = tmp_path / "scope.yaml"
    scope_file.write_text("""
targets:
  - 192.168.1.0/24
operator: test@example.com
""")
    return scope_file
```

## Coverage

### Rust Coverage

**Requirements:** Minimum 80% (measured by cargo-tarpaulin in CI)

**Tool:** cargo-tarpaulin (CI only, not configured locally)

### Python Coverage

**Requirements:** Minimum 85%

**Tool:** pytest-cov

**Config (`pyproject.toml`):**
```toml
[tool.coverage.run]
source = ["kri0k"]
omit = ["tests/*", "kri0k/_native.pyi"]

[tool.coverage.report]
precision = 2
show_missing = true
exclude_lines = [
    "pragma: no cover",
    "if TYPE_CHECKING:",
    "raise NotImplementedError",
    "@abstractmethod",
    "@overload",
]
```

**Run:**
```bash
pytest --cov=kri0k --cov-report=html
# Open htmlcov/index.html
```

**CI:** Coverage uploaded to Codecov

## CI Integration

**Config:** `.github/workflows/ci.yaml`

### Rust CI Tests

```yaml
- name: Run tests
  run: cargo test --all-features --verbose

- name: Run doctests
  run: cargo test --doc --all-features
```

### Python CI Tests

```yaml
- name: Run pytest (unit)
  run: pytest -m unit --cov=kri0k --cov-report=xml

- name: Upload coverage to Codecov
  uses: codecov/codecov-action@v4
```

### Integration CI Tests

```yaml
integration:
  runs-on: ubuntu-latest
  needs: [rust-check, python-check]
  steps:
    - name: Build Rust extension
      run: maturin develop --release

    - name: Run integration tests
      run: pytest -m integration -v

    - name: Run graph fixture tests
      run: pytest -m graph -v
```

## Test Patterns

### Async Testing (Python)

**Framework:** pytest-asyncio (in dev dependencies)

**Pattern:**
```python
import pytest

from kri0k.agent.nodes import sense
from kri0k.agent.state import AgentState


@pytest.mark.asyncio
async def test_sense_node_returns_empty_dict() -> None:
    """Test that sense node placeholder returns empty dict."""
    state = _minimal_state()
    result = await sense(state)
    assert result == {}
```

**Rules:**
- Import `pytest` at module level
- Use `@pytest.mark.asyncio` decorator
- Use `async def` for test function
- Use `await` for async function calls

### Error Testing (Rust)

```rust
#[test]
fn test_scope_validation_todo() {
    let scope = Scope {
        targets: vec!["192.168.1.0/24".to_string()],
        operator: "test@example.com".to_string(),
        metadata: HashMap::new(),
    };

    // Should return error until implemented
    let result = validate_target(&scope, "192.168.1.10");
    assert!(
        matches!(result, Err(crate::Error::Generic(ref s)) if s.contains("not implemented"))
    );
}
```

### Graph State Testing (Rust)

```rust
#[test]
fn test_add_edge() {
    let mut graph = Graph::new();
    let node1 = Node::new(NodeKind::Host { ip: "192.168.1.1".to_string() });
    let node2 = Node::new(NodeKind::Network { cidr: "192.168.1.0/24".to_string() });

    let id1 = graph.add_node(node1);
    let id2 = graph.add_node(node2);

    let edge = Edge::new(id1, id2, EdgeKind::BelongsTo);
    let result = graph.add_edge(edge);

    assert!(result.is_ok());
    assert_eq!(graph.edge_count(), 1);
}
```

### TypedDict Field Testing (Python)

```python
def test_agent_state_has_required_fields() -> None:
    """Test that AgentState TypedDict has all 7 required fields."""
    expected_fields = {
        "snapshot",
        "analysis",
        "proposal",
        "decision",
        "iteration_count",
        "history",
        "engagement_context",
    }
    actual_fields = set(AgentState.__annotations__.keys())
    assert expected_fields == actual_fields, (
        f"Field mismatch. Expected: {expected_fields}, Got: {actual_fields}"
    )
```

### Cross-Language Testing (Python)

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

    # Check first node has required fields
    node = nodes[0]
    assert "id" in node
    assert "kind" in node
```

### Routing Function Testing (Python)

```python
def test_route_after_reflect_continues_under_max() -> None:
    """Test that router returns 'sense' when under MAX_ITERATIONS."""
    state = _minimal_state()
    state["iteration_count"] = 5
    result = route_after_reflect(state)
    assert result == "sense"


def test_route_after_reflect_ends_at_max() -> None:
    """Test that router returns END when at MAX_ITERATIONS."""
    state = _minimal_state()
    state["iteration_count"] = 10
    result = route_after_reflect(state)
    assert result == END
```

## Pre-commit Test Hooks

**Fast feedback (pre-commit):**
- `cargo test --lib --all-features` (Rust unit tests)
- `pytest -m unit -x` (Python unit tests, stop on first failure)

**Skip for CI speed:**
```bash
SKIP=cargo-test-unit,pytest-unit pre-commit run --all-files
```

## Test Requirements

**New features:**
- Must include tests
- Must maintain or improve coverage

**Bug fixes:**
- Must include regression test

**Pre-merge checklist (from PR template):**
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated (if cross-language changes)
- [ ] Graph fixture tests added/updated (if graph state changes)
- [ ] Manual testing performed

## Current Test Summary

| Category | Location | Test Count |
|----------|----------|------------|
| Rust unit tests | `crates/*/src/*.rs` | ~15 |
| Python smoke tests | `tests/test_smoke.py` | 3 |
| Python agent tests | `tests/test_agent_graph.py` | 12 |
| **Total** | | **~30** |

---

*Testing analysis: 2026-05-15*
