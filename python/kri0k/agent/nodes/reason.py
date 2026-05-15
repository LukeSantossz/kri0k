"""Reason node for LangGraph agent.

The reason node analyzes observations from the sense node,
identifies patterns, and prepares context for planning.
"""

from typing import Any

from kri0k.agent.state import AgentState


async def reason(state: AgentState) -> dict[str, Any]:  # noqa: ARG001
    """Reason node: analyze observations and identify patterns.

    Placeholder implementation returns empty dict (no-op).

    Args:
        state: Current agent state.

    Returns:
        State updates (empty for placeholder).
    """
    return {}
