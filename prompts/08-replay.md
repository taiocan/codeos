# Stage 8: Replay Verification

## Your Role

You verify that the system produces deterministic, replayable behavior.
The runtime event log is the source of truth.

## Purpose

Replay verification confirms:
1. Every event in `events/runtime_events.jsonl` conforms to the approved schema
2. The event sequence matches the expected flow from the contracts
3. Correlation IDs form complete, unbroken chains
4. Re-running the same inputs against the same implementation produces the same events

This creates a guarantee: same inputs + same module version + same constraints → same resulting truth.

## Preconditions

- [ ] Stage 7 reconciliation review — APPROVED
- [ ] `events/runtime_events.jsonl` — populated
- [ ] Replay tests in `tests/replay/` — present

## What You Verify

### 1. Schema Conformance

For every event in `runtime_events.jsonl`:
- `event_type` appears in the approved schema
- All required base fields present (`event_id`, `event_type`, `timestamp`, `correlation_id`, `source_module`)
- Payload fields match the schema definition

### 2. Event Sequence Conformance

Does the observed event sequence in the log match the expected flow from the event schema?
Flag any:
- Events occurring out of expected order
- Events that appear without their prerequisite events
- Terminal events (BEHAVIORAL or FAILURE) missing from chains

### 3. Correlation Chain Integrity

For each unique `correlation_id` in the log:
- Does the chain start with an OBSERVATIONAL or trigger event?
- Does the chain end with either a BEHAVIORAL (success) or FAILURE event?
- Are there orphaned events (correlation_id that appears only once)?

### 4. Determinism Check

Run the replay tests in `tests/replay/`:
- Do they pass against the current log?
- Does re-running the system produce a new log consistent with the schema?

## Replay Report

### Event Log Summary

| Metric | Value |
|---|---|
| Total events in log | N |
| Events matching schema | N |
| Events NOT in schema | N (list them) |
| Unique correlation chains | N |
| Complete chains (start + end) | N |
| Broken chains | N (list correlation_ids) |

### Sequence Conformance

| Expected Flow Step | Observed in Log | Status |
|---|---|---|
| [EventA → EventB] | [yes/no] | PASS / FAIL |

### Replay Test Results

| Test | Result | Notes |
|---|---|---|
| [test name] | PASS / FAIL | |

### Conclusion

Either:
- "Log conforms to schema and contracts — ready for Stage 9 (if refinements needed) or COMPLETE"
- "N conformance issues found — return to Stage [X]"

State: **`AWAITING HUMAN APPROVAL TO PROCEED TO STAGE 9 OR MARK COMPLETE`**
