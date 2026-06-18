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

## Structural Orientation (before writing any code)

Identify any critical hubs, high-risk modules, or dependency chokepoints affected
by this change:

- If `docs/codebase-digest.md` exists: read it now and note which listed functions
  this implementation will touch.
- If no digest exists: derive manually — scan `modules/` and identify any function
  that appears to coordinate multiple downstream calls or is called from many sites.

This is a *thinking step*, not an artifact requirement. The goal is to know where
blast radius is concentrated before the first line of code is written.

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

**If this feature consumes vocabulary: apply the Representation Ban.**

Domain logic in this feature must not store, compare, branch on, or pattern-match
vocabulary representations (aliases or canonical strings). Only concept identity —
resolved by the vocabulary module's API — is valid.

Before implementation begins:
1. Identify every site in domain logic that will operate on vocabulary-defined concepts
2. Choose one resolution strategy (normalize-on-write, normalize-on-read, or concept
   identifiers) and apply it uniformly — mixing strategies within one feature is a violation
3. If you find a string literal representing a vocabulary concept in domain logic,
   replace it with a vocabulary-resolved equivalent before proceeding

The wrong pattern in any strategy: comparing a type or concept against a hardcoded
string literal (`== "risk"`, `== Some("risk")`). The correct pattern: concept equality
via the vocabulary's resolution API applied uniformly on both sides of every comparison.

See: `.codeos/patterns/vocabulary-architecture.md`

## Structure

Place implementation in `modules/[feature_id]/` or follow the existing project module layout.

Emit events to `events/runtime_events.jsonl` as append-only JSONL. Each line is one complete JSON event object.

## Output Format

1. Present the implementation
2. Present a **Contract Satisfaction Table**:

| Contract Clause | Satisfied By | Line/Function | Structural Risk |
|---|---|---|---|
| [clause from contract] | [code location] | [file:line] | [LOW / MEDIUM / HIGH / —] |

Structural Risk levels (only populate when a Critical Hub or God Function is touched):
- **LOW** — renaming, extracting helpers, testability changes; behavior visible outside the module is unchanged
- **MEDIUM** — modifying internal logic; external behavior likely unchanged but must be verified
- **HIGH** — behavior visible outside the module may change (callers, emitted events, return values, error modes)
- **—** — no Critical Hub or God Function touched by this clause

3. Present an **Event Emission Table**:

| Event in Schema | Emitted At | Condition |
|---|---|---|
| [EventName] | [file:line] | [when] |

4. Present the Review Package using `.codeos/templates/review-package.md` (Stage 4–5 format, inline only):
   - Artifact: `modules/[feature_id]/`
   - Stage purpose: Implement only what the three approved artifacts specify.
   - Files changed: [list all files created or modified]
   - Key architectural decisions: [choices not fully determined by the approved artifacts — e.g., internal data structure, error propagation strategy]
   - What is not covered yet: [explicit list of what stages 5–9 still need to verify]
   - Suggested areas: (1) Are there contract clauses technically satisfied but implemented in a surprising or fragile way? (2) Does the implementation introduce any behavior not traceable to the approved intent, contract, or schema? (3) What is the most likely Stage 7 gap or mismatch, given what was implemented?
   - Known tensions: from schema design decisions or contract boundary cases, or "none"
5. State: **`AWAITING HUMAN APPROVAL TO PROCEED TO STAGE 5`**

**STOP.** Do not write tests until the human explicitly approves the implementation.
