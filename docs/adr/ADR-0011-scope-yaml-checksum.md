# ADR-0011: scope.yaml versionado, checksum embarcado em todo snapshot

- **Status:** Accepted
- **Data:** 2026-05-13
- **Inputs:** T1 §2 anti-uso, §3.5 scope creep

## Contexto
`scope.yaml` é a **única** fonte da verdade sobre o que é autorizado. Riscos:
- Operador edita scope a quente para "abrir" alvo no meio do engagement → o
  validador aceita, mas o audit log não tem como provar quando isso aconteceu.
- LLM lê snapshot e "esquece" do scope (o snapshot não menciona scope).

## Decisão
1. `scope.yaml` é parseado uma vez no `open_engagement`; um `scope_hash =
   sha256(canonical_yaml)` é calculado e fica em `Engagement`.
2. Todo `Snapshot` carrega `scope_hash` no payload — o LLM vê e o validador
   re-verifica antes de cada `execute`.
3. Mudança de `scope.yaml` durante engagement exige `kri0k scope reload`, que:
   - Recalcula hash.
   - Registra `scope_change` no audit log com hash antigo e novo + diff
     resumido.
   - Pode rejeitar reload se o novo scope **reduzir** alvos já com ações
     pendentes (configurável; default = aceita mas marca proposals stale).
4. Schema:
   ```yaml
   v: 1
   engagement_id: KRK-2026-001
   profile: internal_redteam   # internal_redteam | ctf | lab | training
   targets:
     ipv4: ["10.0.0.0/24"]
     ipv6: []
     domains: ["lab.example.com"]
   enabled_ttps: ["T1046", "T1590.001", "T1590.002", "T1595.001", "T1596.003"]
   destructive_ttps: []         # explicitamente vazio salvo opt-in
   rate_limits:
     T1595.001: "100/min"
   time_window:
     start: "2026-05-13T08:00:00-03:00"
     end:   "2026-05-15T18:00:00-03:00"
   operator:
     name: "Marina Souza"
     contract_ref: "RoE-2026-Acme-12"
   ```

## Consequências
- ✅ Tampering de scope detectável.
- ✅ LLM tem o hash visível — pode citar no rationale ("scope_hash:9f86…
  inclui o target X").
- ❌ Reloading scope é fricção; intencional.

## Alternativas consideradas
- **Scope embedded no binário:** rejeitado, não escala para múltiplos
  engagements.
- **Scope sem hash:** rejeitado, perde audit trail.
