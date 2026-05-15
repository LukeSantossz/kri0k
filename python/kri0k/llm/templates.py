"""Jinja2 prompt-template loader.

Lazy: the environment is only constructed on the first `render()` call,
so `import kri0k.llm.templates` is cheap and side-effect free (D-25).

We intentionally disable autoescape — these templates emit plain text
sent to a chat completion API, not HTML. HTML-escaping would corrupt
prompt content (e.g. ampersands inside JSON).
"""

from functools import lru_cache
from pathlib import Path
from typing import Any

import jinja2


@lru_cache(maxsize=1)
def _get_env() -> jinja2.Environment:
    """Return the singleton Jinja2 environment, building it on first call."""
    prompts_dir = Path(__file__).parent / "prompts"
    return jinja2.Environment(
        loader=jinja2.FileSystemLoader(str(prompts_dir)),
        autoescape=jinja2.select_autoescape(disabled_extensions=("jinja2",)),
        undefined=jinja2.StrictUndefined,
        keep_trailing_newline=True,
    )


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
