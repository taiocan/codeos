# Refinement Log: [feature_id]

<!--
PURPOSE OF THIS FILE:
Records the history of targeted behavioral refinements for this feature.
Each entry must be problem-driven — triggered by observed operational evidence.

REFINEMENT TRIGGERS (only these justify a refinement):
1. Recurring failure — same failure mode appearing repeatedly in runtime_events.jsonl
2. Reconciliation gap — identified in a Stage 7 reconciliation report
3. Replay failure — identified in a Stage 8 replay report
4. Observability gap — behavior occurring but cannot be diagnosed
5. Human-approved evolution — new requirement that extends (not replaces) intent

FORBIDDEN TRIGGERS:
- Theoretical improvements not backed by observation
- Architecture changes for elegance or future-proofing
- Single incidents that haven't recurred

REFINEMENT COST ORDER (prefer cheapest):
1. Observability refinement — add metric, improve event, add correlation field
2. Behavioral refinement — clarify ambiguous contract clause
3. Reliability refinement — add idempotency, retry limit, transaction boundary
4. Performance refinement — only after measured bottleneck
5. Structural refinement — split module, isolate workflow (most expensive, rare)
-->

---

## Refinement [YYYY-MM-DD]: [Short Description]

### Trigger

<!-- Which of the 5 valid triggers applies? -->
Trigger type: RECURRING_FAILURE | RECONCILIATION_GAP | REPLAY_FAILURE | OBSERVABILITY_GAP | HUMAN_APPROVED_EVOLUTION

### Observed Problem

[What was observed — specific runtime events, reconciliation table row, replay report finding]

### Evidence

```
[Paste relevant runtime_events.jsonl lines or reconciliation table rows]
```

### Root Cause

[The specific behavioral mechanism that caused the problem — not a guess]

### Refinement Type

OBSERVABILITY | BEHAVIORAL | RELIABILITY | PERFORMANCE | STRUCTURAL

### Minimal Change

[The smallest change that addresses the root cause without touching unrelated code]

Artifacts changed:
- [ ] `intents/[feature_id].md`
- [ ] `contracts/[feature_id]_contract.md`
- [ ] `events/[feature_id]_schema.md`
- [ ] Implementation in `modules/`
- [ ] `tests/behavioral/`
- [ ] `tests/replay/`

### Stages Re-run

<!-- List which DBA stages were re-run due to this refinement and in what order -->
- [ ] Stage 2: Contracts (if intent changed)
- [ ] Stage 3: Event Schema (if contracts changed)
- [ ] Stage 4: Implementation (if schema changed)
- [ ] Stage 5: Tests (always, if implementation changed)
- [ ] Stage 7: Reconciliation Review
- [ ] Stage 8: Replay Verification

### Validation

[How we confirmed the refinement solved the problem and didn't introduce regressions]

---

<!-- Add new refinement entries above this line, newest first -->
