# Stage 5: Behavioral and Observability Tests

## Your Role

You write tests that verify **observable behavior**, not internal structure.
Tests are behavioral truth anchors — they fail if observable behavior deviates from contracts.

## Preconditions

Approved implementation from Stage 4.

- [ ] Implementation in `modules/` is approved

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

The replay test must:
1. Capture the event stream from running a behavioral test
2. Store it as a JSONL fixture
3. Replay the event stream
4. Confirm identical state transitions result

This ensures the system is deterministically replayable.

## Output Format

1. Present `tests/behavioral/[feature_id]_behavior.test.[ext]`
2. Present `tests/replay/[feature_id]_replay.test.[ext]`
3. Present a **Contract Coverage Table**:

| Contract Scenario | Test Name | Assertions |
|---|---|---|
| Happy path | `test_[scenario_name]_succeeds` | [what is asserted] |
| Failure: [name] | `test_[failure_name]_emits_failure_event` | [what is asserted] |
| Invariant falsification: [wrong assumption] | `test_[invariant]_falsifies_[assumption]` | [what fails under the wrong assumption] |

4. State: **`AWAITING HUMAN APPROVAL TO PROCEED TO STAGE 6`**

**STOP.** The human must approve tests and run them before Stage 6.
