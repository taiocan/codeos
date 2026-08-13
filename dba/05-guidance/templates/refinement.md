# Refinement Log: [feature_id]

<!--
PURPOSE OF THIS FILE:
Records the history of targeted behavioral refinements for this feature.
Each entry must be problem-driven — triggered by evidence or an explicit human evolution decision.

REFINEMENT TRIGGERS (only these justify a refinement):
1. Observed behavioral failure
2. Reconciliation GAP, MISMATCH, or MISSING result
3. Final-verification failure
4. Observability gap
5. Human-approved evolution

FORBIDDEN TRIGGERS:
- Theoretical improvements not backed by observation
- Architecture changes for elegance or future-proofing
- Unapproved scope expansion

A single safety, authorization, or integrity failure is sufficient evidence; recurrence is not
required. Choose the smallest effective change for the actual cause, without a fixed type or cost
ordering.
-->

---

## Refinement [YYYY-MM-DD]: [Short Description]

### Trigger

Trigger: [observed failure, reconciliation result, final-verification failure, observability gap,
or human-approved evolution]

### Observed Problem

[What was observed — specific runtime events, reconciliation table row, replay report finding]

### Evidence

```
[Paste the smallest relevant runtime/external-observation evidence, test result, reconciliation
row, or human decision]
```

### Root Cause

[The specific behavioral mechanism that caused the problem — not a guess]

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
- [ ] Stage 1: Intent (if approved meaning changed)
- [ ] Stage 2: Contract (if observable behavior changed)
- [ ] Stage 3: Event Schema and package approval (if any Specification Package artifact changed)
- [ ] Stage 4: Implementation (if conformance work is required)
- [ ] Stage 5: Tests (always, if implementation changed)
- [ ] Stage 6: Runtime Evidence (when representative execution is applicable)
- [ ] Stage 7: Reconciliation Review
- [ ] Stage 8: Final Verification

### Validation

[How we confirmed the refinement solved the problem and didn't introduce regressions]

---

<!-- Add new refinement entries above this line, newest first -->
