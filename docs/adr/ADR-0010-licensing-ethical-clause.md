# ADR-0010: Licença — cláusula ética suplementar

- **Status:** Proposed (decisão final em card específico)
- **Data:** 2026-05-13
- **Inputs:** T1 §6 (open question), persona D (anti-uso)

## Contexto
Licenciar uma ferramenta de red team agêntico envolve trade-offs:
- OSI puro (MIT/Apache 2.0): máxima adoção, mas permite forks remova-guards.
- Custom non-OSI: pode incluir cláusula ética, mas perde "open source" formal,
  dificulta inclusão em distros e empresas.
- AGPL: força reciprocidade, mas hostil ao uso corporativo legítimo.

## Decisão (proposta)
**Apache 2.0 + cláusula ética suplementar não-OSI** (modelo "Hippocratic
License v3" ou "Anti-996"), comunicada claramente como:
- O **código** é Apache 2.0.
- A licença suplementar é uma cláusula de uso aceitável (AUP) não-OSI; uso em
  violação não invalida copyright mas constitui descumprimento contratual.

## Status atual
**MIT temporário para MVP-0** (decisão de 2026-05-13, T10.1). Repo nasceu com
MIT (T5) como placeholder e permanecerá MIT até card KRK-LICENSE finalizar a
decisão Apache 2.0 + cláusula ética antes do MVP-1 release. Todas as fontes
(LICENSE, Cargo.toml, pyproject.toml, README.md) foram sincronizadas para MIT
com comentários apontando para este ADR e o card KRK-LICENSE.

## Consequências (se aprovada)
- ✅ Comunidade entende que forks removendo guards estão fora do espírito.
- ✅ Sinaliza para auditores corporativos que o projeto leva ética a sério.
- ❌ Pode ser rejeitado por OSI-puristas; impacto em adoção desconhecido.

## Alternativas consideradas
- **Apache 2.0 puro:** simplicidade máxima, mas perde sinal ético explícito.
- **MIT (atual):** mesma crítica.
- **AGPL:** rejeitado, hostil a uso corporativo legítimo (Marina não consegue
  usar em cliente bancário sob AGPL).
