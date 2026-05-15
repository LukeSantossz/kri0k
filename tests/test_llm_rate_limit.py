"""Tests for kri0k.llm.rate_limit.TokenBucket."""

import asyncio

import pytest

from kri0k.llm.rate_limit import TokenBucket


class _Clock:
    """Manual monotonic clock for deterministic tests."""

    def __init__(self, start: float = 1000.0) -> None:
        self.now = start

    def monotonic(self) -> float:
        return self.now

    def advance(self, dt: float) -> None:
        self.now += dt


@pytest.fixture
def clock(monkeypatch):
    c = _Clock()
    monkeypatch.setattr("kri0k.llm.rate_limit.time.monotonic", c.monotonic)
    return c


@pytest.mark.asyncio
@pytest.mark.usefixtures("clock")
async def test_initial_capacity_allows_burst() -> None:
    bucket = TokenBucket(capacity=10, refill_per_window=10, window_s=60.0)
    for _ in range(10):
        await bucket.acquire()
    assert bucket.available() == pytest.approx(0.0, abs=1e-9)


@pytest.mark.asyncio
async def test_eleventh_acquire_blocks_until_refill(clock, monkeypatch) -> None:
    sleep_calls: list[float] = []

    async def fake_sleep(delay: float) -> None:
        sleep_calls.append(delay)
        # Advance clock by the requested delay so refill makes progress.
        clock.advance(delay)

    monkeypatch.setattr("kri0k.llm.rate_limit.asyncio.sleep", fake_sleep)

    bucket = TokenBucket(capacity=10, refill_per_window=10, window_s=60.0)
    for _ in range(10):
        await bucket.acquire()

    await bucket.acquire()
    assert sleep_calls, "Expected acquire to sleep when bucket empty"
    # Refill rate is 10/60s = 1/6 tok/s, so we wait ~6s for one token.
    assert sleep_calls[0] == pytest.approx(6.0, rel=1e-3)


@pytest.mark.asyncio
async def test_refill_is_linear(clock) -> None:
    bucket = TokenBucket(capacity=10, refill_per_window=10, window_s=60.0)
    for _ in range(10):
        await bucket.acquire()
    clock.advance(30.0)  # half a window
    assert bucket.available() == pytest.approx(5.0, abs=1e-6)


@pytest.mark.asyncio
async def test_capacity_never_exceeded(clock) -> None:
    bucket = TokenBucket(capacity=5, refill_per_window=10, window_s=1.0)
    clock.advance(3600.0)
    assert bucket.available() == pytest.approx(5.0)


@pytest.mark.asyncio
async def test_concurrent_acquire(clock, monkeypatch) -> None:
    async def fake_sleep(delay: float) -> None:
        clock.advance(delay)

    monkeypatch.setattr("kri0k.llm.rate_limit.asyncio.sleep", fake_sleep)

    bucket = TokenBucket(capacity=3, refill_per_window=10, window_s=10.0)
    results: list[int] = []

    async def worker(n: int) -> None:
        await bucket.acquire()
        results.append(n)

    await asyncio.gather(*(worker(i) for i in range(6)))
    assert sorted(results) == [0, 1, 2, 3, 4, 5]
    # Bucket consumed all 6 tokens; never exceeded capacity along the way.
    assert bucket.available() <= 3.0


def test_constructor_validates_args() -> None:
    with pytest.raises(ValueError, match="capacity"):
        TokenBucket(capacity=0)
    with pytest.raises(ValueError, match="refill_per_window"):
        TokenBucket(refill_per_window=-1)
    with pytest.raises(ValueError, match="window_s"):
        TokenBucket(window_s=0)
