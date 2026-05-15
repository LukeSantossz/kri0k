"""Sense node for LangGraph agent.

The sense node observes the current state by fetching a snapshot
from the Rust core graph. This is the first node in each iteration.
"""

from typing import Any

from kri0k.agent.state import AgentState


async def sense(state: AgentState) -> dict[str, Any]:  # noqa: ARG001
    """Sense node: observe current state from Rust snapshot.

    Placeholder implementation returns empty dict (no-op).

    Args:
        state: Current agent state.

    Returns:
        State updates (empty for placeholder).
    """
    return {}
