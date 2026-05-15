# Codebase Concerns

**Analysis Date:** 2026-05-15

## Tech Debt

**T7 Security Stubs - Core Functionality Not Implemented:**
- Issue: All security-critical modules in `kri0k-core` contain only stub implementations with `todo!()` macros
- Files:
  - `crates/kri0k-core/src/ttp.rs` (lines 47-48, 57-58, 69-72, 166-168)
  - `crates/kri0k-core/src/scope.rs` (lines 37-38, 45-46, 57-63)
  - `crates/kri0k-core/src/audit.rs` (lines 91-101, 115-117)
- Impact: The entire security model (scope validation, TTP execution flow, audit logging) is non-functional. System cannot safely execute any operations
- Fix approach: Implement T7 milestones M-01 through M-36 as documented in code comments referencing `THREAT_MODEL.md`. Phases 7 (Scope Validation) and 8 (Audit Logging) in ROADMAP.md address this

**NoOpAuditSink Used as Default:**
- Issue: `create_audit_sink()` returns a no-op implementation that silently discards all audit events
- Files: `crates/kri0k-core/src/audit.rs` (lines 86-118)
- Impact: No audit trail is created for any operations - critical for security compliance and forensics
- Fix approach: Implement append-only JSONL audit sink (M-13), hash-chained verification (M-22), and credential redaction (M-14). Phase 8 addresses this

**Pending License Finalization:**
- Issue: License marked as "TODO(KRK-LICENSE)" with placeholder MIT license
- Files: `Cargo.toml` (line 12), `pyproject.toml` (line 16)
- Impact: Ethical use clause per ADR-0010 not enforced; legal ambiguity for distribution
- Fix approach: Finalize ADR-0010 decision and implement chosen license (Apache 2.0 + ethical clause recommended in THREAT_MODEL.md)

**Python Agent Nodes Are Placeholders (Phase 1 Output):**
- Issue: All five agent nodes (sense, reason, plan, act, reflect) are placeholder implementations returning empty dicts
- Files:
  - `python/kri0k/agent/nodes/sense.py` (line 23: `return {}`)
  - `python/kri0k/agent/nodes/reason.py` (line 23: `return {}`)
  - `python/kri0k/agent/nodes/plan.py` (line 23: `return {}`)
  - `python/kri0k/agent/nodes/act.py` (line 23: `return {}`)
  - `python/kri0k/agent/nodes/reflect.py` (line 23: only increments `iteration_count`)
- Impact: LangGraph structure exists but performs no actual work. Agent loop runs but produces no useful output
- Fix approach: Phases 2-6 in ROADMAP.md implement real functionality for each node

**noqa: ARG001 Suppression in All Nodes:**
- Issue: All placeholder nodes use `# noqa: ARG001` to suppress unused argument warnings
- Files: `python/kri0k/agent/nodes/{sense,reason,plan,act}.py` (line 12 in each)
- Impact: Code quality lint suppression hides the fact that state is ignored
- Fix approach: Remove noqa comments when implementing real node logic in Phases 2-5

**Python Root Module Is Minimal Wrapper:**
- Issue: Python package only exposes `hello()` and `get_dummy_graph()` from Rust - no high-level Python API
- Files: `python/kri0k/__init__.py` (10 lines total, only re-exports `_native` functions)
- Impact: No Python API for actual pentest operations. Agent module not exported at package level
- Fix approach: Export `agent` submodule and add high-level orchestration functions after Phase 6

## Known Bugs

**validate_target Always Returns Error:**
- Symptoms: All target validation fails with "scope validation not implemented"
- Files: `crates/kri0k-core/src/scope.rs` (lines 56-64)
- Trigger: Any call to `validate_target()` regardless of input
- Workaround: None - this is a blocking stub, not a bug. Phase 7 implements this

**requires_human_gate Will Panic:**
- Symptoms: Calling `requires_human_gate()` on any TTP ID causes panic via `todo!()`
- Files: `crates/kri0k-core/src/ttp.rs` (lines 163-169)
- Trigger: Any attempt to classify TTP risk level
- Workaround: Do not call this function until T7/M-21 is implemented

