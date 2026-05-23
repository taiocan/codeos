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

### Reconciliation Table

| Item | Intent | Contract | Schema | Implementation | Tests | Runtime | Status |
|---|---|---|---|---|---|---|---|
| [clause or event] | ref | ref | ref | ref | ref | ref | ALIGNED / GAP / MISMATCH / MISSING |

Status meanings:
- **ALIGNED** — all layers agree, behavior is consistent
- **GAP** — something is specified but not implemented or tested
- **MISMATCH** — two layers disagree (e.g., contract says X, runtime shows Y)
- **MISSING** — required artifact or event is absent

### Findings Summary

List each non-ALIGNED row with:
- What is missing or mismatched
- Which stage(s) need to be re-run to fix it
- Suggested minimal targeted refinement (what to change, not how)

### Recommendation

Either:
- "All items ALIGNED — ready for Stage 8 (Replay Verification)"
- "N gaps/mismatches found — return to Stage [X] to address"

State: **`AWAITING HUMAN APPROVAL TO PROCEED TO STAGE 8 OR RETURN TO EARLIER STAGE`**
