---
phase: 04-act-node-ttp-whois
plan: "01"
subsystem: kri0k-graph
tags: [graph, data-model, serde, whois, domain, tdd]
dependency_graph:
  requires: []
  provides:
    - NodeKind::Domain { name }
    - NodeKind::Organization { name }
    - NodeKind::Nameserver { hostname }
    - EdgeKind::RegisteredBy
    - EdgeKind::HasNameserver
  affects:
    - crates/kri0k-pybridge/src/lib.rs (Plan 04-05 consumes these variants)
tech_stack:
  added: []
  patterns:
    - serde tagged enums (tag = "type", rename_all = "snake_case")
key_files:
  modified:
    - crates/kri0k-graph/src/lib.rs
decisions:
  - New variants appended after existing ones (Host, Network, Service, Finding) — additive, non-breaking
  - Unit variants for EdgeKind (RegisteredBy, HasNameserver) — metadata deferred to Phase 5+ per D-39
  - Rustdoc on each variant referencing D-39/D-40/D-42
metrics:
  duration: "~5 min"
  completed: "2026-05-18"
  tasks_completed: 2
  files_modified: 1
  tests_added: 9
  tests_total: 13
---

# Phase 4 Plan 01: Graph Data Model Whois Extensions Summary

**One-liner:** Extends kri0k-graph enums with Domain/Organization/Nameserver NodeKind and RegisteredBy/HasNameserver EdgeKind variants using serde snake_case tagged serialization.

## What Was Built

Extended `crates/kri0k-graph/src/lib.rs` with 5 new enum variants required by the whois TTP (TTP-04 data model partial):

### NodeKind additions (3 variants)

| Variant | Fields | JSON tag |
|---------|--------|----------|
| `Domain { name: String }` | `name` | `"type": "domain"` |
| `Organization { name: String }` | `name` | `"type": "organization"` |
| `Nameserver { hostname: String }` | `hostname` | `"type": "nameserver"` |

### EdgeKind additions (2 unit variants)

| Variant | JSON tag |
|---------|----------|
| `RegisteredBy` | `"type": "registered_by"` |
| `HasNameserver` | `"type": "has_nameserver"` |

All existing variants (Host, Network, Service, Finding, BelongsTo, RunsOn, RelatesTo) preserved — no regressions.

## Tests Added

9 new inline tests added to `#[cfg(test)] mod tests`:

- `test_node_kind_domain_serialization`
- `test_node_kind_organization_serialization`
- `test_node_kind_nameserver_serialization`
- `test_node_kind_deserialization_roundtrip` (covers all 3 NodeKind variants)
- `test_node_kind_existing_variants_still_work` (regression)
- `test_edge_kind_registered_by_serialization`
- `test_edge_kind_has_nameserver_serialization`
- `test_edge_kind_new_variants_roundtrip` (covers both EdgeKind variants)
- `test_edge_kind_existing_variants_still_work` (regression)

**Total tests in kri0k-graph:** 13 (4 existing + 9 new) — all pass.

## Verification Results

```
cargo test --package kri0k-graph
  running 13 tests
  13 passed; 0 failed — ok

cargo clippy --package kri0k-graph --all-targets -- -D warnings
  Finished dev profile — no warnings

cargo check --workspace
  Finished dev profile — no errors
```

## Requirement Coverage

- **TTP-04 (partial):** Graph types for whois domain now present. Full coverage closes when Plan 04-05 implements `Engagement::apply_whois_output` consuming these variants.

## TDD Execution

- RED: Tests added first (compile-fail confirmed — variants not found).
- GREEN: Variants added to enums — 13/13 tests pass.
- REFACTOR: Not needed — code is already minimal.

## Deviations from Plan

None — plan executed exactly as written.

Both tasks (Task 1: NodeKind extension; Task 2: EdgeKind extension) were committed atomically in a single commit `f2e7da3` since both tasks modify only `crates/kri0k-graph/src/lib.rs` and the tests cover both simultaneously.

## Known Stubs

None. This plan is data-model only — no stubs or placeholders.

## Threat Flags

None. This plan introduces no new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries (Rust internal only, per threat model T-04-01-01).

## Self-Check: PASSED

- [x] `crates/kri0k-graph/src/lib.rs` modified and committed (`f2e7da3`)
- [x] `pub enum NodeKind` contains `Domain {`, `Organization {`, `Nameserver {`
- [x] `pub enum EdgeKind` contains `RegisteredBy`, `HasNameserver`
- [x] All pre-existing variants preserved (Host, Network, Service, Finding, BelongsTo, RunsOn, RelatesTo)
- [x] 13 tests pass, 0 failures
- [x] Clippy strict zero warnings
- [x] Workspace `cargo check` clean
