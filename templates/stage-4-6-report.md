# Stage 4–6 Report Templates

> These templates produce the structured evidence that **Stage 7 Reconciliation** consumes.
> After completing each of **Stage 4** (Implementation), **Stage 5** (Tests), and **Stage 6**
> (Runtime Evidence), fill in the corresponding section below and attach it to the stage
> record. Do not leave fields blank — the reviewer and Stage 7 Reconciliation need explicit
> statements, not omissions.
>
> **Template instruction (not script enforcement):** Every field in each section must be
> filled in. When there is no content for a field, write `none`, `not run`, or
> `not applicable`. Do not leave fields empty.

---

## Stage 4 Implementation Report

*Complete after Stage 4 (Implementation). Evidence consumed by Stage 7 Reconciliation.*

**Template instruction:** Every field must be filled. When there is no content, write `none`,
`not run`, or `not applicable`. This is a template instruction, not script enforcement.

Feature:

Approved artifacts used:
- Intent:
- Contract:
- Event schema:

Files changed:

Files inspected but not changed:

Contract clauses implemented:

Schema events emitted:

Correlation ID propagation:

Runtime artifacts touched:

Unimplemented clauses:

Assumptions:

Blocked items:

Requires earlier-stage change:

Unexpected complexity:

---

## Stage 5 Test Report

*Complete after Stage 5 (Tests). Evidence consumed by Stage 7 Reconciliation.*

**Template instruction:** Every field must be filled. When there is no content, write `none`,
`not run`, or `not applicable`. This is a template instruction, not script enforcement.

Feature:

Approved artifacts used:

Behavioral tests added:

Failure-mode tests added:

Invariant tests added:

Telemetry/event tests added:

Replay tests added:

Tests run:

Tests passed:

Tests failed:

Tests skipped:

Tests not run:

Known test gaps:

Why gaps are acceptable or not acceptable:

---

## Stage 6 Runtime Evidence Report

*Complete after Stage 6 (Runtime Evidence). Evidence consumed by Stage 7 Reconciliation.*

**Template instruction:** Every field must be filled. When there is no content, write `none`,
`not run`, or `not applicable`. This is a template instruction, not script enforcement.

Feature:

How the system was run:

Input fixture/scenario:

Runtime command:

Runtime log path:

Events captured:

Unexpected events:

Missing expected events:

Correlation chains observed:

Sanitization status:

Raw logs committed:
- yes/no:
- if yes, why safe:

Derived replay fixtures produced:

Ready for reconciliation:

Known runtime gaps:
