# Stage 6: Runtime Execution and Event Capture

## Your Role

This stage is primarily **human-executed** — you run the implementation and observe what happens.
Claude's role here is advisory: help set up observation infrastructure if needed.

## What Happens in This Stage

1. Human runs the implementation (or automated tests)
2. System emits events to `events/runtime_events.jsonl`
3. The event log becomes operational truth

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

## Handoff to Stage 7

Once the system has been run and `events/runtime_events.jsonl` contains data, Stage 7 (Reconciliation Review) can begin.

Human signals readiness: "events captured, ready for Stage 7" or equivalent.

Claude does not advance to Stage 7 until the human confirms runtime events are available.

State when advisory work is done: **`STAGE 6 COMPLETE — AWAITING HUMAN CONFIRMATION OF RUNTIME EVENT CAPTURE`**
