"""Plan node for LangGraph agent.

The plan node generates action proposals based on analysis
from the reason node. Proposals are subject to scope validation.
"""

from typing import Any

from kri0k.agent.state import AgentState


async def plan(state: AgentState) -> dict[str, Any]:  # noqa: ARG001
    """Plan node: generate action proposals.

    Placeholder implementation returns empty dict (no-op).

    Args:
        state: Current agent state.

    Returns:
        State updates (empty for placeholder).
    """
    return {}
