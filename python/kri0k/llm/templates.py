"""Jinja2 prompt-template loader.

Lazy: the environment is only constructed on the first `render()` call,
so `import kri0k.llm.templates` is cheap and side-effect free (D-25).
"""

from pathlib import Path
from typing import Any

import jinja2

# Module-level cache; populated on first call to `_get_env`.
_ENV: jinja2.Environment | None = None


def _get_env() -> jinja2.Environment:
    """Return the singleton Jinja2 environment, building it if needed."""
    global _ENV
    if _ENV is None:
        prompts_dir = Path(__file__).parent / "prompts"
        _ENV = jinja2.Environment(
            loader=jinja2.FileSystemLoader(str(prompts_dir)),
            autoescape=False,  # Plain-text prompts, not HTML.
            undefined=jinja2.StrictUndefined,
            keep_trailing_newline=True,
        )
    return _ENV


def render(template_name: str, /, **context: Any) -> str:
    """Render a template by file name (e.g. ``"sense.jinja2"``).

    Raises:
        jinja2.TemplateNotFound: If the template does not exist in
            `python/kri0k/llm/prompts/`.
        jinja2.UndefinedError: If a referenced variable is missing
            (StrictUndefined).
    """
    template = _get_env().get_template(template_name)
    return template.render(**context)
