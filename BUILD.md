# Kri0k Development Setup

## Prerequisites

- Rust 1.75+ (MSRV)
- Python 3.11+
- Maturin (`cargo install maturin` or `pip install maturin`)

## Building

### Rust-only tests

```bash
cargo test --all-features
```

### Python package build

```bash
# Development build (editable install)
maturin develop

# Release build
maturin build --release
```

### Python tests

After running `maturin develop`:

```bash
pytest tests/
```

## Quick start

```bash
# 1. Build and install locally
maturin develop

# 2. Try it in Python
python -c "import kri0k; print(kri0k.hello())"
python -c "import kri0k; print(kri0k.get_dummy_graph())"

# 3. Run smoke tests
pytest tests/test_smoke.py -v
```

## Project structure

```
├── Cargo.toml              # Workspace root
├── pyproject.toml          # Python package config + maturin
├── crates/
│   ├── kri0k-core/         # Common types, error handling
│   ├── kri0k-graph/        # Graph logic with petgraph
│   └── kri0k-pybridge/     # PyO3 bindings
├── python/kri0k/           # Python source
│   ├── __init__.py         # Public API
│   └── _native.pyi         # Type stubs for Rust extension
└── tests/                  # Python tests
    └── test_smoke.py       # Cross-language smoke tests
```

## See also

- `docs/ARCHITECTURE.md` — System architecture and ADRs
- `CONTRIBUTING.md` — Code style and quality requirements
- `COMMIT_CONVENTION.md` — Git commit message format
