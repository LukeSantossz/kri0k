# ADR-0002: petgraph::StableGraph com IDs ULID externos

- **Status:** Accepted
- **Data:** 2026-05-13
- **Inputs:** T5 §petgraph (StableGraph mantém índices)

## Contexto
`petgraph::Graph` remapeia índices internos quando há remoções, o que quebra
referências persistidas em disco e em audit logs. Precisamos de IDs estáveis
que sobrevivam a remoções, e que sejam legíveis para o LLM (ULID é monotônico
e ordenável por tempo).

## Decisão
- Backend: `petgraph::stable_graph::StableGraph<NodeKind, EdgeKind>`.
- IDs públicos: ULID gerado em Rust no momento de inserção, mantido num
  `HashMap<Ulid, NodeIndex>` ao lado do grafo.
- `NodeIndex` interno **nunca cruza a fronteira** PyO3 ou o audit log.

## Consequências
- ✅ Remoções não invalidam audit logs antigos.
- ✅ Snapshots são portáveis entre processos / hosts.
- ✅ LLM consegue se referir a "01HX…" no rationale e o validador resolve.
- ❌ Overhead de um lookup hash por mutação (negligível).

## Alternativas consideradas
- UUID v7: ok, mas ULID é mais compacto e a literatura de logs prefere ULID.
- Index inteiro simples: rejeitado pelo problema de remapeamento.
