"""Tests for kri0k.llm.healthcheck.ping_ollama."""

import pytest

from kri0k.llm.config import LLMConfig
from kri0k.llm.healthcheck import ping_ollama
from kri0k.llm.ollama import LLMRetryExhaustedError


class _StubProvider:
    """Minimal LLMProvider for tests; honors the ping contract."""

    def __init__(self, *, response: str | None = None, error: Exception | None = None) -> None:
        self._response = response
        self._error = error
        self.calls: list[str] = []
        self.closed = False

    async def chat(self, *, prompt: str, system: str | None = None) -> str:  # noqa: ARG002
        self.calls.append(prompt)
        if self._error is not None:
            raise self._error
        assert self._response is not None
        return self._response

    async def aclose(self) -> None:
        self.closed = True


@pytest.mark.asyncio
async def test_success_returns_ok_result() -> None:
    provider = _StubProvider(response="pong")
    result = await ping_ollama(LLMConfig(), provider=provider)
    assert result.ok is True
    assert result.model == "deepseek-r1:32b"
    assert result.response_excerpt == "pong"
    assert result.error is None
    assert result.latency_ms >= 0
    assert len(provider.calls) == 1
    # Caller-injected provider must NOT be closed.
    assert provider.closed is False


@pytest.mark.asyncio
async def test_failure_captured_in_error_field() -> None:
    provider = _StubProvider(error=LLMRetryExhaustedError("dead"))
    result = await ping_ollama(LLMConfig(), provider=provider)
    assert result.ok is False
    assert result.response_excerpt == ""
    assert result.error is not None
    assert "LLMRetryExhaustedError" in result.error
    assert "dead" in result.error


@pytest.mark.asyncio
async def test_excerpt_truncated_to_200_chars() -> None:
    long_text = "x" * 1000
    provider = _StubProvider(response=long_text)
    result = await ping_ollama(LLMConfig(), provider=provider)
    assert len(result.response_excerpt) == 200
    assert result.response_excerpt == "x" * 200
