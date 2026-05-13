# ADR-0012: TTP trait + adapter externo com timeout e cancelation

- **Status:** Accepted
- **Data:** 2026-05-13
- **Inputs:** T2/T4 (MVP-1 TTP list), T1 §3.4 (validador antes de exec)

## Contexto
TTPs precisam ser:
- Tipados (assinatura clara de args e output).
- Cancelaveis (kill switch).
- Auditáveis (cada execução gera evento no audit log).
- Plug-and-play (adicionar novo TTP não toca o core).

## Decisão
Trait `Ttp` em `kri0k-ttp`:

```rust
#[async_trait::async_trait]
pub trait Ttp: Send + Sync {
    fn id(&self) -> &'static str;                       // "T1046"
    fn destructive(&self) -> bool { false }
    fn args_schema(&self) -> &'static schemars::Schema;
    fn default_timeout(&self) -> Duration { Duration::from_secs(300) }

    async fn run(
        &self,
        ctx: &TtpCtx,
        target: &Target,
        args: &serde_json::Value,
    ) -> Result<TtpOutput, TtpError>;
}

pub struct TtpCtx {
    pub scope: Arc<ScopeSnapshot>,
    pub audit: AuditHandle,
    pub kill: CancellationToken,    // tokio_util
    pub deadline: Instant,
}

pub struct TtpOutput {
    pub new_nodes: Vec<NodeBuilder>,
    pub new_edges: Vec<EdgeBuilder>,
    pub findings: Vec<Finding>,
    pub raw_log: Option<Bytes>,     // saída crua opcional do tool externo
}
```

- Registro: `inventory::submit!` para auto-descoberta.
- Adapter para tools externos (`nmap`, `dig`, `whois`): wrapper em
  `kri0k-ttp::adapters::cmd` que respeita `kill` e `deadline` via
  `tokio::process::Child` + `select!`.

## Consequências
- ✅ Kill switch realmente para tudo (CancellationToken propaga).
- ✅ Timeout enforced no core, não no adapter (uniform).
- ✅ Adicionar TTP = um arquivo + `inventory::submit!` + entrada em
  `enabled_ttps` do scope.

## Alternativas consideradas
- **Subprocess sync (`std::process`):** rejeitado, bloqueia runtime.
- **Plugins dinâmicos (.so/.dll):** rejeitado para MVP, supply chain risk.
