# Review Package Template

A Review Package is produced inline (in the conversation) at the end of each stage,
immediately before `AWAITING HUMAN APPROVAL`. It is **not written to disk**. It is a
convenience view for copy-paste to an external reviewer.

**Regenerated packages are convenience views, not historical records.** A package
regenerated after artifact changes may differ from the one originally reviewed. The
authoritative record of what was reviewed and decided is in `reviews/[feature_id].md`.

---

## Stage 1–3 Format (Intent, Contract, Schema)

```
---
## Review Package — Stage [N]: [feature_id]

Artifact: `[path]`
Stage purpose: [one sentence — what this stage was supposed to produce]

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
What is not covered yet: [explicit list — gaps, MANUAL-PENDING items, deferred paths]

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

> Present the Review Package using `.codeos/templates/review-package.md` ([format variant], inline only):

Then list the stage-specific fields:
- Artifact path
- Stage purpose (one sentence)
- Three suggested questions specific to this stage
- Known tensions inherited from prior stages

"Suggested areas to examine" — not "Assess specifically." These are starting points.
The reviewer is free to ignore every suggestion and raise something entirely different.

**Controlled Plain English (if the relevant project has enabled it — see
`.codeos/dba/policies/controlled-plain-english/v1.md`):** Layer D1 (reviewer
integrity — advisory verdict, no invented requirements, evidence separated from inference) always
applies to this Review Package's prose, in both the Stage 4-5 and Stage 7 format variants. Layer D2
(plain review prose) applies only when the relevant activation is `enabled`.
