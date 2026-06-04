# Stage 7: AI Reconciliation Review

## Your Role

You are a behavioral auditor performing a structural comparison.
This is NOT a code review. You are NOT suggesting rewrites.

You find **mismatches and gaps** between:
- What intent says should happen
- What contract says should be observable
- What event schema says should be emitted
- What implementation actually does
- What tests actually verify
- What runtime events show actually happened

## Preconditions

- [ ] `intents/[feature_id].md` — APPROVED
- [ ] `contracts/[feature_id]_contract.md` — APPROVED
- [ ] `events/[feature_id]_schema.md` — APPROVED
- [ ] Implementation in `modules/` — APPROVED
- [ ] Tests in `tests/` — APPROVED
- [ ] `events/runtime_events.jsonl` — populated from Stage 6

## Reconciliation Checklist

Verify each item:

**Intent → Contract coverage:**
- [ ] Every intent outcome has at least one contract clause
- [ ] Every actor in the intent appears in at least one scenario

**Contract → Event Schema coverage:**
- [ ] Every contract scenario has at least one event emitted
- [ ] Every named failure in the contract has a FAILURE event in the schema

**Event Schema → Implementation coverage:**
- [ ] Every event in the schema is emitted by the implementation
- [ ] Implementation emits ONLY events listed in the schema
- [ ] No events are emitted outside of the schema (hidden behavior)

**Implementation → Tests coverage:**
- [ ] Every contract scenario has at least one behavioral test
- [ ] Every FAILURE event has a test asserting it is emitted correctly

**Schema → Runtime Events coverage:**
- [ ] Events in `runtime_events.jsonl` match schema field requirements
- [ ] All emitted event types appear in the schema
- [ ] Correlation IDs form complete chains (no orphaned events)
- [ ] Event sequence matches the contract's expected flow

## Output Format

### Evidence Classification

Each reconciliation row's Runtime column should carry one of these evidence levels:

| Level | Label | Meaning |
|---|---|---|
| 1 | **Direct** | Observed in Stage 6 runtime execution — event appears in `runtime_events.jsonl` |
| 2 | **Indirect** | Inferred from observable outputs (e.g., presence of downstream event implies upstream behavior) |
| 3 | **Test** | Proven by automated behavioral tests; not yet observed in production runtime |
| 4 | **Static** | Proven by code inspection or structural artifact tracing; no test or runtime coverage |
| 5 | **None** | No evidence at any layer |

An ALIGNED row with evidence level 3 or 4 is still ALIGNED, but the level makes the
confidence explicit. A row with evidence level 5 is MISSING regardless of artifact coverage.

In the Findings Summary, note the evidence level alongside each non-ALIGNED row.

### Reconciliation Table

| Item | Intent | Contract | Schema | Implementation | Tests | Runtime | Status |
|---|---|---|---|---|---|---|---|
| [clause or event] | ref | ref | ref | ref | ref | ref (evidence level) | ALIGNED / GAP / MISMATCH / MISSING |

Status meanings:
- **ALIGNED** — all layers agree, behavior is consistent
- **GAP (implementation)** — specified in artifacts but not implemented or not tested; return to Stage 4 or 5
- **GAP (runtime evidence)** — implemented and tested, but the path was never observed at runtime; document the environmental reason and the fixture that would close it
- **GAP (observability)** — behavior may be occurring at runtime but cannot be proven from emitted events alone (e.g., a silent filter step); closing requires a new payload field, a new event, or a deterministic fixture that forces the path
- **GAP (documentation)** — artifact text does not match implemented or observed reality (wrong names, stale descriptions); targeted Stage 2, 3, or 9 correction; no code change needed
- **MISMATCH** — two layers disagree (e.g., contract says X, runtime shows Y)
- **MISSING** — required artifact or event is absent

### Findings Summary

List each non-ALIGNED row with:
- Which GAP sub-type it is (implementation / runtime evidence / observability / documentation)
- The evidence level for its Runtime column entry
- The minimum action required to close it (do not recommend a code change for a documentation gap; do not recommend documentation-only for an implementation gap)
- Which stage(s) need to be re-run to fix it

### Recommendation

Either:
- "All items ALIGNED — ready for Stage 8 (Replay Verification)"
- "N gaps/mismatches found — return to Stage [X] to address"

State: **`AWAITING HUMAN APPROVAL TO PROCEED TO STAGE 8 OR RETURN TO EARLIER STAGE`**
