"""Tests for kri0k.llm.ollama.OllamaProvider using httpx.MockTransport."""

import asyncio
import json

import httpx
import pytest

from kri0k.llm.config import LLMConfig
from kri0k.llm.ollama import (
    RETRY_MAX_ATTEMPTS,
    LLMResponseError,
    LLMRetryExhaustedError,
    OllamaProvider,
)
from kri0k.llm.rate_limit import TokenBucket


def _ok_response(content: str = "hello") -> httpx.Response:
    body = {"message": {"role": "assistant", "content": content}}
    return httpx.Response(200, json=body)


def _make_provider(
    handler,
    *,
    config: LLMConfig | None = None,
    bucket: TokenBucket | None = None,
) -> tuple[OllamaProvider, list[httpx.Request]]:
    captured: list[httpx.Request] = []

    def wrapped(request: httpx.Request) -> httpx.Response:
        captured.append(request)
        return handler(request)

    transport = httpx.MockTransport(wrapped)
    cfg = config or LLMConfig()
    client = httpx.AsyncClient(transport=transport, base_url=cfg.base_url)
    provider = OllamaProvider(cfg, client=client, bucket=bucket)
    return provider, captured


@pytest.fixture(autouse=True)
def _no_real_sleep(monkeypatch):
    """Replace asyncio.sleep inside the provider with an immediate noop."""

    async def fake_sleep(_delay: float) -> None:
        return None

    monkeypatch.setattr("asyncio.sleep", fake_sleep)


@pytest.fixture(autouse=True)
def _deterministic_jitter(monkeypatch):
    """Force jitter to 0 for predictable assertions."""
    monkeypatch.setattr("kri0k.llm.ollama._RNG.uniform", lambda _a, _b: 0.0)


@pytest.mark.asyncio
async def test_happy_path_returns_content() -> None:
    provider, requests = _make_provider(lambda _r: _ok_response("pong"))
    try:
        out = await provider.chat(prompt="hi")
    finally:
        await provider.aclose()
    assert out == "pong"
    assert len(requests) == 1
    body = json.loads(requests[0].content)
    assert body["model"] == "deepseek-r1:32b"
    assert body["stream"] is False


@pytest.mark.asyncio
async def test_retry_on_500_then_success() -> None:
    calls = {"n": 0}

    def handler(_request: httpx.Request) -> httpx.Response:
        calls["n"] += 1
        if calls["n"] == 1:
            return httpx.Response(500, text="boom")
        return _ok_response("ok")

    provider, requests = _make_provider(handler)
    try:
        out = await provider.chat(prompt="hi")
    finally:
        await provider.aclose()
    assert out == "ok"
    assert len(requests) == 2


@pytest.mark.asyncio
async def test_retry_on_429_then_success() -> None:
    calls = {"n": 0}

    def handler(_request: httpx.Request) -> httpx.Response:
        calls["n"] += 1
        if calls["n"] == 1:
            return httpx.Response(429, text="slow down")
        return _ok_response("ok")

    provider, requests = _make_provider(handler)
    try:
        out = await provider.chat(prompt="hi")
    finally:
        await provider.aclose()
    assert out == "ok"
    assert len(requests) == 2


@pytest.mark.asyncio
async def test_retry_on_connect_error_then_success() -> None:
    calls = {"n": 0}

    def handler(request: httpx.Request) -> httpx.Response:
        calls["n"] += 1
        if calls["n"] == 1:
            raise httpx.ConnectError("refused", request=request)
        return _ok_response("ok")

    provider, _ = _make_provider(handler)
    try:
        out = await provider.chat(prompt="hi")
    finally:
        await provider.aclose()
    assert out == "ok"


@pytest.mark.asyncio
async def test_retry_on_read_timeout_then_success() -> None:
    calls = {"n": 0}

    def handler(request: httpx.Request) -> httpx.Response:
        calls["n"] += 1
        if calls["n"] == 1:
            raise httpx.ReadTimeout("slow", request=request)
        return _ok_response("ok")

    provider, _ = _make_provider(handler)
    try:
        out = await provider.chat(prompt="hi")
    finally:
        await provider.aclose()
    assert out == "ok"


@pytest.mark.asyncio
async def test_4xx_other_than_429_propagates_without_retry() -> None:
    provider, requests = _make_provider(lambda _r: httpx.Response(400, text="bad"))
    try:
        with pytest.raises(httpx.HTTPStatusError):
            await provider.chat(prompt="hi")
    finally:
        await provider.aclose()
    assert len(requests) == 1


