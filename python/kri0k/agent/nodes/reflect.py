"""Reflect node for LangGraph agent.

The reflect node evaluates results from the act node, updates
history, and increments the iteration counter for loop control.
"""

from typing import Any

from kri0k.agent.state import AgentState


async def reflect(state: AgentState) -> dict[str, Any]:
    """Reflect node: evaluate results and update iteration count.

    Placeholder implementation increments iteration_count only.

    Args:
        state: Current agent state.

    Returns:
        State updates with incremented iteration_count.
    """
    return {"iteration_count": state["iteration_count"] + 1}
