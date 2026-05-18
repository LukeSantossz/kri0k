# Phase 4: Act Node + TTP Whois — Pattern Map

**Mapped:** 2026-05-18
**Files analyzed:** 22 (10 Rust source, 4 Python source, 5 Python tests/fixtures, 4 docs/config) — net new + modified
**Analogs found:** 17 strong / 22 — 5 files have **NO ANALOG** and Phase 4 establishes the canonical pattern

> **Reading order para o executor.** Esta phase introduz três padrões inéditos no projeto: (a) trait Rust + impl concreto + abstração de subprocess, (b) `#[pyclass]` *stateful* com `Mutex<Graph>` e `Box<dyn Trait>` por trás de `block_on`, (c) helper Python que materializa `engagement_context`. Sempre que houver "NO ANALOG", a seção documenta o shape canônico extraído do `04-RESEARCH.md` para que Phase 5+ tenha referência local.

---

## File Classification

| Path (new ✚ / modify ✎) | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| ✎ `crates/kri0k-core/Cargo.toml` | manifest (deps + features) | config | (none — pure manifest edit) | manifest-edit |
| ✎ `crates/kri0k-core/src/lib.rs` | error taxonomy + module declaration | data-shape | `crates/kri0k-core/src/lib.rs:17-29` (current `Error` enum) | exact (extend in place) |
| ✚ `crates/kri0k-core/src/scope.rs` (replace stub) | config parser + validator | file-I/O + request-response | `python/kri0k/llm/config.py:16-69` (`LLMConfig.from_scope_dict`) | role-match (cross-language analog) |
| ✎ `crates/kri0k-core/src/audit.rs` | trait + no-op impl (rename `NoOpAuditSink` → `NoopAuditSink`) | event-driven | `crates/kri0k-core/src/audit.rs:83-108` (existing `NoOpAuditSink`) | exact (rename only) |
| ✚ `crates/kri0k-core/src/ttp/mod.rs` (was `ttp.rs`) | async trait + types | request-response | `crates/kri0k-core/src/ttp.rs:13-92` (current sync trait skeleton) | role-match (sync→async refactor) |
| ✚ `crates/kri0k-core/src/ttp/subprocess.rs` | trait + Real/Mock impl | streaming (stdout/stderr) | `python/kri0k/llm/protocol.py:15-32` + `python/kri0k/llm/ollama.py:82-154` (Protocol + concrete provider) | **NO ANALOG IN RUST** — Phase 4 establishes pattern |
| ✚ `crates/kri0k-core/src/ttp/whois.rs` | TTP impl + parser + struct | request-response + transform | (Ttp trait skeleton has zero impls) | **NO ANALOG IN RUST** — Phase 4 establishes pattern |
| ✎ `crates/kri0k-graph/src/lib.rs` | enum extension (NodeKind/EdgeKind) | data-shape | `crates/kri0k-graph/src/lib.rs:9-49` (existing `NodeKind`/`EdgeKind`) | exact (variant addition) |
| ✎ `crates/kri0k-pybridge/Cargo.toml` | manifest (deps) | config | (none — pure manifest edit) | manifest-edit |
| ✚ `crates/kri0k-pybridge/src/lib.rs` (extend with `Engagement` pyclass) | stateful pyclass + bridge methods | request-response | `crates/kri0k-pybridge/src/lib.rs:34-78` (`get_dummy_graph` — stateless pyfunction) | role-match (stateless → stateful) |
| ✚ `python/kri0k/_native.pyi` (extend with `class Engagement`) | type stub | data-shape | `python/kri0k/_native.pyi:1-23` (existing `hello`/`get_dummy_graph` stubs) | exact (extend in place) |
| ✚ `python/kri0k/engagement.py` (new) | bootstrap factory | request-response | `python/kri0k/llm/__init__.py:28-37` (`build_provider` factory) | role-match (factory pattern) |
| ✎ `python/kri0k/agent/nodes/act.py` (rewrite) | LangGraph node — gate + dispatch | event-driven (gated) | `python/kri0k/agent/nodes/reason.py:32-83` (engagement_context lookup + async) | role-match (Phase 3 node pattern) |
| ✎ `python/kri0k/agent/nodes/sense.py` (modify) | LangGraph node — backward-compat fallback | request-response | `python/kri0k/agent/nodes/sense.py:19-35` (current) | exact (extend with fallback) |
| ✚ `tests/test_act_node.py` (new) | unit tests with mocked engagement | test | `tests/test_reason_node.py:9-122`, `tests/test_plan_node.py:9-150` (Mock provider + asyncio + `_make_state`) | exact (Phase 3 test pattern) |
| ✚ `tests/test_engagement_smoke.py` (new) | integration tests | test (integration-marker) | `tests/test_llm_ollama.py:1-275` (integration via `MockTransport`) + `tests/test_sense_node.py:1-77` (integration marker + real `_native`) | role-match (hybrid: integration marker + real PyO3) |
| ✚ `tests/fixtures/whois_google_com.txt` | static fixture (capture) | data-shape | (no fixture dir exists yet) | **NO ANALOG** — Phase 4 creates `tests/fixtures/` |
| ✚ `tests/fixtures/whois_example_com.txt` | static fixture (capture) | data-shape | (no fixture dir exists yet) | **NO ANALOG** — same |
| ✚ `tests/fixtures/whois_invalid.txt` | static fixture (synthetic) | data-shape | (no fixture dir exists yet) | **NO ANALOG** — same |
| ✎ `config/scope.example.yaml` | YAML example | config | `config/scope.example.yaml:1-36` (current) | exact (rewrite to v1 schema) |
| ✚ `docs/adr/ADR-0013-ttp-trait-subprocess-abstraction.md` | architecture decision | docs | `docs/adr/ADR-0012-ttp-trait-adapters.md:1-62` (existing ADR template) | exact (ADR template) |
| ✎ `README.md` / ✎ `CONTRIBUTING.md` / ✚ `CHANGELOG.md` | docs | docs | (no canonical analog; README exists, CHANGELOG new) | partial (text additions only) |

> **Categoria "manifest-edit" e "docs"** não recebem extração de excerpt — esses files são modificações textuais cobertas pelo File-by-File Manifest do RESEARCH.md (linhas 1076-1390). Pattern Map foca em padrões de código.

---

## Pattern Assignments (Rust)

### `crates/kri0k-core/src/lib.rs` (extend `Error` enum)

**Analog:** `crates/kri0k-core/src/lib.rs:17-29` (current `Error` enum)

**Current shape (DO NOT remove `Json`/`Generic`):**
```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Generic(String),
}
pub type Result<T> = std::result::Result<T, Error>;
```

**Differences to apply (D-53 + D-62):**
- Add 7 new variants in the same style: `ScopeViolation { target: String, reason: String }`, `RateLimitExceeded { ttp_id: String, retry_in_ms: u64 }`, `SubprocessTimeout { ttp_id: String, timeout_ms: u64 }`, `ParseError { source: String, detail: String }`, `MissingDependency { binary: String }`, `UnknownTtp { ttp_id: String }`, `Io(#[from] std::io::Error)`, `Cancelled`.
- Keep `#[error("...")]` attributes — they're the only public-facing strings.
- Change `pub mod ttp;` to remain as-is (the file becomes `src/ttp/mod.rs`, Rust resolves the module without manifest change). Verify by `cargo check --package kri0k-core` after rename.

**Risk / landmine:**
- `Io(#[from] std::io::Error)` only works if NO other variant already wraps `std::io::Error` via `From` — verified: current `Json(#[from] serde_json::Error)` is the only `#[from]`. Adding `Io` is safe.
- Variant order in the enum affects nothing functional but DO put `Cancelled` last to match the convention "transient/runtime errors at the bottom."

---

### `crates/kri0k-core/src/scope.rs` (REPLACE STUB)

**Analog (cross-language — Python config dataclass):** `python/kri0k/llm/config.py:16-69`

