# Coding Conventions

**Analysis Date:** 2026-05-14

## Languages

**Rust (Primary):**
- Edition: 2021
- MSRV: 1.75.0
- Location: `crates/kri0k-core/`, `crates/kri0k-graph/`, `crates/kri0k-pybridge/`

**Python (Primary):**
- Version: 3.11+
- Location: `python/kri0k/`, `tests/`

## Naming Patterns

**Rust Files:**
- Snake_case: `lib.rs`, `audit.rs`, `scope.rs`, `safeguards.rs`, `ttp.rs`
- Module files: `src/lib.rs` as crate root

**Rust Types:**
- PascalCase for structs/enums: `NodeId`, `EdgeId`, `NodeKind`, `SafeguardsConfig`
- Enum variants: PascalCase (`Host`, `Network`, `Service`, `BelongsTo`)
- Tagged enums use `#[serde(tag = "type", rename_all = "snake_case")]`

**Rust Functions:**
- snake_case: `validate_target()`, `compute_hash()`, `add_node()`
- Constructor pattern: `Type::new()` returns `Self`
- Fallible constructors: `Type::from_*()` returns `Result<Self, Error>`

**Rust Constants:**
- SCREAMING_SNAKE_CASE (defined in comments as convention)

**Rust Modules:**
- snake_case: `pub mod audit;`, `pub mod safeguards;`

**Python Files:**
- snake_case: `test_smoke.py`, `__init__.py`
- Test files: `test_*.py` or `*_test.py`

**Python Functions/Variables:**
- snake_case: `get_dummy_graph()`, `hello()`

**Python Types:**
- PascalCase for classes (none observed yet in minimal codebase)

## Code Style

### Rust Formatting

**Tool:** rustfmt via `rustfmt.toml`

**Key Settings (`rustfmt.toml`):**
```toml
edition = "2021"
max_width = 100
hard_tabs = false
tab_spaces = 4
reorder_imports = true
match_block_trailing_comma = true
use_field_init_shorthand = true
chain_width = 60
```

**Run:** `cargo fmt --all`

### Python Formatting

**Tool:** ruff format via `pyproject.toml`

**Key Settings (`pyproject.toml [tool.ruff]`):**
```toml
target-version = "py311"
line-length = 100
indent-width = 4

[tool.ruff.format]
quote-style = "double"
indent-style = "space"
line-ending = "auto"
```

**Run:** `ruff format python/kri0k tests`

## Linting

### Rust Linting

**Tool:** clippy with workspace lints in `Cargo.toml`

**Deny (errors):**
- `correctness`, `suspicious`, `perf` (lint groups)
- `unwrap_used`, `panic`, `unimplemented` (specific lints)

**Warn:**
- `complexity`, `style`, `pedantic`, `nursery` (lint groups)
- `expect_used`, `todo`, `missing_errors_doc`, `missing_panics_doc`

**Allowed:**
- `module_name_repetitions`, `similar_names`, `too_many_arguments`

**Security Lints (via workspace):**
- `unsafe_code = "warn"`

**Run:** `cargo clippy --all-targets --all-features -- -D warnings`

### Python Linting

**Tool:** ruff with comprehensive rules in `pyproject.toml`

**Enabled Rules (selected):**
- `E`, `W` (pycodestyle)
- `F` (pyflakes)
- `I` (isort)
- `N` (pep8-naming)
- `ANN` (type annotations)
- `S` (bandit security)
- `B` (bugbear)
- `PT` (pytest-style)
- `PL` (pylint)
- `RUF` (ruff-specific)

**Ignored Rules:**
- `ANN101`, `ANN102` (self/cls annotations)
- `S603`, `S607` (subprocess - intentional for TTPs)
- `PLR0913` (too many arguments)

**Per-file Ignores:**
- `tests/**/*.py`: `S101` (assert), `ANN` (annotations), `PLR2004` (magic values)
- `__init__.py`: `F401` (unused imports)

**Run:** `ruff check python/kri0k tests`

## Type Checking

### Python Type Checking

**Tool:** mypy with strict mode via `pyproject.toml`

**Settings:**
```toml
[tool.mypy]
python_version = "3.11"
strict = true
disallow_untyped_defs = true
warn_return_any = true
warn_unreachable = true
show_error_codes = true
```

**Overrides:**
- `kri0k._native`: `ignore_missing_imports = true` (Rust module)
- `tests.*`: `disallow_untyped_defs = false`
- `langgraph.*`, `langchain.*`: `ignore_missing_imports = true`

**Run:** `mypy python/kri0k`

## Import Organization

### Rust Imports

**Order:**
1. std library imports
2. External crate imports
3. Local crate imports (`use crate::...`)

**Pattern:** `use serde::{Deserialize, Serialize};` (grouped derives)

**Enforced by:** `reorder_imports = true` in `rustfmt.toml`

### Python Imports

**Order (via ruff isort):**
1. Standard library
2. Third-party packages
3. First-party (`kri0k`)

**Settings:**
```toml
[tool.ruff.lint.isort]
known-first-party = ["kri0k"]
force-sort-within-sections = true
```

## Error Handling

### Rust Error Handling

**Pattern:** Use `Result<T, E>` with thiserror-derived errors

