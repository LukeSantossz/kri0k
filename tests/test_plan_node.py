"""Tests for the plan node with mocked LLM."""

import pytest

from kri0k.agent.nodes.plan import plan
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
    analysis: dict | None = None,
) -> AgentState:
    """Create a minimal AgentState for testing."""
    return AgentState(
        snapshot={"raw": {}, "formatted": "Test snapshot"},
        analysis=analysis or {},
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
async def test_plan_returns_proposal_dict() -> None:
    """Plan node returns proposal as dict with expected keys."""
    mock_response = """{
        "target": "test.example.com",
        "ttp_id": "T1590.001",
        "params": {"verbose": true},
        "rationale": "Need domain info"
    }"""
    provider = MockLLMProvider(mock_response)
    state = _make_state(llm_provider=provider)

    result = await plan(state)

    assert "proposal" in result
    proposal = result["proposal"]
    assert proposal["target"] == "test.example.com"
    assert proposal["ttp_id"] == "T1590.001"
    assert proposal["params"] == {"verbose": True}
    assert proposal["rationale"] == "Need domain info"


@pytest.mark.asyncio
@pytest.mark.unit
async def test_plan_without_provider_returns_empty_proposal() -> None:
    """Plan node without LLM provider returns graceful empty proposal."""
    state = _make_state(llm_provider=None)

    result = await plan(state)

    assert "proposal" in result
    proposal = result["proposal"]
    assert proposal["target"] == ""
    assert proposal["ttp_id"] == "NONE"
    assert "Cannot plan without LLM provider" in proposal["rationale"]


@pytest.mark.asyncio
@pytest.mark.unit
async def test_plan_includes_analysis_in_prompt() -> None:
    """Plan node includes analysis summary in LLM prompt."""
    mock_response = """{
        "target": "example.com",
        "ttp_id": "T1590.001",
        "params": {},
        "rationale": "test"
    }"""
    provider = MockLLMProvider(mock_response)
    state = _make_state(
        llm_provider=provider,
        analysis={
            "observations": ["Custom observation"],
            "gaps": ["Custom gap"],
            "priority_targets": ["priority.example.com"],
            "reasoning": "Custom reasoning",
        },
    )

    await plan(state)

    assert provider.last_prompt is not None
    assert "Custom observation" in provider.last_prompt
    assert "Custom gap" in provider.last_prompt
    assert "priority.example.com" in provider.last_prompt


@pytest.mark.asyncio
@pytest.mark.unit
async def test_plan_handles_think_tags() -> None:
    """Plan node correctly parses response with think tags."""
    mock_response = """<think>
I should propose a whois lookup.
</think>
{
    "target": "example.com",
    "ttp_id": "T1590.001",
    "params": {},
    "rationale": "Whois lookup needed"
}"""
    provider = MockLLMProvider(mock_response)
    state = _make_state(llm_provider=provider)

    result = await plan(state)

    proposal = result["proposal"]
    assert proposal["target"] == "example.com"
    assert proposal["ttp_id"] == "T1590.001"


@pytest.mark.asyncio
@pytest.mark.unit
async def test_plan_handles_none_proposal() -> None:
    """Plan node handles 'no action' proposal correctly."""
    mock_response = """{
        "target": "",
        "ttp_id": "NONE",
        "params": {},
        "rationale": "Objective already achieved"
    }"""
    provider = MockLLMProvider(mock_response)
    state = _make_state(llm_provider=provider)

    result = await plan(state)

    proposal = result["proposal"]
    assert proposal["target"] == ""
    assert proposal["ttp_id"] == "NONE"