@pytest.mark.asyncio
async def test_401_propagates_without_retry() -> None:
    provider, requests = _make_provider(lambda _r: httpx.Response(401, text="nope"))
    try:
        with pytest.raises(httpx.HTTPStatusError):
            await provider.chat(prompt="hi")
    finally:
        await provider.aclose()
    assert len(requests) == 1


@pytest.mark.asyncio
async def test_exhausted_retries_raises_typed_error() -> None:
    provider, requests = _make_provider(lambda _r: httpx.Response(500, text="boom"))
    try:
        with pytest.raises(LLMRetryExhaustedError) as exc_info:
            await provider.chat(prompt="hi")
    finally:
        await provider.aclose()
    assert len(requests) == RETRY_MAX_ATTEMPTS
    assert exc_info.value.__cause__ is not None


@pytest.mark.asyncio
async def test_malformed_json_propagates_without_retry() -> None:
    def handler(_request: httpx.Request) -> httpx.Response:
        return httpx.Response(200, content=b"not-json")

    provider, requests = _make_provider(handler)
    try:
        with pytest.raises(LLMResponseError):
            await provider.chat(prompt="hi")
    finally:
        await provider.aclose()
    assert len(requests) == 1


@pytest.mark.asyncio
async def test_missing_message_content_raises_response_error() -> None:
    def handler(_request: httpx.Request) -> httpx.Response:
        return httpx.Response(200, json={"weird": "shape"})

    provider, requests = _make_provider(handler)
    try:
        with pytest.raises(LLMResponseError):
            await provider.chat(prompt="hi")
    finally:
        await provider.aclose()
    assert len(requests) == 1


@pytest.mark.asyncio
async def test_rate_limit_blocks_when_bucket_drained(monkeypatch) -> None:
    sleeps: list[float] = []

    async def recording_sleep(delay: float) -> None:
        sleeps.append(delay)

    monkeypatch.setattr("asyncio.sleep", recording_sleep)

    bucket = TokenBucket(capacity=1, refill_per_window=0, window_s=60.0)
    provider, _ = _make_provider(lambda _r: _ok_response("ok"), bucket=bucket)
    try:
        await provider.chat(prompt="first")
        # Second call drains, refill is 0 — bucket.acquire will sleep at
        # least once before giving up the loop. We don't await the second
        # call indefinitely; just assert sleep was invoked while trying.
        task = asyncio.create_task(provider.chat(prompt="second"))
        await asyncio.sleep(0)  # let the task spin once
        task.cancel()
        with pytest.raises(asyncio.CancelledError):
            await task
    finally:
        await provider.aclose()
    assert sleeps, "Expected rate limiter to sleep when bucket drained"


@pytest.mark.asyncio
async def test_backoff_sequence_is_deterministic(monkeypatch) -> None:
    sleeps: list[float] = []

    async def recording_sleep(delay: float) -> None:
        sleeps.append(delay)

    monkeypatch.setattr("asyncio.sleep", recording_sleep)

    provider, _ = _make_provider(lambda _r: httpx.Response(500, text="boom"))
    try:
        with pytest.raises(LLMRetryExhaustedError):
            await provider.chat(prompt="hi")
    finally:
        await provider.aclose()
    # 5 attempts → 5 backoff sleeps. With jitter pinned to 0:
    # 1, 2, 4, 8, 16.
    assert sleeps == [1.0, 2.0, 4.0, 8.0, 16.0]


@pytest.mark.asyncio
async def test_aclose_closes_owned_client_only() -> None:
    transport = httpx.MockTransport(lambda _r: _ok_response("ok"))
    injected = httpx.AsyncClient(transport=transport, base_url="http://x")
    provider = OllamaProvider(LLMConfig(), client=injected)
    await provider.aclose()
    # Injected client must remain usable.
    response = await injected.post("/ping")
    assert response.status_code == 200
    await injected.aclose()


@pytest.mark.asyncio
async def test_aclose_closes_self_owned_client() -> None:
    cfg = LLMConfig()
    provider = OllamaProvider(cfg)
    # Force a clean shutdown without any HTTP traffic.
    await provider.aclose()
    assert provider._client.is_closed  # type: ignore[attr-defined]
