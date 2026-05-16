# Phase 2 — Live Verification

**Date:** 2026-05-15
**Branch:** `feat/phase-2-sense-ollama`
**Verifier:** kri0k-agent (Claude, Modo Desenvolvimento)

This document records the manual end-to-end verification of the Phase 2
deliverables: `LLMConfig`, `OllamaProvider`, `ping_ollama`, and the
`scope.yaml::llm.model` override.

## 1. Tooling sanity

```bash
$ .venv/Scripts/python.exe -m pytest tests/
============================= 70 passed in 0.92s =============================

$ .venv/Scripts/python.exe -m ruff check python tests
All checks passed!

$ .venv/Scripts/python.exe -m ruff format --check python tests
24 files already formatted

$ .venv/Scripts/python.exe -m mypy python/kri0k
Success: no issues found in 19 source files
```

## 2. Scope override resolution

```bash
$ .venv/Scripts/python.exe -c "
import yaml
from kri0k.llm import LLMConfig
cfg = LLMConfig.from_scope_dict(yaml.safe_load(open('config/scope.example.yaml')))
print(f'Resolved model: {cfg.model}')
print(f'base_url: {cfg.base_url}')
"
Resolved model: deepseek-r1:32b
base_url: http://localhost:11434
```

Confirms D-19/D-20: only `model` is read from scope; defaults apply to
the rest. Strict-key behavior is asserted in
`tests/test_llm_config.py::test_from_scope_dict_rejects_unknown_key`.

## 3. ping_ollama against local daemon — failure mode

The verification host did not have Ollama running on `localhost:11434`
during this session:

```bash
$ curl -s --max-time 5 http://localhost:11434/api/tags
(empty — daemon unreachable)
```

The health-check correctly captures this without re-raising:

```bash
$ .venv/Scripts/python.exe -c "
import asyncio
from kri0k.llm import LLMConfig, ping_ollama
print(asyncio.run(ping_ollama(LLMConfig())))
"
PingResult(
    ok=False,
    model='deepseek-r1:32b',
    latency_ms=42812.99999999828,
    response_excerpt='',
    error='LLMRetryExhaustedError: Ollama call failed after 5 attempts',
)
```

Observed behavior:

* All 5 retries exhausted (D-29 default).
* Total wall-clock ~42.8s — consistent with the deterministic backoff
  series `1 + 2 + 4 + 8 + 16 = 31s` plus per-attempt connect timeouts
  on Windows TCP stack (~2s each).
* `LLMRetryExhaustedError` is wrapped, original `httpx.ConnectError`
  preserved as `__cause__`.
* Diagnostic flow never raises: the caller receives a `PingResult` and
  inspects `ok` / `error`.

## 4. ping_ollama against local daemon — success mode (expected shape)

When Ollama is running and the configured model is available, the call
returns a populated `PingResult`. Schematic of the expected response
(literal output depends on the model and system load):

```python
PingResult(
    ok=True,
    model='deepseek-r1:32b',
    latency_ms=<positive float>,
    response_excerpt='<think>\n...reasoning...\n</think>\n\npong',
    error=None,
)
```

Notes:

* Per **D-18**, the default `deepseek-r1:32b` model emits
  `<think>...</think>` tags before its actual answer. We do **not** strip
  them in Phase 2; that handling lands in Phase 3 (`reason` node).
* `response_excerpt` is truncated to 200 characters
  (`tests/test_llm_healthcheck.py::test_excerpt_truncated_to_200_chars`).

## 5. Re-run instructions for Phase 3 prep

To reproduce a green ping locally before starting Phase 3:

```bash
ollama serve &                    # daemon on :11434
ollama pull deepseek-r1:32b       # one-time
.venv/Scripts/python.exe -c "
import asyncio
from kri0k.llm import LLMConfig, ping_ollama
print(asyncio.run(ping_ollama(LLMConfig())))
"
```

To probe a different model without code changes, edit
`config/scope.example.yaml::llm.model` and use `LLMConfig.from_scope_dict`.

## 6. Phase 2 acceptance summary

| Check | Result |
|---|---|
| `cargo test --workspace` | not re-run (Phase 2 is Python-only; no Rust changes) |
| `cargo clippy` | not re-run (same reason) |
| `pytest tests/` | **70 passed** |
| `ruff check` | **clean** |
| `ruff format --check` | **clean** |
| `mypy python/kri0k` | **clean** |
| `ping_ollama` failure mode | reproduced live (daemon down) |
| `ping_ollama` success mode | shape documented (live success requires daemon) |
| `scope.yaml::llm.model` override | resolved correctly |

Phase 2 is **complete pending green Ollama success run**, which is a
deployment-environment concern and not a code-quality gate.
