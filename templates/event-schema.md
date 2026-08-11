# Event Schema: [feature_id]

<!--
PURPOSE OF THIS FILE:
Defines the complete event spine for this feature.
This is the most constraining artifact in the DBA loop.

Implementation may emit only governed events authorized here. Changes follow the selected doctrine
through the specification-approval adapter and the applicable verification path.

DERIVED FROM:
- intents/[feature_id].md (actors, outcomes)
- contracts/[feature_id]_contract.md (state transitions, failure modes)
-->

## Naming Convention

See `docs/conventions.md` (source: `.codeos/templates/conventions.md`).

## Required Base Fields (all events)

Every event must include these fields:

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

`correlation_id` is mandatory and must propagate through the entire execution chain.

## Design Notes

<!--
Record implementation context, timing, and rationale here.
This section is for implementers — it is NOT part of the observable event contract.
Put here: processing loops, ordering assumptions, cross-module timing, architectural decisions.
Do NOT put these in Event Definitions or the Event Flow diagram.
-->

## Event Definitions

<!--
Categories:
  OBSERVATIONAL — raw runtime facts (e.g., RequestReceived)
  BEHAVIORAL    — verified outcomes (e.g., CartItemAdded)
  FAILURE       — classified error conditions (e.g., CartItemAddFailed)
  EXTERNAL      — side effects on outside systems (e.g., EmailSent)
-->

### [EventName]

- category: OBSERVATIONAL | BEHAVIORAL | FAILURE | EXTERNAL
- emitted when: [specific condition that triggers this event]
- payload:
  - `[field_name]`: `[type]` — [description]
  - `[field_name]`: `[type]` — [description]

### [EventName]

- category: OBSERVATIONAL | BEHAVIORAL | FAILURE | EXTERNAL
- emitted when: [condition]
- payload:
  - `[field_name]`: `[type]` — [description]

### [FailureEventName]

- category: FAILURE
- emitted when: [failure condition from contracts/[feature_id]_contract.md]
- payload:
  - `failure_reason`: `string` — [snake_case reason code]
  - `[field_name]`: `[type]` — [description]

## Event Flow

<!--
Show the expected sequence of events for the happy path.
Show branching for failure paths.
-->

```text
[EventA]              ← emitted on: [trigger]
  ↓
[EventB]              ← emitted on: [condition]
  ↓ (on success)
[EventC]
  ↓ (on failure: [failure_name])
[FailureEventD]
```

## Cross-module events relied upon

<!--
Events emitted by OTHER modules that this feature depends on for its observable behavior.
These are NOT emitted by this module. List them here to make the dependency explicit.
If none, state "none".
-->

| Event | Source module | Contract clause |
|---|---|---|
| (none) | — | — |

## Coverage Check

<!--
Every failure in contracts/[feature_id]_contract.md must appear here as a FAILURE event.
Every new payload field or event must trace to an approved contract clause.
Fill this table before submitting for approval.
-->

| Contract Failure | Event Here | Status |
|---|---|---|
| [failure_name from contract] | [FailureEventName] | COVERED / MISSING |

<!--
Also verify before submitting:
- [ ] No new observable (payload field, event, ordering guarantee) introduced beyond the contract
- [ ] Validation ordering not prescribed unless the contract explicitly requires it
- [ ] Event flow diagram contains events only — no processing steps
-->

---

<!-- METADATA — recording behavior is owned by the specification-approval doctrine adapter -->
status: DRAFT
feature_id: [feature_id]
approved_by:
approved_at:
derived_from_intent: intents/[feature_id].md
derived_from_contract: contracts/[feature_id]_contract.md