**Ttp Trait Methods Panic:**
- Symptoms: Calling `propose()`, `execute_dry_run()`, or `execute()` on any TTP causes panic
- Files: `crates/kri0k-core/src/ttp.rs` (lines 46-73)
- Trigger: Any TTP execution attempt
- Workaround: None - Phase 4 implements first TTP (whois)

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
- Files: Multiple functions in `crates/kri0k-core/src/ttp.rs`, `scope.rs`, `audit.rs` (6 total `todo!()` calls)
- Current mitigation: Clippy lint `todo = "warn"` configured; all `todo!()` calls have explicit `#[allow(clippy::todo)]`
- Recommendations: Track all T7 milestones to completion before any production use

**No Encryption at Rest:**
- Risk: Graph snapshots containing collected credentials stored in plaintext
- Files: Not yet implemented (M-09 pending)
- Current mitigation: MVP-0 has no credential persistence
- Recommendations: Implement KRK-CRYPTO card before MVP-1 adds credential storage (chacha20-poly1305 + argon2id as per THREAT_MODEL.md)

**External LLM Data Exposure:**
- Risk: Snapshot data sent to external LLM providers (Anthropic, OpenAI) when opted-in
- Files: Dependencies declared in `pyproject.toml` (anthropic, openai optional deps at lines 52-53)
- Current mitigation: Ollama local-first is default (ADR-0008); M-27-M-30 mitigations planned
- Recommendations: Implement M-29 aggressive sanitization before enabling external providers

**Agent State Contains Sensitive Data:**
- Risk: `AgentState` TypedDict holds `snapshot`, `analysis`, `proposal` fields that may contain target data
- Files: `python/kri0k/agent/state.py` (lines 24-30)
- Current mitigation: All fields are `dict[str, Any]` - no schema enforcement
- Recommendations: Define strict schemas for state fields; implement redaction before external LLM calls

**MAX_ITERATIONS Hardcoded:**
- Risk: No runtime control over agent loop iterations
- Files: `python/kri0k/agent/graph.py` (line 15: `MAX_ITERATIONS: int = 10`)
- Current mitigation: Constant value prevents runaway loops
- Recommendations: Move to `engagement_context` config with validation bounds

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

**Agent Nodes Are Async But Not Concurrent:**
- Problem: All five agent nodes are `async def` but run sequentially in LangGraph
- Files: `python/kri0k/agent/nodes/*.py` (all 5 node files)
- Cause: LangGraph executes nodes in graph order, not parallel
- Improvement path: This is intentional for security (sequential execution with gates). No change needed

## Fragile Areas

**PyO3 Bridge Error Handling:**
- Files: `crates/kri0k-pybridge/src/lib.rs`
- Why fragile: Multiple `expect()` calls (lines 20, 69, 73) can crash Python interpreter; no graceful error propagation
- Safe modification: Always convert Rust errors to `PyResult<T>` instead of panicking
- Test coverage: Only smoke tests exist (`tests/test_smoke.py` - 3 tests)

**Graph Node/Edge ID Mapping:**
- Files: `crates/kri0k-graph/src/lib.rs` (lines 102-107)
- Why fragile: `node_map: HashMap<NodeId, NodeIndex>` can become stale if nodes are removed
- Safe modification: Implement node removal that updates `node_map`; add invariant assertions
- Test coverage: Basic add operations tested; no removal or edge cases

**LangGraph Version Dependency:**
- Files: `pyproject.toml` (line 32: `langgraph>=0.2.0`)
- Why fragile: LangGraph API has changed significantly between minor versions
- Safe modification: Pin to exact version; test upgrades before merging
- Test coverage: `tests/test_agent_graph.py` (12 tests) covers current API

**AgentState TypedDict Not Validated:**
- Files: `python/kri0k/agent/state.py` (lines 11-30)
- Why fragile: TypedDict provides type hints but no runtime validation. Invalid state passes silently
- Safe modification: Consider Pydantic model for runtime validation
- Test coverage: `test_agent_state_has_required_fields` only checks annotations exist

## Scaling Limits

**In-Memory Graph Storage:**
- Current capacity: All graph data held in memory via petgraph `StableGraph`
- Limit: Memory exhaustion on large engagements (thousands of nodes/edges)
- Scaling path: Implement snapshot checkpointing to disk; consider SQLite backend for large graphs

**Single-File Audit Log:**
- Current capacity: All audit events appended to single JSONL file (when implemented)
- Limit: File I/O bottleneck; single point of failure
- Scaling path: Implement log rotation per THREAT_MODEL.md recommendations; add external sink support

**Agent History List Unbounded:**
- Current capacity: `history: list[dict[str, Any]]` grows with each iteration
- Limit: Memory exhaustion on long-running engagements
- Files: `python/kri0k/agent/state.py` (line 29)
- Scaling path: Implement sliding window or persist to disk after N iterations

