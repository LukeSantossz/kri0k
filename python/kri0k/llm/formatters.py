"""Snapshot formatters for LLM consumption.

Implements the hybrid format chosen in D-22: a short textual summary
(node-kind histogram) followed by a JSON code-fence with the sanitized
graph payload.

Sanitization implements threat-model mitigation **M-16**: snapshot data
that originates from external sources (banners, headers, response bodies,
free-text metadata) must not be able to break out of the prompt boundary,
inject control characters, or smuggle hidden instructions through fence
tokens.
"""

from collections.abc import Mapping, Sequence
import json
from typing import Any

# --- Sanitization knobs -----------------------------------------------------

#: Maximum byte length per leaf string after UTF-8 encoding. Anything longer
#: is truncated and tagged with an ellipsis marker. 1 KiB is enough for
#: banners and short headers without flooding the prompt window.
MAX_FIELD_BYTES = 1024

#: Field names whose values are most likely to carry attacker-controlled
#: text. They are still emitted, but always go through the full sanitization
#: pipeline regardless of nesting depth.
SENSITIVE_FIELD_NAMES: frozenset[str] = frozenset({"banner", "headers", "body", "raw_response"})

#: Tokens that could close the JSON fence or inject role markers used by
#: chat templates (Qwen, ChatML, Llama). Any occurrence in a leaf string is
#: neutralized by inserting a zero-width space.
PROMPT_FENCE_TOKENS: tuple[str, ...] = (
    "```",
    "<|im_start|>",
    "<|im_end|>",
    "<|system|>",
    "<|user|>",
    "<|assistant|>",
)

_ELLIPSIS = "…[truncated]"
_ZWSP = "\u200b"  # Zero-width space used to break fence tokens.
_FIRST_PRINTABLE_CODEPOINT = 0x20  # ASCII space; everything below is C0 control.


def _strip_control_chars(value: str) -> str:
    """Remove ASCII control chars except \\n and \\t."""
    return "".join(
        ch for ch in value if ch in ("\n", "\t") or ord(ch) >= _FIRST_PRINTABLE_CODEPOINT
    )


def _strip_fence_tokens(value: str) -> str:
    """Neutralize prompt-fence and role tokens by inserting ZWSPs."""
    sanitized = value
    for token in PROMPT_FENCE_TOKENS:
        if token in sanitized:
            # Insert a ZWSP between every char so the literal token can no
            # longer match. Cheap, deterministic, and reversible by humans.
            broken = _ZWSP.join(token)
            sanitized = sanitized.replace(token, broken)
    return sanitized


def _truncate(value: str, max_bytes: int = MAX_FIELD_BYTES) -> str:
    """Truncate `value` so its UTF-8 encoding does not exceed `max_bytes`.

    Uses an iterative shrink to ensure we never split a multi-byte rune.
    Marks the result with `_ELLIPSIS` when truncation occurred.
    """
    encoded = value.encode("utf-8")
    if len(encoded) <= max_bytes:
        return value
    # Reserve room for the ellipsis suffix.
    suffix_bytes = _ELLIPSIS.encode("utf-8")
    budget = max_bytes - len(suffix_bytes)
    if budget <= 0:
        # Pathological case: cap < ellipsis. Fall back to byte slice.
        return encoded[:max_bytes].decode("utf-8", errors="ignore")
    # Decode with errors="ignore" trims any partial trailing rune.
    head = encoded[:budget].decode("utf-8", errors="ignore")
    return head + _ELLIPSIS


def _sanitize_value(value: Any, *, sensitive: bool = False) -> Any:
    """Recursively sanitize an arbitrary JSON-compatible value.

    Args:
        value: Leaf or container.
        sensitive: True when the value sits under a key that matches
            `SENSITIVE_FIELD_NAMES`. Currently used for documentation
            symmetry; sanitization is applied to all strings either way.

    Returns:
        A new value with the same structure, fully sanitized.
    """
    del sensitive  # Reserved for future per-field policies.
    if isinstance(value, str):
        return _truncate(_strip_fence_tokens(_strip_control_chars(value)))
    if isinstance(value, Mapping):
        return {
            str(k): _sanitize_value(
                v,
                sensitive=str(k) in SENSITIVE_FIELD_NAMES,
            )
            for k, v in value.items()
        }
    if isinstance(value, (list, tuple)):
        return [_sanitize_value(item) for item in value]
    # bool / int / float / None pass through unchanged. Anything exotic
    # is coerced to its repr to keep the JSON serializer happy.
    if isinstance(value, (bool, int, float)) or value is None:
        return value
    return _truncate(_strip_fence_tokens(_strip_control_chars(repr(value))))


def _node_kind_label(node: Any) -> str:
    """Pick a stable, short label for a node's kind.

    The Rust serializer emits `kind` as a tagged dict like
    ``{"type": "host", "ip": "..."}``. We use the inner ``type`` field
    when present; otherwise fall back to a plain string `kind`.
    """
    if not isinstance(node, Mapping):
        return "unknown"
    kind = node.get("kind")
    if isinstance(kind, Mapping):
        type_tag = kind.get("type")
        if isinstance(type_tag, str):
            return type_tag
        return "unknown"
    if isinstance(kind, str):
        return kind
    return "unknown"


def _node_kind_histogram(nodes: Sequence[Any]) -> list[tuple[str, int]]:
    """Return a sorted (kind, count) list for the summary header."""
    counts: dict[str, int] = {}
    for node in nodes:
        label = _node_kind_label(node)
        counts[label] = counts.get(label, 0) + 1
    return sorted(counts.items())


def format_snapshot_hybrid(raw: Mapping[str, Any]) -> str:
    """Format a Rust graph snapshot for safe injection into an LLM prompt.

    Output shape (deterministic, sort_keys, 2-space indent):

        Graph snapshot
        - nodes: N (kind_a=K, kind_b=L)
        - edges: M

        ```json
        { ...sanitized payload... }
        ```

    Args:
        raw: Snapshot dict as returned by `kri0k._native.get_dummy_graph()`.
            Expected to expose `nodes` and `edges` lists, but missing keys
            are tolerated (treated as empty).

    Returns:
        A single string with the textual summary followed by the JSON
        block. Safe to embed in a Jinja2 template or chat message.
    """
    sanitized = _sanitize_value(raw)
    assert isinstance(sanitized, dict)  # noqa: S101 - invariant after sanitize

    nodes = sanitized.get("nodes", [])
    edges = sanitized.get("edges", [])
    nodes_seq: Sequence[Any] = nodes if isinstance(nodes, list) else []
    edges_seq: Sequence[Any] = edges if isinstance(edges, list) else []

    histogram = _node_kind_histogram(nodes_seq)
    if histogram:
        kinds_str = ", ".join(f"{kind}={count}" for kind, count in histogram)
        nodes_line = f"- nodes: {len(nodes_seq)} ({kinds_str})"
    else:
        nodes_line = f"- nodes: {len(nodes_seq)}"

    summary = "\n".join(
        [
            "Graph snapshot",
            nodes_line,
            f"- edges: {len(edges_seq)}",
        ]
    )

    payload = json.dumps(sanitized, indent=2, sort_keys=True, ensure_ascii=False)
    return f"{summary}\n\n```json\n{payload}\n```"
