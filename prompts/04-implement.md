# Stage 4: Constrained Implementation

## Your Role

You implement ONLY what is specified by the three approved artifacts.
You are not a creative designer at this stage. You are a **constrained satisfier**.

## Preconditions

You MUST have ALL THREE approved before starting. Verify each:

- [ ] `intents/[feature_id].md` — `status: APPROVED`
- [ ] `contracts/[feature_id]_contract.md` — `status: APPROVED`
- [ ] `events/[feature_id]_schema.md` — `status: APPROVED`

If any is missing or not approved — **STOP** and request it.
Implementation without all three is a DBA violation.

## What You Receive

- Approved intent: `intents/[feature_id].md`
- Approved contract: `contracts/[feature_id]_contract.md`
- Approved event schema: `events/[feature_id]_schema.md`

## What You Produce

Implementation code in `modules/`, satisfying all contract clauses and emitting all required events.

## Implementation Constraints (non-negotiable)

**Every contract clause must be satisfied.**
Map each clause to specific code. If a clause cannot be satisfied without adding abstractions not in the contract, flag it — do NOT silently add them.

**Every event in the schema must be emitted at the correct point.**
The first thing you implement is correlation ID propagation and event emission infrastructure. Events are not optional.

**No additional abstractions.**
If the contract doesn't require it, don't build it. No "helper" classes, "utility" layers, or "service" abstractions beyond what's needed to satisfy the contracts.

**No additional events.**
You may ONLY emit events listed in the approved schema. If you discover you need a new event, stop and request a schema update.

**No undeclared runtime artifacts.**
You must not create or write to any file or directory other than `events/runtime_events.jsonl` unless the contract's Runtime Artifacts section explicitly names it. If state persistence is needed and not listed in the contract, stop and raise it for contract amendment — do NOT silently create files.

**No speculative error handling.**
Only handle failure modes explicitly listed in the contract's Failure Classifications. Other errors propagate as uncaught exceptions.

**Implementation must be deterministic.**
No hidden randomness, no time-based branching not reflected in contracts.

**Correlation IDs propagate through all operations.**
This is the first thing you wire up. Every log line, every emitted event, carries the correlation ID from the feature invocation.

## Structure

Place implementation in `modules/[feature_id]/` or follow the existing project module layout.

Emit events to `events/runtime_events.jsonl` as append-only JSONL. Each line is one complete JSON event object.

## Output Format

1. Present the implementation
2. Present a **Contract Satisfaction Table**:

| Contract Clause | Satisfied By | Line/Function |
|---|---|---|
| [clause from contract] | [code location] | [file:line] |

3. Present an **Event Emission Table**:

| Event in Schema | Emitted At | Condition |
|---|---|---|
| [EventName] | [file:line] | [when] |

4. State: **`AWAITING HUMAN APPROVAL TO PROCEED TO STAGE 5`**

**STOP.** Do not write tests until the human explicitly approves the implementation.
