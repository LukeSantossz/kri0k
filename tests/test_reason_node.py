"""Tests for the reason node with mocked LLM."""

import pytest

from kri0k.agent.nodes.reason import reason
from kri0k.agent.state import AgentState


class MockLLMProvider:
    """Mock LLM provider for testing."""

    def __init__(self, response: str) -> None:
        self.response = response
        self.last_prompt: str | None = None

    async def chat(self, *, prompt: str, system: str | None = None) -> str:  # noqa: ARG002
        self.last_prompt = prompt
        return self.response


def _make_state(
    *,
    llm_provider: MockLLMProvider | None = None,
    snapshot: dict | None = None,
) -> AgentState:
    """Create a minimal AgentState for testing."""
    return AgentState(
        snapshot=snapshot or {"raw": {}, "formatted": "Test snapshot"},
        analysis={},
        proposal={},
        decision={},
        iteration_count=1,
        history=[],
        engagement_context={
            "llm_provider": llm_provider,
            "scope": "*.example.com",
            "objective": "Test objective",
        },
    )


@pytest.mark.asyncio
@pytest.mark.unit
async def test_reason_returns_analysis_dict() -> None:
    """Reason node returns analysis as dict with expected keys."""
    mock_response = """{
        "observations": ["Found 2 hosts"],
        "gaps": ["Need port scan"],
        "priority_targets": ["host1.example.com"],
        "reasoning": "Based on network discovery"
    }"""
    provider = MockLLMProvider(mock_response)
    state = _make_state(llm_provider=provider)

    result = await reason(state)

    assert "analysis" in result
    analysis = result["analysis"]
    assert analysis["observations"] == ["Found 2 hosts"]
    assert analysis["gaps"] == ["Need port scan"]
    assert analysis["priority_targets"] == ["host1.example.com"]
    assert analysis["reasoning"] == "Based on network discovery"


@pytest.mark.asyncio
@pytest.mark.unit
async def test_reason_without_provider_returns_empty_analysis() -> None:
    """Reason node without LLM provider returns graceful empty analysis."""
    state = _make_state(llm_provider=None)

    result = await reason(state)

    assert "analysis" in result
    analysis = result["analysis"]
    assert analysis["observations"] == []
    assert "LLM provider not configured" in analysis["gaps"]


@pytest.mark.asyncio
@pytest.mark.unit
async def test_reason_includes_snapshot_in_prompt() -> None:
    """Reason node includes formatted snapshot in LLM prompt."""
    mock_response = """{
        "observations": [],
        "gaps": [],
        "priority_targets": [],
        "reasoning": "test"
    }"""
    provider = MockLLMProvider(mock_response)
    state = _make_state(
        llm_provider=provider,
        snapshot={"raw": {}, "formatted": "CUSTOM_SNAPSHOT_CONTENT"},
    )

    await reason(state)

    assert provider.last_prompt is not None
    assert "CUSTOM_SNAPSHOT_CONTENT" in provider.last_prompt


@pytest.mark.asyncio
@pytest.mark.unit
async def test_reason_handles_think_tags() -> None:
    """Reason node correctly parses response with think tags."""
    mock_response = """<think>
Let me analyze this carefully...
The network has 2 hosts.
</think>
{
    "observations": ["2 hosts discovered"],
    "gaps": [],
    "priority_targets": [],
    "reasoning": "Analysis complete"
}"""
    provider = MockLLMProvider(mock_response)
    state = _make_state(llm_provider=provider)

    result = await reason(state)

    analysis = result["analysis"]
    assert analysis["observations"] == ["2 hosts discovered"]
