# Phase 1: LangGraph Structure - Discussion Log

**Date:** 2026-05-14
**Mode:** Interactive (default)
**Areas Discussed:** 4

---

## 1. Graph State Structure

### Question 1: Core State Fields
**Options:**
- Minimal (Recommended) — 4 core fields only
- With history — Core fields + iteration_count, history
- With context — Core fields + engagement_context

**User Selection:** "context and history" (combined both options)
**Outcome:** State includes all 7 fields: snapshot, analysis, proposal, decision, iteration_count, history, engagement_context

### Question 2: State Type Enforcement
**Options:**
- TypedDict (Recommended) — enforced by mypy
- Plain dict — flexible but no type checking
- Pydantic BaseModel — runtime validation

**User Selection:** TypedDict (Recommended)
**Outcome:** `class AgentState(TypedDict)` enforced by mypy

### Question 3: History List Structure
**Options:**
- Full state snapshots — complete state at each iteration
- Deltas only (Recommended) — compact HistoryEntry
- Claude decides — implementation detail

**User Selection:** Claude decides
**Outcome:** Left to planner/researcher discretion

### Question 4: Engagement Context Source
**Options:**
- Passed at graph invoke (Recommended) — caller provides at start
- Read from scope.yaml — sense node reads on first iteration
- Global config — separate config object

**User Selection:** Passed at graph invoke (Recommended)
**Outcome:** Context provided via `graph.invoke({'engagement_context': ctx, ...})`

---

## 2. Node Signatures

### Question 1: Node Function Typing
**Options:**
- Typed functions (Recommended) — typed input, generic return
- Full return type — each node has TypedDict output
- Untyped placeholders — add types later

**User Selection:** Typed functions (Recommended)
**Outcome:** `def sense(state: AgentState) -> dict[str, Any]`

### Question 2: Placeholder Behavior
**Options:**
- Pass-through (Recommended) — `return {}`
- Logging stubs — print node name
- Mock data — return hardcoded values

**User Selection:** Pass-through (Recommended)
**Outcome:** All nodes return empty dict

### Question 3: State Scope
**Options:**
- Full state (Recommended) — each node sees entire AgentState
- Scoped state — configure node inputs

**User Selection:** Full state (Recommended)
**Outcome:** Nodes see everything, decide what to use

### Question 4: Async vs Sync
**Options:**
- Async (Recommended) — future-proof for LLM/Rust
- Sync — simpler for placeholders
- Claude decides

**User Selection:** Async (Recommended)
**Outcome:** All nodes defined as `async def`

---

## 3. Edge Topology

### Question 1: Loop Completeness
**Options:**
- Full loop (Recommended) — complete with conditional edge
- Linear only — simpler, add conditional later
- Claude decides

**User Selection:** Full loop (Recommended)
**Outcome:** sense→reason→plan→act→reflect→[conditional]

### Question 2: Conditional Implementation
**Options:**
- Router function (Recommended) — named function with typed signature
- Inline lambda — compact
- Claude decides

**User Selection:** Router function (Recommended)
**Outcome:** Named router function

### Question 3: Entry Point
**Options:**
- START → sense (Recommended)
- START → init node

**User Selection:** START → sense (Recommended)
**Outcome:** No init node, sense is always first

### Question 4: Compile Timing
**Options:**
- Lazy (Recommended) — `get_graph()` function
- Eager — module-level compile

**User Selection:** Lazy (Recommended)
**Outcome:** Compiled on first call via function

### Question 5: Max Iterations Guard
**Options:**
- Yes (Recommended) — router checks limit
- No guard — trust reflect
- Add in Phase 5

**User Selection:** Yes (Recommended)
**Outcome:** Guard in router logic

### Question 6: MAX_ITERATIONS Value
**Options:**
- 10 (Recommended)
- Configurable
- Claude decides

**User Selection:** 10 (Recommended)
**Outcome:** Hardcoded default of 10

---

## 4. File Organization

### Question 1: Code Organization
**Options:**
- Single file (Recommended for Phase 1)
- Separate files now

**User Selection:** Separate files now
**Outcome:** Split from start: state.py, nodes/, graph.py

### Question 2: Node File Structure
**Options:**
- One file per node (Recommended)
- Grouped files
- All in __init__.py

**User Selection:** One file per node (Recommended)
**Outcome:** sense.py, reason.py, plan.py, act.py, reflect.py

### Question 3: Router Location
**Options:**
- In graph.py (Recommended)
- Separate routing.py

**User Selection:** In graph.py (Recommended)
**Outcome:** Router alongside graph builder

---

## Deferred Ideas

None captured — discussion stayed within phase scope.

---

*Discussion completed: 2026-05-14*
