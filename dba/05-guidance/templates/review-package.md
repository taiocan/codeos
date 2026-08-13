# Review Package Template

A Review Package is produced inline when the selected review policy and the applicable doctrine
adapter call for one. It is **not written to disk**. It is a convenience view for copy-paste to an
external reviewer and does not define a decision boundary.

**Regenerated packages are convenience views, not historical records.** A package
regenerated after artifact changes may differ from the one originally reviewed. The
authoritative record of what was reviewed and decided is in `reviews/[feature_id].md`.

---

## Specification Package Format (Intent, Contract, Event Schema)

```
---
## Review Package — Specification Package: [feature_id]

Artifacts: `intents/[feature_id].md`, `contracts/[feature_id]_contract.md`, `events/[feature_id]_schema.md`
Purpose: verify mutual consistency and readiness to govern implementation

Suggested areas to examine:
- [question 1]
- [question 2]
- [question 3]

Known tensions from brief or prior stages: [or "none"]

[Full artifact content]
---
```

---

## Stage 4–5 Format (Implementation, Tests)

```
---
## Review Package — Stage [N]: [feature_id]

Artifact: `[path(s)]`
Stage purpose: [one sentence]

Files changed: [list]
Key architectural decisions: [decisions not fully determined by prior approved artifacts]
What is not covered yet: [explicit list — gaps or deferred paths]

Suggested areas to examine:
- [question 1]
- [question 2]
- [question 3]

Known tensions from prior stages: [or "none"]

[Key output tables]
---
```

---

## Stage 7 Format (Reconciliation)

```
---
## Review Package — Stage 7 (Reconciliation): [feature_id]

Stage purpose: Structural comparison of all artifacts for gaps, mismatches, and schema drift.

Verdict: N ALIGNED / N GAP / N MISMATCH / N MISSING

Non-ALIGNED items:
- [item] — [one-line description]
(or: "none")

Not measured by this reconciliation:
- Performance / benchmarks
- Security audit
- [any other explicitly out-of-scope dimension]

Suggested areas to examine:
- [question 1]
- [question 2]
- [question 3]

[Full reconciliation table and Findings Summary]
---
```

---

## Stage 8–9 Format (Replay, Refinement)

```
---
## Review Package — Stage [N] ([Replay/Refinement]): [feature_id]

Stage purpose: [one sentence]

What was verified / changed:
- [item]

Verdict: PASS / FAIL ([N] issues found)

What would make this stage stronger: [or "none — evidence is sufficient"]

Suggested areas to examine:
- [question 1]
- [question 2]
- [question 3]

[Full stage output]
---
```

---

## Usage Notes for Stage Prompts

Stage prompts reference this template by saying:

> Present the Review Package using `.codeos/dba/05-guidance/templates/review-package.md` ([format variant], inline only):

Then list the stage-specific fields:
- Artifact path
- Stage purpose (one sentence)
- Three suggested questions specific to this stage
- Known tensions inherited from prior stages

"Suggested areas to examine" — not "Assess specifically." These are starting points.
The reviewer is free to ignore every suggestion and raise something entirely different.
