# ADR-0005: Validador determinístico em Rust precede toda execução

- **Status:** Accepted
- **Data:** 2026-05-13
- **Inputs:** T1 §3.4, §3.5; T3 (CheckMate "Planner-Executor-Perceptor")

## Contexto
O LLM é não-determinístico e pode propor ações fora do escopo, destrutivas, ou
mal formadas. Confiar no LLM para auto-policiar (prompt + system message) é
insuficiente — jailbreaks e alucinações são realidade observada.

CheckMate (T3) demonstra que separar "decisor de O QUÊ atacar" do "executor"
melhora coerência em 20%. Aplicamos a mesma divisão: o LLM propõe, o validador
em Rust decide se pode executar.

## Decisão
Toda `Proposal` passa por `validator::check(p, &scope, &graph, &policy)` antes
de qualquer side-effect. A função é **pura** e **fail-closed**: se qualquer
checagem retornar dúvida, a ação é rejeitada (não permitida).

Checagens MVP-1:
1. `p.target` pertence a algum range em `scope.yaml`.
2. `p.ttp_id` está em `enabled_ttps`.
3. Se `p.destructive` ou `ttp_id ∈ destructive_set`, exige `human_gate_token`
   válido (HMAC sobre `proposal_hash` + nonce, gerado pela CLI ao confirmar).
4. Schema do `args` valida via `jsonschema`.
5. Rate-limit por TTP (default: 10/min, configurável em scope.yaml).

## Consequências
- ✅ Mesmo um LLM totalmente comprometido (prompt injection) não consegue
  executar fora do escopo.
- ✅ Decisões auditáveis: o resultado do validador entra no audit log.
- ❌ Algumas ações legítimas são rejeitadas (false positives); mitigação:
  mensagem de erro inclui exatamente qual regra falhou para o operador ajustar
  scope.yaml.

## Alternativas consideradas
- **Validador em Python:** rejeitado — Python pode ser monkey-patched, e o
  validador precisa ser invariante.
- **Validador opcional:** rejeitado — viola o princípio "nunca confie no LLM".
