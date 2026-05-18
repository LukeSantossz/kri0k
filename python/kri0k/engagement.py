"""Engagement bootstrap helper.

Creates an Engagement instance and returns the engagement_context dict
suitable for injection into AgentState before ``graph.invoke()``.
"""

from typing import Any

from kri0k import _native


def create(
    scope_dict: dict[str, Any],
    objective: str,
    propose_only: bool = True,
) -> dict[str, Any]:
    """Create an Engagement and return the engagement_context dict.

    Args:
        scope_dict: Parsed scope.yaml as dict (must contain version=1, targets).
        objective: Engagement objective (free-form string).
        propose_only: If True (default; D-49), act node returns proposals
            without invoking Rust execution. Set False for full execution.

    Returns:
        Dict suitable as AgentState["engagement_context"] with keys:
        engagement, objective, propose_only, scope_hash.

    Raises:
        RuntimeError: If whois binary not found (propagated from Rust per D-50).
    """
    engagement = _native.Engagement(scope_dict)
    return {
        "engagement": engagement,
        "objective": objective,
        "propose_only": propose_only,
        "scope_hash": engagement.scope_hash(),
    }
