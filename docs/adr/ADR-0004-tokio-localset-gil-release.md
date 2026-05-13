# ADR-0004: Tokio LocalSet + GIL release; nenhum await sob GIL

- **Status:** Accepted
- **Data:** 2026-05-13
- **Inputs:** T5 §async pitfalls ("Python event loops são thread-local; use LocalSet")

## Contexto
Tokio + GIL é território espinhoso. Padrões anti que já apareceram em
projetos PyO3:
- `await` segurando o GIL → deadlock determinístico se outra task tentar Python.
- Spawn em `tokio::spawn` quando o callback precisa de `!Send` (Python objects).
- Event loop Python morto enquanto Rust ainda tenta callbacks.

## Decisão
1. Toda chamada de TTP roda num **Tokio runtime multi-thread** dedicado em
   Rust, separado do thread Python.
2. Antes de qualquer `await`, **liberamos o GIL** via `Python::allow_threads`.
3. Calls de volta para Python (raras) usam `tokio::task::LocalSet` para
   garantir afinidade de thread.
4. Objetos Python **não cruzam tasks Tokio** — só dados owned (Strings,
   Vec<u8>, structs `#[pyclass]` Send).
5. Não usamos `pyo3-asyncio`/`pyo3-async-runtimes` no MVP-0; LangGraph roda
   síncrono no thread Python e chama Rust síncrono que internamente bloqueia
   no runtime Tokio via `Runtime::block_on`.

## Consequências
- ✅ Modelo simples: Python é síncrono na borda; concorrência é Tokio interno.
- ✅ Stress test de 1000 chamadas concorrentes (critério MVP-0 T1 §5.1.3) é
  viável.
- ❌ TTPs longos (nmap full scan) bloqueiam a iteração do LangGraph — ok no
  MVP-1; MVP-2 pode introduzir TTPs em background com handle.

## Alternativas consideradas
- **pyo3-async-runtimes:** maduro, mas adiciona dependência e modelo mental
  complexo; revisitar no MVP-2 se TTPs assíncronos forem mandatórios.