**Excerpt to mimic (shape + validation pattern):**
```python
@dataclass(frozen=True, slots=True)
class LLMConfig:
    provider: Literal["ollama"] = "ollama"
    model: str = "deepseek-r1:32b"
    base_url: str = "http://localhost:11434"
    # ...

    @classmethod
    def from_scope_dict(cls, scope: Mapping[str, Any]) -> "LLMConfig":
        llm_block = scope.get("llm") or {}
        if not isinstance(llm_block, Mapping):
            raise TypeError(...)
        unknown = set(llm_block.keys()) - _ALLOWED_SCOPE_KEYS
        if unknown:
            raise ValueError(f"Unknown llm config key: {key}")
        # ...
        return cls(model=model)
```

**Why this Python file is the analog:** It demonstrates the project's house style for **dict-to-typed-config** conversion: explicit keys, fail-loud on unknown fields, returns immutable struct. The Rust `ScopeConfig` mirrors this with `serde` derive + `#[serde(default)]` for lookahead fields.

**Canonical shape (from RESEARCH.md Pattern 7 — Phase 4 establishes for Rust side):**
```rust
// crates/kri0k-core/src/scope.rs
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScopeConfig {
    pub version: u32,                              // Required; v1 only
    pub objective: Option<String>,
    pub operator: Option<String>,                  // Parsed but unused in Phase 4

    #[serde(default)]
    pub targets: Vec<String>,                      // USED (D-48)

    #[serde(default)]
    pub targets_cidr: Vec<String>,                 // Phase 7 stub

    #[serde(default)]
    pub targets_wildcards: Vec<String>,            // Phase 7 stub

    #[serde(default)]
    pub safeguards: SafeguardsSection,

    #[serde(default)]
    pub rate_limits: RateLimitsSection,

    #[serde(default)]
    pub audit_path: Option<String>,

    #[serde(skip)]
    pub raw_yaml: String,                          // Captured for compute_hash()
}

impl ScopeConfig {
    pub fn from_yaml(path: &Path) -> Result<Self, crate::Error> { /* read file, parse, version check */ }
    pub fn from_dict_value(value: serde_json::Value) -> Result<Self, crate::Error> { /* Engagement::new path */ }
    pub fn compute_hash(&self) -> String { /* sha256(raw_yaml) */ }
    pub fn validate_target(&self, target: &str) -> Result<(), crate::Error> {
        if self.targets.iter().any(|t| t == target) {
            Ok(())
        } else {
            Err(crate::Error::ScopeViolation { target: target.into(), reason: format!(...) })
        }
    }
}
```

**Differences to apply vs old stub:**
- DELETE existing `pub struct Scope`, `Scope::from_yaml`, `Scope::compute_hash`, free function `validate_target`. The new `validate_target` is a method.
- The old tests in `crates/kri0k-core/src/scope.rs:66-84` need full rewrite (current test asserts the stub error message; replace with: "parses minimal yaml", "rejects version=2", "exact-match accept", "non-match returns ScopeViolation").

**Risk / landmine:**
- **`SafeguardsSection::default()`** gives `propose_only: false` (because `bool::default()` is `false`) which would silently make engagements *execute by default* if a YAML omits the section entirely. See Pitfall 11 in `04-RESEARCH.md:1054-1058`. Implement `Default` MANUALLY on `SafeguardsSection` so default is `propose_only: true` — D-49 default.
- `sha2 = "0.10"` may already be transitively present via `ulid`; verify with `cargo tree -p kri0k-core | grep sha2`. Add to `Cargo.toml` only if absent.
- `serde_yaml_ng` is API-compatible with `serde_yaml`. Do NOT use `serde_yaml` (deprecated) nor `serde_yml` (RUSTSEC-2025-0068). See Pitfall 9.

---

### `crates/kri0k-core/src/audit.rs` (rename only)

**Analog:** `crates/kri0k-core/src/audit.rs:83-108` (existing `NoOpAuditSink`)

**Excerpt:**
```rust
#[derive(Debug, Default)]
pub struct NoOpAuditSink;

impl AuditSink for NoOpAuditSink {
    fn log_ttp_execution(&mut self, _event: TtpExecutionEvent) -> Result<(), crate::Error> {
        Ok(())
    }
    // ...
}
```

**Differences to apply:**
- Rename `NoOpAuditSink` → `NoopAuditSink` everywhere (per D-38). The rest of the file is untouched in Phase 4. Real impl lands in Phase 8.

**Risk / landmine:**
- Verify `create_audit_sink` at `crates/kri0k-core/src/audit.rs:114-119` still returns `Box::new(NoopAuditSink)` after rename.
- No other crate references `NoOpAuditSink` today (`grep` confirms zero call-sites outside this file), so the rename is local-only.

---

### `crates/kri0k-core/src/ttp/mod.rs` (promote single file to module)

**Analog:** `crates/kri0k-core/src/ttp.rs:13-92` (current sync trait skeleton)

**Current shape (sync, methods `todo!()`):**
```rust
pub trait Ttp: Send + Sync {
    fn id(&self) -> &str;
    fn description(&self) -> &str;
    fn requires_human_gate(&self) -> bool { false }
    fn rate_limits(&self) -> RateLimits { RateLimits::default() }
    fn propose(&self, _ctx: &ExecutionContext) -> Result<ExecutionPlan, crate::Error> { todo!(...) }
    fn execute_dry_run(&self, _plan: &ExecutionPlan) -> Result<DryRunOutput, crate::Error> { todo!(...) }
    fn execute(&self, _plan: &ExecutionPlan) -> Result<ExecutionResult, crate::Error> { todo!(...) }
}
```

**Canonical async shape (from RESEARCH.md Pattern 3 — Phase 4 establishes):**
```rust
// crates/kri0k-core/src/ttp/mod.rs
use async_trait::async_trait;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub mod subprocess;
pub mod whois;

#[async_trait]
pub trait Ttp: Send + Sync {
    fn id(&self) -> &str;
    fn description(&self) -> &str;
    fn risk_level(&self) -> RiskLevel { RiskLevel::Safe }
    fn rate_limits(&self) -> RateLimits;
    fn default_timeout(&self) -> Duration { Duration::from_secs(30) }

    async fn execute(
        &self,
        target: &str,
        cancel: CancellationToken,
    ) -> Result<TtpOutput, crate::Error>;
}

#[derive(Debug, Clone)]
pub enum TtpOutput {
    Whois(crate::ttp::whois::WhoisOutput),
    // Phase 5+: Nmap(...), Dig(...), etc.
}

// Keep: RateLimits, RiskLevel
// Delete: ExecutionContext, ExecutionPlan, DryRunOutput, ExecutionResult, DestructiveTtp,
//         requires_human_gate free fn — none of these are used in Phase 4 (D-49 propose-only
//         is a Python-side boolean; D-52 dispatch is target+cancel).
```

**Differences to apply vs current sync trait:**
- File migration: `crates/kri0k-core/src/ttp.rs` → `crates/kri0k-core/src/ttp/mod.rs`. Rust resolves the module identically (`pub mod ttp;` in `lib.rs` is unchanged). Use `git mv` for clean diff.
- `#[async_trait]` annotation on the trait AND on each `impl` (mandatory; see Pitfall 6).
- Remove `propose`/`execute_dry_run`/`execute(&self, &ExecutionPlan)`/`requires_human_gate(&self)` — Phase 4 ditches the propose/dry-run/execute trichotomy and exposes a single `execute(target, cancel)`. The propose-only gate moves to Python (`act.py`).
- Add `default_timeout()` (D-51 — 30s) and `risk_level()` (placeholder for Phase 11 TUI).
- Add `TtpOutput` enum — single-variant in Phase 4 (`Whois(WhoisOutput)`); future TTPs add variants.

**Risk / landmine:**
- **`#[async_trait]` is required.** Native `async fn in trait` (stable since Rust 1.85) does NOT support `Box<dyn Trait>` — the `Engagement` registry needs `HashMap<String, Box<dyn Ttp>>`, so dyn-compatibility is non-negotiable. Without `#[async_trait]`: compile error `the trait Ttp cannot be made into an object`. See Pitfall 6 in `04-RESEARCH.md:1024-1028`.
- Keep `RateLimits` struct (lines 76-92 of current ttp.rs) — `WhoisTtp::rate_limits()` returns it.

---

### `crates/kri0k-core/src/ttp/subprocess.rs` (NEW — Subprocess trait + Real/Mock)

