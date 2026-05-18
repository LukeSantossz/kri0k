# KRK-001 — Kri0K

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/badge/build-TBD-lightgrey)]()

> ⚠️ **DISCLAIMER**: Esta ferramenta é destinada EXCLUSIVAMENTE para uso em ambientes de laboratório controlados, máquinas virtuais isoladas, testes de penetração autorizados e competições CTF. O uso em redes não autorizadas ou ambientes de produção é estritamente proibido e pode ser ilegal. Os usuários são os únicos responsáveis por garantir que possuem autorização explícita antes de executar este software.

## O que isso faz?

Orquestrador autônomo de kill-chains para red team com raciocínio baseado em grafo de estado persistente.

**Funcionalidades planejadas:**

- Orquestração autônoma de kill-chains sem intervenção manual entre etapas
- Grafo de estado de ataque tipado e persistente (serializado como contexto para o LLM)
- Mapeamento automático de ações para TTPs do framework MITRE ATT&CK
- Loop de raciocínio via LangGraph sobre o estado acumulado do ataque
- Módulos de reconhecimento com hickory-dns
- Persistência de estado entre sessões para cadeia de ataque contínua

## O que é?

Biblioteca Rust com CLI e camada de raciocínio em Python via LangGraph. Não é aplicação web, desktop ou mobile — é uma ferramenta de linha de comando para pesquisa em segurança ofensiva.

O núcleo assíncrono em Rust mantém o grafo de estado e executa operações de rede/sistema; a camada Python (LangGraph) raciocina sobre o grafo serializado e decide próximas etapas táticas.

## Quais tecnologias são usadas?

| Camada | Tecnologias |
|--------|-------------|
| Núcleo assíncrono | Rust (Tokio, petgraph, hickory-dns) |
| Interoperabilidade | PyO3 (Rust ↔ Python) |
| Raciocínio / Orquestração | Python (LangGraph) |
| Framework tático | MITRE ATT&CK |

**Stack principal:** Rust + Python via PyO3, com LangGraph para o loop de decisão baseado em estado.

## Qual é a ambição?

Ferramenta de pesquisa para red teamers, CTFs e laboratórios de segurança ofensiva autorizados. **Não é produto comercial** nem destinado a redes não autorizadas.

A ambição é explorar orquestração de ataques com um **grafo de estado persistente** que elimina a dependência exclusiva do histórico de mensagens do LLM — permitindo raciocínio sobre o contexto completo do ataque acumulado, não apenas sobre o último comando executado. Isso reduz fricção operacional e aumenta a profundidade das cadeias exploradas de forma autônoma.

## Qual é o estágio?

