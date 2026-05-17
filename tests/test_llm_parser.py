"""Tests for LLM response parser module."""

import pytest

from kri0k.llm import (
    Analysis,
    ParseError,
    Proposal,
    extract_json,
    parse_analysis,
    parse_proposal,
    strip_think_tags,
)


class TestStripThinkTags:
    """Tests for strip_think_tags function."""

    def test_removes_simple_think_tag(self) -> None:
        text = "<think>some reasoning</think>actual response"
        assert strip_think_tags(text) == "actual response"

    def test_removes_multiline_think_tag(self) -> None:
        text = """<think>
        line 1
        line 2
        </think>
        response here"""
        assert "response here" in strip_think_tags(text)
        assert "<think>" not in strip_think_tags(text)

    def test_removes_multiple_think_tags(self) -> None:
        text = "<think>first</think>middle<think>second</think>end"
        result = strip_think_tags(text)
        assert result == "middleend"

    def test_case_insensitive(self) -> None:
        text = "<THINK>reasoning</THINK>response"
        assert strip_think_tags(text) == "response"

    def test_preserves_text_without_tags(self) -> None:
        text = "just regular text"
        assert strip_think_tags(text) == "just regular text"

    def test_strips_whitespace(self) -> None:
        text = "  <think>x</think>  response  "
        assert strip_think_tags(text) == "response"


class TestExtractJson:
    """Tests for extract_json function."""

    def test_extracts_from_markdown_json_block(self) -> None:
        text = """Here is the response:
```json
{"key": "value"}
```
Done."""
        result = extract_json(text)
        assert result == {"key": "value"}

    def test_extracts_from_plain_code_block(self) -> None:
        text = """```
{"key": "value"}
```"""
        result = extract_json(text)
        assert result == {"key": "value"}

    def test_extracts_raw_json_object(self) -> None:
        text = 'Some text {"key": "value"} more text'
        result = extract_json(text)
        assert result == {"key": "value"}

    def test_strips_think_tags_before_extraction(self) -> None:
        text = '<think>reasoning</think>{"key": "value"}'
        result = extract_json(text)
        assert result == {"key": "value"}

    def test_raises_on_no_json(self) -> None:
        text = "no json here at all"
        with pytest.raises(ParseError, match="No JSON object found"):
            extract_json(text)

    def test_raises_on_invalid_json(self) -> None:
        text = '{"key": invalid}'
        with pytest.raises(ParseError, match="Invalid JSON"):
            extract_json(text)


class TestProposal:
    """Tests for Proposal dataclass."""

    def test_from_dict_valid(self) -> None:
        data = {
            "target": "example.com",
            "ttp_id": "T1590.001",
            "params": {"verbose": True},
            "rationale": "Testing",
        }
        proposal = Proposal.from_dict(data)
        assert proposal.target == "example.com"
        assert proposal.ttp_id == "T1590.001"
        assert proposal.params == {"verbose": True}
        assert proposal.rationale == "Testing"

    def test_from_dict_missing_field(self) -> None:
        data = {"target": "example.com", "ttp_id": "T1590.001"}
        with pytest.raises(ParseError, match="Missing required fields"):
            Proposal.from_dict(data)

    def test_from_dict_wrong_type_target(self) -> None:
        data = {
            "target": 123,
            "ttp_id": "T1590.001",
            "params": {},
            "rationale": "Test",
        }
        with pytest.raises(ParseError, match="target must be str"):
            Proposal.from_dict(data)

    def test_from_dict_wrong_type_params(self) -> None:
        data = {
            "target": "example.com",
            "ttp_id": "T1590.001",
            "params": "not a dict",
            "rationale": "Test",
        }
        with pytest.raises(ParseError, match="params must be dict"):
            Proposal.from_dict(data)

    def test_is_frozen(self) -> None:
        proposal = Proposal(
            target="example.com",
            ttp_id="T1590.001",
            params={},
            rationale="Test",
        )
        with pytest.raises(AttributeError):
            proposal.target = "other.com"  # type: ignore[misc]


class TestAnalysis:
    """Tests for Analysis dataclass."""

    def test_from_dict_valid(self) -> None:
        data = {
            "observations": ["obs1", "obs2"],
            "gaps": ["gap1"],
            "priority_targets": ["target1"],
            "reasoning": "Because reasons",
        }
        analysis = Analysis.from_dict(data)
        assert analysis.observations == ["obs1", "obs2"]
        assert analysis.gaps == ["gap1"]
        assert analysis.priority_targets == ["target1"]
        assert analysis.reasoning == "Because reasons"

    def test_from_dict_missing_field(self) -> None:
        data = {"observations": ["obs1"]}
        with pytest.raises(ParseError, match="Missing required fields"):
            Analysis.from_dict(data)

    def test_from_dict_wrong_type_observations(self) -> None:
        data = {
            "observations": "not a list",
            "gaps": [],
            "priority_targets": [],
            "reasoning": "Test",
        }
        with pytest.raises(ParseError, match="observations must be list"):
            Analysis.from_dict(data)

    def test_converts_list_items_to_str(self) -> None:
        data = {
            "observations": [1, 2, 3],
            "gaps": [True],
            "priority_targets": [None],
            "reasoning": "Test",
        }
        analysis = Analysis.from_dict(data)
        assert analysis.observations == ["1", "2", "3"]
        assert analysis.gaps == ["True"]
        assert analysis.priority_targets == ["None"]


class TestParseAnalysis:
    """Tests for parse_analysis function."""

    def test_parses_valid_response(self) -> None:
        text = """```json
{
    "observations": ["obs1"],
    "gaps": ["gap1"],
    "priority_targets": ["target1"],
    "reasoning": "Test reasoning"
}
```"""
        analysis = parse_analysis(text)
        assert analysis.observations == ["obs1"]
        assert analysis.reasoning == "Test reasoning"


class TestParseProposal:
    """Tests for parse_proposal function."""

    def test_parses_valid_response(self) -> None:
        text = """```json
{
    "target": "example.com",
    "ttp_id": "T1590.001",
    "params": {},
    "rationale": "Testing"
}
```"""
        proposal = parse_proposal(text)
        assert proposal.target == "example.com"
        assert proposal.ttp_id == "T1590.001"

    def test_handles_think_tags(self) -> None:
        text = """<think>
Let me think about this...
</think>
{"target": "example.com", "ttp_id": "T1590.001", "params": {}, "rationale": "Test"}"""
        proposal = parse_proposal(text)
        assert proposal.target == "example.com"
