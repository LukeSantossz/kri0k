# ADR-0007: Audit log append-only com hash chain (JSONL)

- **Status:** Accepted
- **Data:** 2026-05-13
- **Inputs:** T1 §1.3 (red team interno: artefato auditável), §2 (anti-bypass)

## Contexto
Engagements de red team autorizado precisam entregar artefatos forensicamente
defensáveis. Logs editáveis pós-hoc minam essa defesa. Ferramentas comparáveis
(Cobalt Strike, CALDERA) têm logging fraco para esse fim.

## Decisão
- Formato: JSONL (`audit.jsonl`), uma linha por evento.
- Cada linha inclui `prev` = `sha256(canonical_json(linha anterior))` e
  `hash` = `sha256(canonical_json(self_without_hash))`.
- Eventos registrados: `engagement_open`, `scope_load`, `proposal`,
  `validation`, `human_gate`, `execution_start`, `execution_end`, `ttp_output`,
  `engagement_close`, `kill_switch`.
- **Não criptografado** por padrão. Justificativa: o audit log é o que
  exporta para o cliente / juízo; segredos (credenciais coletadas) ficam no
  grafo, que **é** criptografado em iteração futura.
- Sanitização: campos sensíveis (`password`, `hash_ntlm`, `cookie`,
  `private_key`) são automaticamente redacted no audit (gravados como
  `"<REDACTED:sha256:...>"`).

## Consequências
- ✅ Detecção trivial de tampering (recomputar a cadeia).
- ✅ Exportação ATT&CK direta: `jq` extrai `ttp_id` de cada execução.
- ❌ Append-only no FS exige cuidado (operador não deve rotacionar manualmente
  o arquivo durante engagement); CLI bloqueia rotate.

## Alternativas consideradas
- **SQLite com triggers:** mais features mas pior portabilidade e hash chain
  fica complicado.
- **Blockchain interno:** overkill — chain hash linear basta.
