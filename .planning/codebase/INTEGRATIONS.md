# External Integrations

**Analysis Date:** 2025-05-14

## APIs & External Services

**LLM Providers:**
- Ollama (default, local-first per ADR-0008)
  - SDK/Client: `ollama` Python package (>=0.3.0)
  - Endpoint: `http://localhost:11434`
  - Auth: None (local service)
  - Models: qwen3:32b, deepseek-r1:32b (recommended)

- Anthropic (opt-in)
  - SDK/Client: `anthropic` Python package (>=0.39.0)
  - Auth: API key via env var (standard Anthropic SDK pattern)
  - Warning displayed when used (per ADR-0008)

- OpenAI (opt-in)
  - SDK/Client: `openai` Python package (>=1.54.0)
  - Auth: API key via env var (standard OpenAI SDK pattern)
  - Can point to local OpenAI-compatible endpoints

**Reconnaissance Tools (TTP Adapters, planned per ADR-0012):**
- nmap - Port scanning (T1046)
  - Adapter: `kri0k-ttp::adapters::cmd` (subprocess wrapper)
  - Communication: `tokio::process::Child`
  - Timeout/cancellation: via `CancellationToken`

- dig - DNS enumeration (T1590.002)
  - Adapter: External command or hickory-dns (planned)

- whois - Domain registration lookup (T1590.001)
  - Adapter: External command wrapper

- crt.sh - Certificate transparency (T1596.003)
  - Adapter: HTTP client (reqwest+rustls planned)

## Data Storage

**Graph State:**
- In-memory: `petgraph::StableGraph` wrapped in `kri0k-graph::Graph`
- Persistence: JSONL snapshots on engagement close
- Location: `engagement/<id>/` directory structure (planned)

**Audit Log:**
- Format: Append-only hash-chained JSONL (ADR-0007)
- Location: `engagement/<id>/audit.jsonl`
- Fields: `ts`, `audit_id` (ULID), `kind`, `payload`, `prev_hash`, `hash`
- Not encrypted by default

**Scope Configuration:**
- Format: YAML
- Location: `config/scope.yaml` (user-provided)
- Example: `config/scope.example.yaml`
- SHA256 checksum embedded in all snapshots (ADR-0011)

**File Storage:**
- Local filesystem only
- No cloud storage integration

**Caching:**
- None (state is authoritative in Rust, snapshots are read-only views)

## Authentication & Identity

**Auth Provider:**
- Custom (no external auth service)
- Operator identification via `operator` field in scope.yaml
- Human gate tokens for destructive TTPs (generated at runtime)

**Scope Authorization:**
- `scope.yaml` defines authorized targets (CIDR notation)
- Target validation enforced in Rust before any TTP execution
- Scope hash propagates through all snapshots

## Monitoring & Observability

**Tracing:**
- `tracing` crate (Rust) - Structured logging infrastructure
- OTLP export: Planned (subscriber pluggable)
- Audit log separate from tracing (forensics vs debug)

**Error Tracking:**
- None (no external service)
- Errors captured in audit log

**Logs:**
- Rust: `tracing` crate
- Python: Standard logging (via LangGraph)
- Audit: Separate append-only JSONL

## CI/CD & Deployment

**Hosting:**
- Local/on-premise (air-gapped capable per ADR-0008)
- No cloud deployment configured

**CI Pipeline:**
- Pre-commit hooks via `pre-commit` (>=3.8)
- Cargo clippy with `-D warnings` (deny all warnings)
- Ruff + mypy for Python
- Cargo test + pytest for testing

**Build:**
- maturin builds PyO3 extension
- Release profile: LTO thin, single codegen unit, stripped symbols

## Environment Configuration

**Required env vars:**
- None strictly required for basic operation

**Optional env vars:**
- `KRI0K_LLM` - LLM provider selection (e.g., `ollama:qwen3-32b`, `anthropic:...`)
- Provider-specific API keys when using remote LLMs

**User config:**
- `~/.kri0k/config.toml` - LLM provider settings (documented in architecture)

**Secrets location:**
- API keys in environment variables (not committed)
- No secrets checked into repository

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None (all integrations are client-initiated)

## Cross-Language Bridge (PyO3)

**Interface:**
- Module: `kri0k._native` (cdylib)
- Build: maturin with `pyo3/extension-module` feature
- GIL handling: Released during graph operations (`py.allow_threads()`)

**Data Exchange:**
- JSON serialization for snapshots
- Python dicts for graph structures
- Compact JSON keys for token efficiency (ADR-0003)

**Runtime:**
- Global Tokio runtime initialized on module load
- 2 worker threads named `kri0k-tokio`

## Planned Integrations (MVP-1+)

**TTP Registry:**
- `inventory` crate for auto-discovery of TTP implementations
- Each TTP registers via `inventory::submit!`

**DNS Resolver:**
- hickory-dns (embedded resolver for T1590.002, T1596.001)
- Replaces external `dig` command

**HTTP Client (Rust):**
- reqwest + rustls for HTTPS requests (crt.sh, etc.)

**UI Extensions:**
- ratatui - TUI interface (planned)
- axum + SSE - Web interface (planned)
- Events via `kri0k-core::events` broadcast channel

---

*Integration audit: 2025-05-14*
