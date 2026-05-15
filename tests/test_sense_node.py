"""Tests for the wired sense node (Phase 2)."""

import pytest

from kri0k import _native
from kri0k.agent.nodes.sense import sense
from kri0k.agent.state import AgentState


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
async def test_sense_raw_matches_native_call() -> None:
    result = await sense(_state())
    assert result["snapshot"]["raw"] == _native.get_dummy_graph()


@pytest.mark.asyncio
async def test_sense_formatted_includes_summary_header() -> None:
    result = await sense(_state())
    assert "Graph snapshot" in result["snapshot"]["formatted"]


@pytest.mark.asyncio
async def test_sense_is_idempotent() -> None:
    a = await sense(_state())
    b = await sense(_state())
    assert a == b
