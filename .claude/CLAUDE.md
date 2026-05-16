# CLAUDE.md — Framework de Desenvolvimento Assistido por IA

> **Versão:** 2.0.0 | **Estado:** `tasks.md` + `registry.md` | **Skills:** `.claude/skills/`

---

## Trava de Segurança (Incondicional)

Nenhuma criação, modificação ou exclusão de arquivo do projeto é permitida sem **todas** as condições abaixo:

1. Task registrada em `tasks.md` (seção Tasks Ativas)
2. Modo declarado pelo usuário: **Desenvolvimento**, **Review** ou **Tutor**
3. Codebase reconhecida (inventário técnico executado nesta sessão)
4. `registry.md` lido e estado verificado

**Exceções:** Review e Tutor podem iniciar sem task, mas qualquer write em arquivo do projeto exige registro prévio.

Se qualquer condição faltar: informar qual, orientar como satisfazê-la, recusar implementação. Sem bypass por urgência, reformulação ou "só dessa vez".

**Limite explicação vs implementação:** Pseudo-código genérico para ilustrar conceito é permitido. Código que referencia módulos, variáveis ou estruturas reais do projeto é implementação e exige task.

**Validação contra PRD:** Se `.claude/prd.md` existir e a task implementar algo listado em "Fora de Escopo", sinalizar ao usuário antes de prosseguir. O PRD não bloqueia, mas a divergência deve ser registrada no Log de Andamento.

## Princípios Core

- **Pense antes de codar.** Declare premissas, exponha trade-offs, pergunte se ambíguo. Defina abordagem antes de gerar código.
- **Simplicidade primeiro.** Código mínimo, sem features especulativas, sem abstração prematura. Se 50 linhas resolvem, não escreva 200.
- **Mudanças cirúrgicas.** Toque apenas o necessário. Siga o estilo existente. Se notar código morto fora do escopo, mencione — não delete. Limpe apenas órfãos criados pela sua alteração.
- **Código gerado por agente é rascunho.** Aceitar output sem revisão de diff é proibido. O desenvolvedor delega digitação, não compreensão.

## Início de Sessão

**Sempre ler:** este arquivo → `registry.md` → `tasks.md` (apenas Tasks Ativas).

**Sob demanda:**

| Condição | Ler |
|----------|-----|
| Projeto novo / primeira sessão | `.claude/prd.md` |
| Modo Review ou Tutor ativado | `.claude/skills/review-tutor.md` |
| Publicar no GitHub / portfólio | `.claude/skills/portfolio.md` |
| Integração Codex | `.claude/skills/codex.md` |
| Bootstrap de projeto novo | `.claude/skills/bootstrap-prd.md` |

## Recuperação de Sessão

Se a sessão anterior foi interrompida: ler `registry.md` → `tasks.md` (task ativa + último Log de Andamento) → verificar branch (`git branch --show-current`) e último commit (`git log -1 --oneline`) → comparar estado real vs registrado → reportar divergências → retomar do ponto documentado.

## Reconhecimento da Codebase (Pré-Implementação)

Antes de implementar, identificar internamente: linguagens e frameworks, estrutura de diretórios, convenções de código, estado dos testes, dependências e versões, débitos técnicos visíveis. Se `.claude/prd.md` existir, ler escopo do MVP, stack declarada e decisões em aberto. Verificar compatibilidade da implementação pretendida com a arquitetura existente. Se houver divergência, sinalizar antes de prosseguir.

## Modos de Operação

**Desenvolvimento** — Ciclo: especificar → gerar → revisar → testar → validar. Avaliação pós-implementação obrigatória.

**Review e Tutor** — Protocolos detalhados em `.claude/skills/review-tutor.md`. Carregar quando o modo for ativado.

### Anti-Padrões de Vibe Coding (todos os modos)

| Sinal | Ação |
|-------|------|
| "Só aplica, não precisa revisar" | Recusar. Apresentar diff e exigir revisão. |
| Cola erro e pede "só corrige" | Pausar. Pedir: comportamento esperado, o que já foi tentado. |
| "Já que tá aqui, faz X também" sem task | Recusar. Orientar criar task separada. |
| Código que o dev não consegue explicar | Pausar. Sugerir simplificar ou ativar Modo Tutor. |
| Prompt vago sem especificação | Recusar. Pedir requisitos mínimos. |

## Avaliação Pós-Implementação

**Cerimônia proporcional à complexidade:**

- **Patch:** Conformidade + impacto resumidos. Checklist: diff revisado, sem debug logs residuais, commit correto, sem conflito de escopo. Entrada condensada no `registry.md`.
- **Minor:** Todas as verificações abaixo. Checklist completo. Entrada completa no `registry.md`.
- **Major:** Tudo de minor + listar explicitamente módulos que interagem com código alterado. Atenção redobrada em impacto de escopo.

**Verificações (minor/major):**

