"""Tests for kri0k.llm.templates."""

import jinja2
import pytest

from kri0k.llm import templates


def test_get_env_is_lazy() -> None:
    # Ensure the cached singleton has not been built yet for this test.
    templates._get_env.cache_clear()
    assert templates._get_env.cache_info().currsize == 0
    templates._get_env()
    assert templates._get_env.cache_info().currsize == 1


def test_render_sense_template_with_all_variables() -> None:
    out = templates.render(
        "sense.jinja2",
        formatted_snapshot="Graph snapshot\n- nodes: 0",
        scope="10.0.0.0/8",
        objective="enumerate hosts",
        iteration_count=2,
        history_summary="no prior actions",
    )
    assert "Engagement objective: enumerate hosts" in out
    assert "Iteration: 2" in out
    assert "Authorized scope (canonical): 10.0.0.0/8" in out
    assert "Graph snapshot" in out
    assert "no prior actions" in out


def test_render_missing_variable_raises_undefined() -> None:
    with pytest.raises(jinja2.UndefinedError):
        templates.render(
            "sense.jinja2",
            formatted_snapshot="x",
            scope="x",
            objective="x",
            iteration_count=0,
            # history_summary intentionally omitted
        )


def test_render_unknown_template_raises_not_found() -> None:
    with pytest.raises(jinja2.TemplateNotFound):
        templates.render("does_not_exist.jinja2")


def test_render_healthcheck_returns_exact_text() -> None:
    out = templates.render("healthcheck.jinja2")
    assert out == "Reply with the single word: pong\n"
