# Behavioral Contract: [feature_id]

<!--
PURPOSE OF THIS FILE:
Defines observable truths derived from the approved intent.
Contracts describe OBSERVABLE behavior, not internal logic.
Every clause must be independently testable.
This file must be APPROVED before Stage 4 (implementation) begins.

DERIVED FROM: intents/[feature_id].md
-->

## Scenarios

### Happy Path

```gherkin
Given [precondition describing valid starting state]
When [actor performs trigger action]
Then [observable outcome — state change or event emitted]
And [additional observable outcome if needed]
```

### Failure Path 1: [FailureName]

```gherkin
Given [precondition]
When [trigger with failure condition]
Then [failure event is emitted]
And [system state is unchanged OR specific known state]
```

### Failure Path 2: [FailureName]

```gherkin
Given [precondition]
When [trigger with failure condition]
Then [failure event is emitted]
And [system state is unchanged OR specific known state]
```

<!--
Add additional failure paths to exhaustively cover failure_classifications below.
Every failure in the table must have a scenario.
-->

## Invariants

<!--
What must ALWAYS be true, regardless of inputs or execution path.
-->

- [invariant — always true]
- [invariant — always true]

## Preconditions

<!--
What must be true BEFORE this feature can execute.
-->

- [precondition]
- [precondition]

## Postconditions

<!--
What must be true AFTER successful execution.
-->

- [postcondition]
- [postcondition]

## Runtime Artifacts

<!--
Files and directories this feature creates or modifies at runtime, beyond the shared
event log. List every artifact explicitly, or state "none beyond events/runtime_events.jsonl".
This section must be completed before Stage 4 begins — undeclared files created during
implementation are a DBA violation.
-->

| Artifact | Path | Lifecycle |
|---|---|---|
| (none beyond events/runtime_events.jsonl) | — | — |

## Failure Classifications

<!--
Exhaustive list of named failure modes. Every failure here needs:
1. A scenario above
2. An event in the event schema
3. A test in tests/behavioral/
-->

| Failure Name | Trigger Condition | Observable Signal |
|---|---|---|
| [failure_name] | [when it occurs] | [event emitted or error returned] |
| [failure_name] | [when it occurs] | [event emitted or error returned] |

---

<!-- METADATA -->
status: DRAFT
feature_id: [feature_id]
approved_by:
approved_at:
derived_from_intent: intents/[feature_id].md
derived_event_schema: events/[feature_id]_schema.md
