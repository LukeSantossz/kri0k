# Skill: Integração Claude Code + Codex

> Carregar quando o projeto usa integração Codex (verificar `registry.md`).

---

## Princípio

Claude Code é orquestrador primário (fluxo, estado, regras). Codex é ferramenta especializada invocada em momentos definidos. Nenhum opera fora do CRURA.

## Setup

```bash
/plugin marketplace add openai/codex-plugin-cc
/plugin install codex@openai-codex
/reload-plugins
/codex:setup
```

Config local (`.codex/config.toml`):
```toml
model = "gpt-5.4-mini"
model_reasoning_effort = "high"
```

Registrar no `registry.md` (Decisões Técnicas): "Integração Codex ativa. Modelo: [modelo]."

## Comandos

| Comando | Uso | Quando no CRURA |
|---------|-----|-----------------|
| `/codex:review` | Review padrão | Etapa R |
| `/codex:review --background` | Review sem bloquear | Etapa R |
| `/codex:adversarial-review --background [foco]` | Review adversarial | Etapa R — tasks major |
| `/codex:rescue [descrição]` | Delegar bug complexo | Etapa C |
| `/codex:status` | Checar progresso | Após --background |
| `/codex:result` | Ver resultado | Após conclusão |
| `/codex:cancel` | Cancelar job | Quando desnecessário |
| `/codex:setup --enable-review-gate` | Review a cada resposta | Tasks major |
| `/codex:setup --disable-review-gate` | Desativar auto-review | Após concluir major |

## Uso por Complexidade

- **Patch:** Codex não necessário.
- **Minor:** `/codex:review` após avaliação pós.
- **Major:** `/codex:adversarial-review` + review gate opcional.

## Regras

- Resultado do Codex = código de IA → sujeito ao checklist de auto-revisão.
- Findings aceitos → corrigir antes do Upload. Findings rejeitados → registrar justificativa nas Observações.
- Codex não é usado no Modo Tutor (contradiz objetivo pedagógico).

## Registro

No relatório pós-implementação: `✓ Review cruzado (Codex): [ok / findings e resolução | N/A]`

## Anti-Padrões

| Anti-Padrão | Ação |
|-------------|------|
| Codex como implementador primário | Claude Code implementa, Codex revisa |
| Aceitar rescue sem revisão | Sempre revisar diff |
| Review gate sem supervisão | Apenas para major |
| Delegar sem task | Registrar task primeiro |
| Ignorar findings sem justificativa | Registrar concordância ou discordância |
