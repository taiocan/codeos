# Stage 8: Replay Verification

<!-- DOCTRINE ADAPTER: final-acceptance
Operationalizes the active doctrine's final feature decision boundary. -->

## Your Role

You verify that the system produces deterministic, replayable behavior.
The runtime event log is evidence of what occurred; it does not redefine approved requirements.

## Purpose

Replay verification confirms:
1. Every event in `events/runtime_events.jsonl` conforms to the approved schema
2. The event sequence matches the expected flow from the contracts
3. Correlation IDs form complete, unbroken chains
4. Re-running the same inputs against the same implementation produces the same events

This creates a guarantee: same inputs + same module version + same constraints → same resulting truth.

## Preconditions

- [ ] Stage 7 reconciliation is complete and its findings are available

**Controlled Plain English check (if `architecture/controlled-plain-english.yaml` exists):** read
its `status` per the Optional Mechanism Status Convention's four-outcome table
(`.codeos/templates/conventions.md`). Absent or `disabled` → proceed unaffected. `enabled` → read
`.codeos/patterns/controlled-plain-english.md`; if missing/unreadable, **STOP** and report a
pattern-access error; otherwise apply **factual reporting plus Layer D1's evidence-vs-inference
separation** (not Layer B) — replay compares observed outcomes against expectation, closer to
review than to spec-writing (Layer C1 always applies regardless). Malformed status file → **STOP**
and report a configuration error.
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

Present the final Review Package using `.codeos/templates/review-package.md`:
- Stage purpose: Verify deterministic, schema-conforming, correlation-intact event replay.
- What was verified: [N events inspected for schema conformance, N correlation chains verified, replay test results]
- Verdict: PASS / FAIL ([N] issues found)
- What would make this stage stronger: [e.g., "Observe failure path X at real boundary — currently EQ 3", or "none — evidence is sufficient"]
- Suggested areas: (1) Are schema conformance issues indicating implementation drift rather than test gaps? (2) Do broken correlation chains point to a specific event category or module? (3) Would real-boundary observation of any failure path change the verdict?

Run the default advisory review with Stage ID `8`, then state: **`AWAITING FINAL HUMAN ACCEPTANCE`**.
The human accepts the feature or requests targeted refinement.