**Status: MVP-0 concluído ✅ ([v0.0.1-mvp0](https://github.com/LukeSantossz/kri0k/releases/tag/v0.0.1-mvp0))**

| Estágio | Status | Descrição |
|---------|--------|-----------|
| **MVP-0** | ✅ Concluído | Scaffold Rust+Python, estrutura do repositório, interop PyO3 base, safeguards stubs |
| **MVP-1** | ⏳ Próximo | Agente LangGraph operando sobre o grafo serializado, mapeamento ATT&CK |
| **Visão completa** | ⏳ Planejado | Persistência de estado entre sessões, módulos recon hickory-dns, expansão de TTPs |

**O que está pronto (v0.0.1-mvp0):**

- Repositório público configurado (MIT License, .gitignore Rust+Python)
- Scaffold do núcleo Rust (Tokio runtime, petgraph para grafo de estado)
- Estrutura PyO3 para interoperabilidade Rust/Python
- README com DISCLAIMER de uso responsável e badges
- **Framework de safeguards de segurança** (stubs com TODOs para T7):
  - Validação de escopo
  - Verificações de permissão
  - Rate limiting
  - Activity logging
- **Quality gates aprovados**:
  - ✅ cargo build (clean)
  - ✅ cargo test (10/10 pass)
  - ✅ cargo clippy --deny warnings (clean)
  - ✅ cargo fmt --check (clean)
- SECURITY.md com política de disclosure

**O que está pendente:**

- Implementação do agente LangGraph (MVP-1)
- Serialização do grafo de estado e injeção no prompt do LLM
- Mapeamento de saídas para MITRE ATT&CK
- Módulos de reconhecimento via hickory-dns
- Persistência de estado entre sessões
- Testes cross-language (Rust ↔ Python)

## Problemas conhecidos / limitações

**Restrições de escopo:**

- ⚠️ **APENAS para ambientes autorizados:** labs, VMs isoladas, CTFs. **NÃO** usar em produção ou redes sem autorização explícita.
- ⚠️ **Ferramenta dual-use:** O software pode ser usado tanto para defesa (red team em exercícios autorizados) quanto para fins maliciosos. Uso inadequado é responsabilidade exclusiva do operador.

**Limitações técnicas:**

- **Interop Rust↔Python via PyO3:** Ainda não validada sob carga. Possíveis gargalos de serialização/deserialização em grafos grandes.
- **Ausência de testes cross-language:** A integração entre camadas Rust e Python ainda não possui suíte de testes end-to-end.
- **Performance do grafo:** petgraph funciona bem para grafos médios, mas escalabilidade ainda não foi testada para kill-chains muito longas.

**Roadmap:**

1. Validar interop PyO3 com benchmarks de carga
2. Implementar suite de testes Rust+Python integrada
3. Adicionar limitações de profundidade/tempo para evitar loops infinitos no agente
4. Documentar práticas de uso responsável e checklists de autorização pré-execução

---

## Running the whois TTP

Phase 4 entrega a primeira TTP concreta do kri0k: `T1590.001` (whois reconnaissance via subprocess).

### Pré-requisito

- **Windows:** `winget install Microsoft.Sysinternals.Whois`
- **Linux:** `apt install whois`
- **Primeira execução:** rode `whois -accepteula -nobanner example.com` uma vez para aceitar a EULA do Sysinternals. Subsequentes não prompt.

`Engagement::new()` faz fail-fast com `RuntimeError` se o binário não estiver no `PATH`.

### scope.yaml mínimo

Veja `config/scope.example.yaml`. Para uma execução real (não propose-only):

```yaml
version: 1
operator: you@example.com
targets:
  - example.com
safeguards:
  propose_only: false
```

### Exemplo end-to-end (Python)

```python
import asyncio
import yaml

from kri0k.agent import get_graph
from kri0k.engagement import create
from kri0k.agent.state import AgentState


async def main() -> None:
    with open("config/scope.example.yaml") as f:
        scope_dict = yaml.safe_load(f)

    context = create(scope_dict, objective="recon example.com", propose_only=False)
    graph = get_graph()
    state: AgentState = {
        "snapshot": {"raw": {}, "formatted": ""},
        "analysis": {},
        "proposal": {"ttp_id": "T1590.001", "target": "example.com"},
        "decision": {},
        "iteration_count": 1,
        "history": [],
        "engagement_context": context,
    }
    result = await graph.ainvoke(state)
    print(result["decision"])


if __name__ == "__main__":
    asyncio.run(main())
```

### Limitações conhecidas (Phase 4)

- Allowlist exact-match domínio apenas (D-48). CIDR e wildcards (`*.example.com`) chegam em Phase 7.
- Audit log é no-op (D-38). Hash-chained JSONL chega em Phase 8.
- Sem TUI interativa. Aprovação via flag `propose_only`. Keybindings TUI chegam em Phase 11.
- TLDs não-ICANN (`.br`, `.uk`) podem ter parse parcial (D-41). Ver `WhoisOutput.raw_unparsed`.

---

## Inspirações

- Arquiteturas de agentes com grafo de estado persistente (LangGraph)
- Frameworks de red team: Metasploit, CALDERA
- MITRE ATT&CK como estrutura taxonômica de referência

## License

MIT License - see [LICENSE](LICENSE) for details.