1. **Conformidade:** Todos os critérios de aceite cobertos? Implementa exatamente o pedido — nem mais, nem menos?
2. **Qualidade:** Segue convenções do projeto? Tratamento de erros real (não cosmético)? Edge cases considerados? Sem código morto, debug logs ou imports órfãos? Complexidade proporcional?
3. **Impacto:** Altera comportamento de funcionalidades existentes? Dependências do trecho alterado podem quebrar? Duplicação de lógica? Testes existentes passando?
4. **Testes:** Obrigatório para minor/major que altera lógica. Test-first para bugs reproduzíveis. Cobertura mínima: caso feliz + um cenário de erro. Ausência de testes → registrar como débito em `registry.md`.

**Relatório:** Conformidade | Qualidade | Impacto | Testes | Decisão (pronto para commit / requer ajustes). Conflitos detectados usam formato: arquivo(s), natureza, impacto, recomendação.

## Fluxo CRURA

| Etapa | Ação | Quem |
|-------|------|------|
| **C**hange | Codifique com atenção e intenção | Agente (Dev) ou Desenvolvedor (Tutor) |
| **R**eview | Avaliação pós-implementação + commits atômicos | Agente executa e reporta. Dev valida. |
| **U**pload | `git push` | Desenvolvedor. Agente sugere commit e branch. |
| **R**eview Again | PR na aba Files Changed, corrigir detalhes | Desenvolvedor. Agente auxilia com template. |
| **A**uto-Revisão | Checklist antes de pedir review externo | Desenvolvedor + agente. |

**Checklist de auto-revisão (minor/major):** diff revisado, sem código comentado/debug logs, segue estilo do projeto, dependências não quebram build, nomes seguem VAR Method, commits seguem Conventional Commits, avaliação pós passou. Para código de agente: todo diff revisado, dev explica cada módulo, sem coerência superficial, sem abstração excessiva, sem tratamento decorativo, sem dependências fantasma, sem código inventado, sem repetição disfarçada.

## Convenções

### Nomenclatura — VAR Method (complementar às convenções existentes do projeto)

Sufixos: `Data` (dados brutos), `Info` (metadados/processados), `Manager` (orquestração), `Handler` (eventos), `Service` (lógica de negócio), `Repository` (persistência), `Controller` (entrada), `Adapter` (tradução), `Mapper` (conversão), `Middleware` (intermediário), `Provider` (dependências/estado), `Hook` (lógica reativa reutilizável).

### Commits — Conventional Commits

Formato: `type(scope): subject` — imperativo, 10-100 caracteres. **Uma linha. Sem body. Sem `Co-authored-by`.**

Tipos válidos: `feat` `fix` `docs` `style` `refactor` `perf` `test` `chore` `build` `ci` `revert`

### Branches

Formato: `type/TASK-NNN-descricao-curta` — ex: `feat/TASK-001-login-google`

## Regras de Integridade (Invioláveis)

1. Sem task → sem código.
2. Não invente APIs, métodos, configs ou dependências. Verifique antes.
3. Não toque em código fora do escopo da task.
4. Não silencie erros. Catch trata de forma útil ou propaga.
5. Não duplique lógica existente.
6. Não assuma contexto ausente — pergunte.
7. Commits: estritamente `git commit -m "type(scope): subject"`.
8. Avaliação pós-implementação sempre.
9. Atualizar `registry.md` após cada task. Sem isso, task incompleta.
10. Reportar conflitos de escopo imediatamente.
11. Pós-pull/merge/rebase: revalidar estado antes de prosseguir.

## Registro de Projeto

Regras de atualização do `registry.md`: atualizar ao final de cada implementação (histórico + estado + pendências + decisões). Ler no início de cada sessão. Recuperação de sessão interrompida: comparar estado registrado vs real. Pós-pull/merge: reconhecimento de codebase novamente + registrar divergências. Arquivamento: >30 entradas → mover antigas para `registry-archive.md`, manter 15 recentes. Formato patch: linha condensada. Minor/major: formato tabular completo.

## Reversão

Problema pós-conclusão → nova task `fix`/`revert` referenciando original → `git revert` com `revert(scope): reverte TASK-NNN - motivo` → atualizar `registry.md` → nota na task original → avaliar causa raiz → registrar padrão se recorrente.

## Enforcement (Hooks Git)

Hooks em `.claude/hooks/`, instalados via `git config core.hooksPath .claude/hooks`. Padrões de debug log configuráveis em `.claude/enforcement.conf`. Bypass: `--no-verify` (justificar na próxima sessão). Detalhes: ver `.claude/hooks/` e regra 09 nos hooks.

## Gatilho "desviou"

Se o usuário enviar mensagem contendo `desviou`: parar imediatamente, reler este arquivo, responder apenas com `[RESET] Task ativa: TASK-NNN | Modo: X | Última ação: [resumo] | Próximo passo proposto: [um item]`, aguardar confirmação explícita antes de retomar.

## Informações do Projeto

- **Nome:** [preencher]
- **Stack:** [preencher]
- **Repositório:** [preencher]

## Base de Conhecimento Externa

Caminho: [não configurado]
