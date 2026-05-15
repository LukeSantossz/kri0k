# External Integrations

**Analysis Date:** 2026-05-15

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
- Node types: `Host` (ip), `Network` (cidr), `Service` (port, protocol), `Finding` (description)
- Edge types: `BelongsTo`, `RunsOn`, `RelatesTo` (relation)
- Persistence: JSONL snapshots on engagement close
- Location: `engagement/<id>/` directory structure (planned)

**Agent State (Phase 1):**
- In-memory: `AgentState` TypedDict passed between LangGraph nodes
- Fields: `snapshot`, `analysis`, `proposal`, `decision`, `iteration_count`, `history`, `engagement_context`
- Persistence: Not yet implemented (planned for future phases)

**Audit Log:**
- Format: Append-only hash-chained JSONL (ADR-0007)
- Location: `engagement/<id>/audit.jsonl`
- Fields: `ts`, `audit_id` (ULID), `kind`, `payload`, `prev_hash`, `hash`
- Not encrypted by default
- Implementation: Stub in `crates/kri0k-core/src/audit.rs`

**Scope Configuration:**
- Format: YAML
- Location: `config/scope.yaml` (user-provided)
- Example: `config/scope.example.yaml`
- SHA256 checksum embedded in all snapshots (ADR-0011)
- Implementation: Stub in `crates/kri0k-core/src/scope.rs`

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
- Implementation: `validate_target()` in `crates/kri0k-core/src/scope.rs` (stub)

## Monitoring & Observability

**Tracing:**
- `tracing` crate (Rust) - Structured logging infrastructure (planned)
- OTLP export: Planned (subscriber pluggable)
- Audit log separate from tracing (forensics vs debug)

**Error Tracking:**
- None (no external service)
- Errors captured in audit log

**Logs:**
- Rust: `tracing` crate (planned)
- Python: Standard logging (via LangGraph)
- Audit: Separate append-only JSONL

## CI/CD & Deployment

**Hosting:**
- Local/on-premise (air-gapped capable per ADR-0008)
- No cloud deployment configured

**CI Pipeline:**
- Pre-commit hooks via `pre-commit` (>=3.8)
  - cargo-fmt, cargo-clippy, cargo-test-unit
  - ruff lint, ruff format, mypy
  - pytest-unit
  - detect-secrets
- Cargo clippy with `-D warnings` (deny all warnings)
- Ruff + mypy for Python

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
- `.env` files in `.gitignore`

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
- Type stubs: `python/kri0k/_native.pyi`

**Exposed Functions:**
- `hello() -> str` - Initialization confirmation
- `get_dummy_graph() -> dict[str, Any]` - Test graph structure with nodes/edges

**Data Exchange:**
- JSON serialization for snapshots
- Python dicts for graph structures
- Compact JSON keys for token efficiency (ADR-0003)

**Runtime:**
- Global Tokio runtime initialized on module load
- 2 worker threads named `kri0k-tokio`
- `OnceLock<tokio::runtime::Runtime>` for lazy initialization

## LangGraph Integration (Phase 1)

**Graph Structure:**
- Location: `python/kri0k/agent/graph.py`
- Entry: `get_graph()` returns compiled `StateGraph[AgentState]`
- Nodes: sense, reason, plan, act, reflect (linear sequence)
- Edges: START -> sense -> reason -> plan -> act -> reflect -> (conditional)
- Conditional routing: `route_after_reflect()` checks `iteration_count >= MAX_ITERATIONS`

**State Flow:**
- State TypedDict flows through all nodes
- Each node returns partial state updates (merged by LangGraph)
- `reflect` node increments `iteration_count`
- Loop terminates when `iteration_count >= 10`

**Async Execution:**
- All node functions are `async def`
- Compatible with pytest-asyncio for testing

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

*Integration audit: 2026-05-15*
