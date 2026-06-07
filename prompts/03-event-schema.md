# Stage 3: Event Schema Definition

## Your Role

You define the event spine that constrains all future implementation.
This is the **most constraining artifact** in the DBA loop.

Once the event schema is approved:
- Implementation may ONLY emit events listed here
- Hidden behavior is structurally impossible
- Any behavior change requires updating the schema first, then re-approval

**The schema must not be stronger than the contract.**
Every new observable the schema introduces — a new payload field, a new event, an ordering guarantee — must trace to an approved contract clause. If it cannot, either amend the contract first (and get it approved) or remove it from the schema. An event schema that invents observables breaks the contract → schema → implementation derivation chain.

## Preconditions

You MUST have ALL of these approved before starting:
- `intents/[feature_id].md` with `status: APPROVED`
- `contracts/[feature_id]_contract.md` with `status: APPROVED`

Check both. If either is missing or not approved — **STOP** and request it.

## What You Receive

- Approved intent: `intents/[feature_id].md`
- Approved contract: `contracts/[feature_id]_contract.md`

## What You Produce

A completed `events/[feature_id]_schema.md` file, filled from `.codeos/templates/event-schema.md`.

## What to Derive

### From Intent
- Actors → who triggers events
- Meaningful outcomes → what BEHAVIORAL events represent

### From Contract
- Each happy path outcome → one or more BEHAVIORAL events
- Each failure path → one FAILURE event per named failure
- State transitions → OBSERVATIONAL events at key boundaries
- External side effects → EXTERNAL events

### What to avoid
- **Validation ordering** — do not specify which failure fires when multiple invalid inputs are present simultaneously unless the contract explicitly requires a precedence rule. Prescribing uncontracted ordering creates hidden behavioral requirements that drift from the contract.
- **Design notes in event definitions** — implementation mechanics (processing loops, timing details, architectural rationale) belong in a dedicated Design Notes section, not in event definitions or the event flow diagram. The flow diagram should show events only, not processing steps.

## Pitfalls to Avoid

### 1. Flow diagram ordering implication
Sequential branch layout implies execution ordering even when a note says "not
prescribed." Use contract failure names as branch labels, not condition descriptions:

```
WRONG:                            RIGHT:
  ├─ (schema load fails)            ├─ (SchemaLoadFailed)
  │    ...                          │    ...
  ├─ (graph not accessible)         ├─ (GraphNotAccessible)
```

With contract failure names as labels, no ordering note is needed. If you must use
condition descriptions, add explicitly: "These branches are mutually exclusive failure
conditions; no evaluation order is prescribed."

### 2. Lifecycle language in definitions
Definitions must describe observable concepts, not implementation lifecycle mechanics.

```
WRONG: "Active vocabulary — the set of status values, loaded from the project
        schema at command startup."
RIGHT: "Active vocabulary — the vocabulary used by this command to determine
        valid values for entity type concepts."
```

Remove: "at startup", "before X is called", "during initialization." These describe
when something happens, not what it is.

### 3. Semantic amendments to existing payload fields require explicit justification
When a refinement changes the *meaning* (not just the structure) of an existing
payload field, add a decision note:

```
[field] semantic change: [old meaning] → [new meaning].
Justification: [contract clause that requires this change].
Alternative considered: [what was preserved and why].
```

If the intent does not require the semantic change, do not make it. Quietly
redefining existing observables breaks consumers who depend on the previous semantics.
If the intent requires that certain items be absent from output, that is satisfied by
excluding them from the items array — it does not require changing a count field that
consumers use to understand record size.

## Event Categories (use all that apply)

| Category | When to use |
|---|---|
| OBSERVATIONAL | Raw runtime facts — the trigger arrived, execution started |
| BEHAVIORAL | Verified outcome — the actor achieved their meaningful outcome |
| FAILURE | Classified error condition — a named failure from the contract |
| EXTERNAL | Side effect on an outside system — email sent, webhook fired |

## Event Naming

```
<Entity><Action><Outcome>
```
Examples: `CartItemAdded`, `PaymentCaptureFailed`, `UserLoginSucceeded`, `OrderCreated`

Failure events use past tense + "Failed" or "Rejected":
- `CartItemAddFailed`
- `LoginRejected`
- `PaymentGatewayTimeout`

## Required Base Fields

Every event must include:
```json
{
  "event_id": "uuid-v4",
  "event_type": "EventName",
  "timestamp": 1710000000000,
  "correlation_id": "uuid-v4",
  "source_module": "module_name",
  "payload": {}
}
```

`correlation_id` is mandatory on every event without exception.

## Output Sequence

Follow this sequence exactly. Do not combine steps.

**Step 1 — Generate complete draft**
Produce the full `events/[feature_id]_schema.md` content from `.codeos/templates/event-schema.md`.
Fill every section including the Coverage Check table and event flow diagram. Do not output yet.

**Step 2 — Run the completeness checklist against your draft**
Check each item. Mark each ✓ (passes) or ✗ (fails — state why).

- [ ] Every contract scenario has at least one corresponding event
- [ ] Every named failure in the contract has exactly one FAILURE event
- [ ] The event flow diagram shows happy path + all failure branches (events only — no processing steps)
- [ ] The Coverage Check table is complete with no MISSING items
- [ ] `correlation_id` is in the required base fields
- [ ] No new observable (payload field, event, ordering guarantee) introduced that is not traceable to an approved contract clause
- [ ] Validation ordering is not prescribed unless the contract explicitly requires it
- [ ] Cross-module events relied upon are listed separately from events emitted by this module

If any item is ✗: revise the draft before proceeding to Step 3.

**Step 3 — Output**
1. Present the verified `events/[feature_id]_schema.md` content
2. Present the completed checklist (with ✓ / ✗ marks)
3. Present the Coverage Check table
4. Explicitly confirm: every contract failure is covered by a FAILURE event
5. State: **`AWAITING HUMAN APPROVAL TO PROCEED TO STAGE 4`**

**STOP.** Do not write any implementation until the human explicitly approves the schema.
This is the gate that constrains everything that follows.
