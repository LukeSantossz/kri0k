"""Tests for kri0k.llm.formatters (M-16 snapshot sanitization)."""

import json

import pytest

from kri0k.llm.formatters import (
    MAX_FIELD_BYTES,
    PROMPT_FENCE_TOKENS,
    format_snapshot_hybrid,
)

pytestmark = pytest.mark.unit


def _extract_json_block(rendered: str) -> dict:
    start = rendered.index("```json\n") + len("```json\n")
    end = rendered.rindex("\n```")
    return json.loads(rendered[start:end])


def test_empty_snapshot_renders_summary_and_empty_payload() -> None:
    out = format_snapshot_hybrid({})
    assert "Graph snapshot" in out
    assert "- nodes: 0" in out
    assert "- edges: 0" in out
    assert _extract_json_block(out) == {}


def test_node_kind_histogram_in_summary() -> None:
    snapshot = {
        "nodes": [
            {"id": "n1", "kind": "host"},
            {"id": "n2", "kind": "host"},
            {"id": "n3", "kind": "service"},
        ],
        "edges": [{"id": "e1", "src": "n1", "dst": "n2", "kind": "link"}],
    }
    out = format_snapshot_hybrid(snapshot)
    assert "- nodes: 3 (host=2, service=1)" in out
    assert "- edges: 1" in out


def test_control_chars_are_stripped() -> None:
    snapshot = {"nodes": [{"id": "n1", "kind": "host", "banner": "OK\x07\x08bad\x1f"}]}
    out = format_snapshot_hybrid(snapshot)
    payload = _extract_json_block(out)
    banner = payload["nodes"][0]["banner"]
    assert "\x07" not in banner
    assert "\x08" not in banner
    assert "\x1f" not in banner
    assert "OK" in banner
    assert "bad" in banner


def test_newline_and_tab_are_preserved() -> None:
    snapshot = {"nodes": [{"id": "n1", "kind": "host", "banner": "line1\nline2\tcol"}]}
    out = format_snapshot_hybrid(snapshot)
    payload = _extract_json_block(out)
    assert payload["nodes"][0]["banner"] == "line1\nline2\tcol"


def test_truncation_above_max_field_bytes() -> None:
    long = "A" * (MAX_FIELD_BYTES * 2)
    snapshot = {"nodes": [{"id": "n1", "kind": "host", "banner": long}]}
    out = format_snapshot_hybrid(snapshot)
    payload = _extract_json_block(out)
    sanitized = payload["nodes"][0]["banner"]
    assert sanitized.endswith("…[truncated]")
    assert len(sanitized.encode("utf-8")) <= MAX_FIELD_BYTES


def test_truncation_handles_multibyte_boundary() -> None:
    # Each emoji is 4 bytes in UTF-8; pile enough to exceed cap.
    payload_value = "🎯" * (MAX_FIELD_BYTES // 2)
    snapshot = {"nodes": [{"id": "n1", "kind": "host", "banner": payload_value}]}
    out = format_snapshot_hybrid(snapshot)
    payload = _extract_json_block(out)
    sanitized = payload["nodes"][0]["banner"]
    # Must remain valid UTF-8 (no orphan surrogate / partial rune).
    sanitized.encode("utf-8").decode("utf-8")
    assert len(sanitized.encode("utf-8")) <= MAX_FIELD_BYTES


def test_fence_tokens_are_neutralized() -> None:
    for token in PROMPT_FENCE_TOKENS:
        snapshot = {"nodes": [{"id": "n1", "kind": "host", "banner": f"prefix{token}suffix"}]}
        out = format_snapshot_hybrid(snapshot)
        payload = _extract_json_block(out)
        assert token not in payload["nodes"][0]["banner"], f"Token leaked: {token!r}"


def test_nested_dicts_and_lists_are_sanitized() -> None:
    snapshot = {
        "nodes": [
            {
                "id": "n1",
                "kind": "host",
                "metadata": {
                    "tags": ["safe", "with```fence"],
                    "deep": {"banner": "x\x00y"},
                },
            }
        ]
    }
    out = format_snapshot_hybrid(snapshot)
    payload = _extract_json_block(out)
    tags = payload["nodes"][0]["metadata"]["tags"]
    assert "```" not in tags[1]
    assert "\x00" not in payload["nodes"][0]["metadata"]["deep"]["banner"]


def test_output_is_deterministic() -> None:
    snapshot = {
        "nodes": [
            {"id": "n2", "kind": "host"},
            {"id": "n1", "kind": "host"},
        ],
        "edges": [],
    }
    a = format_snapshot_hybrid(snapshot)
    b = format_snapshot_hybrid(snapshot)
    assert a == b


def test_unknown_node_kind_is_labeled() -> None:
    snapshot = {"nodes": [{"id": "x"}]}  # missing 'kind'
    out = format_snapshot_hybrid(snapshot)
    assert "unknown=1" in out
