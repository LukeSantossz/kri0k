# KRK-001 — Kri0K

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/badge/build-TBD-lightgrey)]()

> ⚠️ **DISCLAIMER**: This tool is intended EXCLUSIVELY for use in controlled laboratory environments, isolated virtual machines, authorized penetration testing engagements, and CTF competitions. Use in unauthorized networks or production environments is strictly prohibited and may be illegal. Users are solely responsible for ensuring they have explicit authorization before running this software.

## O quê

O Kri0K (KRK-001) é um orquestrador autônomo de kill chain para red team, construído sobre um núcleo assíncrono em Rust com uma camada de raciocínio em Python via LangGraph. O sistema mantém um grafo de estado de ataque tipado e persistente — serializado e fornecido como contexto ao LLM a cada iteração do loop de raciocínio — eliminando a dependência exclusiva do histórico de mensagens para manter coerência tática entre etapas.

A saída do sistema é mapeada para TTPs do framework MITRE ATT\&CK, permitindo rastreabilidade formal de cada ação executada pelo agente dentro da cadeia de ataque.

---

## Pra quem

Red teamers, pesquisadores de segurança ofensiva e equipes de CTF que operam em ambientes de laboratório controlado, VMs isoladas ou competições. O sistema não é destinado a ambientes de produção ou redes não autorizadas.

---

## Por quê

Ferramentas de red team existentes exigem intervenção humana constante para encadear etapas táticas. O Kri0K resolve a ausência de um orquestrador que raciocine sobre o estado acumulado do ataque — não apenas sobre o último comando executado — e tome decisões de próximo passo com base no grafo completo, reduzindo fricção operacional e aumentando a profundidade das cadeias exploradas.

---

## MVP vs. Visão Completa

|Escopo|Conteúdo|
|-|-|
|**MVP-0**|Scaffolding do núcleo Rust com runtime Tokio, integração PyO3 para interop com Python, estrutura base do grafo com petgraph.|
|**MVP-1**|Agente LangGraph operando sobre o grafo serializado com mapeamento de saída para ATT\&CK.|
|**Visão completa**|Persistência de estado entre sessões, módulos de reconhecimento via hickory-dns, expansão do catálogo de TTPs cobertos pelo agente.|

---

## Restrições

* **Stack obrigatória:** Rust (Tokio, hickory-dns, PyO3, petgraph) e Python (LangGraph).
* **Escopo de execução:** Restrito a ambientes isolados — lab, VM, CTF. Sem suporte a redes não autorizadas.
* **Infraestrutura:** Sem dependência de cloud para execução do núcleo.
* **Restrição arquitetural:** Interop Rust/Python via PyO3 é não negociável.

---

## Inspirações e Referências

* Arquiteturas de agentes com grafo de estado persistente (LangGraph nativo).
* Frameworks de red team: Metasploit, CALDERA.
* Modelo de raciocínio sobre estado acumulado presente em sistemas de planejamento autônomo.
* MITRE ATT\&CK como estrutura taxonômica de referência.

---

## O que já existe

Pura ideação por hora

---

## License

MIT License - see [LICENSE](LICENSE) for details.
