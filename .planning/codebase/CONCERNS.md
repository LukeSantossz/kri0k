# Codebase Concerns

**Analysis Date:** 2025-05-14

## Tech Debt

**T7 Security Stubs - Core Functionality Not Implemented:**
- Issue: All security-critical modules in `kri0k-core` contain only stub implementations with `todo!()` macros
- Files:
  - `crates/kri0k-core/src/ttp.rs` (lines 47-48, 57-58, 69-72, 166-168)
  - `crates/kri0k-core/src/scope.rs` (lines 37-38, 45-46, 57-63)
  - `crates/kri0k-core/src/audit.rs` (lines 91-101, 115-117)
- Impact: The entire security model (scope validation, TTP execution flow, audit logging) is non-functional. System cannot safely execute any operations
- Fix approach: Implement T7 milestones M-01 through M-36 as documented in code comments referencing `THREAT_MODEL.md`

**NoOpAuditSink Used as Default:**
- Issue: `create_audit_sink()` returns a no-op implementation that silently discards all audit events
- Files: `crates/kri0k-core/src/audit.rs` (lines 86-118)
- Impact: No audit trail is created for any operations - critical for security compliance and forensics
- Fix approach: Implement append-only JSONL audit sink (M-13), hash-chained verification (M-22), and credential redaction (M-14)

**Pending License Finalization:**
- Issue: License marked as "TODO(KRK-LICENSE)" with placeholder MIT license
- Files: `Cargo.toml` (line 12), `pyproject.toml` (line 16)
- Impact: Ethical use clause per ADR-0010 not enforced; legal ambiguity for distribution
- Fix approach: Finalize ADR-0010 decision and implement chosen license (Apache 2.0 + ethical clause recommended in THREAT_MODEL.md)

**Python Module is Minimal Wrapper:**
- Issue: Python package only exposes `hello()` and `get_dummy_graph()` - no real functionality
- Files: `python/kri0k/__init__.py` (10 lines total)
- Impact: No Python API for actual pentest operations; LangGraph/LangChain integration not started
- Fix approach: Implement Python orchestration layer with LangGraph state machine as planned in dependencies

## Known Bugs

**validate_target Always Returns Error:**
- Symptoms: All target validation fails with "scope validation not implemented"
- Files: `crates/kri0k-core/src/scope.rs` (lines 56-64)
- Trigger: Any call to `validate_target()` regardless of input
- Workaround: None - this is a blocking stub, not a bug

**requires_human_gate Will Panic:**
- Symptoms: Calling `requires_human_gate()` on any TTP ID causes panic via `todo!()`
- Files: `crates/kri0k-core/src/ttp.rs` (lines 163-169)
- Trigger: Any attempt to classify TTP risk level
- Workaround: Do not call this function until T7/M-21 is implemented

## Security Considerations

**expect() and unwrap() Usage:**
- Risk: Panics in production code can cause denial of service
- Files:
  - `crates/kri0k-pybridge/src/lib.rs` (lines 20, 69, 73) - Tokio runtime and JSON serialization
  - `crates/kri0k-graph/src/lib.rs` (lines 255, 258) - Test code only
  - `crates/kri0k-core/src/lib.rs` (lines 113-114) - Test code only
- Current mitigation: Clippy lint `unwrap_used = "deny"` and `expect_used = "warn"` configured; pybridge uses `#[allow(clippy::expect_used)]` with comment
- Recommendations: Replace pybridge `expect()` calls with proper error propagation to Python via `PyResult`

**todo!() Macros in Security-Critical Paths:**
- Risk: Calling any core security function will panic, bypassing safety guarantees
- Files: Multiple functions in `crates/kri0k-core/src/ttp.rs`, `scope.rs`, `audit.rs`
- Current mitigation: Clippy lint `todo = "warn"` configured; all `todo!()` calls have explicit `#[allow(clippy::todo)]`
- Recommendations: Track all T7 milestones to completion before any production use

**No Encryption at Rest:**
- Risk: Graph snapshots containing collected credentials stored in plaintext
- Files: Not yet implemented (M-09 pending)
- Current mitigation: MVP-0 has no credential persistence
- Recommendations: Implement KRK-CRYPTO card before MVP-1 adds credential storage (chacha20-poly1305 + argon2id as per THREAT_MODEL.md)

**External LLM Data Exposure:**
- Risk: Snapshot data sent to external LLM providers (Anthropic, OpenAI) when opted-in
- Files: Dependencies declared in `pyproject.toml` (anthropic, openai optional deps)
- Current mitigation: Ollama local-first is default (ADR-0008); M-27-M-30 mitigations planned
- Recommendations: Implement M-29 aggressive sanitization before enabling external providers

