"""Tests for kri0k.llm.build_provider factory and public exports."""

import asyncio
import inspect

import pytest

import kri0k.llm as kllm
from kri0k.llm import LLMConfig, OllamaProvider, build_provider

pytestmark = pytest.mark.unit


def test_factory_returns_ollama_provider() -> None:
    provider = build_provider(LLMConfig())
    try:
        assert isinstance(provider, OllamaProvider)
    finally:
        asyncio.run(provider.aclose())


def test_factory_rejects_unknown_provider() -> None:
    cfg = LLMConfig()
    # Bypass frozen via object.__setattr__ to fabricate an invalid value.
    object.__setattr__(cfg, "provider", "bogus")
    with pytest.raises(ValueError, match="Unknown provider"):
        build_provider(cfg)


def test_chat_is_coroutine_function() -> None:
    provider = build_provider(LLMConfig())
    try:
        assert inspect.iscoroutinefunction(provider.chat)
    finally:
        asyncio.run(provider.aclose())


def test_public_exports() -> None:
    expected = {
        "LLMConfig",
        "LLMError",
        "LLMProvider",
        "LLMResponseError",
        "LLMRetryExhaustedError",
        "OllamaProvider",
        "PingResult",
        "build_provider",
        "format_snapshot_hybrid",
        "ping_ollama",
    }
    assert set(kllm.__all__) == expected
    for name in expected:
        assert hasattr(kllm, name), f"Missing export: {name}"
