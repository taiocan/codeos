# Stage 5: Behavioral and Observability Tests

## Your Role

You write tests that verify **observable behavior**, not internal structure.
Tests are behavioral truth anchors — they fail if observable behavior deviates from contracts.

## Preconditions

Implementation admitted by the Stage 4 `delivery-entry` adapter.

- [ ] Implementation in `modules/` is available for verification

## What You Produce

Two test files per feature:
1. `tests/behavioral/[feature_id]_behavior.test.[ext]`
2. `tests/replay/[feature_id]_replay.test.[ext]`

## Test Rules

**Tests must NOT test internal implementation details.**
No testing private methods, internal state, or intermediate computations.

**Tests test observable outcomes:**
- State changes after execution
- Events emitted (type, payload fields, correlation_id presence)
- Error signals for each failure mode
- Idempotency (only if the contract specifies it)

**All assertions reference the event schema language.**
Use event names from `events/[feature_id]_schema.md` exactly as they appear there.

## Behavioral Test Categories (all required)

### 1. Happy Path Tests
One test per happy path scenario in the contract.
Assert: correct state change + correct events emitted in correct order.

### 2. Failure Mode Tests
One test per named failure in the contract's Failure Classifications table.
Assert: the correct FAILURE event is emitted with the correct `failure_reason`.
Assert: system state is unchanged (unless contract specifies otherwise).

Per the `doctrine` component selected by `.codeos/dba-system.md` →
"Contract-to-Implementation Failure Boundary," also verify the
negative direction — these are not optional extras, they are part of what "Failure Mode Tests"
means for this category:
- **No masquerading**: trigger a technical/internal error not listed in the Failure
  Classifications table (e.g. a simulated storage or serialization failure) and assert that it
  does **not** produce any approved FAILURE event — it propagates as the internal error type
  itself (or an uncaught exception), never as a misleading classified outcome.
- **No unapproved events**: across the failure-path tests in this category, assert that every
  emitted event's type is present in the approved event schema — an internal error must never
  result in an event the schema doesn't list.

### 3. Telemetry Tests
Assert: correlation_id is present and non-empty in every emitted event.
Assert: all required event fields are present (`event_id`, `event_type`, `timestamp`, `source_module`).

### 4. Idempotency Tests (only if contract specifies idempotency)
Assert: running the same operation twice produces the same outcome without duplicate state changes.

### 5. Invariant Falsification Tests

One test per row in the contract's Invariant Falsification Scenarios table.

Each test:
1. Sets up exactly the falsifying fixture described in the contract row
2. Runs the feature
3. Asserts the observable outcome listed in the contract row

After writing each test, fill in the Test ID column in the contract's Invariant
Falsification Scenarios table — this is required for traceability in Stages 7 and 8.

Name each test `test_[invariant_slug]_falsifies_[wrong_assumption]` and group
them under a `// ── Invariant Falsification` comment block.

Verification rule: for each test, ask — "Would this test still pass if the wrong
implementation assumption from the contract row were present?" If yes, the fixture
is not a falsifier and must be revised before proceeding.

### 6. Contract Note Tests (vocabulary-dependent features)
For each behavioral claim in a contract *Note* (not only in scenarios) that is
testable, add at least one test asserting the noted behavior. Common examples:
- "this also applies when [status set / type set] is empty"
- "this applies regardless of stored representation (canonical or alias)"

Contract notes represent conscious design decisions. Leaving them untested creates
gaps between the contract and the test suite that surface only at reconciliation
(Stage 7), not during development.

### 7. Vocabulary Test Setup
When the feature under test uses vocabulary-driven validation or display, test setup
MUST write a `project-schema.yaml` to the temp directory. Do not rely on
system-installed schemas (e.g., `~/.lucidpm/default-schema.yaml`) — that makes tests
machine-dependent and non-portable in CI.

Recommended pattern:
```
const DEFAULT_SCHEMA: &str = r#"schemaVersion: 1
statuses: { ... }
pageTypes: { ... }
"#;

fn setup_temp_dir() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("events")).unwrap();
    write_project_schema(&dir, DEFAULT_SCHEMA);  // required for vocab features
    dir
}
```

`DEFAULT_SCHEMA` serves double duty: test isolation AND backward-compat regression
baseline. Its status sets must exactly match what the previous hardcoded table provided.

**Environment isolation:** When the binary under test resolves a global configuration root
at runtime (e.g., `HOME`, `XDG_CONFIG_HOME`, `APPDATA`), the test runner must override
that variable to point to the temp directory, preventing any globally-installed schema from
merging with the test schema. Without this, a system-installed vocabulary may introduce alias
collisions or unexpected types that silently break test assertions.

Name this runner variant clearly (e.g., `run_binary_schema_isolated`). Apply it to all tests
that write a `project-schema.yaml`. Tests that deliberately exercise the *no-schema* path
must **not** use the isolated variant — they should let the global configuration resolve
normally, which exercises backward-compatibility with the system vocabulary.

### 8. Backward-Compatibility Regression Test (vocabulary refinements)
When a refinement replaces hardcoded vocabulary behavior with schema-driven behavior,
one test must prove the default schema produces outcomes identical to the old hardcoded
behavior. The important requirement is the assertion — use a descriptive name:
- `test_default_vocabulary_matches_legacy_behavior`
- `test_default_schema_preserves_prior_visibility`
- `test_no_schema_produces_no_additional_exclusions`

