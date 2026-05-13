# ADR-0006: Propose-only é default; `--execute` é opt-in por engagement

- **Status:** Accepted
- **Data:** 2026-05-13
- **Inputs:** T1 §3.4 mitigação, §5.2 critério MVP-1

## Contexto
Tooling ofensivo agêntico tem precedente histórico de "fire and forget"
catastrófico — agente decide pivotar para sistema errado, deleta dados, ou
quebra RoE em 3 segundos.

## Decisão
1. CLI default: `kri0k run` ⇒ `--propose-only`. O agente itera, popula o grafo
   com nós `kind=proposal`, mas **nenhum TTP toca a rede**.
2. `kri0k run --execute` é exigido para execução real. Imprime um aviso em
   vermelho e exige confirmação interativa (`yes/no`) salvo `--yes` explícito.
3. Mesmo em `--execute`, TTPs destrutivos (ADR-0005 §3) exigem **segundo gate
   humano** por ação.

## Consequências
- ✅ Operador novo não destrói nada por acidente.
- ✅ Modo `--propose-only` serve como "dry-run de plano" para review pré-engagement.
- ❌ Mais cliques para casos legítimos; aceito como custo do design.

## Alternativas consideradas
- **`--execute` default + `--dry-run`:** rejeitado, viola fail-safe defaults.
