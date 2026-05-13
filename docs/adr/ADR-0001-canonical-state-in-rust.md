# ADR-0001: Estado canônico vive em Rust; Python recebe snapshots

- **Status:** Accepted
- **Data:** 2026-05-13
- **Decisores:** architect (T6)
- **Inputs:** T1 §3.8, T5 §async pitfalls, T3 (Red-MIRROR / CheckMate findings)

## Contexto
O agente combina raciocínio LLM (Python/LangGraph) com execução de TTPs e
auditoria forense (Rust). Há duas opções razoáveis para localizar o estado:
(a) viver em Python como objeto compartilhado por referência, ou (b) viver em
Rust com Python recebendo cópias serializadas por iteração.

A opção (a) é o que LangChain/CrewAI/AutoGen fazem nativamente, mas trás
problemas em PyO3+Tokio: GIL contention, race conditions silenciosas quando o
LLM "imagina" mutações que não chegaram a aplicar, e auditoria fraca (qualquer
nó do grafo pode mudar sem trilha).

## Decisão
O grafo canônico é uma `StableGraph` em `kri0k-graph`. A camada Python só
acessa o estado via `Snapshot` JSON imutável; mutação só acontece via
`Engagement.execute(Proposal)`, que valida e aplica atomicamente em Rust.

## Consequências
- ✅ Auditoria trivial: toda mutação tem um caminho único.
- ✅ GIL nunca segura o estado quando um TTP roda em background Tokio.
- ✅ LLM pode "viajar" sem efeitos: alucinações ficam confinadas a `Proposal`
  rejeitadas pelo validador.
- ❌ Custo de serialização por iteração (medido em T5: ~1ms para 1k nós, ok).
- ❌ Python perde flexibilidade de patches monkey — intencional.

## Alternativas consideradas
- **Estado em Python com Rust como mero serviço:** rejeitada por causa do
  acoplamento entre LangGraph e auditoria.
- **Estado replicado nos dois lados com CRDT:** overkill para single-operator.
