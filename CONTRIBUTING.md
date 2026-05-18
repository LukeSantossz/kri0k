# Contributing to KRK-001 Kri0K

Thank you for considering contributing to Kri0K! This document outlines the quality standards, development workflow, and testing strategy for the project.

## Table of Contents

1. [Quality Bar](#quality-bar)
2. [Development Setup](#development-setup)
3. [Code Conventions](#code-conventions)
4. [Testing Strategy](#testing-strategy)
5. [Pre-commit Hooks](#pre-commit-hooks)
6. [Continuous Integration](#continuous-integration)
7. [Commit Convention](#commit-convention)
8. [Pull Request Process](#pull-request-process)

---

## Quality Bar

Kri0K maintains high code quality standards across both Rust and Python codebases:

### Rust (MSRV: 1.75.0)

- **Formatting**: `rustfmt` (enforced via `rustfmt.toml`)
- **Linting**: `clippy` with `-D warnings` (deny all warnings)
- **Testing**: Unit tests, doctests, integration tests
- **Safety**: No `unwrap()`, `panic!()`, or `unimplemented!()` in production code
- **Documentation**: Public APIs must have rustdoc comments

### Python (3.11+)

- **Formatting**: `ruff format` (100-char line length)
- **Linting**: `ruff` (comprehensive rule set)
- **Type Checking**: `mypy --strict`
- **Testing**: `pytest` with coverage tracking
- **Documentation**: Google-style docstrings

---

## Development Setup

### Prerequisites

- Rust 1.75.0+ ([rustup](https://rustup.rs/))
- Python 3.11+ ([python.org](https://www.python.org/))
- Git

### Clone and Setup

```bash
# Clone the repository
git clone https://github.com/LukeSantossz/kri0k.git
cd kri0k

# Install Rust components
rustup component add rustfmt clippy

# Create Python virtual environment
python -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate

# Install Python dependencies
pip install --upgrade pip
pip install maturin ruff mypy pytest pytest-cov pre-commit

# Install development dependencies
pip install -e .[dev]

# Install pre-commit hooks
pre-commit install
```

### Build the Project

```bash
# Build Rust crates
cargo build --all-features

# Build Python extension (maturin)
maturin develop --release

# Run all tests
cargo test --all-features
pytest
```

---

## Code Conventions

### Rust

#### Formatting

Automatically enforced by `rustfmt.toml`:
- 100-character line width
- 4-space indentation
- Grouped imports (std → external → crate)

Run: `cargo fmt --all`

#### Linting

Enforced by `clippy.toml` and workspace lints:
- **Deny**: `correctness`, `suspicious`, `perf`, `unwrap-used`, `panic`, `unimplemented`
- **Warn**: `complexity`, `style`, `pedantic`, `nursery`

Run: `cargo clippy --all-targets --all-features -- -D warnings`

#### Naming Conventions

- Types: `PascalCase` (e.g., `Snapshot`, `ValidationResult`)
- Functions: `snake_case` (e.g., `validate_proposal`, `execute_ttp`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `MAX_RATIONALE_LEN`)
- Modules: `snake_case` (e.g., `graph_store`, `audit_log`)

#### Error Handling

- Use `Result<T, E>` for fallible operations
- Use `expect()` with descriptive messages instead of `unwrap()`
- Document panic conditions in rustdoc with `# Panics` section
- Prefer `?` operator for error propagation

#### Unsafe Code

- Requires explicit justification in code comments
- Must be reviewed by a second maintainer
- Document safety invariants with `# Safety` section

### Python

#### Formatting

Automatically enforced by `ruff format`:
- 100-character line width
- Double quotes for strings
- Google-style docstrings

Run: `ruff format python/kri0k tests`

#### Linting

Enforced by `ruff` with comprehensive rules:
- Security checks (bandit)
- Type annotations required (except in tests)
- No commented-out code
- Import sorting (isort-compatible)

Run: `ruff check python/kri0k tests`

#### Type Annotations

All production code must have type annotations:

```python
from typing import Literal

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
    ...
```

Run: `mypy python/kri0k`

---

## Testing Strategy

Kri0K uses a multi-layered testing approach to ensure correctness across the Rust ↔ Python boundary.

### Test Categories

#### 1. Unit Tests (Rust)

**Location**: `crates/*/tests/` and inline `#[cfg(test)]` modules

**Purpose**: Test individual Rust functions and modules in isolation

**Markers**: None (default)

**Examples**:
- `kri0k-graph`: StableGraph operations, Node/Edge serialization
- `kri0k-scope`: scope.yaml parsing, CIDR matching
- `kri0k-core`: Validator logic, audit log hash chaining

**Run**: `cargo test --lib --all-features`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_serialization() {
        let snapshot = Snapshot::new(42, "sha256:abc...");
        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: Snapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snapshot, deserialized);
    }
}
```

#### 2. Unit Tests (Python)

**Location**: `tests/unit/`

**Purpose**: Test Python LangGraph nodes, LLM providers, prompt rendering

**Markers**: `@pytest.mark.unit`

**Run**: `pytest -m unit`

```python
import pytest
from kri0k.agent import build_graph

@pytest.mark.unit
def test_graph_topology():
    """Verify LangGraph has all required nodes."""
    graph = build_graph()
    nodes = list(graph.nodes.keys())
    assert "sense" in nodes
    assert "reason" in nodes
    assert "validate" in nodes
    assert "act" in nodes
    assert "update" in nodes
```

#### 3. Integration Tests (Cross-Language)

**Location**: `tests/integration/`

**Purpose**: Test Rust ↔ Python interactions via PyO3 bridge

**Markers**: `@pytest.mark.integration`

**Run**: `pytest -m integration`

**Examples**:
- `test_pybridge.py`: Snapshot codec, Proposal validation, execution round-trip
- `test_engagement_lifecycle.py`: open → snapshot → validate → execute → close

```python
import pytest
from kri0k._native import open_engagement, Proposal

@pytest.mark.integration
def test_propose_and_validate(tmp_scope_yaml):
    """Test proposal validation through PyO3 bridge."""
    eng = open_engagement(tmp_scope_yaml)
    
    proposal = Proposal(
        ttp_id="T1046",
        target="192.168.1.0/24",
        args={"ports": "1-1000"},
        rationale="Initial network scan",
        destructive=False,
    )
    
    result = eng.validate(proposal)
    assert result.ok is True
    assert not result.requires_human
    
    eng.close()
```

#### 4. Graph Fixture Tests

**Location**: `tests/graph/`

**Purpose**: Test graph state transitions, invariant preservation

**Markers**: `@pytest.mark.graph`

**Fixtures**: `fixtures/graph_*.jsonl` (petgraph snapshots)

**Run**: `pytest -m graph`

**Examples**:
- Node ID stability after add/remove
- Edge integrity after node removal
- Snapshot idempotency (serialize → deserialize → serialize)

```python
import pytest
from kri0k._native import open_engagement

@pytest.mark.graph
def test_graph_node_removal_preserves_ids(tmp_scope_yaml):
    """Removing a node doesn't reassign IDs of other nodes."""
    eng = open_engagement(tmp_scope_yaml)
    
    # Add 3 nodes, get ULIDs
    node1_id = eng.add_node("host", {"ip": "192.168.1.10"})
    node2_id = eng.add_node("host", {"ip": "192.168.1.20"})
    node3_id = eng.add_node("host", {"ip": "192.168.1.30"})
    
    # Remove middle node
    eng.remove_node(node2_id)
    
    # Check that node1 and node3 still exist with same IDs
    snapshot = eng.snapshot()
    node_ids = [n.id for n in snapshot.nodes]
    assert node1_id in node_ids
    assert node3_id in node_ids
    assert node2_id not in node_ids
    
    eng.close()
```

#### 5. TTP Tests (External Tools)

**Location**: `tests/ttp/`

**Purpose**: Test TTP implementations with external tools (nmap, dig, whois)

**Markers**: `@pytest.mark.ttp`

**Requirements**: External tools must be installed (skipped if missing)

**Run**: `pytest -m ttp` (slow, requires network)

```python
import pytest
import shutil
from kri0k.ttp.t1046 import NmapScanner

@pytest.mark.ttp
@pytest.mark.skipif(not shutil.which("nmap"), reason="nmap not installed")
def test_nmap_scanner_alive_hosts():
    """Test T1046 nmap scanner finds alive hosts."""
    scanner = NmapScanner()
    result = scanner.run(target="127.0.0.1", ports="80,443")
    
    assert result.ok
    assert len(result.new_nodes) >= 1
    assert result.new_nodes[0].kind == "host"
```

#### 6. Audit Log Tests

**Location**: `tests/audit/`

**Purpose**: Validate audit log integrity, hash chaining, sanitization

**Markers**: `@pytest.mark.audit`

**Run**: `pytest -m audit`

```python
import pytest
from kri0k._native import open_engagement, Proposal

@pytest.mark.audit
def test_audit_log_hash_chain(tmp_scope_yaml, tmp_path):
    """Verify audit log entries are hash-chained."""
    eng = open_engagement(tmp_scope_yaml)
    
    # Execute two proposals
    p1 = Proposal(ttp_id="T1046", target="192.168.1.1", args={}, rationale="Scan 1", destructive=False)
    r1 = eng.execute(p1)
    
    p2 = Proposal(ttp_id="T1046", target="192.168.1.2", args={}, rationale="Scan 2", destructive=False)
    r2 = eng.execute(p2)
    
    # Read audit log
    audit_path = tmp_path / "audit.jsonl"
    entries = [json.loads(line) for line in audit_path.read_text().splitlines()]
    
    # Verify hash chain
    assert entries[0]["prev_hash"] == "0" * 64  # Genesis
    assert entries[1]["prev_hash"] == entries[0]["hash"]
    
    eng.close()
```

### Test Fixtures

#### Graph Fixtures (`tests/fixtures/`)

Pre-constructed graph states for regression testing:

- `empty_graph.jsonl`: Empty engagement, scope only
- `single_host.jsonl`: One discovered host
- `multi_host_network.jsonl`: Small network topology
- `post_pivot.jsonl`: After credential discovery and lateral movement

#### Scope Fixtures (`tests/fixtures/scope_*.yaml`)

Sample scope.yaml files for different scenarios:

- `scope_single_cidr.yaml`: Single /24 network
- `scope_multi_domain.yaml`: Multiple domains + subdomains
- `scope_strict.yaml`: Minimal permissions, propose-only
- `scope_permissive.yaml`: Execute-enabled, broader permissions

### Running Tests

```bash
# All tests (unit + integration + graph)
cargo test --all-features  # Rust
pytest                      # Python

# Only fast tests (unit)
cargo test --lib
pytest -m unit

# Only integration tests
pytest -m integration

# Only graph fixture tests
pytest -m graph

# With coverage
pytest --cov=kri0k --cov-report=html
# Open htmlcov/index.html

# Slow tests (TTPs with network/external tools)
pytest -m "slow or ttp"

# Specific test file
pytest tests/integration/test_pybridge.py -v
```

### Test Coverage Requirements

- **Rust**: Minimum 80% coverage (measured by `cargo-tarpaulin` in CI)
- **Python**: Minimum 85% coverage (measured by `pytest-cov`)
- New features must include tests
- Bug fixes must include regression tests

---

## Pre-commit Hooks

Pre-commit hooks run automatically before each commit to catch issues early.

### Install

```bash
pip install pre-commit
pre-commit install
```

### Hooks

1. **Rust**:
   - `cargo fmt --all` (auto-fix formatting)
   - `cargo clippy -- -D warnings` (deny warnings)
   - `cargo test --lib` (unit tests only)

2. **Python**:
   - `ruff check --fix` (auto-fix lint issues)
   - `ruff format` (auto-format code)
   - `mypy --strict` (type checking)
   - `pytest -m unit` (unit tests only)

3. **General**:
   - Trim trailing whitespace
   - Fix end-of-file newlines
   - Check YAML/TOML/JSON syntax
   - Detect large files (>500KB)
   - Detect merge conflicts
   - Detect hardcoded secrets

### Manual Run

```bash
# Run on all files
pre-commit run --all-files

# Skip slow hooks (tests)
SKIP=cargo-test-unit,pytest-unit pre-commit run --all-files
```

---

## Continuous Integration

CI runs on GitHub Actions for all PRs and pushes to `main`/`master`/`develop`.

### Jobs

#### 1. `rust-check` (matrix)

- **OS**: Ubuntu, Windows, macOS
- **Toolchain**: `stable`, `beta` (beta Ubuntu-only for speed)
- **Steps**:
  - `cargo fmt --check`
  - `cargo clippy -- -D warnings`
  - `cargo build --all-features`
  - `cargo test --all-features`
  - `cargo test --doc`

#### 2. `python-check` (matrix)

- **OS**: Ubuntu, Windows, macOS
- **Python**: 3.11, 3.12 (3.12 Ubuntu-only)
- **Steps**:
  - `ruff check`
  - `ruff format --check`
  - `mypy --strict`
  - `pytest -m unit --cov`
  - Upload coverage to Codecov

#### 3. `integration`

- **OS**: Ubuntu only (requires Rust + Python)
- **Steps**:
  - Build Rust extension with `maturin develop --release`
  - Run `pytest -m integration`
  - Run `pytest -m graph`

#### 4. `msrv`

- **Toolchain**: Rust 1.75.0 (MSRV)
- **Steps**: `cargo check --all-features`

#### 5. `security`

- **Rust**: `cargo audit` (RustSec advisories)
- **Python**: `pip-audit` (PyPI vulnerabilities)

### CI Passing Requirements

All jobs must pass before a PR can be merged:
- ✅ Formatting (rustfmt, ruff)
- ✅ Linting (clippy -D warnings, ruff)
- ✅ Type checking (mypy --strict)
- ✅ Tests (unit + integration + graph)
- ✅ MSRV check
- ✅ Security audit

---

## Commit Convention

Kri0K follows [Conventional Commits](https://www.conventionalcommits.org/) v1.0.0.

### Format

```
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code refactoring
- `perf`: Performance
- `test`: Tests
- `build`: Dependencies
- `ci`: CI/CD
- `chore`: Maintenance

### Examples

```
feat(ttp): add T1046 network scanner TTP

Implements T1046 (Network Service Discovery) using nmap.

Refs: #42
```

```
fix(graph)!: change Node.id to ULID format

BREAKING CHANGE: Node.id is now ULID instead of u32.

Fixes: #78
```

See [COMMIT_CONVENTION.md](./COMMIT_CONVENTION.md) for full details.

---

## Pull Request Process

### Before Submitting

1. ✅ Run pre-commit hooks: `pre-commit run --all-files`
2. ✅ Run all tests: `cargo test && pytest`
3. ✅ Update documentation (if applicable)
4. ✅ Add tests for new features
5. ✅ Squash commits into logical units
6. ✅ Write conventional commit messages

### PR Template

The PR template (`.github/PULL_REQUEST_TEMPLATE.md`) includes:
- Description of changes
- Type of change (feat/fix/docs/etc.)
- Scope (which crate/module)
- Breaking changes (if any)
- Checklist:
  - Code quality (rustfmt, clippy, ruff, mypy)
  - Testing (unit, integration, graph)
  - Security (no secrets, no unsafe without justification)
  - Documentation (ADRs, ARCHITECTURE.md, inline comments)

### Review Process

1. **Automated checks**: CI must pass (all jobs green)
2. **Code review**: At least one maintainer approval required
3. **Security review**: Required if PR touches:
   - TTP execution logic
   - Scope validation
   - PyO3 bridge
   - Audit log
   - Unsafe Rust code
4. **Merge**: Squash merge (preserves Conventional Commits for changelog)

### Post-Merge

- Changelog is auto-generated from commit messages
- Version bumps follow [SemVer](https://semver.org/):
  - `feat`: Minor version bump
  - `fix`: Patch version bump
  - `BREAKING CHANGE`: Major version bump

---

## Additional Resources

- [ARCHITECTURE.md](./ARCHITECTURE.md): System design and component contracts
- [docs/adr/](./docs/adr/): Architecture Decision Records
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Google Python Style Guide](https://google.github.io/styleguide/pyguide.html)

---

## Adding a new TTP

O kri0k abstrai TTPs atrás de um trait async (`kri0k_core::ttp::Ttp`) e injeta o subprocess via `Subprocess` trait (ADR-0013). Pattern para adicionar nova TTP:

### 1. Implementar o struct + impl

Em `crates/kri0k-core/src/ttp/<seu_ttp>.rs`:

- Struct `MyTtp { subprocess: Arc<dyn Subprocess>, /* rate limit state */ }`
- `#[async_trait] impl Ttp for MyTtp` com `id()`, `description()`, `rate_limits()`, `default_timeout()`, `async fn execute(target, cancel)`.

Reuse `WhoisTtp` (`crates/kri0k-core/src/ttp/whois.rs`) como referência. Sempre passar `cancel.clone()` ao subprocess.

### 2. Registrar no Engagement

Em `crates/kri0k-pybridge/src/lib.rs`, dentro de `Engagement::new()`:

```rust
registry.insert("MITRE.ID".to_string(), Box::new(MyTtp::new(subprocess.clone())));
```

Quando o registry tiver >= 5 TTPs, refatorar para auto-discovery via crate `inventory` (deferido em D-52).

### 3. Capturar fixture + testes

1. Rode o binário externo manualmente, capture stdout para `tests/fixtures/<ttp>_<target>.txt`.
2. Duplique em `crates/kri0k-core/tests/fixtures/` (Pitfall 13 — path do cargo).
3. Unit tests inline em `<seu_ttp>.rs` usando `MockSubprocess::from_fixture(...)`. Cobertura mínima: `parses_<target>`, `rate_limit_enforced`, `cancellation_returns_cancelled`.
4. Integration test gated por `#[cfg(feature = "integration")]` para o binary real.

### 4. Validação

```bash
cargo test --workspace                             # unit only (CI default)
cargo test --workspace --features integration      # + binary real (local/nightly)
cargo clippy --workspace --all-targets -- -D warnings
```

### 5. Documentar

- `CHANGELOG.md`: entry sob próxima versão.
- `README.md` "Running ..." section se aplicável.
- ADR novo se a TTP introduzir pattern arquitetural.

---

## Questions?

Open an issue or discussion on GitHub: https://github.com/LukeSantossz/kri0k

Thank you for contributing to Kri0K! 🚀
