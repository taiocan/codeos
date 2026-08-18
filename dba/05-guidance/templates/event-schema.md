# Event Schema: [feature_id]

<!--
PURPOSE OF THIS FILE:
Defines the complete event spine for this feature.
This is the most constraining artifact in the DBA loop.

Implementation may emit only governed events authorized here. Changes follow the selected doctrine
through the specification-approval adapter and the applicable verification path.

Every Specification Package contains this artifact. In `external-observation` mode it records that
the feature defines no governed internal events and maps Contract outcomes to the declared external
observation artifact.

DERIVED FROM:
- .codeos/01-specification/intents/<feature-id>.md (actors, outcomes)
- .codeos/01-specification/contracts/<feature-id>_contract.md (state transitions, failure modes)
-->

## Naming Convention

Use exact, stable event names in `<Entity><Action><Outcome>` form. Failure events normally end in
`Failed`, `Rejected`, or `Timeout`. Project-specific exceptions belong in project instructions or
approved artifacts, not in a synchronized conventions copy.

## Observation Mode

**Mode:** `events` | `external-observation`

For `events`, complete the event sections below. For `external-observation`, state:

- Governed internal events: `none`
- Observation artifact: [exact Contract reference]
- Outcome coverage: [Contract outcome → observable evidence]

Then remove Required Base Fields, Event Definitions, Event Flow, and Cross-module events as
inapplicable; retain the Coverage Check using Contract outcomes instead of failures/events.

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
When the project uses `events/runtime_events.jsonl`, append one event per line and never rewrite
existing evidence.

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
- emitted when: [failure condition from .codeos/01-specification/contracts/<feature-id>_contract.md]
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
In `events` mode, every governed failure that uses an event signal must appear here as an authorized
FAILURE event and every event/payload field must trace to the Contract. In
`external-observation` mode, map every Contract outcome to the declared observation evidence.
Fill this table before submitting for approval.
-->

| Contract Requirement | Governed Event / Observation Evidence | Status |
|---|---|---|
| [scenario, outcome, or failure] | [EventName or external evidence] | COVERED / MISSING |

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
derived_from_intent: .codeos/01-specification/intents/<feature-id>.md
derived_from_contract: .codeos/01-specification/contracts/<feature-id>_contract.md
