"""LLM provider protocol.

Defines the minimal contract every concrete provider must satisfy.
Per D-33, Phase 2 returns the raw text of `message.content`. Structured
parsing (JSON mode, Pydantic schemas) is the responsibility of Phase 3.

Note: with the default `deepseek-r1:32b` model (D-18) the returned text
includes `<think>...</think>` tags. Stripping or interpreting them is
also Phase 3 work; Phase 2 keeps the contract neutral.
"""

from typing import Protocol, runtime_checkable


@runtime_checkable
class LLMProvider(Protocol):
    """Asynchronous chat-completion provider contract."""

    async def chat(self, *, prompt: str, system: str | None = None) -> str:
        """Send a single-turn chat request and return the raw response text.

        Args:
            prompt: User-role content. Already-rendered prompt (no template
                substitution at this layer).
            system: Optional system-role preamble.

        Returns:
            Raw `message.content` from the provider. No parsing, no
            sanitization. Downstream consumers (Phase 3+) are responsible
            for structured handling.
        """
        ...
