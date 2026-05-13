# ADR-0009: Cargo workspace de 5 crates + pacote Python via maturin

- **Status:** Accepted
- **Data:** 2026-05-13
- **Inputs:** T6 §1.2

## Contexto
Precisamos organizar Rust (núcleo) + Python (LangGraph) num único repositório
com build reproduzível e fronteira PyO3 isolada num crate dedicado.

## Decisão
Cargo workspace (`Cargo.toml` raiz com `[workspace]`, `resolver = "2"`):

```
crates/
  kri0k-core      lib  (runtime, scope, audit, kill switch)
  kri0k-graph     lib  (petgraph + serde)
  kri0k-ttp       lib  (Ttp trait + adapters)
  kri0k-scope     lib  (scope.yaml DSL)
  kri0k-pybridge  cdylib  (PyO3 → kri0k._native)
python/kri0k/         pacote pip; build via maturin (`pyproject.toml`)
```

- Build: `maturin develop` para dev, `maturin build --release` para release.
- Distribuição: wheels por plataforma (linux x86_64, macOS arm64, windows
  x86_64) via GH Actions matrix.
- Python package version = Rust workspace version (sincronizado por script).

## Consequências
- ✅ `cargo test` testa Rust puro sem Python.
- ✅ `pytest` testa Python + smoke tests cross-language.
- ✅ Crates internos podem ser reusados (ex.: `kri0k-graph` standalone).
- ❌ Build duplo (Rust + Python) aumenta CI matrix; mitigação: cache em GH
  Actions.

## Alternativas consideradas
- **Crate único monolítico:** rejeitado, dificulta testes isolados.
- **uv workspace ao invés de maturin:** maturin é a referência para PyO3.