Fixture: `DEFAULT_SCHEMA` must exactly match the previously hardcoded table. This test
is the primary verifiable evidence for any "no additional exclusions when no schema"
guarantee.

## Replay Test (required)

The replay test verifies that the system is deterministically replayable: given the same
inputs, the same events are emitted in the same order.

### Required interface

The replay test file (`tests/replay/[feature_id]_replay.test.[ext]`) must:

1. **Invoke the real feature** (not a mock) with a known input that exercises the happy path
2. **Capture the event stream** from `events/runtime_events.jsonl` (or a temp copy) after execution
3. **Assert schema conformance** for every event:
   - All six base fields present and non-null (`event_id`, `event_type`, `timestamp`, `correlation_id`, `source_module`, `payload`)
   - `event_type` is one of the types listed in `events/[feature_id]_schema.md`
4. **Assert correlation chain integrity**:
   - All events from one invocation share a single `correlation_id`
   - The chain starts with the feature's trigger event and ends with a BEHAVIORAL or FAILURE event
   - No orphaned events (events with a `correlation_id` not matching any chain start)
5. **Assert event sequence**:
   - The ordered list of `event_type` values matches the expected happy path sequence from the event flow diagram
   - The sequence is deterministic: running the feature twice with the same input produces the same sequence

### What replay tests do NOT do

- Replay tests do not re-inject the JSONL file into the system and re-run it (that would require
  a replay engine outside the current scope)
- Replay tests do not mock internal components — they invoke the real binary/function
- Replay tests do not assert payload content beyond field presence (payload assertions belong
  in the behavioral tests)

### File format

The event log is JSONL: one complete JSON object per line, no trailing comma, UTF-8.
Each line must be independently parseable. An empty file or a file with only partial lines
is a test failure.

### Determinism verification

Name the primary replay assertion `test_[feature_id]_event_sequence_is_deterministic`.
Run the feature twice in sequence within the same test. Assert that both runs produce
the same `event_type` sequence. Differences indicate hidden non-determinism
(randomness, timestamp-based branching, external state dependency) that must be fixed.

## Output Sequence

**Step 1 — Write both test files**
Write `tests/behavioral/[feature_id]_behavior.test.[ext]` and
`tests/replay/[feature_id]_replay.test.[ext]`. Apply all rules above.

**Step 2 — Fill in the Contract Coverage Table**
Every contract scenario and every invariant falsification row must have exactly one test:

| Contract Scenario | Test Name | Assertions |
|---|---|---|
| Happy path | `test_[scenario_name]_succeeds` | [what is asserted] |
| Failure: [name] | `test_[failure_name]_emits_failure_event` | [what is asserted] |
| Invariant falsification: [wrong assumption] | `test_[invariant]_falsifies_[assumption]` | [what fails under the wrong assumption] |

After completing this table, fill in the Test ID column in the contract's Invariant
Falsification Scenarios table. This is required for Stage 7 and Stage 8 traceability.

**Step 3 — Self-check**
Verify before outputting:
- [ ] One behavioral test per contract happy path scenario
- [ ] One behavioral test per named failure in Failure Classifications
- [ ] Failure boundary: a technical/internal failure does not masquerade as an approved FAILURE event, and no unapproved failure event type is emitted
- [ ] One invariant falsification test per row in the contract's Invariant Falsification Scenarios table
- [ ] Telemetry test asserts `correlation_id` and all six base fields on every event
- [ ] Replay test asserts: base fields, schema event types, correlation chain integrity, deterministic sequence
- [ ] Contract Coverage Table has no empty rows
- [ ] Test IDs filled in the contract's Invariant Falsification Scenarios table
- [ ] **If the contract has a Runtime Context section:** check whether any test executes
  across the real boundary (Electron harness, Docker container, real binary execution —
  not only mocks). If no real-boundary test exists:
  - Document why (infeasible, deferred, etc.)
  - Classify the uncovered contract clauses as MANUAL-PENDING
  - Carry those forward into Stage 6 reconciliation as evidence gaps
  This is not a hard blocker — the goal is to surface gaps, not prohibit progress.

**Step 4 — Output**
1. Present `tests/behavioral/[feature_id]_behavior.test.[ext]`
2. Present `tests/replay/[feature_id]_replay.test.[ext]`
3. Present the Contract Coverage Table (from Step 2) with ✓ / ✗ marks from Step 3
4. Preserve the following delivery evidence for final review (inline; do not create a new artifact):
   - Artifacts: `tests/behavioral/[feature_id]_behavior.test.[ext]`, `tests/replay/[feature_id]_replay.test.[ext]`
   - Stage purpose: Write behavioral tests that would fail if observable behavior deviates from contracts.
   - Files changed: [list test files created or modified]
   - Key decisions: [test design choices — isolation strategy, mock vs real boundary, what was classified MANUAL-PENDING]
   - What is not covered: [contract clauses classified MANUAL-PENDING or deferred to Stage 6 real-boundary observation]
   - Suggested areas: (1) Do the tests verify observable behavior, or do they inadvertently test internal implementation details? (2) Are the invariant falsification tests set up so they would fail if the named wrong assumption were present? (3) Is there any contract clause technically covered but not actually verifying the right outcome?
   - Known tensions: from implementation decisions or contract boundary cases, or "none"
5. Hand the test evidence to `.codeos/dba/03-prompts/workflow/06-observe.md`.
