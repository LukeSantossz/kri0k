"""Act node for LangGraph agent.

Phase 4: gates execution via propose_only flag and dispatches approved
proposals to the Rust Engagement (D-49, D-56).
"""

import asyncio
from typing import Any

from kri0k.agent.state import AgentState


async def act(state: AgentState) -> dict[str, Any]:
    """Act node: execute approved proposal through Rust Engagement.

    If propose_only=True (default; D-49), returns a proposed status without
    invoking Rust. If False, dispatches to engagement.execute_proposal via
    asyncio.to_thread to avoid blocking the event loop (D-46).

    Args:
        state: Current agent state. Must contain ``proposal`` (dict or
            empty) and ``engagement_context`` with ``propose_only`` and
            optionally ``engagement``.

    Returns:
        State updates with ``decision`` (outcome dict) and ``history``
        appended with a D-56 entry. Empty dict when no proposal exists.

    Raises:
        RuntimeError: If propose_only=False but ``engagement`` is missing
            from context (programming error — engagement.create() must run
            before invoke()).
    """
    proposal = state["proposal"]
    if not proposal:
        return {}

    context = state["engagement_context"]
    propose_only = context.get("propose_only", True)
    iteration = state["iteration_count"]
    ttp_id = proposal.get("ttp_id", "")
    target = proposal.get("target", "")

    if propose_only:
        # M-05: propose-only gate — never reach Rust execution.
        history_entry: dict[str, Any] = {
            "iteration": iteration,
            "ttp_id": ttp_id,
            "target": target,
            "status": "proposed",
            "summary": f"propose-only: would_execute {ttp_id} on {target}",
            "graph_delta": {"nodes_added": 0, "edges_added": 0},
            "audit_id": None,
        }
        return {
            "decision": {
                "status": "proposed",
                "would_execute": ttp_id,
                "proposal": proposal,
            },
            "history": state["history"] + [history_entry],
        }

    engagement = context.get("engagement")
    if engagement is None:
        raise RuntimeError(
            "act: engagement missing from engagement_context (run kri0k.engagement.create first)"
        )

    # D-46: dispatch to thread pool so the Tokio runtime block_on call does
    # not stall the asyncio event loop.
    outcome = await asyncio.to_thread(engagement.execute_proposal, proposal)

    delta = outcome.get("graph_delta", {"nodes_added": 0, "edges_added": 0})
    summary = _format_summary(ttp_id, target, outcome, delta)
    history_entry = {
        "iteration": iteration,
        "ttp_id": ttp_id,
        "target": target,
        "status": outcome.get("status", "error"),
        "summary": summary,
        "graph_delta": delta,
        "audit_id": outcome.get("audit_id"),
    }
    return {
        "decision": outcome,
        "history": state["history"] + [history_entry],
    }


def _format_summary(
    ttp_id: str,
    target: str,
    outcome: dict[str, Any],
    delta: dict[str, int],
) -> str:
    """Build human-readable summary per D-56."""
    status = outcome.get("status", "error")
    if status == "executed":
        nodes = delta.get("nodes_added", 0)
        edges = delta.get("edges_added", 0)
        if nodes == 0:
            return f"{ttp_id} {target} -> no new nodes (already known)"
        return f"{ttp_id} {target} -> +{nodes} nodes +{edges} edges"
    error_msg = outcome.get("error", "")
    return f"{ttp_id} {target} -> {status}: {error_msg}".strip()
