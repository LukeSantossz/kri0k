"""Asynchronous token-bucket rate limiter.

Used by `OllamaProvider` to honor LLM-04 (≤10 requests/minute interpreted
as a 10-token bucket refilling 10 tokens per 60s window — D-28).
"""

import asyncio
import time


class TokenBucket:
    """Async-safe token bucket with linear refill.

    Tokens accrue at `refill_per_window / window_s` per second up to
    `capacity`. `acquire()` waits until at least one token is available,
    then consumes it.

    Args:
        capacity: Maximum number of tokens the bucket can hold.
        refill_per_window: Tokens added per `window_s`.
        window_s: Length of the refill window in seconds.
    """

    __slots__ = ("_capacity", "_last_refill", "_lock", "_rate", "_tokens")

    def __init__(
        self,
        capacity: int = 10,
        refill_per_window: int = 10,
        window_s: float = 60.0,
    ) -> None:
        if capacity <= 0:
            raise ValueError("capacity must be > 0")
        if refill_per_window < 0:
            raise ValueError("refill_per_window must be >= 0")
        if window_s <= 0:
            raise ValueError("window_s must be > 0")
        self._capacity = float(capacity)
        self._rate = refill_per_window / window_s  # tokens per second
        self._tokens = float(capacity)
        self._last_refill = time.monotonic()
        self._lock = asyncio.Lock()

    def _refill(self) -> None:
        now = time.monotonic()
        elapsed = now - self._last_refill
        if elapsed <= 0:
            return
        self._tokens = min(self._capacity, self._tokens + elapsed * self._rate)
        self._last_refill = now

    def available(self) -> float:
        """Return the current token count after a synchronous refill."""
        self._refill()
        return self._tokens

    async def acquire(self) -> None:
        """Block until a token is available, then consume it."""
        while True:
            async with self._lock:
                self._refill()
                if self._tokens >= 1.0:
                    self._tokens -= 1.0
                    return
                if self._rate <= 0:
                    # No refill scheduled — would deadlock. Pick a small
                    # wait so callers get a chance to observe the empty
                    # state via tests.
                    wait_s = 0.05
                else:
                    deficit = 1.0 - self._tokens
                    wait_s = deficit / self._rate
            # Release the lock before sleeping so other coroutines can
            # observe state changes (and tests can monkeypatch sleep).
            await asyncio.sleep(wait_s)
