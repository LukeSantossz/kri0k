"""Sense node for LangGraph agent.

The sense node observes the current state by fetching a snapshot from
the Rust core graph and formatting it for downstream LLM consumption.
It is the first node in each engagement loop iteration.

Phase 2 scope (D-23): sense is **pure** — it reads from `_native` and
writes the formatted snapshot to `AgentState.snapshot`. It does **not**
call the LLM; the sense→reason wiring lands in Phase 3.
"""

from typing import Any

from kri0k import _native
from kri0k.agent.state import AgentState
from kri0k.llm.formatters import format_snapshot_hybrid


async def sense(state: AgentState) -> dict[str, Any]:  # noqa: ARG001
    """Sense node: fetch and format the current graph snapshot.

    Args:
        state: Current agent state (unused — sense is read-only on Rust).

    Returns:
        State update with the new snapshot. Per D-21 the value is a dict
        with two keys:

        * ``raw``: the unmodified dict returned by `_native.get_dummy_graph()`.
        * ``formatted``: a string ready to be injected into a prompt
          (sanitized via M-16 in `format_snapshot_hybrid`).
    """
    raw = _native.get_dummy_graph()
    formatted = format_snapshot_hybrid(raw)
    return {"snapshot": {"raw": raw, "formatted": formatted}}