**NO ANALOG IN RUST.** Phase 4 establishes the canonical pattern. Future TTPs MUST inject a `Subprocess` in their constructor for testability.

**Cross-language analog (Python provider Protocol):** `python/kri0k/llm/protocol.py:15-32` + `python/kri0k/llm/ollama.py:82-154`

**Excerpt from Python Protocol pattern (shape only):**
```python
# protocol.py
@runtime_checkable
class LLMProvider(Protocol):
    async def chat(self, *, prompt: str, system: str | None = None) -> str: ...

# ollama.py — concrete impl, injectable in constructor
class OllamaProvider:
    def __init__(
        self, config: LLMConfig, *,
        bucket: TokenBucket | None = None,
        client: httpx.AsyncClient | None = None,   # ← injectable for tests
    ) -> None: ...
```

The shape Phase 4 imports: trait surface + concrete `Real*` impl + the impl is injectable via constructor (so tests substitute a mock).

**Canonical Rust shape (from RESEARCH.md Pattern 1):**
```rust
// crates/kri0k-core/src/ttp/subprocess.rs
use async_trait::async_trait;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

#[async_trait]
pub trait Subprocess: Send + Sync {
    async fn run(
        &self,
        cmd: &str,
        args: &[&str],
        timeout: Duration,
        cancel: CancellationToken,
    ) -> Result<SubprocessOutput, crate::Error>;
}

#[derive(Debug, Clone)]
pub struct SubprocessOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Default)]
pub struct RealSubprocess;

#[async_trait]
impl Subprocess for RealSubprocess {
    async fn run(&self, cmd: &str, args: &[&str], timeout: Duration, cancel: CancellationToken)
        -> Result<SubprocessOutput, crate::Error>
    {
        let mut child = Command::new(cmd)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)             // belt-and-suspenders
            .spawn()
            .map_err(crate::Error::Io)?;

        let output = tokio::select! {
            biased;
            _ = cancel.cancelled() => {
                let _ = child.start_kill();
                let _ = child.wait().await;
                return Err(crate::Error::Cancelled);
            }
            _ = tokio::time::sleep(timeout) => {
                let _ = child.start_kill();
                let _ = child.wait().await;
                return Err(crate::Error::SubprocessTimeout {
                    ttp_id: "<subprocess>".into(),
                    timeout_ms: timeout.as_millis() as u64,
                });
            }
            res = child.wait_with_output() => res.map_err(crate::Error::Io)?,
        };

        Ok(SubprocessOutput {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            exit_code: output.status.code(),
        })
    }
}

// MockSubprocess for unit tests
pub struct MockSubprocess { /* fixture path OR hanging mode */ }
impl MockSubprocess {
    pub fn from_fixture(path: PathBuf) -> Self { /* reads file at run-time */ }
    pub fn hanging() -> Self { /* returns a future that never completes */ }
}
```

**Risk / landmine:**
- **`tokio::select!` with `biased`** is critical — cancel branch must fire BEFORE timeout (see Pitfall 4 in `04-RESEARCH.md:1012-1016`).
- **`kill_on_drop(true)`** is belt-and-suspenders only; rely on explicit `start_kill()` + `wait()` in the cancel/timeout branches to avoid Windows "OS error 6: invalid handle" log noise.
- **`from_utf8_lossy`** (not `from_utf8`) — whois.exe on Windows can emit localized non-UTF-8 byte sequences in error messages (see Pitfall 3 in `04-RESEARCH.md:1006-1010`).

---

### `crates/kri0k-core/src/ttp/whois.rs` (NEW — first concrete TTP)

**NO ANALOG IN RUST.** The current `Ttp` trait has zero impls. Phase 4 establishes the canonical "client struct + `impl Trait`" shape.

**Cross-language analog (shape only):** `python/kri0k/llm/ollama.py:82-154` (concrete provider with injected dep + per-instance state)

**Excerpt (shows the "client struct that takes injectable dep + holds per-instance runtime state"):**
```python
class OllamaProvider:
    def __init__(self, config: LLMConfig, *, bucket: TokenBucket | None = None, client: httpx.AsyncClient | None = None):
        self._config = config
        self._bucket = bucket if bucket is not None else TokenBucket()   # ← rate limit bucket
        # ...
    async def chat(self, *, prompt: str, system: str | None = None) -> str:
        # ... uses self._bucket and self._client
```

The Phase 4 Rust counterpart applies the same pattern: `WhoisTtp` takes `Arc<dyn Subprocess>` (the injectable dep, analogous to `client`) and holds `Mutex<Instant>` for the rate limit (analogous to `_bucket`).

**Canonical Rust shape (from RESEARCH.md Pattern 8 + D-45):**
```rust
// crates/kri0k-core/src/ttp/whois.rs
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use async_trait::async_trait;
use tokio_util::sync::CancellationToken;
use tracing::instrument;

use crate::ttp::{Ttp, TtpOutput, RateLimits};
use crate::ttp::subprocess::Subprocess;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WhoisOutput {
    pub registrant: Option<String>,
    pub registrar: Option<String>,
    pub nameservers: Vec<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub expires_at: Option<String>,
    pub raw_unparsed: Vec<String>,
}

pub struct WhoisTtp {
    subprocess: Arc<dyn Subprocess>,
    last_call: Mutex<Option<Instant>>,
}

impl WhoisTtp {
    pub fn new(subprocess: Arc<dyn Subprocess>) -> Self {
        Self { subprocess, last_call: Mutex::new(None) }
    }
}

#[async_trait]
impl Ttp for WhoisTtp {
    fn id(&self) -> &str { "T1590.001" }
    fn description(&self) -> &str { "whois reconnaissance (MITRE T1590.001)" }
    fn rate_limits(&self) -> RateLimits { RateLimits { max_rps: Some(1), max_concurrent: Some(1) } }
    fn default_timeout(&self) -> Duration { Duration::from_secs(30) }

    #[instrument(skip(self), fields(ttp_id = %self.id()))]
    async fn execute(&self, target: &str, cancel: CancellationToken)
        -> Result<TtpOutput, crate::Error>
    {
        // 1. Rate limit (D-45): wait at least 1s since last_call
        let wait = {
            let last = self.last_call.lock()
                .map_err(|e| crate::Error::Generic(format!("rate-limit mutex: {e}")))?;
            last.and_then(|t| Duration::from_secs(1).checked_sub(t.elapsed()))
        };
        if let Some(d) = wait {
            tokio::time::sleep(d).await;
        }

        // 2. Subprocess (with -accepteula per Pitfall 1, -v per Pitfall 2)
        let out = self.subprocess
            .run("whois", &["-v", "-nobanner", "-accepteula", target],
                 self.default_timeout(), cancel.clone())
            .await?;

        // 3. Update last_call BEFORE parsing (so rate limit honoured on parse-error retries)
        {
            let mut last = self.last_call.lock()
                .map_err(|e| crate::Error::Generic(format!("rate-limit mutex: {e}")))?;
            *last = Some(Instant::now());
        }

        // 4. Parse (heuristic — never panics)
        let parsed = parse_whois_output(&out.stdout);
        Ok(TtpOutput::Whois(parsed))
    }
}

pub fn parse_whois_output(raw: &str) -> WhoisOutput {
    let mut out = WhoisOutput::default();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('%') || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once(':') {
            let key_lower = key.trim().to_ascii_lowercase();
            let value = value.trim();
            if value.is_empty() { continue; }
            match key_lower.as_str() {
                "registrant organization" => out.registrant = Some(value.to_string()),  // D-42: only Registrant
                "registrar" => out.registrar = Some(value.to_string()),
                "name server" | "nserver" => {
                    let ns = value.to_ascii_lowercase();
                    if !out.nameservers.contains(&ns) { out.nameservers.push(ns); }
                }
                "creation date" | "created" | "created on" => out.created_at = Some(value.to_string()),
                "updated date" | "last updated" | "modified" => out.updated_at = Some(value.to_string()),
                "registry expiry date" | "expiration date" | "expires on" =>
                    out.expires_at = Some(value.to_string()),
                _ => out.raw_unparsed.push(trimmed.to_string()),
            }
        } else {
            out.raw_unparsed.push(trimmed.to_string());
        }
    }
    out
}
```