## Dependencies at Risk

**PyO3 Version Lock:**
- Risk: PyO3 0.22 has breaking API changes between minor versions
- Impact: Python binding code may need updates on upgrade
- Files: `Cargo.toml` (line 19)
- Migration plan: Pin version in `Cargo.toml`; track PyO3 migration guides for 0.23+

**LangGraph/LangChain Rapid Evolution:**
- Risk: LangGraph >=0.2.0 and LangChain >=0.3.0 have frequent breaking changes
- Impact: Orchestration layer may break on dependency updates
- Files: `pyproject.toml` (lines 32-33)
- Migration plan: Lock versions in `uv.lock`; add integration tests before upgrades

**pytest-asyncio Compatibility:**
- Risk: pytest-asyncio 0.24+ changed default mode behavior
- Impact: Async tests may fail without explicit mode configuration
- Files: `pyproject.toml` (line 46)
- Migration plan: Add `asyncio_mode = "auto"` to pytest config if needed

## Missing Critical Features

**No CLI Entry Point:**
- Problem: `kri0k.cli:main` referenced in `pyproject.toml` but module does not exist
- Files: `pyproject.toml` (line 60: `kri0k = "kri0k.cli:main"`)
- Blocks: Cannot run `kri0k` command from terminal
- Fix: Phase 12 implements CLI commands

**No TTP Implementations:**
- Problem: `Ttp` trait exists but no concrete TTP implementations (nmap, dig, whois, etc.)
- Files: `crates/kri0k-core/src/ttp.rs` (trait definition only)
- Blocks: Cannot perform any reconnaissance operations
- Fix: Phase 4 implements first TTP (whois)

**No Ollama Integration:**
- Problem: Ollama declared as optional dependency but no provider implementation
- Files: `pyproject.toml` (line 51: `ollama = ["ollama>=0.3.0"]`)
- Blocks: Cannot use local LLM for reasoning
- Fix: Phase 2 implements Ollama provider

**No scope.yaml Parser:**
- Problem: `Scope::from_yaml()` is a stub that panics
- Files: `crates/kri0k-core/src/scope.rs` (lines 35-38)
- Blocks: Cannot load engagement scope configuration
- Fix: Phase 7 implements scope validation

**Sense Node Does Not Call Rust:**
- Problem: Sense node placeholder does not call `_native.get_dummy_graph()` as intended
- Files: `python/kri0k/agent/nodes/sense.py` (returns empty dict, no Rust call)
- Blocks: Agent has no observation of Rust graph state
- Fix: Phase 2 implements real sense node with snapshot retrieval

**No State Initialization:**
- Problem: No function to create properly initialized `AgentState`
- Files: `python/kri0k/agent/state.py` (TypedDict only, no factory)
- Blocks: Callers must manually construct state with all 7 fields
- Fix: Add `create_initial_state(engagement_context)` factory function

## Test Coverage Gaps

**No Rust Integration Tests:**
- What's not tested: Cross-crate interactions (kri0k-core <-> kri0k-graph)
- Files: No `tests/` directory for Rust workspace
- Risk: Crate API contracts may break silently
- Priority: Medium

**Python Integration Tests Limited:**
- What's not tested: LangGraph state transitions, error handling across PyO3 boundary
- Files: `tests/test_smoke.py` (3 tests), `tests/test_agent_graph.py` (12 tests)
- Risk: Complex orchestration failures undetected
- Priority: High - Phase 6 should add E2E tests

**Security Module Stubs Not Testable:**
- What's not tested: Scope validation, audit logging, TTP execution flow
- Files: All of `crates/kri0k-core/src/{scope,audit,ttp}.rs`
- Risk: Implementation bugs in security-critical code
- Priority: Critical - blocked until stubs are implemented (Phases 7-8)

**No End-to-End Test Harness:**
- What's not tested: Full engagement flow from scope load to audit close
- Files: No E2E test infrastructure
- Risk: Integration failures between components
- Priority: High for pre-release

**Agent Graph Execution Not Tested:**
- What's not tested: `graph.invoke()` with real state transitions
- Files: `tests/test_agent_graph.py` tests structure but not execution
- Risk: Runtime errors in graph execution undetected
- Priority: Medium - test after Phase 2 adds real node logic

---

*Concerns audit: 2026-05-15*
