"""Plan node for LangGraph agent.

The plan node generates action proposals based on analysis
from the reason node. Proposals are subject to scope validation.

Phase 3: Calls LLM to propose next action as structured Proposal.
"""

from dataclasses import asdict
from typing import TYPE_CHECKING, Any

from kri0k.agent.state import AgentState
from kri0k.llm import parse_proposal
from kri0k.llm.templates import render

if TYPE_CHECKING:
    from kri0k.llm.protocol import LLMProvider


def _format_analysis(analysis: dict[str, Any]) -> str:
    """Format analysis dict into summary string for prompt."""
    if not analysis:
        return "(No analysis available)"

    lines = []

    observations = analysis.get("observations", [])
    if observations:
        lines.append("Observations:")
        for obs in observations:
            lines.append(f"  - {obs}")

    gaps = analysis.get("gaps", [])
    if gaps:
        lines.append("Information gaps:")
        for gap in gaps:
            lines.append(f"  - {gap}")

    priority_targets = analysis.get("priority_targets", [])
    if priority_targets:
        lines.append(f"Priority targets: {', '.join(priority_targets)}")

    reasoning = analysis.get("reasoning", "")
    if reasoning:
        lines.append(f"Reasoning: {reasoning}")

    return "\n".join(lines) if lines else "(Empty analysis)"


async def plan(state: AgentState) -> dict[str, Any]:
    """Plan node: generate action proposals.

    Calls the LLM with the analysis from reason node to produce
    a structured Proposal (target, ttp_id, params, rationale).

    Args:
        state: Current agent state with analysis from reason node.

    Returns:
        State updates with 'proposal' key containing Proposal as dict.
    """
    # Get LLM provider from engagement context
    engagement_ctx = state.get("engagement_context", {})
    llm_provider: LLMProvider | None = engagement_ctx.get("llm_provider")

    if llm_provider is None:
        # No provider configured - return empty proposal
        return {
            "proposal": {
                "target": "",
                "ttp_id": "NONE",
                "params": {},
                "rationale": "Cannot plan without LLM provider",
            }
        }

    # Extract context for prompt
    analysis = state.get("analysis", {})
    scope = engagement_ctx.get("scope", "(No scope defined)")
    objective = engagement_ctx.get("objective", "(No objective defined)")
    iteration_count = state.get("iteration_count", 0)

    # Render prompt
    prompt = render(
        "plan.jinja2",
        analysis_summary=_format_analysis(analysis),
        scope=scope,
        objective=objective,
        iteration_count=iteration_count,
    )

    # Call LLM
    response = await llm_provider.chat(prompt=prompt)

    # Parse response into Proposal
    proposal = parse_proposal(response)

    return {"proposal": asdict(proposal)}
