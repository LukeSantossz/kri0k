# ADR-0013: TTP trait async + Subprocess abstraction para testabilidade

- **Status:** Accepted
- **Data:** 2026-05-18
- **Inputs:** Phase 4 implementação (TTP-01..05), ADR-0012 (TTP trait skeleton sync), D-44, D-54, D-46

## Contexto

ADR-0012 estabeleceu o trait `Ttp` como ponto de extensão para TTPs (M-15, M-34..M-36). Phase 4 implementou a primeira TTP concreta (T1590.001 whois) e precisou resolver dois pontos arquiteturais:

1. **Async vs sync `execute()`.** Subprocess via `std::process::Command` (sync) bloqueia o Tokio runtime do pybridge — incompatível com `tokio::time::timeout` e `CancellationToken`. Migração para `tokio::process::Command` (async) é necessária.
2. **Testabilidade.** Unit tests com binary real são lentos, flaky (rede), e exigem dependência externa instalada. Tests devem rodar em CI greenfield sem `whois.exe`.

## Decisão

1. **`Ttp` trait async via `async-trait`.** Método `async fn execute(&self, target: &str, cancel: CancellationToken) -> Result<TtpOutput, Error>`. Macro `#[async_trait]` torna o trait dyn-compatible (`Box<dyn Ttp>` para registry `HashMap`).

2. **Abstração `Subprocess` trait com Real + Mock impls.** TTP não chama `tokio::process::Command` diretamente; injeta `Arc<dyn Subprocess>` no construtor:

```rust
pub struct WhoisTtp {
    subprocess: Arc<dyn Subprocess>,
    last_call: Mutex<Option<Instant>>,
}

impl WhoisTtp {
    pub fn new(subprocess: Arc<dyn Subprocess>) -> Self { /* ... */ }
}
```

Implementações:
- `RealSubprocess` — usa `tokio::process::Command` com `tokio::select! { biased; cancel, timeout, output }`.
- `MockSubprocess::from_fixture(path)` — lê fixture file, retorna `SubprocessOutput` sintético.
- `MockSubprocess::hanging()` — future que nunca completa, usado em timeout/cancel tests.

3. **Cancel + timeout uniformes.** O `tokio::select!` na `RealSubprocess::run` cobre 3 branches com `biased` (cancel ganha):
   - `cancel.cancelled()` -> `start_kill() + wait()` + `Err(Cancelled)`.
   - `tokio::time::sleep(timeout)` -> `start_kill() + wait()` + `Err(SubprocessTimeout)`.
   - `child.wait_with_output()` -> `Ok(SubprocessOutput)`.

## Consequências

- **+** Kill switch funcional (`CancellationToken` propaga, biased select garante prioridade).
- **+** Timeout uniforme no nível do `Subprocess` — TTPs não reinventam.
- **+** Unit tests rápidos via `MockSubprocess` (sem rede, sem binary).
- **+** Adicionar TTP novo = 1 arquivo + 1 linha no registry do `Engagement`.
- **-** Custo de 1 heap allocation por chamada async (preço do `#[async_trait]` para dyn-compat). Negligível para 1 req/sec; revisitar se hot path.
- **-** `Box<dyn Subprocess>` impede inline; tradeoff aceitável.

## Alternativas consideradas

- **Native `async fn` em trait (Rust 1.85 estável):** rejeitado — NÃO suporta `Box<dyn Trait>` (vtable não pode armazenar diferentes `impl Future`). Registry `HashMap<String, Box<dyn Ttp>>` é requisito.
- **`std::process::Command` sync direto na TTP:** rejeitado — bloqueia runtime, incompatível com `CancellationToken` + timeout via `tokio::select`.
- **`pyo3-asyncio` para async pyclass methods:** deferido — `runtime().block_on` + `py.allow_threads` + Python side `asyncio.to_thread` é suficiente para Phase 4. Reavaliar quando ato chamar Python de dentro de async task.
- **TTP-aware subprocess (sem trait abstraction):** rejeitado — cada TTP duplicaria timeout/cancel/kill_on_drop logic.

## Referências

- ADR-0001 (canonical state in Rust)
- ADR-0012 (TTP trait skeleton — superceded para subprocess shape, complementa para risk_level)
- `crates/kri0k-core/src/ttp/{mod,subprocess,whois}.rs`
- RESEARCH.md Phase 4 seção "Pattern 1: tokio::process::Command + timeout + CancellationToken" + "Pattern 3: async-trait + Box<dyn Ttp> dispatch"
