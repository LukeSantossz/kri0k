"""Ollama health-check.

Decoupled probe used to validate connectivity and model availability
without dragging the LangGraph runtime into Phase 2 (D-23, D-27). The
future ``kri0k doctor`` CLI command (Phase 12) will reuse this primitive.

Note: with the default ``deepseek-r1:32b`` model (D-18) the response
includes ``<think>...</think>`` tags. We do not strip them here — the
excerpt is meant as a diagnostic blob, not parsed output.
"""

from dataclasses import dataclass
import time

import httpx

from kri0k.llm.config import LLMConfig
from kri0k.llm.ollama import LLMError, OllamaProvider
from kri0k.llm.protocol import LLMProvider
from kri0k.llm.templates import render

_EXCERPT_LIMIT = 200


@dataclass(frozen=True, slots=True)
class PingResult:
    """Outcome of a single health-check call.

    Attributes:
        ok: True when the provider returned a successful response.
        model: Model tag that was probed.
        latency_ms: Wall-clock time spent in the call (always populated).
        response_excerpt: First 200 chars of the response, or ``""`` on
            failure. May contain `<think>` tags for reasoning models.
        error: Stringified exception when ``ok=False``, else ``None``.
    """

    ok: bool
    model: str
    latency_ms: float
    response_excerpt: str
    error: str | None


async def ping_ollama(
    config: LLMConfig,
    *,
    provider: LLMProvider | None = None,
) -> PingResult:
    """Send a trivial prompt and report success/failure.

    Diagnostic helper: never re-raises. Network/parse/timeout errors are
    captured into `PingResult.error`.

    Args:
        config: Provider configuration (model, base URL, timeout).
        provider: Optional pre-built provider (used by tests). When
            omitted, a fresh `OllamaProvider` is created and closed.

    Returns:
        PingResult populated with timing and either an excerpt or an
        error string.
    """
    prompt = render("healthcheck.jinja2")

    # Track the auto-created provider separately so we own its lifecycle
    # without leaking that responsibility into the LLMProvider Protocol
    # (which does not require `aclose`).
    owned: OllamaProvider | None = None
    if provider is None:
        owned = OllamaProvider(config)
        target: LLMProvider = owned
    else:
        target = provider

    started = time.monotonic()
    try:
        text = await target.chat(prompt=prompt)
    except (LLMError, httpx.HTTPError) as exc:
        latency_ms = (time.monotonic() - started) * 1000.0
        return PingResult(
            ok=False,
            model=config.model,
            latency_ms=latency_ms,
            response_excerpt="",
            error=f"{type(exc).__name__}: {exc}",
        )
    finally:
        if owned is not None:
            await owned.aclose()

    latency_ms = (time.monotonic() - started) * 1000.0
    excerpt = text[:_EXCERPT_LIMIT]
    return PingResult(
        ok=True,
        model=config.model,
        latency_ms=latency_ms,
        response_excerpt=excerpt,
        error=None,
    )
