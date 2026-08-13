# Feature Specification: [feature_id]

<!--
PURPOSE OF THIS FILE:
Detailed operational guidance for a single observable transformation.
Derived from approved intent + contract.

This is NOT the same as intent (which states WHY).
This states WHAT observable transformation occurs and HOW to verify it.

Features exist between capabilities and implementation.
A feature represents one observable transformation, one measurable outcome,
one traceable execution boundary.

RULES:
- One feature = one observable transformation = one meaningful outcome
- If this spans multiple outcomes, split into multiple features
- All events here must match events/[feature_id]_schema.md exactly
- Implementation-independent — describes behavior, not code
-->

## Purpose

[Single sentence: what observable transformation this feature performs]
Formula: `[Feature] transforms [Input] into [Observable Outcome]`

## Inputs

- `[input_name]`: `[type]` — [description]
- `[input_name]`: `[type]` — [description]
- `correlation_id`: `uuid` — REQUIRED, must propagate to all emitted events

## Outcome

[Single sentence: what state of the world is true after successful execution]

## Transformation

```text
Given [precondition A]:
  → [action performed]
  → emit [BehavioralEventName]

Given [precondition B — failure condition]:
  → [action performed]
  → emit [FailureEventName]
    with failure_reason: [snake_case_reason]
```

## Observability

### Events
<!-- Must match events/[feature_id]_schema.md exactly. No additions. -->

- `[EventName]`: emitted when [specific condition]
- `[EventName]`: emitted when [specific condition]
- `[FailureEventName]`: emitted when [failure condition]

### Metrics

- `[feature_id]_duration_ms`: duration from trigger to terminal event
- `[feature_id]_success_rate`: ratio of BEHAVIORAL to FAILURE events
- `[metric_name]`: [what it measures]

### Required Log Fields

Every log entry during this feature's execution must include:
- `correlation_id`
- `feature_id`: `[feature_id]`
- `outcome`: `success` | `[failure_reason]`
- `[additional_field]`: [description]

## Errors

<!-- All errors must appear as FAILURE events in the event schema -->

| Error Name | Condition | Event Emitted | Failure Reason |
|---|---|---|---|
| `[error_name]` | [when it occurs] | `[FailureEventName]` | `[snake_case_reason]` |
| `[error_name]` | [when it occurs] | `[FailureEventName]` | `[snake_case_reason]` |

---

<!-- METADATA -->
status: DRAFT
feature_id: [feature_id]
approved_by:
approved_at:
derived_from_intent: intents/[feature_id].md
derived_from_contract: contracts/[feature_id]_contract.md
derived_from_schema: events/[feature_id]_schema.md
