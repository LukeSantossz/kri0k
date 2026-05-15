"""Tests for kri0k.llm.templates."""

import importlib

import jinja2
import pytest


def test_import_does_not_load_environment(monkeypatch) -> None:
    # Reset global before import to confirm laziness.
    import kri0k.llm.templates as templates

    monkeypatch.setattr(templates, "_ENV", None)
    reloaded = importlib.reload(templates)
    assert reloaded._ENV is None


def test_render_sense_template_with_all_variables() -> None:
    from kri0k.llm import templates

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
    from kri0k.llm import templates

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
    from kri0k.llm import templates

    with pytest.raises(jinja2.TemplateNotFound):
        templates.render("does_not_exist.jinja2")


def test_render_healthcheck_returns_exact_text() -> None:
    from kri0k.llm import templates

    out = templates.render("healthcheck.jinja2")
    assert out == "Reply with the single word: pong\n"