**Error Type (`crates/kri0k-core/src/lib.rs`):**
```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Generic(String),
}

pub type Result<T> = std::result::Result<T, Error>;
```

**Rules:**
- NO `unwrap()` in production code (clippy `unwrap_used = "deny"`)
- Use `expect()` with descriptive message when panic is justified
- `#[allow(clippy::expect_used)]` requires comment justification
- Use `?` operator for error propagation
- Document panic conditions with `# Panics` rustdoc section

**Test Exception:**
```rust
#[cfg(test)]
#[allow(clippy::expect_used)] // expect is ok in tests
mod tests { ... }
```

### Python Error Handling

**Pattern:** Raise specific exceptions with clear messages

**Docstrings (Google style):**
```python
def validate_proposal(...) -> ValidationResult:
    """Validate a proposal against scope constraints.

    Raises:
        ValueError: If scope_hash doesn't match engagement
    """
```

## Logging

**Rust:** Not yet implemented (TODO in codebase)

**Python:** No logging framework observed yet in minimal codebase

## Documentation

### Rust Documentation

**Style:** rustdoc comments

**Requirements:**
- `missing_docs = "warn"` in workspace lints
- Public APIs must have `///` doc comments
- Functions document errors with `# Errors` section
- Panic conditions documented with `# Panics` section
- Unsafe code requires `# Safety` section

**Example:**
```rust
/// Validate if a target is within the authorized scope.
///
/// This is the core pre-execution validator (M-02, ADR-0005).
///
/// # Errors
/// Returns error if target is out of scope or validation fails.
pub fn validate_target(_scope: &Scope, _target: &str) -> Result<(), crate::Error>
```

### Python Documentation

**Style:** Google-style docstrings

**Setting:** `[tool.ruff.lint.pydocstyle] convention = "google"`

**Example:**
```python
def validate_proposal(
    proposal: Proposal,
    scope_hash: str,
) -> ValidationResult:
    """Validate a proposal against scope constraints.

    Args:
        proposal: The proposal to validate
        scope_hash: SHA256 hash of the current scope.yaml

    Returns:
        ValidationResult with ok=True if valid

    Raises:
        ValueError: If scope_hash doesn't match engagement
    """
```

## Function Design

### Rust Functions

**Return Types:**
- Infallible: Return `T` directly
- Fallible: Return `Result<T, Error>`
- Constructor: `#[must_use]` attribute on `fn new() -> Self`

**Pattern:**
```rust
impl NodeId {
    #[must_use]
    pub fn new() -> Self { Self(ulid::Ulid::new()) }

    #[must_use]
    pub const fn inner(&self) -> ulid::Ulid { self.0 }
}
```

### Python Functions

**Type Annotations:** Required in production code, optional in tests

**Pattern:**
```python
def test_hello() -> None:
    """Test hello() function from Rust core."""
    result = kri0k.hello()
    assert isinstance(result, str)
```

## Module Design

### Rust Modules

**Exports:** Re-export from `lib.rs` for public API

**Pattern:**
```rust
// crates/kri0k-core/src/lib.rs
pub mod audit;
pub mod safeguards;
pub mod scope;
pub mod ttp;
```

### Python Modules

**Exports:** Use `__all__` in `__init__.py`

**Pattern (`python/kri0k/__init__.py`):**
```python
from kri0k._native import get_dummy_graph, hello

__version__ = "0.1.0"
__all__ = ["get_dummy_graph", "hello"]
```

## Commit Convention

**Standard:** Conventional Commits v1.0.0

**Format:**
```
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

**Types:** `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`

**Scopes:** `core`, `graph`, `ttp`, `scope`, `pybridge`, `agent`, `cli`, `deps`, `adr`

**Subject Rules:**
- Imperative mood ("add" not "added")
- Lowercase first letter
- No period at end
- Max 72 characters

**Full details:** `COMMIT_CONVENTION.md`

## Pre-commit Hooks

**Configuration:** `.pre-commit-config.yaml`

**Hooks (in order):**
1. `cargo fmt` (Rust formatting)
2. `cargo clippy` (Rust linting, -D warnings)
3. `cargo test --lib` (Rust unit tests)
4. `ruff` (Python lint + auto-fix)
5. `ruff-format` (Python formatting)
6. `mypy` (Python type checking)
7. `pytest -m unit` (Python unit tests)
8. General checks (whitespace, EOF, YAML/TOML/JSON syntax)
9. `detect-secrets` (security)

**Install:** `pre-commit install`
**Run:** `pre-commit run --all-files`
**Skip slow hooks:** `SKIP=cargo-test-unit,pytest-unit pre-commit run --all-files`

## Security Conventions

**Rust:**
- No `unwrap()`, `panic!()`, `unimplemented!()` in production code
- `unsafe` code requires justification in comments + second maintainer review
- `unsafe_code = "warn"` in workspace lints

**Python:**
- Bandit security rules enabled via ruff (`S` rule set)
- `S603`, `S607` ignored for TTP subprocess execution (intentional)

**Secrets:**
- `detect-secrets` pre-commit hook with baseline file
- Large file prevention (>500KB blocked)

---

*Convention analysis: 2026-05-14*
