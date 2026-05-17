"""Reason node for LangGraph agent.

The reason node analyzes observations from the sense node,
identifies patterns, and prepares context for planning.

Phase 3: Calls LLM to analyze current state and produce structured analysis.
"""

from dataclasses import asdict
from typing import TYPE_CHECKING, Any

from kri0k.agent.state import AgentState
from kri0k.llm import parse_analysis
from kri0k.llm.templates import render

if TYPE_CHECKING:
    from kri0k.llm.protocol import LLMProvider


def _format_history(history: list[dict[str, Any]]) -> str:
    """Format history list into summary string for prompt."""
    if not history:
        return "(No previous iterations)"

    lines = []
    for i, entry in enumerate(history[-5:], 1):  # Last 5 entries
        summary = entry.get("summary", "No summary")
        lines.append(f"- Iteration {i}: {summary}")
    return "\n".join(lines)


async def reason(state: AgentState) -> dict[str, Any]:
    """Reason node: analyze observations and identify patterns.

    Calls the LLM with the current snapshot and context to produce
    a structured analysis (observations, gaps, priority_targets).

    Args:
        state: Current agent state with snapshot from sense node.

    Returns:
        State updates with 'analysis' key containing Analysis as dict.
    """
    # Get LLM provider from engagement context
    engagement_ctx = state.get("engagement_context", {})
    llm_provider: LLMProvider | None = engagement_ctx.get("llm_provider")

    if llm_provider is None:
        # No provider configured - return empty analysis
        return {
            "analysis": {
                "observations": [],
                "gaps": ["LLM provider not configured"],
                "priority_targets": [],
                "reasoning": "Cannot analyze without LLM provider",
            }
        }

    # Extract context for prompt
    snapshot = state.get("snapshot", {})
    formatted_snapshot = snapshot.get("formatted", "(No snapshot available)")
    scope = engagement_ctx.get("scope", "(No scope defined)")
    objective = engagement_ctx.get("objective", "(No objective defined)")
    iteration_count = state.get("iteration_count", 0)
    history = state.get("history", [])

    # Render prompt
    prompt = render(
        "reason.jinja2",
        formatted_snapshot=formatted_snapshot,
        scope=scope,
        objective=objective,
        iteration_count=iteration_count,
        history_summary=_format_history(history),
    )

    # Call LLM
    response = await llm_provider.chat(prompt=prompt)

    # Parse response into Analysis
    analysis = parse_analysis(response)

    return {"analysis": asdict(analysis)}
