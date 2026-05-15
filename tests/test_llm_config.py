"""Tests for kri0k.llm.config.LLMConfig."""

import dataclasses

import pytest

from kri0k.llm.config import LLMConfig


def test_defaults() -> None:
    cfg = LLMConfig()
    assert cfg.provider == "ollama"
    assert cfg.model == "deepseek-r1:32b"
    assert cfg.base_url == "http://localhost:11434"
    assert cfg.timeout_s == 30.0
    assert cfg.temperature == 0.2
    assert cfg.max_tokens is None


def test_is_frozen() -> None:
    cfg = LLMConfig()
    with pytest.raises(dataclasses.FrozenInstanceError):
        cfg.model = "other"  # type: ignore[misc]


def test_from_scope_dict_empty_returns_defaults() -> None:
    cfg = LLMConfig.from_scope_dict({})
    assert cfg == LLMConfig()


def test_from_scope_dict_missing_llm_block_returns_defaults() -> None:
    cfg = LLMConfig.from_scope_dict({"targets": ["10.0.0.0/8"]})
    assert cfg.model == "deepseek-r1:32b"


def test_from_scope_dict_overrides_model() -> None:
    cfg = LLMConfig.from_scope_dict({"llm": {"model": "qwen3:32b"}})
    assert cfg.model == "qwen3:32b"
    # Other defaults untouched.
    assert cfg.base_url == "http://localhost:11434"


def test_from_scope_dict_rejects_unknown_key() -> None:
    with pytest.raises(ValueError, match="Unknown llm config key: temperature"):
        LLMConfig.from_scope_dict({"llm": {"temperature": 0.7}})


def test_from_scope_dict_rejects_non_mapping_block() -> None:
    with pytest.raises(ValueError, match="scope.yaml::llm must be a mapping"):
        LLMConfig.from_scope_dict({"llm": ["not", "a", "mapping"]})


def test_from_scope_dict_rejects_non_string_model() -> None:
    with pytest.raises(ValueError, match="must be a string"):
        LLMConfig.from_scope_dict({"llm": {"model": 42}})


def test_from_scope_dict_handles_null_llm_block() -> None:
    # `llm:` with no value parses to None in YAML; treat as empty.
    cfg = LLMConfig.from_scope_dict({"llm": None})
    assert cfg == LLMConfig()
