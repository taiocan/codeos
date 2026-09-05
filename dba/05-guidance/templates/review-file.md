---
artifact_type: review_file
---

# Review-Process Measurement

<!-- Exempt from the Summary block and reader_model declaration: already a compact measurement. -->

> Optional, non-authoritative analytical evidence. The reviewer tool owns automated review records;
> the applicable doctrine boundary owns approval. This file evaluates the review process and is
> neither a reviewer log nor a feature-status artifact.

Create an instance only when measuring review effectiveness or cost will inform a concrete process
decision. Do not create one for every feature or review.
When saved, use `.codeos/05-review/measurements/<name>.md`.

## Measurement Scope

Period or experiment: [bounded measurement scope]

Tool-owned record references: [existing review/change identifiers or paths]

Question being evaluated: [the process question this analysis answers]

## Derived Measures

Derivation: [query or calculation against tool-owned records]

Result: [rounds, findings, rework, elapsed effort, or other relevant measure]

Do not manually synchronize values that can be recalculated from the authoritative records.

## Evaluation

Useful findings: [human assessment, referencing existing finding identifiers where needed]

False or irrelevant findings: [human assessment, referencing existing identifiers where needed]

Rework or review cost: [effect on the work]

Effectiveness conclusion: [what was learned about the review process]

Process action: [specific change justified by the measurement, or none]

Do not copy or independently maintain finding bodies, verdicts, approval state, feature lifecycle
state, or review history. Reference the tool-owned record when that context is required.
