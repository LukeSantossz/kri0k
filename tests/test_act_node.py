"""Tests for the act node (Phase 4) with mocked Engagement.

Validates AGENT-05 (act sends proposal, receives outcome), the propose_only
gate (D-49), the asyncio.to_thread dispatch (D-46), and the history entry
shape (D-56).
"""

from typing import Any

import pytest

from kri0k.agent.nodes.act import act
from kri0k.agent.state import AgentState


class MockEngagement:
    """Mock Engagement for act node tests.

    Records calls and returns a configurable execute_proposal result.
    """

    def __init__(self, execute_result: dict[str, Any] | None = None) -> None:
        self.execute_result = execute_result or {
            "status": "executed",
            "result": None,
            "error": None,
            "graph_delta": {"nodes_added": 1, "edges_added": 0},
            "audit_id": "test-audit-001",
        }
        self.last_proposal: dict[str, Any] | None = None
        self.execute_call_count: int = 0

    def snapshot(self) -> dict[str, Any]:
        return {"nodes": [], "edges": []}

    def execute_proposal(self, proposal: dict[str, Any]) -> dict[str, Any]:
        self.execute_call_count += 1
        self.last_proposal = proposal
        return self.execute_result

    def scope_hash(self) -> str:
        return "deadbeef" * 8

    def kill(self) -> None:
        pass


def _make_state(
    *,
    engagement: MockEngagement | None = None,
    propose_only: bool = True,
    proposal: dict[str, Any] | None = None,
    iteration: int = 1,
    history: list[dict[str, Any]] | None = None,
) -> AgentState:
    """Create a minimal AgentState for testing."""
    return AgentState(
        snapshot={"raw": {}, "formatted": ""},
        analysis={},
        proposal=proposal or {},
        decision={},
        iteration_count=iteration,
        history=history or [],
        engagement_context={
            "engagement": engagement,
            "propose_only": propose_only,
            "objective": "test objective",
        },
    )


@pytest.mark.asyncio
@pytest.mark.unit
async def test_act_returns_empty_when_no_proposal() -> None:
    """Empty proposal -> no-op."""
    state = _make_state(proposal={})
    result = await act(state)
    assert result == {}


@pytest.mark.asyncio
@pytest.mark.unit
async def test_act_propose_only_skips_rust() -> None:
    """propose_only=True must NOT invoke engagement.execute_proposal (D-49, M-05)."""
    engagement = MockEngagement()
    proposal = {"ttp_id": "T1590.001", "target": "example.com"}
    state = _make_state(engagement=engagement, propose_only=True, proposal=proposal)
    result = await act(state)
    assert engagement.execute_call_count == 0, "Rust must not be called in propose_only mode"
    assert result["decision"]["status"] == "proposed"
    assert result["decision"]["would_execute"] == "T1590.001"
    assert result["decision"]["proposal"] == proposal


@pytest.mark.asyncio
@pytest.mark.unit
async def test_act_propose_only_appends_history_d56() -> None:
    """History entry in propose_only must match D-56 shape."""
    engagement = MockEngagement()
    proposal = {"ttp_id": "T1590.001", "target": "example.com"}
    state = _make_state(engagement=engagement, propose_only=True, proposal=proposal, iteration=3)
    result = await act(state)
    history = result["history"]
    assert len(history) == 1
    entry = history[0]
    assert set(entry.keys()) == {
        "iteration",
        "ttp_id",
        "target",
        "status",
        "summary",
        "graph_delta",
        "audit_id",
    }
    assert entry["iteration"] == 3
    assert entry["ttp_id"] == "T1590.001"
    assert entry["target"] == "example.com"
    assert entry["status"] == "proposed"
    assert "propose-only" in entry["summary"]
    assert entry["graph_delta"] == {"nodes_added": 0, "edges_added": 0}
    assert entry["audit_id"] is None


@pytest.mark.asyncio
@pytest.mark.unit
async def test_act_calls_execute_proposal_when_not_propose_only() -> None:
    """propose_only=False dispatches to engagement.execute_proposal (AGENT-05)."""
    engagement = MockEngagement()
    proposal = {"ttp_id": "T1590.001", "target": "example.com"}
    state = _make_state(engagement=engagement, propose_only=False, proposal=proposal)
    await act(state)
    assert engagement.execute_call_count == 1
    assert engagement.last_proposal == proposal


@pytest.mark.asyncio
@pytest.mark.unit
async def test_act_outcome_appended_to_history() -> None:
    """Executed outcome populates history entry per D-56."""
    engagement = MockEngagement(
        execute_result={
            "status": "executed",
            "result": {"registrant": "Example Inc"},
            "error": None,
            "graph_delta": {"nodes_added": 3, "edges_added": 2},
            "audit_id": "audit-xyz",
        }
    )
    proposal = {"ttp_id": "T1590.001", "target": "example.com"}
    state = _make_state(engagement=engagement, propose_only=False, proposal=proposal)
    result = await act(state)
    entry = result["history"][0]
    assert entry["status"] == "executed"
    assert entry["graph_delta"] == {"nodes_added": 3, "edges_added": 2}
    assert entry["audit_id"] == "audit-xyz"
    assert "+3 nodes" in entry["summary"]


@pytest.mark.asyncio
@pytest.mark.unit
async def test_act_raises_when_engagement_missing() -> None:
    """propose_only=False AND engagement is None -> RuntimeError (no silent failure)."""
    state = _make_state(
        engagement=None,
        propose_only=False,
        proposal={"ttp_id": "T1590.001", "target": "example.com"},
    )
    with pytest.raises(RuntimeError, match="engagement missing"):
        await act(state)


@pytest.mark.asyncio
@pytest.mark.unit
async def test_act_summary_idempotent_format() -> None:
    """nodes_added=0 produces 'no new nodes (already known)' (D-56)."""
    engagement = MockEngagement(
        execute_result={
            "status": "executed",
            "result": None,
            "error": None,
            "graph_delta": {"nodes_added": 0, "edges_added": 0},
            "audit_id": "audit-idem",
        }
    )
    proposal = {"ttp_id": "T1590.001", "target": "example.com"}
    state = _make_state(engagement=engagement, propose_only=False, proposal=proposal)
    result = await act(state)
    assert "no new nodes (already known)" in result["history"][0]["summary"]


@pytest.mark.asyncio
@pytest.mark.unit
async def test_act_summary_error_format() -> None:
    """Error status produces error string in summary."""
    engagement = MockEngagement(
        execute_result={
            "status": "scope_violation",
            "result": None,
            "error": "target not in allowlist",
            "graph_delta": {"nodes_added": 0, "edges_added": 0},
            "audit_id": None,
        }
    )
    proposal = {"ttp_id": "T1590.001", "target": "evil.com"}
    state = _make_state(engagement=engagement, propose_only=False, proposal=proposal)
    result = await act(state)
    summary = result["history"][0]["summary"]
    assert "scope_violation" in summary
    assert "target not in allowlist" in summary