**Risk / landmine:**
- **The `-v` flag is mandatory** to get `Registrant Organization` past GDPR redaction (Pitfall 2 — verified live for google.com). Without `-v`, the parser will silently produce `WhoisOutput { registrant: None, ... }` and `Organization` nodes never appear in the graph — tests pass but the feature is dead.
- **`-accepteula` is mandatory** on first run on a fresh Windows machine; otherwise the binary hangs forever waiting for keyboard input (Pitfall 1 — verified).
- **Rate-limit update BEFORE parse:** if parse fails and the caller retries, the rate limit must still hold. Update `last_call` before any fallible parsing.
- **Test fixture path:** Use `concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/whois_google_com.txt")` for absolute paths in `MockSubprocess::from_fixture` calls. `cargo test` working dir is the package root, but a path like `"tests/fixtures/..."` is fragile when running from workspace root or via IDE (Pitfall 13).

---

### `crates/kri0k-graph/src/lib.rs` (extend `NodeKind` + `EdgeKind` enums)

**Analog:** `crates/kri0k-graph/src/lib.rs:9-49` (existing tagged enums)

**Excerpt (current shape — copy this style for new variants):**
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NodeKind {
    Host { ip: String },
    Network { cidr: String },
    Service { port: u16, protocol: String },
    Finding { description: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EdgeKind {
    BelongsTo,
    RunsOn,
    RelatesTo { relation: String },
}
```

**Differences to apply (D-39, D-40):**
- Add `NodeKind::Domain { name: String }`, `NodeKind::Organization { name: String }`, `NodeKind::Nameserver { hostname: String }`. All natural keys go on the variant; dates and `last_whois_at` go to `Node.metadata` (per D-40 — already supported by current `Node` struct at lines 52-72).
- Add `EdgeKind::RegisteredBy`, `EdgeKind::HasNameserver`. Unit variants (no payload) — same shape as `BelongsTo`/`RunsOn`.
- Keep all existing variants — D-39/D-40 are additions, not refactors. Phase 1-3 tests (`tests/test_smoke.py:51-58`) check for `host`/`network`/`service` strings and continue passing.

**Risk / landmine:**
- The `#[serde(rename_all = "snake_case")]` means the variants serialize as `"domain"`, `"organization"`, `"nameserver"`, `"registered_by"`, `"has_nameserver"`. Python tests (and any future TUI) MUST consume the snake_case form.
- No existing code pattern-matches `NodeKind` exhaustively (verified by `grep` — only `get_dummy_graph` constructs variants). So adding variants is safe.

---

### `crates/kri0k-pybridge/src/lib.rs` (extend with `Engagement` pyclass)

**Analog (closest):** `crates/kri0k-pybridge/src/lib.rs:34-78` (`get_dummy_graph`)

**Excerpt (closest existing pattern — sync pyfunction wrapping `runtime().block_on`):**
```rust
#[pyfunction]
#[allow(clippy::expect_used)]
#[allow(clippy::useless_conversion)]
fn get_dummy_graph(py: Python<'_>) -> PyResult<Py<PyAny>> {
    let json_value = py.allow_threads(|| {
        let mut graph = Graph::new();
        // ... build graph ...
        graph.to_json().expect("Failed to serialize graph")
    });

    let json_str = serde_json::to_string(&json_value).expect("Failed to stringify JSON");
    let json_module = py.import("json")?;
    let loads_fn = json_module.getattr("loads")?;
    let result = loads_fn.call1((json_str,))?;
    Ok(result.into())
}
```

**What to copy verbatim:**
- The `py.allow_threads(|| { ... })` envelope — releases the GIL before doing Rust work. Critical to prevent Tokio-vs-GIL deadlock (Pitfall 7 in `04-RESEARCH.md:1030-1034`).
- The `serde_json::to_string` → `json.loads` round-trip for converting `serde_json::Value` to a Python dict. Reuse this as a helper `fn json_value_to_pydict(py, &value)`.

**What's NEW (Phase 4 establishes pattern for stateful pyclass):**

Per RESEARCH.md Pattern 2 (lines 427-578) — the `Engagement` pyclass holds `Mutex<Graph>`, `Box<dyn Trait>`, `CancellationToken`, and a `HashMap<String, Box<dyn Ttp>>` registry. PyO3 0.24 requires `Sync` — Pitfalls 5 and 12 cover the gotchas:

```rust
// crates/kri0k-pybridge/src/lib.rs (add below existing pyfunctions)
use kri0k_core::{
    audit::{AuditSink, NoopAuditSink},
    scope::ScopeConfig,
    ttp::{Ttp, whois::WhoisTtp, subprocess::RealSubprocess, TtpOutput},
    Error, NodeId,
};
use kri0k_graph::Graph;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;
use pyo3::types::PyDict;
use tracing::instrument;

#[pyclass]
pub struct Engagement {
    graph: Mutex<Graph>,
    scope: ScopeConfig,
    audit: Mutex<Box<dyn AuditSink>>,                    // Mutex<...> needed: &mut self methods (Pitfall 5)
    registry: HashMap<String, Box<dyn Ttp>>,
    cancel: CancellationToken,
    dedupe: Mutex<HashMap<(String, String), NodeId>>,    // (kind_tag, natural_key) → NodeId
}

#[pymethods]
impl Engagement {
    #[new]
    fn new(scope_dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        // 1. PyDict → serde_json::Value → ScopeConfig (json round-trip)
        // 2. which::which("whois") fail-fast (D-50) — map error to PyRuntimeError with install hint
        // 3. Build registry with WhoisTtp(Arc::new(RealSubprocess))
        // ...
    }

    #[instrument(skip(self, py))]
    fn snapshot(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let json_value = py.allow_threads(|| {
            self.graph.lock()
                .map_err(|e| Error::Generic(format!("graph mutex: {e}")))
                .and_then(|g| g.to_json())
        }).map_err(error_to_pyerr)?;
        json_value_to_pydict(py, &json_value)
    }

    #[instrument(skip(self, py, proposal))]
    fn execute_proposal(&self, py: Python<'_>, proposal: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
        // 1. Extract target/ttp_id from PyDict (GIL held)
        // 2. py.allow_threads(|| runtime().block_on(async { ... validate → dispatch → apply_delta → audit }))
        // 3. Convert outcome serde_json::Value → PyDict
    }

    fn scope_hash(&self) -> PyResult<String> { Ok(self.scope.compute_hash()) }
    fn kill(&self) { self.cancel.cancel(); }
}

// Helper (extract from existing get_dummy_graph pattern):
fn json_value_to_pydict(py: Python<'_>, v: &serde_json::Value) -> PyResult<Py<PyAny>> {
    let json_str = serde_json::to_string(v)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("json: {e}")))?;
    let json_module = py.import("json")?;
    let result = json_module.getattr("loads")?.call1((json_str,))?;
    Ok(result.into())
}

// In #[pymodule] fn _native:
m.add_class::<Engagement>()?;
```

**Differences to apply vs `get_dummy_graph`:**
- `get_dummy_graph` is stateless (builds a graph per call) — `Engagement` is **stateful** (mutex-guarded fields persist across calls). Use `std::sync::Mutex` (NOT `tokio::sync::Mutex`) because locks are held only inside `block_on` for short non-async sections, and PyO3 0.24 needs `Sync` (Pitfall 12).
- `get_dummy_graph` doesn't use `block_on` — it's pure sync graph construction. `Engagement::execute_proposal` MUST wrap async work with `runtime().block_on(...)` inside `py.allow_threads(...)`.
- `get_dummy_graph` uses `#[pyfunction]` (free function) — `Engagement` uses `#[pyclass]` + `#[pymethods]`. The `runtime()` helper at `crates/kri0k-pybridge/src/lib.rs:13-22` is reused as-is.

**Risk / landmine:**
- **`Box<dyn AuditSink>` is NOT `Sync` by default.** PyO3 0.24 requires `Sync`. Wrap in `Mutex<Box<dyn AuditSink>>` to get `Sync` + interior mutability (Pitfall 5).
- **GIL deadlock:** ALWAYS wrap `runtime().block_on(...)` in `py.allow_threads(|| ...)`. Inside the async block, NEVER call back into Python (Pitfall 7). If you must, use `Python::with_gil` AFTER the runtime returns control.
- **`#[allow(clippy::useless_conversion)]`** at the top of the existing file (line 2) is required because `PyResult<Py<PyAny>>` triggers a false positive. Keep it.
- The `apply_whois_output` helper (called inside `execute_proposal` after `ttp.execute()` returns) needs to acquire BOTH `self.graph.lock()` AND `self.dedupe.lock()`. Always lock in the same order across all helpers to avoid deadlock; recommend `dedupe` first, then `graph`.

---

## Pattern Assignments (Python)

### `python/kri0k/_native.pyi` (extend with `class Engagement`)

**Analog:** `python/kri0k/_native.pyi:1-23` (existing stubs)

**Excerpt:**
```python
"""Type stubs for kri0k._native (Rust extension module).

Generated from Rust PyO3 bindings.
"""

from typing import Any

def hello() -> str:
    """Return a greeting message from the Rust core.

    Returns:
        A greeting string confirming initialization.
    """

def get_dummy_graph() -> dict[str, Any]:
    """Return a dummy graph structure for testing.

    Returns:
        A dictionary with 'nodes' and 'edges' keys, where:
        - nodes: list of dicts with 'id', 'kind', and optional 'metadata'
        - edges: list of dicts with 'id', 'src', 'dst', and 'kind'
    """
```

**Differences to apply (Manifest lines 1154-1162):**
- Append a `class Engagement:` block. Match the existing Google-style docstring convention.
- Methods: `__new__(cls, scope_dict: dict[str, Any]) -> "Engagement"`, `snapshot() -> dict[str, Any]`, `execute_proposal(self, proposal: dict[str, Any]) -> dict[str, Any]`, `scope_hash() -> str`, `kill() -> None`.

**Canonical shape:**
```python
class Engagement:
    """Stateful engagement container exposed by the Rust core.

    Holds the canonical graph, scope config, audit sink, TTP registry, and
    cancellation token for a single engagement. All execution flows through
    `execute_proposal`; `snapshot` returns a read-only view.

    Phase 4: only the whois TTP (T1590.001) is registered.
    """

    def __new__(cls, scope_dict: dict[str, Any]) -> "Engagement":
        """Construct an Engagement from a parsed scope dict.

        Raises:
            RuntimeError: If whois binary not found in PATH (D-50 fail-fast).
            RuntimeError: If scope_dict.version is not 1, or required fields missing.
        """

    def snapshot(self) -> dict[str, Any]:
        """Return current graph state as a dict.

        Returns:
            A dictionary with 'nodes' and 'edges' keys (same shape as get_dummy_graph).
        """

    def execute_proposal(self, proposal: dict[str, Any]) -> dict[str, Any]:
        """Validate scope, dispatch TTP, apply graph mutations, return outcome.

        Returns:
            Outcome dict with keys: status, result, error, graph_delta, audit_id.
        """

    def scope_hash(self) -> str:
        """Return SHA-256 hash of the raw scope YAML (M-03)."""

    def kill(self) -> None:
        """Signal the cancellation token. All in-flight executions abort.

        Terminal: after kill(), the engagement cannot execute further proposals.
        """
```

**Risk / landmine:**
- mypy strict mode requires every method's signature exactly matches the Rust PyO3 binding. If `execute_proposal` returns `serde_json::Value` that wasn't dict-shaped (e.g., a list), mypy passes but runtime fails. Keep the Rust side disciplined to always return a dict (see `build_outcome` in RESEARCH.md Pattern 2).

---

### `python/kri0k/engagement.py` (NEW — bootstrap factory)

**Analog:** `python/kri0k/llm/__init__.py:28-37` (`build_provider` factory pattern)

**Excerpt:**
```python
# python/kri0k/llm/__init__.py
def build_provider(config: LLMConfig) -> LLMProvider:
    """Construct the concrete provider selected by `config.provider`.

    Phase 2 supports only the Ollama provider; the factory exists so the
    Phase 12 CLI bootstrap can wire `engagement_context["llm_provider"]`
    without importing `OllamaProvider` directly.
    """
    if config.provider == "ollama":
        return OllamaProvider(config)
    raise ValueError(f"Unknown provider: {config.provider}")
```

The shape Phase 4 imports: small, single-responsibility helper that the Phase 12 CLI will reuse. Returns a value that the LangGraph bootstrap stuffs into `engagement_context`.

**Canonical shape (from Manifest lines 1164-1199):**
```python
"""Engagement bootstrap helper.

Creates an Engagement instance and returns the engagement_context dict
suitable for injection into AgentState before `graph.invoke()`.
"""

from typing import Any

from kri0k import _native


def create(
    scope_dict: dict[str, Any],
    objective: str,
    propose_only: bool = True,
) -> dict[str, Any]:
    """Create an Engagement and return the engagement_context dict.

    Args:
        scope_dict: Parsed scope.yaml as dict (must contain version=1, targets).
        objective: Engagement objective (free-form string).
        propose_only: If True (default; D-49), act node returns proposals
            without invoking Rust execution. Set False for full execution.

    Returns:
        Dict suitable as AgentState["engagement_context"] with keys:
        engagement, objective, propose_only, scope_hash.

    Raises:
        RuntimeError: If whois binary not found (propagated from Rust per D-50).
    """
    engagement = _native.Engagement(scope_dict)
    return {
        "engagement": engagement,
        "objective": objective,
        "propose_only": propose_only,
        "scope_hash": engagement.scope_hash(),
    }
```

**Differences to apply vs `build_provider`:**
- Returns a `dict`, not a single object — because the helper materializes the FULL `engagement_context` slot (instance + scalars), whereas `build_provider` returns just the provider instance and the caller assembles the context.
- Uses `_native.Engagement(scope_dict)` directly (no factory branch — only one TTP suite in Phase 4). When Phase 12 CLI adds CLI flags, the helper can grow conditionals.

**Risk / landmine:**
- mypy strict: the `dict[str, Any]` return is intentional — `engagement_context` is heterogeneous (`Engagement` instance + `str` + `bool`). Do not narrow to `TypedDict` yet (Phase 5 reflect may add fields).
- `propose_only: bool = True` is correct per D-49 default. Do NOT change.
- The `RuntimeError` from Rust `Engagement.__new__` propagates as-is (PyO3 maps `PyRuntimeError::new_err` → `RuntimeError`). The helper should NOT swallow it.

---

### `python/kri0k/agent/nodes/act.py` (REWRITE)

**Analog:** `python/kri0k/agent/nodes/reason.py:32-83` (engagement_context lookup + async node pattern)

**Excerpt (the pattern to copy):**
```python
async def reason(state: AgentState) -> dict[str, Any]:
    """Reason node: analyze observations and identify patterns."""
    # Get LLM provider from engagement context
    engagement_ctx = state.get("engagement_context", {})
    llm_provider: LLMProvider | None = engagement_ctx.get("llm_provider")

    if llm_provider is None:
        # No provider configured - return empty analysis
        return { ... }

    # Extract context for prompt
    snapshot = state.get("snapshot", {})
    # ...
    response = await llm_provider.chat(prompt=prompt)
    analysis = parse_analysis(response)
    return {"analysis": asdict(analysis)}
```

**What to copy verbatim:**
- The `engagement_ctx = state.get("engagement_context", {})` lookup pattern.
- The "graceful degradation when key missing" return-empty-dict pattern (in `act`'s case, when `proposal` is empty).
- The `async def act(state: AgentState) -> dict[str, Any]:` signature with Google-style docstring + Args/Returns.
- Returning state-partial updates (LangGraph merges them).

**Differences to apply vs `reason` (per D-49 + D-56 + Manifest lines 1201-1275):**
- Replace `llm_provider` lookup with `propose_only` flag + `engagement` instance lookup.
- Two branches:
  - `propose_only=True` (default) → build a `proposed` history entry, return `{"decision": {...}, "history": state["history"] + [entry]}`. Do NOT touch Rust.
  - `propose_only=False` → `outcome = await asyncio.to_thread(engagement.execute_proposal, proposal)`, then build the `executed`/`error` history entry per D-56.
- Helper `_format_summary(proposal, outcome, delta) -> str` — module-private, builds the human string per D-56 (`"<ttp_id> <target> → +<N> nodes +<N> edges"` or `"<ttp_id> <target> → no new nodes (already known)"`).
- Raise `RuntimeError` when `propose_only=False` but `engagement` is `None` — fail loud per `.claude/CLAUDE.md` Regra 4 "não silencie erros."

**Canonical shape (verbatim from Manifest):**
```python
"""Act node: gate propose_only and execute via Engagement."""
import asyncio
from typing import Any

from kri0k.agent.state import AgentState


async def act(state: AgentState) -> dict[str, Any]:
    """Act node: execute approved proposal through Rust Engagement.

    If propose_only=True (default), returns a proposed status without
    invoking Rust. If False, dispatches to engagement.execute_proposal via
    asyncio.to_thread to avoid blocking the event loop.
    """
    proposal = state["proposal"]
    if not proposal:
        return {}

    context = state["engagement_context"]
    propose_only = context.get("propose_only", True)
    iteration = state["iteration_count"]

    if propose_only:
        history_entry = {
            "iteration": iteration,
            "ttp_id": proposal.get("ttp_id", ""),
            "target": proposal.get("target", ""),
            "status": "proposed",
            "summary": f"propose-only: would_execute {proposal.get('ttp_id', '')} on {proposal.get('target', '')}",
            "graph_delta": {"nodes_added": 0, "edges_added": 0},
            "audit_id": None,
        }
        return {
            "decision": {"status": "proposed", "would_execute": proposal.get("ttp_id", ""), "proposal": proposal},
            "history": state["history"] + [history_entry],
        }

    engagement = context.get("engagement")
    if engagement is None:
        raise RuntimeError("act: engagement missing from engagement_context (run kri0k.engagement.create first)")

    outcome = await asyncio.to_thread(engagement.execute_proposal, proposal)
    # ... build history_entry per D-56, return {"decision": outcome, "history": ...}
```

**Risk / landmine:**
- **`asyncio.to_thread` is mandatory** (D-46) — `engagement.execute_proposal` is sync (PyO3 pyclass method) and would block the event loop otherwise. Do NOT call `engagement.execute_proposal(proposal)` directly inside `act`.
- The `history` update uses `state["history"] + [entry]` (immutable append). LangGraph merges state, but appending a new list ensures the merge doesn't lose entries. Do NOT use `.append()` on `state["history"]` — TypedDict warns and mutations don't survive merges cleanly.
- Phase 5 reflect (next phase) consumes `entry["status"]` and `entry["graph_delta"]` — the shape is contractual, do not rename fields.

---

### `python/kri0k/agent/nodes/sense.py` (MODIFY — add backward-compat fallback)

**Analog:** `python/kri0k/agent/nodes/sense.py:19-35` (current implementation)

**Excerpt (current):**
```python
async def sense(state: AgentState) -> dict[str, Any]:  # noqa: ARG001
    """Sense node: fetch and format the current graph snapshot."""
    raw = _native.get_dummy_graph()
    formatted = format_snapshot_hybrid(raw)
    return {"snapshot": {"raw": raw, "formatted": formatted}}
```

**Differences to apply (Manifest lines 1277-1293):**
- Replace `# noqa: ARG001` — state is now USED.
- Add Phase 4 path: prefer `engagement.snapshot()` if `engagement_context["engagement"]` is present.
- Fallback to `_native.get_dummy_graph()` if not present — keeps Phase 1/2 tests (`tests/test_sense_node.py`, `tests/test_agent_graph.py`) green without modification.

**Canonical shape:**
```python
async def sense(state: AgentState) -> dict[str, Any]:
    """Sense node: fetch and format the current graph snapshot.

    Phase 4: prefers engagement.snapshot() when an Engagement is present in
    context; falls back to _native.get_dummy_graph() to preserve Phase 1/2
    test compatibility.
    """
    engagement = state.get("engagement_context", {}).get("engagement")
    if engagement is not None:
        raw = engagement.snapshot()
    else:
        raw = _native.get_dummy_graph()
    formatted = format_snapshot_hybrid(raw)
    return {"snapshot": {"raw": raw, "formatted": formatted}}
```

**Risk / landmine:**
- `engagement.snapshot()` returns the same shape as `get_dummy_graph` (dict with `nodes`/`edges`) — verified by `_native.pyi` stub design. If shapes diverge, `format_snapshot_hybrid` breaks silently.
- Remove `# noqa: ARG001` (state is now read). Ruff will flag if you keep it without justification.

---

## Pattern Assignments (Tests)

### `tests/test_act_node.py` (NEW — unit tests with mocked Engagement)

**Analog:** `tests/test_reason_node.py:1-122` + `tests/test_plan_node.py:1-150`

**Excerpt (the test pattern to copy verbatim):**
```python
"""Tests for the reason node with mocked LLM."""

import pytest

from kri0k.agent.nodes.reason import reason
from kri0k.agent.state import AgentState


class MockLLMProvider:
    """Mock LLM provider for testing."""

    def __init__(self, response: str) -> None:
        self.response = response
        self.last_prompt: str | None = None

    async def chat(self, *, prompt: str, system: str | None = None) -> str:  # noqa: ARG002
        self.last_prompt = prompt
        return self.response


def _make_state(
    *,
    llm_provider: MockLLMProvider | None = None,
    snapshot: dict | None = None,
) -> AgentState:
    """Create a minimal AgentState for testing."""
    return AgentState(
        snapshot=snapshot or {"raw": {}, "formatted": "Test snapshot"},
        analysis={},
        proposal={},
        decision={},
        iteration_count=1,
        history=[],
        engagement_context={
            "llm_provider": llm_provider,
            "scope": "*.example.com",
            "objective": "Test objective",
        },
    )


@pytest.mark.asyncio
@pytest.mark.unit
async def test_reason_returns_analysis_dict() -> None:
    """Reason node returns analysis as dict with expected keys."""
    # ...
    provider = MockLLMProvider(mock_response)
    state = _make_state(llm_provider=provider)
    result = await reason(state)
    assert "analysis" in result
```

**Differences to apply (Manifest lines 1297-1305):**
- Replace `MockLLMProvider` with `MockEngagement` — a tiny class exposing `snapshot()`, `execute_proposal(proposal)`, `scope_hash()`, `kill()`. Record calls in `self.calls = []` and `self.last_proposal: dict | None = None`. Configurable `execute_proposal_result` per instance.
- Replace `_make_state(llm_provider=...)` with `_make_state(engagement=None, propose_only=True, proposal=None)`.
- Markers: `@pytest.mark.asyncio` + `@pytest.mark.unit` (no Rust binary needed, no fixture files).
- 8 tests covering: empty proposal → `{}`; `propose_only=True` skips Rust; `propose_only=True` appends history; `propose_only=False` calls `engagement.execute_proposal`; outcome appended to history; raises `RuntimeError` if engagement missing in non-propose-only mode; summary format `executed`; summary format `idempotent`.

**Mock shape (canonical for Phase 4):**
```python
class MockEngagement:
    """Mock Engagement for act node tests."""
    def __init__(self, execute_result: dict | None = None) -> None:
        self.execute_result = execute_result or {
            "status": "executed", "result": None, "error": None,
            "graph_delta": {"nodes_added": 1, "edges_added": 0}, "audit_id": "test-001",
        }
        self.last_proposal: dict | None = None
        self.execute_call_count = 0

    def snapshot(self) -> dict:
        return {"nodes": [], "edges": []}

    def execute_proposal(self, proposal: dict) -> dict:
        self.execute_call_count += 1
        self.last_proposal = proposal
        return self.execute_result

    def scope_hash(self) -> str: return "deadbeef"
    def kill(self) -> None: pass
```

**Risk / landmine:**
- `asyncio.to_thread(engagement.execute_proposal, proposal)` will call the mock's sync `execute_proposal` from a thread pool. This works (sync methods are fine in threads), but tests asserting `mock.execute_call_count` must `await` the act result to ensure the thread completed.
- Do NOT use `unittest.mock.Mock(spec=Engagement)` — `Engagement` is a Rust pyclass and `inspect` doesn't introspect PyO3 types well. Hand-written `MockEngagement` is more reliable.

---

### `tests/test_engagement_smoke.py` (NEW — integration tests)

**Analog:** `tests/test_llm_ollama.py:1-275` (integration via `MockTransport`) + `tests/test_sense_node.py:1-77` (integration marker + real `_native`)

**Excerpt (markers + real PyO3 pattern):**
```python
# tests/test_sense_node.py
"""Tests for the wired sense node (Phase 2).

The Rust `_native.get_dummy_graph()` mints fresh ULIDs on every call, so
we assert structural shape rather than equality across invocations.
"""
import pytest

from kri0k.agent.nodes.sense import sense
from kri0k.agent.state import AgentState

pytestmark = pytest.mark.integration


def _state() -> AgentState:
    return AgentState(...)


@pytest.mark.asyncio
async def test_sense_returns_snapshot_with_raw_and_formatted() -> None:
    result = await sense(_state())
    assert "snapshot" in result
    # ... structural assertions only — IDs differ across calls
```

**What to copy:**
- `pytestmark = pytest.mark.integration` at module top (whole file gated).
- Async tests with `@pytest.mark.asyncio`.
- Structural assertions (no ID equality across calls — ULIDs are non-deterministic).
- Real `_native` imports via `from kri0k import _native` or `from kri0k._native import Engagement`.

**Differences to apply (Manifest lines 1307-1314):**
- 7 tests: construct with minimal scope; reject `version=2`; fail without whois (conditional skip); whois grows graph (real subprocess); whois idempotent; `kill()` cancels execute; scope violation short-circuits.
- The "whois grows graph" + "idempotent" + "kill cancels" tests need the real whois.exe — skip with `pytest.importorskip` or check `shutil.which("whois")` in fixture.
- For `test_kill_cancels_execute`: spawn `execute_proposal` in a thread, call `kill()` from main thread within 100ms, assert `status == "error"` with `Cancelled` cause.

**Canonical shape (minimal example):**
```python
"""Integration tests for the Engagement pyclass.

These tests exercise the full Rust→Python bridge with real subprocess calls.
Marked `integration`; skipped in default CI (which runs `-m "not integration"`).
"""
import shutil
import pytest

from kri0k._native import Engagement

pytestmark = pytest.mark.integration


def _minimal_scope() -> dict:
    return {
        "version": 1,
        "operator": "test@example.com",
        "targets": ["example.com"],
        "safeguards": {"propose_only": False},
    }


@pytest.fixture
def whois_available() -> bool:
    return shutil.which("whois") is not None


def test_engagement_construct_with_minimal_scope(whois_available: bool) -> None:
    if not whois_available:
        pytest.skip("whois binary not in PATH")
    eng = Engagement(_minimal_scope())
    snap = eng.snapshot()
    assert snap == {"nodes": [], "edges": []}


def test_engagement_rejects_unknown_version() -> None:
    scope = _minimal_scope()
    scope["version"] = 2
    with pytest.raises(RuntimeError, match="version"):
        Engagement(scope)


# ... +5 more per Manifest
```

**Risk / landmine:**
- `whois` may not be installed in CI — tests that exercise the real binary MUST `pytest.skip` rather than fail. The CI config runs `pytest -m "not integration"` by default; integration tests run only in `nightly-integration.yml` (Manifest line 1384).
- ULIDs are non-deterministic — never assert `node.id == "..."`. Assert counts and kind histograms (mirrors `tests/test_sense_node.py:73-77`).
- `test_kill_cancels_execute` is the trickiest: timing-sensitive, may flake. Wrap with `@pytest.mark.flaky(reruns=2)` or just allow `status` to be `"error"` OR `"cancelled"` (depending on whether the cancel raced the spawn).

---

### `tests/fixtures/whois_*.txt` (NEW — capture + synthetic fixtures)

**NO ANALOG.** Phase 4 creates the `tests/fixtures/` directory.

**How to populate (per Manifest lines 1316-1320 + RESEARCH.md Pitfall 1+2):**

1. **`tests/fixtures/whois_google_com.txt`** — capture with:
   ```bash
   whois -v -nobanner -accepteula google.com > tests/fixtures/whois_google_com.txt
   ```
   Verify contents include: `Registrant Organization: Google LLC`, `Registrar: MarkMonitor Inc.`, 4× `Name Server: NS*.GOOGLE.COM`, `Creation Date:`, `Updated Date:`, `Registry Expiry Date:`.

2. **`tests/fixtures/whois_example_com.txt`** — capture with the same command for `example.com`. Verify it has NO Registrant Organization (IANA registry redacts) — this is the "redacted output" test path.

3. **`tests/fixtures/whois_invalid.txt`** — hand-crafted. Contents like:
   ```
   ERROR
   garbage data with no colon
   : empty key
   key without value:
   ```
   Used by parser robustness test to verify `parse_whois_output` never panics and graceful-degrades into `raw_unparsed`.

**Risk / landmine:**
- **Locale leak (Pitfall 3):** if captured on a non-English Windows machine, fixtures may contain Portuguese/German/etc. error messages prepended to the WHOIS data. Capture on an English-locale machine OR manually strip leading non-WHOIS paragraphs.
- **Working directory:** Fixtures live at `tests/fixtures/` relative to workspace root for Python; Rust unit tests should use `concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures/whois_*.txt")` to reach them from `kri0k-core` package dir (Pitfall 13). Alternative: duplicate fixtures into `crates/kri0k-core/tests/fixtures/` to keep Rust tests self-contained. Planner picks; the alternative is cleaner per crate boundary.

---

## Pattern Assignments (Docs)

### `docs/adr/ADR-0013-ttp-trait-subprocess-abstraction.md` (NEW)

**Analog:** `docs/adr/ADR-0012-ttp-trait-adapters.md:1-62`

**Excerpt (full template structure):**
```markdown
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
[Code snippet of the trait]

## Consequências
- ✅ Kill switch realmente para tudo (CancellationToken propaga).
- ✅ Timeout enforced no core, não no adapter (uniform).
- ✅ Adicionar TTP = um arquivo + ...

## Alternativas consideradas
- **Subprocess sync (`std::process`):** rejeitado, bloqueia runtime.
- **Plugins dinâmicos (.so/.dll):** rejeitado para MVP, supply chain risk.
```

**Differences to apply:**
- Same headings (Contexto, Decisão, Consequências, Alternativas), Portuguese (matches ADR-0012 style).
- **Decisão:** document the `Subprocess` trait abstraction (`RealSubprocess` + `MockSubprocess`), the `#[async_trait]` requirement for dyn-compatibility, and the `WhoisTtp::new(subprocess: Arc<dyn Subprocess>)` constructor injection.
- **Alternativas consideradas:** "Direct `tokio::process::Command` in TTP impl" (rejected: untestable without real subprocess + locale-dependent assertions); "`pyo3-asyncio` for native async pyclass methods" (deferred: `asyncio.to_thread` suffices per D-46).
- Reference ADR-0012 as superseded **for the subprocess pattern** (or as a sibling — Phase 4 keeps ADR-0012's high-level TTP trait intent but reshapes the surface).

**Risk / landmine:**
- ADR numbering: current max is `ADR-0012`. `ADR-0013` is correct. Update `docs/adr/README.md` index after creation.
- Date format `2026-05-18` (matches the project's ADR convention).

---

### `README.md`, `CONTRIBUTING.md`, `CHANGELOG.md`, `config/scope.example.yaml`

**Analog for `README.md`:** existing file at workspace root.
**Analog for `CONTRIBUTING.md`:** existing file at workspace root.
**Analog for `CHANGELOG.md`:** none — Phase 4 creates the file.
**Analog for `config/scope.example.yaml`:** existing file at workspace root (`config/scope.example.yaml:1-36`).

These are TEXT additions per Manifest lines 1322-1378. No code excerpt extraction needed — the manifest provides the exact YAML to write and the section headers to add. Style: README/CONTRIBUTING are Portuguese, CHANGELOG can be English (Keep-a-Changelog format) per D-60 Claude's Discretion.

**Risk / landmine for `scope.example.yaml`:**
- Current file uses `targets: [192.168.1.0/24, 10.0.0.0/16]` (CIDR). Phase 4 must rewrite to `targets: [example.com]` (domain — D-48 exact-match). Old `excluded:` field has no v1-schema counterpart; remove or move to `metadata`.
- `version: 1` MUST be the first field (mandatory per D-58; parser rejects absent version).

---

## Shared Patterns

These apply across multiple new files; the planner should reference them once and have each affected file's action list cite back.

### Shared 1: GIL release pattern for PyO3 + Tokio

**Source:** `crates/kri0k-pybridge/src/lib.rs:34-78` (`get_dummy_graph`)
**Apply to:** every `#[pymethods]` on `Engagement` that does Rust work (`snapshot`, `execute_proposal`, `scope_hash`).

```rust
fn snapshot(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
    let json_value = py.allow_threads(|| {       // ← release GIL
        // do Rust work (lock mutexes, run runtime().block_on(...), etc.)
    });
    // Re-acquire GIL implicitly by being inside #[pymethods]
    json_value_to_pydict(py, &json_value)
}
```

Reason: prevents the GIL-vs-Tokio deadlock documented in Pitfall 7 (`04-RESEARCH.md:1030-1034`).

### Shared 2: serde_json round-trip for PyDict conversion

**Source:** `crates/kri0k-pybridge/src/lib.rs:73-77`
**Apply to:** every Rust→Python dict return (`snapshot`, `execute_proposal`).

```rust
fn json_value_to_pydict(py: Python<'_>, v: &serde_json::Value) -> PyResult<Py<PyAny>> {
    let json_str = serde_json::to_string(v)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("json: {e}")))?;
    let json_module = py.import("json")?;
    let result = json_module.getattr("loads")?.call1((json_str,))?;
    Ok(result.into())
}
```

Reason: avoids manual `PyDict::new` + per-field setters. Slower than direct conversion but correct + matches existing house style.

### Shared 3: `engagement_context` lookup pattern for nodes

**Source:** `python/kri0k/agent/nodes/reason.py:44-48`, `python/kri0k/agent/nodes/plan.py:62-66`
**Apply to:** `act.py` (Phase 4) and `sense.py` (backward-compat fallback).

```python
context = state.get("engagement_context", {})
dep = context.get("<key>")  # llm_provider, engagement, etc.
if dep is None:
    # graceful degradation OR raise
    return {"<field>": <empty_dict>}
```

Phase 3 nodes (reason/plan) graceful-degrade. Phase 4 `act.py` MUST RAISE `RuntimeError` when `propose_only=False` and `engagement is None` — this is a programming error, not a runtime fallback. Sense.py keeps the graceful fallback (calls `_native.get_dummy_graph()` when no engagement).

### Shared 4: pytest async + marker convention

**Source:** `tests/test_reason_node.py:42-44`, `tests/test_plan_node.py:42-44`, `tests/test_sense_node.py:12`, `tests/test_llm_ollama.py:18`
**Apply to:** `tests/test_act_node.py` and `tests/test_engagement_smoke.py`.

```python
# Unit tests: per-function markers
@pytest.mark.asyncio
@pytest.mark.unit
async def test_foo() -> None: ...

# OR module-level marker (whole file integration):
pytestmark = pytest.mark.integration
```

Markers registered in `pyproject.toml:234-241`: `unit`, `integration`, `slow`, `ttp`, `graph`, `audit`. Phase 4 may also use `@pytest.mark.ttp` for the act/engagement tests since they relate to TTP execution — optional, planner's discretion.

### Shared 5: Conventional Commits + Rust workspace lint discipline

**Source:** `Cargo.toml:24-53` + `.planning/codebase/CONVENTIONS.md:490-512`
**Apply to:** every commit + every Rust file written.

- Commit format: `type(scope): subject` — one line, no body, no `Co-authored-by` (`.claude/CLAUDE.md` Convenções).
- Scopes valid in Phase 4: `core`, `graph`, `ttp`, `scope`, `pybridge`, `agent`, `deps`, `adr`. Per D-59, expected sequence ≈12 commits.
- Rust files MUST pass `cargo clippy --workspace --all-targets -- -D warnings` with workspace lints (`unwrap_used`, `panic`, `unimplemented` denied). NO `unwrap()` in production code; tests may `#[allow(clippy::expect_used)] // expect is ok in tests`.
- Public APIs require rustdoc (`missing_docs = "warn"`).
- Python files MUST pass `ruff check python/ tests/` + `mypy python/kri0k` strict.

---

## No Analog Found (canonical pattern established by Phase 4)

| File | Role | Why no analog | Phase 4 canonical reference |
|---|---|---|---|
| `crates/kri0k-core/src/ttp/subprocess.rs` | Rust trait + Real/Mock impl | First trait+impl pair in Rust code; closest analog is Python `LLMProvider` Protocol | RESEARCH.md Pattern 1 (lines 342-425); Shared 1+2 above |
| `crates/kri0k-core/src/ttp/whois.rs` | First concrete `Ttp` impl | `Ttp` trait has zero impls currently | RESEARCH.md Pattern 8 (lines 869-944); rate-limit pattern from D-45 |
| `python/kri0k/engagement.py` | Bootstrap factory returning `engagement_context` dict | `build_provider` is closest but returns a single object, not a context dict | Manifest lines 1164-1199 |
| `tests/fixtures/whois_*.txt` | Static test fixtures | `tests/fixtures/` directory doesn't exist yet | Manifest lines 1316-1320; capture via real `whois -v -nobanner -accepteula <domain>` on English-locale machine |
| `CHANGELOG.md` | Release notes | File doesn't exist | D-60 + Manifest lines 1350-1353 (Keep-a-Changelog or simple list; planner picks) |

**Future TTPs and pyclasses MUST follow these patterns.** Phase 5 (dig/nmap TTPs?), Phase 9 (TUI snapshot pyclass?), Phase 12 (CLI Engagement bootstrap) all inherit:

- TTP impls: `<Name>Ttp { subprocess: Arc<dyn Subprocess>, last_call: Mutex<Instant> }` + `#[async_trait] impl Ttp`.
- Pyclasses with state: `Mutex<T>` (std, not tokio) + `py.allow_threads(|| runtime().block_on(...))` + serde_json→json.loads bridge.
- Python factories that materialize `engagement_context`: small module returning a dict, NOT a single object.

---

## Metadata

**Analog search scope:**
- Rust: `crates/kri0k-core/src/**`, `crates/kri0k-graph/src/**`, `crates/kri0k-pybridge/src/**` (all current files)
- Python: `python/kri0k/**/*.py` (all current files, focused on `agent/nodes/*`, `llm/*`, `_native.pyi`)
- Tests: `tests/test_*.py` (13 existing test files)
- Docs: `docs/adr/ADR-00*.md` (12 existing ADRs)
- Config: `config/scope.example.yaml`, `Cargo.toml`, `pyproject.toml`

**Files scanned (high-detail Read):** 18
**Files glob-listed (counted, not Read in full):** 24
**Files NOT scanned (intentional):** `python/kri0k/llm/healthcheck.py`, `python/kri0k/llm/rate_limit.py`, `python/kri0k/llm/templates.py`, `python/kri0k/llm/formatters.py` (beyond top 60 lines) — these don't match any Phase 4 file's role/data-flow closely enough to be analogs.

**Pattern extraction date:** 2026-05-18

---

## PATTERN MAPPING COMPLETE

Phase 4 mapeia 22 files com 17 analogs fortes; **5 files são padrões inéditos no projeto** (`subprocess.rs`, `whois.rs`, `engagement.py`, fixtures, CHANGELOG) — para esses, o PATTERNS.md documenta o shape canônico extraído do RESEARCH.md para que Phase 5+ tenha referência local. Os shared patterns (GIL release, serde_json round-trip, `engagement_context` lookup, pytest markers, Conventional Commits + clippy strict) cobrem múltiplos files e devem ser referenciados por cada plan-action ao invés de repetidos.
