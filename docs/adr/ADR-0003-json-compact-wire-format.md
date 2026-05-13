# ADR-0003: JSON compacto na fronteira PyO3 (não MsgPack)

- **Status:** Accepted
- **Data:** 2026-05-13
- **Inputs:** T5 §serialização (JSON compacto vence para LLMs <10k nós)

## Contexto
A serialização cruza a fronteira PyO3 a cada `snapshot()`. Candidatos: JSON
verboso, JSON compacto (chaves curtas), MsgPack, bincode, DSL textual custom.

Testes (T5) mostraram que para grafos ≤10k nós:
- JSON compacto perde ~5% para MsgPack em bytes, mas o LLM gasta ~30% **menos
  tokens** lendo JSON compacto.
- Binário (MsgPack/bincode) não é lido pelo LLM — exige round-trip extra.

## Decisão
Formato canônico de wire é JSON com chaves curtas (`k`, `a`, `t`, `s`, `d`,
`ttp`). Schema versionado em `v`. Disco usa o mesmo formato + gzip
opcional.

## Consequências
- ✅ Debug fácil (`jq` no audit log).
- ✅ LLM lê snapshot diretamente sem decode.
- ❌ Custo CPU de parse maior que MsgPack — não é o gargalo (LLM domina).

## Alternativas consideradas
- **MsgPack para wire, JSON para LLM:** dupla serialização, complicação sem
  ganho.
- **DSL textual custom (DOT-like):** legibilidade alta mas exige parser próprio
  do nosso lado; rejeitado por custo de manutenção.
