"""Tests for the wired sense node (Phase 2).

The Rust `_native.get_dummy_graph()` mints fresh ULIDs on every call, so
we assert structural shape rather than equality across invocations.
"""

import pytest

from kri0k.agent.nodes.sense import sense
from kri0k.agent.state import AgentState

pytestmark = pytest.mark.integration


def _state() -> AgentState:
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
async def test_sense_returns_snapshot_with_raw_and_formatted() -> None:
    result = await sense(_state())
    assert "snapshot" in result
    snap = result["snapshot"]
    assert isinstance(snap, dict)
    assert "raw" in snap
    assert "formatted" in snap
    assert isinstance(snap["raw"], dict)
    assert isinstance(snap["formatted"], str)


@pytest.mark.asyncio
async def test_sense_raw_has_native_graph_shape() -> None:
    result = await sense(_state())
    raw = result["snapshot"]["raw"]
    assert "nodes" in raw
    assert "edges" in raw
    assert isinstance(raw["nodes"], list)
    assert isinstance(raw["edges"], list)
    # Dummy graph should expose at least one host node.
    kinds = {n["kind"]["type"] for n in raw["nodes"] if isinstance(n.get("kind"), dict)}
    assert "host" in kinds


@pytest.mark.asyncio
async def test_sense_formatted_includes_summary_header() -> None:
    result = await sense(_state())
    formatted = result["snapshot"]["formatted"]
    assert "Graph snapshot" in formatted
    assert "- nodes:" in formatted
    assert "- edges:" in formatted


@pytest.mark.asyncio
async def test_sense_is_structurally_stable() -> None:
    """Two consecutive calls produce snapshots with the same shape.

    IDs differ (ULIDs are time/random), so we compare counts and the set
    of node-kind types instead.
    """
    a = await sense(_state())
    b = await sense(_state())
    raw_a, raw_b = a["snapshot"]["raw"], b["snapshot"]["raw"]
    assert len(raw_a["nodes"]) == len(raw_b["nodes"])
    assert len(raw_a["edges"]) == len(raw_b["edges"])

    def kinds(raw: dict) -> set[str]:
        return {n["kind"]["type"] for n in raw["nodes"] if isinstance(n.get("kind"), dict)}

    assert kinds(raw_a) == kinds(raw_b)
