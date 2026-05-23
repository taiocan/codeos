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

4. State: **`AWAITING HUMAN APPROVAL TO PROCEED TO STAGE 6`**

**STOP.** The human must approve tests and run them before Stage 6.
