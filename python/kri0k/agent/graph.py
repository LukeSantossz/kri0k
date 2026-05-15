"""LangGraph agent graph builder.

This module provides the StateGraph definition for the kri0k agent.
The graph connects the five engagement loop nodes in sequence with
conditional routing for iteration control.
"""

from typing import Any

from langgraph.graph import END, START, StateGraph

from kri0k.agent.nodes import act, plan, reason, reflect, sense
from kri0k.agent.state import AgentState

MAX_ITERATIONS: int = 10


def route_after_reflect(state: AgentState) -> str:
    """Route after reflect node: continue loop or end.

    Determines whether to continue the engagement loop or terminate.
    Currently routes based solely on iteration count; future phases
    will add goal completion and blocking conditions.

    Args:
        state: Current agent state with iteration_count.

    Returns:
        "sense" to continue loop, or END to terminate.
    """
    if state["iteration_count"] >= MAX_ITERATIONS:
        return END
    return "sense"


def get_graph() -> Any:
    """Build and compile the agent graph.

    Creates a StateGraph with five nodes (sense, reason, plan, act, reflect)
    connected in sequence. The reflect node routes conditionally based on
    iteration count.

    Returns:
        Compiled StateGraph ready for invocation.
    """
    graph: StateGraph[AgentState] = StateGraph(AgentState)

    # Add nodes
    graph.add_node("sense", sense)
    graph.add_node("reason", reason)
    graph.add_node("plan", plan)
    graph.add_node("act", act)
    graph.add_node("reflect", reflect)

    # Add edges: linear sequence through the engagement loop
    graph.add_edge(START, "sense")
    graph.add_edge("sense", "reason")
    graph.add_edge("reason", "plan")
    graph.add_edge("plan", "act")
    graph.add_edge("act", "reflect")

    # Conditional routing from reflect: continue or end
    graph.add_conditional_edges("reflect", route_after_reflect)

    return graph.compile()
