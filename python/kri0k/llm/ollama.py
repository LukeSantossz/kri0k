"""Ollama HTTP provider.

Implements `LLMProvider` against the Ollama `/api/chat` endpoint with
async rate limiting (D-28), exponential backoff with jitter (D-29) and a
narrow retryable-error set (D-30).

Threat-model notes:

* M-15: this layer returns the provider's raw text. Downstream code in
  Phase 3+ MUST hand it to Rust validators (no execution from raw LLM
  output).
* M-30: the canonical audit log lives in Phase 8. Until then we only emit
  ``logging.debug`` events at request boundaries — never the prompt nor
  the response body.
"""

from __future__ import annotations

from dataclasses import dataclass
import logging
import random
from types import TracebackType
from typing import Any

import httpx

from kri0k.llm.config import LLMConfig
from kri0k.llm.rate_limit import TokenBucket

# --- Constants --------------------------------------------------------------

RETRY_BASE_S: float = 1.0
RETRY_MAX_S: float = 30.0
RETRY_MAX_ATTEMPTS: int = 5
RETRY_JITTER: float = 0.2  # ±20%
RETRYABLE_STATUSES: frozenset[int] = frozenset({429, *range(500, 600)})

_LOGGER = logging.getLogger("kri0k.llm.ollama")
_RNG = random.SystemRandom()


# --- Errors -----------------------------------------------------------------


class LLMError(Exception):
    """Base class for LLM-layer errors."""


class LLMResponseError(LLMError):
    """Provider returned a structurally invalid response (no retry)."""


class LLMRetryExhaustedError(LLMError):
    """All retry attempts exhausted; original error chained as ``__cause__``."""


# --- Helpers ----------------------------------------------------------------


@dataclass(frozen=True, slots=True)
class _BackoffPlan:
    base_s: float = RETRY_BASE_S
    max_s: float = RETRY_MAX_S
    jitter: float = RETRY_JITTER

    def delay_for(self, attempt: int) -> float:
        """Return jittered delay for the given (0-indexed) attempt."""
        raw = min(self.max_s, self.base_s * (2**attempt))
        if self.jitter <= 0:
            return raw
        offset = _RNG.uniform(-self.jitter, self.jitter)
        return max(0.0, raw * (1.0 + offset))


# --- Provider ---------------------------------------------------------------


class OllamaProvider:
    """LLMProvider talking to a local Ollama daemon via httpx.

    The instance owns its `httpx.AsyncClient` unless one is injected; in
    that case the caller keeps responsibility for closing it.
    """

    def __init__(
        self,
        config: LLMConfig,
        *,
        bucket: TokenBucket | None = None,
        client: httpx.AsyncClient | None = None,
    ) -> None:
        self._config = config
        self._bucket = bucket if bucket is not None else TokenBucket()
        if client is None:
            self._client = httpx.AsyncClient(
                base_url=config.base_url,
                timeout=config.timeout_s,
            )
            self._owns_client = True
        else:
            self._client = client
            self._owns_client = False
        self._backoff = _BackoffPlan()

    # -- lifecycle ------------------------------------------------------

    async def __aenter__(self) -> "OllamaProvider":
        return self

    async def __aexit__(
        self,
        exc_type: type[BaseException] | None,
        exc: BaseException | None,
        tb: TracebackType | None,
    ) -> None:
        await self.aclose()

    async def aclose(self) -> None:
        """Close the underlying client iff this instance owns it."""
        if self._owns_client:
            await self._client.aclose()

    # -- public API -----------------------------------------------------

    async def chat(self, *, prompt: str, system: str | None = None) -> str:
        """Send a single-turn chat request, return raw `message.content`."""
        messages: list[dict[str, str]] = []
        if system is not None:
            messages.append({"role": "system", "content": system})
        messages.append({"role": "user", "content": prompt})

        payload: dict[str, Any] = {
            "model": self._config.model,
            "messages": messages,
            "stream": False,
            "options": {"temperature": self._config.temperature},
        }
        if self._config.max_tokens is not None:
            payload["options"]["num_predict"] = self._config.max_tokens

        data = await self._request_with_retry(payload)
        try:
            content = data["message"]["content"]
        except (KeyError, TypeError) as exc:
            raise LLMResponseError(
                "Ollama response missing 'message.content'"
            ) from exc
        if not isinstance(content, str):
            raise LLMResponseError(
                f"Ollama 'message.content' must be str, got {type(content).__name__}"
            )
        return content

    # -- internals ------------------------------------------------------

    async def _request_with_retry(self, payload: dict[str, Any]) -> dict[str, Any]:
        last_exc: BaseException | None = None
        for attempt in range(RETRY_MAX_ATTEMPTS):
            await self._bucket.acquire()
            try:
                _LOGGER.debug(
                    "ollama request",
                    extra={"model": self._config.model, "attempt": attempt},
                )
                response = await self._client.post("/api/chat", json=payload)
            except (httpx.ConnectError, httpx.ReadTimeout) as exc:
                last_exc = exc
                _LOGGER.debug(
                    "ollama transport error",
                    extra={
                        "model": self._config.model,
                        "attempt": attempt,
                        "status_code": None,
                    },
                )
                await self._sleep_backoff(attempt)
                continue

            if response.status_code in RETRYABLE_STATUSES:
                _LOGGER.debug(
                    "ollama retryable status",
                    extra={
                        "model": self._config.model,
                        "attempt": attempt,
                        "status_code": response.status_code,
                    },
                )
                last_exc = httpx.HTTPStatusError(
                    f"retryable status {response.status_code}",
                    request=response.request,
                    response=response,
                )
                await self._sleep_backoff(attempt)
                continue

            response.raise_for_status()
            try:
                data = response.json()
            except ValueError as exc:
                # Malformed JSON is not a transport hiccup — surface it.
                raise LLMResponseError("Ollama returned non-JSON body") from exc
            if not isinstance(data, dict):
                raise LLMResponseError(
                    f"Ollama JSON must be an object, got {type(data).__name__}"
                )
            return data

        raise LLMRetryExhaustedError(
            f"Ollama call failed after {RETRY_MAX_ATTEMPTS} attempts"
        ) from last_exc

    async def _sleep_backoff(self, attempt: int) -> None:
        # Imported lazily so tests can monkeypatch a single symbol.
        import asyncio

        delay = self._backoff.delay_for(attempt)
        await asyncio.sleep(delay)
