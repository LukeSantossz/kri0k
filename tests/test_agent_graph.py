"""Tests for the LangGraph agent structure and execution."""

from langgraph.graph import END
import pytest

from kri0k.agent import AgentState, get_graph
from kri0k.agent.graph import MAX_ITERATIONS, route_after_reflect
from kri0k.agent.nodes import act, plan, reason, reflect, sense

pytestmark = pytest.mark.graph


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


def test_get_graph_compiles() -> None:
    """Test that get_graph() returns a compiled graph."""
    graph = get_graph()
    assert graph is not None
    assert hasattr(graph, "invoke")


def test_graph_has_five_nodes() -> None:
    """Test that the graph contains all five engagement loop nodes."""
    graph = get_graph()
    # Check node names in the compiled graph
    node_names = set(graph.nodes.keys())
    # LangGraph adds __start__ and __end__ nodes
    expected_nodes = {"sense", "reason", "plan", "act", "reflect"}
    assert expected_nodes.issubset(node_names), f"Missing nodes: {expected_nodes - node_names}"


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


@pytest.mark.asyncio
async def test_sense_node_returns_snapshot_dict() -> None:
    """Test that sense node returns a state update with `snapshot`.

    Phase 2 wiring: sense fetches `_native.get_dummy_graph()` and writes
    the formatted snapshot to state. See `tests/test_sense_node.py` for
    detailed coverage.
    """
    state = _minimal_state()
    result = await sense(state)
    assert "snapshot" in result
    snap = result["snapshot"]
    assert isinstance(snap, dict)
    assert "raw" in snap
    assert "formatted" in snap


@pytest.mark.asyncio
async def test_reason_node_returns_analysis_dict() -> None:
    """Test that reason node returns analysis dict (no LLM = graceful fallback)."""
    state = _minimal_state()
    result = await reason(state)
    assert "analysis" in result
    assert isinstance(result["analysis"], dict)


@pytest.mark.asyncio
async def test_plan_node_returns_proposal_dict() -> None:
    """Test that plan node returns proposal dict (no LLM = graceful fallback)."""
    state = _minimal_state()
    result = await plan(state)
    assert "proposal" in result
    assert isinstance(result["proposal"], dict)


@pytest.mark.asyncio
async def test_act_node_returns_empty_dict() -> None:
    """Test that act node placeholder returns empty dict."""
    state = _minimal_state()
    result = await act(state)
    assert result == {}


@pytest.mark.asyncio
async def test_reflect_node_increments_iteration() -> None:
    """Test that reflect node increments iteration_count."""
    state = _minimal_state()
    state["iteration_count"] = 5
    result = await reflect(state)
    assert result == {"iteration_count": 6}


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


def test_route_after_reflect_ends_above_max() -> None:
    """Test that router returns END when above MAX_ITERATIONS."""
    state = _minimal_state()
    state["iteration_count"] = 15
    result = route_after_reflect(state)
    assert result == END


def test_max_iterations_is_ten() -> None:
    """Test that MAX_ITERATIONS constant is 10."""
    assert MAX_ITERATIONS == 10
