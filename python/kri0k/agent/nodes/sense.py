"""Sense node for LangGraph agent.

The sense node observes the current state by fetching a snapshot from
the Rust core graph and formatting it for downstream LLM consumption.
It is the first node in each engagement loop iteration.

Phase 4: prefers ``engagement.snapshot()`` when an Engagement instance is
present in ``engagement_context``; falls back to the Phase 1/2
``_native.get_dummy_graph()`` for backward-compat with legacy tests.
"""

from typing import Any

from kri0k import _native
from kri0k.agent.state import AgentState
from kri0k.llm.formatters import format_snapshot_hybrid


async def sense(state: AgentState) -> dict[str, Any]:
    """Sense node: fetch and format the current graph snapshot.

    Phase 4: prefers ``engagement.snapshot()`` when an Engagement is present
    in context; falls back to ``_native.get_dummy_graph()`` to preserve
    Phase 1/2 test compatibility.

    Args:
        state: Current agent state. If ``engagement_context["engagement"]``
            is present, the snapshot is sourced from it. Otherwise the
            Rust dummy graph fallback is used.

    Returns:
        State update with the new snapshot. The value is a dict with two keys:

        * ``raw``: the unmodified dict from snapshot source.
        * ``formatted``: a string ready to be injected into a prompt
          (sanitized via M-16 in ``format_snapshot_hybrid``).
    """
    engagement = state.get("engagement_context", {}).get("engagement")
    raw = engagement.snapshot() if engagement is not None else _native.get_dummy_graph()
    formatted = format_snapshot_hybrid(raw)
    return {"snapshot": {"raw": raw, "formatted": formatted}}