## Performance Bottlenecks

**Tokio Runtime Global Singleton:**
- Problem: Single global Tokio runtime shared across all Python calls
- Files: `crates/kri0k-pybridge/src/lib.rs` (lines 9-22)
- Cause: `OnceLock` initialization with fixed 2 worker threads
- Improvement path: Consider runtime-per-engagement or configurable thread count for parallelism

**Graph Serialization Allocates Full Copy:**
- Problem: `to_json()` creates intermediate `Vec` collections for all nodes and edges
- Files: `crates/kri0k-graph/src/lib.rs` (lines 160-190)
- Cause: Eager collection into `Vec` before JSON conversion
- Improvement path: Implement streaming serialization for large graphs; consider `serde_json::Serializer` with iterator

## Fragile Areas

**PyO3 Bridge Error Handling:**
- Files: `crates/kri0k-pybridge/src/lib.rs`
- Why fragile: Multiple `expect()` calls can crash Python interpreter; no graceful error propagation
- Safe modification: Always convert Rust errors to `PyResult<T>` instead of panicking
- Test coverage: Only smoke tests exist (`tests/test_smoke.py`)

**Graph Node/Edge ID Mapping:**
- Files: `crates/kri0k-graph/src/lib.rs` (lines 102-107)
- Why fragile: `node_map: HashMap<NodeId, NodeIndex>` can become stale if nodes are removed
- Safe modification: Implement node removal that updates `node_map`; add invariant assertions
- Test coverage: Basic add operations tested; no removal or edge cases

## Scaling Limits

**In-Memory Graph Storage:**
- Current capacity: All graph data held in memory via petgraph `StableGraph`
- Limit: Memory exhaustion on large engagements (thousands of nodes/edges)
- Scaling path: Implement snapshot checkpointing to disk; consider SQLite backend for large graphs

**Single-File Audit Log:**
- Current capacity: All audit events appended to single JSONL file (when implemented)
- Limit: File I/O bottleneck; single point of failure
- Scaling path: Implement log rotation per THREAT_MODEL.md recommendations; add external sink support

## Dependencies at Risk

**PyO3 Version Lock:**
- Risk: PyO3 0.22 has breaking API changes between minor versions
- Impact: Python binding code may need updates on upgrade
- Migration plan: Pin version in `Cargo.toml`; track PyO3 migration guides for 0.23+

**LangGraph/LangChain Rapid Evolution:**
- Risk: LangGraph >=0.2.0 and LangChain >=0.3.0 have frequent breaking changes
- Impact: Orchestration layer may break on dependency updates
- Migration plan: Lock versions in `uv.lock`; add integration tests before upgrades

## Missing Critical Features

**No CLI Entry Point:**
- Problem: `kri0k.cli:main` referenced in `pyproject.toml` but not implemented
- Files: `pyproject.toml` (line 60)
- Blocks: Cannot run `kri0k` command from terminal

**No TTP Implementations:**
- Problem: `Ttp` trait exists but no concrete TTP implementations (nmap, dig, etc.)
- Files: `crates/kri0k-core/src/ttp.rs` (trait definition only)
- Blocks: Cannot perform any reconnaissance operations

**No LangGraph Integration:**
- Problem: LangGraph declared as dependency but no state machine implemented
- Files: `pyproject.toml` (line 32)
- Blocks: AI-driven orchestration loop not functional

**No scope.yaml Parser:**
- Problem: `Scope::from_yaml()` is a stub
- Files: `crates/kri0k-core/src/scope.rs` (lines 35-38)
- Blocks: Cannot load engagement scope configuration

## Test Coverage Gaps

**No Rust Integration Tests:**
- What's not tested: Cross-crate interactions (kri0k-core <-> kri0k-graph)
- Files: No files in `tests/` directory for Rust
- Risk: Crate API contracts may break silently
- Priority: Medium

**No Python Integration Tests:**
- What's not tested: LangGraph state transitions, error handling across PyO3 boundary
- Files: Only `tests/test_smoke.py` with 3 basic tests
- Risk: Complex orchestration failures undetected
- Priority: High - this is core functionality

**Security Module Stubs Not Testable:**
- What's not tested: Scope validation, audit logging, TTP execution flow
- Files: All of `crates/kri0k-core/src/{scope,audit,ttp}.rs`
- Risk: Implementation bugs in security-critical code
- Priority: Critical - blocked until stubs are implemented

**No End-to-End Test Harness:**
- What's not tested: Full engagement flow from scope load to audit close
- Files: No E2E test infrastructure
- Risk: Integration failures between components
- Priority: High for pre-release

---

*Concerns audit: 2025-05-14*
