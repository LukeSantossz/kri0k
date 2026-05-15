"""LLM provider configuration.

Single source of truth for provider settings. Per D-19, only the `model`
field is overridable via `scope.yaml::llm.model`. All other fields are
locked to their defaults in Phase 2 (no env-var fallback, no CLI flag).
"""

from collections.abc import Mapping
from dataclasses import dataclass
from typing import Any, Literal

# Only this key may appear under `llm:` in scope.yaml (D-19, strict mode).
_ALLOWED_SCOPE_KEYS = frozenset({"model"})


@dataclass(frozen=True, slots=True)
class LLMConfig:
    """Immutable LLM provider configuration.

    Attributes:
        provider: Provider identifier. Currently only "ollama" is supported.
        model: Ollama model tag. Default `deepseek-r1:32b` per D-18; emits
            `<think>...</think>` tags handled in Phase 3.
        base_url: Ollama HTTP endpoint.
        timeout_s: Per-request timeout in seconds.
        temperature: Sampling temperature.
        max_tokens: Optional cap on generated tokens (None = provider default).
    """

    provider: Literal["ollama"] = "ollama"
    model: str = "deepseek-r1:32b"
    base_url: str = "http://localhost:11434"
    timeout_s: float = 30.0
    temperature: float = 0.2
    max_tokens: int | None = None

    @classmethod
    def from_scope_dict(cls, scope: Mapping[str, Any]) -> "LLMConfig":
        """Build config from a parsed scope.yaml mapping.

        Reads the optional `llm` block. Only `model` is accepted; any other
        key triggers `ValueError` (strict mode) per the user's choice.

        Args:
            scope: Parsed scope.yaml top-level mapping.

        Returns:
            LLMConfig with defaults overridden by `scope["llm"]["model"]` if
            present.

        Raises:
            ValueError: If `scope["llm"]` contains an unknown key.
        """
        llm_block = scope.get("llm") or {}
        if not isinstance(llm_block, Mapping):
            raise ValueError(f"scope.yaml::llm must be a mapping, got {type(llm_block).__name__}")

        unknown = set(llm_block.keys()) - _ALLOWED_SCOPE_KEYS
        if unknown:
            # Pick a deterministic key for the message.
            key = sorted(unknown)[0]
            raise ValueError(f"Unknown llm config key: {key}")

        model = llm_block.get("model")
        if model is None:
            return cls()
        if not isinstance(model, str):
            raise ValueError(f"scope.yaml::llm.model must be a string, got {type(model).__name__}")
        return cls(model=model)
