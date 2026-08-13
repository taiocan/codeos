# Stage 6: Runtime Execution and Event Capture

## Your Role

Run representative scenarios when the environment permits. If an action is designated as
controlled, obtain human authorization before running it. Never fabricate runtime evidence.

## What Happens in This Stage

1. The agent or human runs the implementation (or automated tests) as permitted
2. System emits events to `events/runtime_events.jsonl`
3. The event log becomes verification evidence for what occurred

## The Runtime Event Log

File: `events/runtime_events.jsonl`

Format: one JSON event object per line (JSONL).
This file is **append-only**. Never delete or modify existing lines.

Example entries:
```jsonl
{"event_id":"evt-001","event_type":"RequestReceived","timestamp":1710000001000,"correlation_id":"c-abc123","source_module":"api_gateway","payload":{"feature_id":"user_login","actor":"user_42"}}
{"event_id":"evt-002","event_type":"UserLoginSucceeded","timestamp":1710000001050,"correlation_id":"c-abc123","source_module":"auth_module","payload":{"user_id":"user_42","session_id":"sess-xyz"}}
```

## Advisory: Event Capture Setup

If the implementation does not yet write to `events/runtime_events.jsonl`, Claude can help add event emission:

- Each event emission writes a complete JSON line to the file
- `correlation_id` must be included in every line
- Events must match the approved schema exactly (no additional fields in `event_type`)

## What to Observe

After running the system, check:
- Are events appearing in `events/runtime_events.jsonl`?
- Do event types match the approved schema?
- Are correlation chains intact (same `correlation_id` across a feature execution)?
- Are there any unexpected events (not in the schema)?

## Failure Path Coverage

Attempt runtime observation of each named failure in the contract's Failure Classifications
table. For each path not yet observed, create a *runtime fixture* — the minimal setup that
reproducibly triggers that path.

Common fixture strategies:

| Failure condition | Fixture strategy |
|---|---|
| Schema absent or invalid | Write broken/missing schema to project dir before running |
| Empty record | Run against a fresh project dir with no incorporated items |
| No recognized types | Write a schema defining only a type no existing item uses; isolate runtime from global schema (e.g., `HOME=<temp>`) |
| External service unavailable | Remove API key or point to unreachable endpoint |
| Resource not found | Supply a non-existent ID in the command arguments |

Where direct runtime observation is impractical or environmentally constrained (cloud
outages, third-party failures, race conditions, production-only infrastructure), document
the reason and classify the row in Stage 7 as **GAP (runtime evidence)**. The goal is
*observe or explicitly justify why not* — not a mandatory pass/fail gate on every path.

## Handoff to Stage 7

Once representative evidence is available, hand it to `.codeos/dba/03-prompts/workflow/07-reconcile.md`. If
evidence cannot be obtained, record the reason as a GAP and do not invent it.
