---
phase: 4
slug: act-node-ttp-whois
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-05-18
completed: 2026-05-18
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from `04-RESEARCH.md §Validation Architecture` (lines 946-989).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | pytest 7.4+ (Python) + cargo test (Rust) |
| **Config file** | `pyproject.toml` `[tool.pytest.ini_options]`; `Cargo.toml` workspace |
| **Quick run command (Python)** | `pytest tests/test_act_node.py -x` |
| **Quick run command (Rust)** | `cargo test --package kri0k-core` |
| **Full suite command** | `cargo test --workspace --features integration && pytest tests/` |
| **Estimated runtime** | ~30s unit (Rust + Python); ~60s with integration (whois calls @ 1 req/s) |
| **Existing markers** | `unit`, `integration`, `slow`, `ttp`, `graph`, `audit` (already in `pyproject.toml`) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --package <touched crate>` and/or `pytest -k <touched test>` (whichever applies to the file changed).
- **After every plan wave:** Run `cargo test --workspace && pytest tests/ -m "not integration"`.
- **Before `/gsd-verify-work`:** Full phase gate must be green:
  ```
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace --features integration
  pytest tests/
  ruff check python/ tests/
  mypy python/kri0k
  ```
- **Max feedback latency:** 30 seconds for unit tests; 60 seconds for integration (rate-limited by whois 1 req/s).

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 04-XX-XX | TBD | TBD | AGENT-05 | M-15 | `act` envia proposal para Rust e recebe outcome; LLM nunca executa direto | unit | `pytest tests/test_act_node.py::test_act_calls_execute_proposal -x` | ❌ W0 | ⬜ pending |
| 04-XX-XX | TBD | TBD | AGENT-05 | M-05 | `act` honra `propose_only=True` sem chamar Rust | unit | `pytest tests/test_act_node.py::test_act_propose_only_skips_rust -x` | ❌ W0 | ⬜ pending |
| 04-XX-XX | TBD | TBD | TTP-01 | — | `WhoisTtp` implementa trait `Ttp` (dyn-compatible) | unit | `cargo test --package kri0k-core ttp::whois::tests::implements_trait` | ❌ W0 | ⬜ pending |
| 04-XX-XX | TBD | TBD | TTP-02 | — | `WhoisTtp::execute` invoca subprocess (via mock) e retorna stdout bytes | unit | `cargo test --package kri0k-core ttp::whois::tests::executes_via_mock_subprocess` | ❌ W0 | ⬜ pending |
| 04-XX-XX | TBD | TBD | TTP-02 | — | `WhoisTtp::execute` invoca whois.exe real e parseia output | integration | `cargo test --features integration ttp::whois::tests::real_whois_smoke` | ❌ W0 | ⬜ pending |
| 04-XX-XX | TBD | TBD | TTP-03 | — | Parser extrai `registrar`, `nameservers[]`, `created/updated/expires` da fixture `google.com` (verbose) | unit | `cargo test --package kri0k-core ttp::whois::tests::parses_google_fixture` | ❌ W0 | ⬜ pending |
| 04-XX-XX | TBD | TBD | TTP-03 | — | Parser lida com output GDPR-redacted (`example.com` sem `-v`) — todos campos `None` exceto `raw_unparsed` | unit | `cargo test --package kri0k-core ttp::whois::tests::handles_redacted_output` | ❌ W0 | ⬜ pending |
| 04-XX-XX | TBD | TBD | TTP-04 | M-15 | `Engagement::execute_proposal` adiciona ≥1 Domain + ≥1 Organization + N Nameserver nodes | integration | `pytest tests/test_engagement_smoke.py::test_whois_grows_graph -m integration` | ❌ W0 | ⬜ pending |
| 04-XX-XX | TBD | TBD | TTP-04 | — | Re-execução idempotente (D-43): mesmo target não cria nós duplicados | integration | `pytest tests/test_engagement_smoke.py::test_whois_idempotent -m integration` | ❌ W0 | ⬜ pending |
| 04-XX-XX | TBD | TBD | TTP-05 | M-34 | Rate limit 1 req/s: 2 chamadas consecutivas levam ≥ 1.0s wall-clock | unit | `cargo test --package kri0k-core ttp::whois::tests::rate_limit_enforced` | ❌ W0 | ⬜ pending |
| 04-05-04 | 04-05 | 3 | D-50 | M-36 | `which::which("whois")` (caminho subjacente de `Engagement::new`) falha quando PATH vazio (BLOCKER 4 fix — Rust-side integration test em pybridge usa temp-env para isolar PATH) | integration | `cargo test --package kri0k-pybridge --test engagement_missing_whois` | ✅ W0 | ⬜ pending |
| 04-XX-XX | TBD | TBD | D-51 | — | Timeout 30s mata subprocess; `WhoisTtp::execute` retorna `status: "timeout"` | unit | `cargo test --package kri0k-core ttp::whois::tests::timeout_kills_child` (mock que dorme 60s) | ❌ W0 | ⬜ pending |
| 04-XX-XX | TBD | TBD | D-62 | M-36 | `Engagement::kill()` cancela execução em andamento via CancellationToken | integration | `pytest tests/test_engagement_smoke.py::test_kill_cancels_execute -m integration` | ❌ W0 | ⬜ pending |
| 04-XX-XX | TBD | TBD | D-48 | M-02 | Scope violation rejeitada antes do subprocess; outcome `status: "scope_violation"` | unit | `cargo test --package kri0k-core engagement::tests::scope_violation_short_circuits` | ❌ W0 | ⬜ pending |
| 04-XX-XX | TBD | TBD | D-63 | AB-03 | Target inválido (regex domain fail) rejeitado com `Error::ParseError` antes do subprocess | unit | `cargo test --package kri0k-core engagement::tests::invalid_target_rejected` | ❌ W0 | ⬜ pending |
| 04-XX-XX | TBD | TBD | D-58 | M-02 | `Scope::from_yaml` rejeita `version` desconhecido com erro estruturado | unit | `cargo test --package kri0k-core scope::tests::unknown_version_rejected` | ❌ W0 | ⬜ pending |
| 04-XX-XX | TBD | TBD | D-58 | — | `Scope::from_yaml` parseia todos campos v1 (lookahead), usa apenas `targets` + `safeguards.propose_only` em Phase 4 | unit | `cargo test --package kri0k-core scope::tests::parses_full_v1_schema` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*
*Task IDs ficam `04-XX-XX` até o planner gerar PLAN.md files — o verifier conecta os IDs reais.*

---

## Wave 0 Requirements

Files needed before any verification command can run green:

- [ ] `tests/test_act_node.py` (novo) — cobre AGENT-05 com `Engagement` mockado (`unittest.mock.Mock`)
- [ ] `tests/test_engagement_smoke.py` (novo) — cobre TTP-04 + D-62 com `Engagement` real, gated por marker `integration`
- [ ] `tests/fixtures/whois_google_com.txt` (novo) — captura de `whois -v -nobanner google.com` (output completo verbose)
- [ ] `tests/fixtures/whois_example_com.txt` (novo) — captura de `whois -v -nobanner example.com` (output redacted GDPR)
- [ ] `tests/fixtures/whois_invalid.txt` (novo) — output vazio/malformado para teste de robustez do parser
- [ ] `crates/kri0k-core/tests/scope_yaml_integration.rs` (novo, opcional) ou inline `mod tests` em `scope.rs` — fixtures de scope.yaml parsing
- [ ] `crates/kri0k-core/src/ttp/whois.rs` (novo) — `WhoisTtp` struct + impl
- [ ] `crates/kri0k-core/src/ttp/subprocess.rs` (novo) — `Subprocess` trait + `RealSubprocess` + `MockSubprocess`
- [ ] `crates/kri0k-core/src/ttp/mod.rs` (novo) — module declaration substituindo `ttp.rs`
- [ ] `crates/kri0k-pybridge/src/lib.rs` (modificar) — adicionar `pyclass Engagement`
- [ ] `python/kri0k/_native.pyi` (modificar) — declarar `class Engagement` para mypy strict
- [ ] `python/kri0k/engagement.py` (novo) — bootstrap helper `create(...)`
- [ ] `python/kri0k/agent/nodes/act.py` (modificar) — wire propose_only gate + execute_proposal call
- [ ] `crates/kri0k-core/Cargo.toml` (modificar) — adicionar deps `tokio`, `tokio-util`, `which`, `async-trait`, `tracing`, `serde_yaml_ng`, feature flag `integration`
- [ ] `Cargo.toml` workspace (modificar) — propagar deps para workspace level se necessário
- [ ] No framework install needed — pytest, cargo test e asyncio já estão configurados.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `Engagement::new` falha quando whois.exe não está no PATH | D-50 | Mockar `which::which` requer hijack do PATH global, frágil em CI paralelo. Mais simples como smoke manual. | Em uma máquina sem whois: rode `python -c "from kri0k.engagement import create; create({'version': 1, 'targets': ['example.com']}, objective='test')"` e confirme `RuntimeError: whois binary not found in PATH`. Documentar como passo de install em README. |
| Output do Sysinternals whois.exe difere entre versões | TTP-03 | Não há controle de versão do binário externo. Phase 4 trava em v1.21 com fixtures dessa versão. Versões futuras podem mudar field names. | Periodicamente (release de nova versão Sysinternals): re-capturar fixtures e diff. Documentar em CONTRIBUTING.md. |
| EULA prompt na primeira execução (Sysinternals) | D-50, Pitfall 1 | `-accepteula` flag mitiga, mas em primeiro run absoluto em CI greenfield pode haver delay enquanto registry key é setada. Verificado nesta sessão. | Documentar no README quickstart: primeiro `whois -accepteula -nobanner example.com` manual antes de rodar tests. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies (16/16 mapped above)
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify (todas as 17 entries são automáticas exceto D-50 que é manual smoke)
- [ ] Wave 0 covers all MISSING references (12 files + fixtures listados)
- [ ] No watch-mode flags (todos comandos são one-shot)
- [ ] Feedback latency < 30s for unit (~60s with rate-limited integration)
- [ ] `nyquist_compliant: true` set in frontmatter (após gsd-plan-checker passar)

**Approval:** pending — preencher após gsd-plan-checker aprovar PLAN.md.

---

## Threat Coverage Cross-Ref

Validation tests cover threat mitigations from `docs/security/THREAT_MODEL.md`:

| Threat / Mitigation | Test that verifies | Phase 4 Decision |
|---|---|---|
| M-02 (scope check before exec) | `engagement::tests::scope_violation_short_circuits` | D-48 |
| M-05 (propose-only default) | `tests/test_act_node.py::test_act_propose_only_skips_rust` | D-49 |
| M-15 (LLM never executes direct) | Façade pattern + `engagement::tests::scope_violation_short_circuits` | D-34, D-35 |
| M-34 (TTP rate limit) | `ttp::whois::tests::rate_limit_enforced` | D-45 |
| M-36 (kill switch) | `cargo test --package kri0k-pybridge --test engagement_missing_whois` + `Engagement.kill()` manual smoke | D-50, D-62 |
| AB-03 (prompt injection) | `engagement::tests::invalid_target_rejected` | D-63 |

---

*Phase: 04-act-node-ttp-whois*
*Validation strategy created: 2026-05-18 from RESEARCH.md §Validation Architecture*
