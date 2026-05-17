"""LLM provider package for kri0k.

Public surface (D-31, D-32). The top-level `kri0k` package intentionally
does **not** re-export these — callers import directly from `kri0k.llm`.
"""

from kri0k.llm.config import LLMConfig
from kri0k.llm.formatters import format_snapshot_hybrid
from kri0k.llm.healthcheck import PingResult, ping_ollama
from kri0k.llm.ollama import (
    LLMError,
    LLMResponseError,
    LLMRetryExhaustedError,
    OllamaProvider,
)
from kri0k.llm.parser import (
    Analysis,
    ParseError,
    Proposal,
    extract_json,
    parse_analysis,
    parse_proposal,
    strip_think_tags,
)
from kri0k.llm.protocol import LLMProvider


def build_provider(config: LLMConfig) -> LLMProvider:
    """Construct the concrete provider selected by `config.provider`.

    Phase 2 supports only the Ollama provider; the factory exists so the
    Phase 12 CLI bootstrap can wire `engagement_context["llm_provider"]`
    without importing `OllamaProvider` directly.
    """
    if config.provider == "ollama":
        return OllamaProvider(config)
    raise ValueError(f"Unknown provider: {config.provider}")


__all__ = [
    "Analysis",
    "LLMConfig",
    "LLMError",
    "LLMProvider",
    "LLMResponseError",
    "LLMRetryExhaustedError",
    "OllamaProvider",
    "ParseError",
    "PingResult",
    "Proposal",
    "build_provider",
    "extract_json",
    "format_snapshot_hybrid",
    "parse_analysis",
    "parse_proposal",
    "ping_ollama",
    "strip_think_tags",
]
