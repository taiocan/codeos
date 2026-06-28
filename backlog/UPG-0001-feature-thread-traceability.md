---
feature_id: UPG-0001
slug: feature-thread-traceability
title: Feature Thread Traceability and Stable ID Nomenclature
status: IN_PROGRESS
priority: P0
class: self-dev-governance
scope: self-dev only
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Feature Brief: Feature Thread Traceability and Stable ID Nomenclature

## Metadata

* feature_id: UPG-0001
* slug: feature-thread-traceability
* title: Feature Thread Traceability and Stable ID Nomenclature
* status: IN_PROGRESS
* priority: P0
* class: self-dev-governance
* scope: self-dev only
* depends_on:

  * UPG for Codeos self-development split, if assigned
  * UPG for implementation roadmap, if assigned
* related_files:

  * backlog/features.md
  * status/roadmap.md
  * status/self-development.md
  * templates/codeos-change.md
  * prompts/codeos-self-dev.md
  * reviews/review-log.md
  * scripts/codeos-review.sh

## Problem

Codeos currently risks confusing four different identities:

1. the stable backlog feature or upgrade;
2. the concrete self-development change that implements it;
3. review rounds for that change;
4. follow-up fixes or backlog items created from reviewer findings.

The recent roadmap/review-fix work showed the failure mode clearly: a review fix for one change was assigned the next numeric ID, making it look like the next feature. Then review rounds created fresh bookkeeping facts, causing the dashboard and change records to fall out of sync.

The system needs a robust traceability model where every related file visibly shows which feature it belongs to, without requiring the user to open every change record and inspect metadata.

## Goal

Introduce a stable Feature Thread model and ID convention so that all work related to the same feature is visible across backlog files, change files, review files, status dashboards, and roadmaps.

The user must be able to answer these questions from filenames and status tables:

* Which feature does this change belong to?
* Which change records implement this feature?
* Which review rounds belong to this change?
* Which review findings were fixed inside the same change?
* Which findings became follow-up backlog features?
* Which features are planned, active, complete, superseded, or blocked?

## Core Concept: Feature Thread

A Feature Thread is the persistent identity and traceability spine for one Codeos upgrade.

A feature is the stable thing.

A change is one execution attempt against a feature.

A review is evidence about one step of one change.

A finding is either resolved inside the current change or promoted to a new backlog feature.

## ID Types

### Stable feature ID

Format:

```text
UPG-####
```

Example:

```text
UPG-0025
```

Meaning:

A stable backlog feature or upgrade ID. It is assigned once, never reused, and never renumbered.

Rules:

* Every non-trivial backlog feature file must have exactly one `feature_id`.
* The `feature_id` is authoritative over filename order.
* IDs are never reused after abandonment, merge, supersession, deletion, or completion.
* Historical/piloted work may receive IDs during migration without pretending it originally followed the new process.

### Change ID

Format:

```text
CHG-YYYYMMDD-NNN
```

Example:

```text
CHG-20260627-001
```

Meaning:

One self-development execution record through the 4-step Codeos loop.

Rules:

* A change ID is not a feature ID.
* A change may implement one feature, part of one feature, or, with explicit approval, a small group of related features.
* Review fixes for in-scope findings stay inside the same change.
* A new change is created only when the current change is closed, scope changes, or an out-of-scope backlog item is accepted.

### Review round ID

Format:

```text
REV__UPG-####__CHG-YYYYMMDD-NNN__S<N>__R<N>
```

Example:

```text
REV__UPG-0025__CHG-20260627-001__S4__R2
```

Meaning:

One reviewer run for one step of one change.

Rules:

* Review rounds are not features.
* Review rounds are not change records.
* Review rounds must not create new change numbers.
* Review history lives in the change record and review files, not as separate feature rows.

### Finding ID

Optional format:

```text
FND__REV__UPG-####__CHG-YYYYMMDD-NNN__S<N>__R<N>__NN
```

Example:

```text
FND__REV__UPG-0025__CHG-20260627-001__S4__R2__01
```

Meaning:

A specific reviewer finding that needs explicit tracking.

Rules:

* Use finding IDs only when the finding needs explicit resolution tracking.
* Findings are classified as:

  * IN-SCOPE BLOCKER
  * IN-SCOPE NON-BLOCKER
  * OUT-OF-SCOPE BACKLOG
  * REJECTED
* IN-SCOPE BLOCKER findings are fixed inside the same change.
* OUT-OF-SCOPE BACKLOG findings create or link to a new `UPG-####`.

## Required Filename Convention

### Backlog feature files

Pattern:

```text
backlog/UPG-####-slug.md
```

Example:

```text
backlog/UPG-0025-feature-thread-traceability.md
```

### Change records

Pattern:

```text
changes/UPG-####__CHG-YYYYMMDD-NNN__slug.md
```

Example:

```text
changes/UPG-0025__CHG-20260627-001__feature-thread-traceability.md
```

Rules:

* Every non-trivial change filename must include the primary `UPG-####`.
* The filename must also include the unique `CHG-*` ID.
* The feature ID in the filename gives immediate visual grouping.
* The change ID preserves execution uniqueness.
* The slug should describe the concrete change, not the entire roadmap.

### Multiple-feature changes

If a change touches more than one feature, use the primary feature ID in the filename and list additional affected features in metadata.

Example filename:

```text
changes/UPG-0025__CHG-20260627-002__id-migration.md
```

Trace header:

```yaml
primary_feature_id: UPG-0025
implements:
  - UPG-0025
related_features:
  - UPG-0012
  - UPG-0018
```

Use `MULTI` only when there is genuinely no primary feature and the human explicitly approves a multi-feature change.

Example:

```text
changes/MULTI__CHG-20260627-003__roadmap-dashboard-alignment.md
```

This should be rare.

### Review files

> **Documented manual convention — not enforced or auto-emitted by this change.** Renaming
> existing review files and adding `scripts/codeos-review.sh` support are a deferred follow-up
> `UPG-####` (see Migration Plan step 9 and Acceptance Criterion 9).

Pattern:

```text
reviews/codex/REV__UPG-####__CHG-YYYYMMDD-NNN__S<N>__R<N>.md
```

Example:

```text
reviews/codex/REV__UPG-0025__CHG-20260627-001__S4__R1.md
```

Review packet:

```text
reviews/codex/REV__UPG-0025__CHG-20260627-001__S4__R1.packet.txt
```

## Trace Header

Every related file should include a compact trace header.

### Backlog feature file header

```yaml
---
feature_id: UPG-0025
slug: feature-thread-traceability
title: Feature Thread Traceability and Stable ID Nomenclature
status: PROPOSED
priority: P0
class: self-dev-governance
scope: self-dev only
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---
```

### Change record header

```yaml
---
feature_id: UPG-0025
primary_feature_id: UPG-0025
change_id: CHG-20260627-001
slug: feature-thread-traceability
state: IN_PROGRESS
current_step: 4-Reconcile
implements:
  - UPG-0025
related_features: []
latest_review: REV__UPG-0025__CHG-20260627-001__S4__R1
fixes_findings: []
follow_up_of: null
---
```

### Review record header

```yaml
---
review_id: REV__UPG-0025__CHG-20260627-001__S4__R1
feature_id: UPG-0025
change_id: CHG-20260627-001
step: selfdev-step-4
round: 1
verdict: CHANGES ADVISED
evidence: A
coverage: FULL_COVERAGE
---
```

## Feature Thread Section

Every backlog feature file must include a `## Feature Thread` section.

Template:

```markdown
## Feature Thread

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260627-001 | changes/UPG-0025__CHG-20260627-001__feature-thread-traceability.md | Initial implementation | IN_PROGRESS |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
```

Rules:

* This section is the canonical thread rollup.
* It may be manually maintained initially.
* It should stay compact: links and IDs, not full review text.
* Full details remain in change records and review files.

## Dashboard Model

`status/self-development.md` should separate feature identity from change identity.

Required columns:

```text
Feature ID | Change ID | Class | Scope | Loop step | Latest review | State | Follow-up
```

Example:

```markdown
| Feature ID | Change ID | Class | Scope | Loop step | Latest review | State | Follow-up |
|---|---|---|---|---|---|---|---|
| UPG-0025 | CHG-20260627-001 | self-dev-governance | self-dev only | 4-Reconcile | CHANGES ADV S4/R1 | IN_PROGRESS | — |
```

Rules:

* `Feature ID` identifies the backlog feature.
* `Change ID` identifies the active implementation record.
* `Latest review` is informational only.
* Detailed review history lives in the change record and review files.
* The dashboard must not try to encode every review finding.
* If review findings are tracked to a future feature, use `Follow-up: UPG-####`.

## Roadmap Model

`status/roadmap.md` should use feature IDs as primary keys.

Required columns:

```text
Wave | Feature ID | Title | Priority | Depends on | Planned/active change | State
```

Example:

```markdown
| Wave | Feature ID | Title | Priority | Depends on | Planned/active change | State |
|---|---|---|---|---|---|---|
| 1 | UPG-0025 | Feature Thread Traceability | P0 | — | CHG-20260627-001 | IN_PROGRESS |
```

Rules:

* Roadmap sequencing is by `UPG-####`.
* Change IDs are execution details.
* Existing pilot work may be recorded as current state without retroactive false sequencing.

## Review-Fix Rule

A review fix remains inside the same `CHG-*` if:

* it addresses an IN-SCOPE BLOCKER;
* it does not alter the approved scope;
* it only repairs implementation, documentation, status, acceptance, or review-record consistency for the current change.

A review fix creates or links a new `UPG-####` only if:

* it is OUT-OF-SCOPE BACKLOG;
* it changes the approved intent or acceptance criteria materially;
* it introduces a new feature, policy, workflow, file type, or tool behavior;
* it would make the current change unreviewably broad.

A review fix must not receive the next feature ID merely because it happened after a review.

## State Rules

Feature state values:

```text
PROPOSED
PLANNED
IN_PROGRESS
PILOTED
COMPLETE
BLOCKED
SUPERSEDED
ABANDONED
```

Change state values:

```text
DRAFT
IN_REVIEW
IN_PROGRESS
BLOCKED
COMPLETE
ABANDONED
SUPERSEDED
```

Review verdict values:

```text
NO OBJECTION
CHANGES ADVISED
DO NOT ADVANCE
SKIPPED
```

Rules:

* A feature can remain `IN_PROGRESS` across multiple changes.
* A change becomes `COMPLETE` only after its in-scope blockers are resolved or explicitly accepted by the human.
* Advisory findings tracked to future `UPG-####` items do not prevent the current change from closing if they are out of scope.
* Historical pilot work may be marked `PILOTED` instead of being forced into false `COMPLETE` sequencing.

## Required File Updates

This feature should update:

```text
prompts/codeos-self-dev.md
templates/codeos-change.md
status/self-development.md
status/roadmap.md
backlog/features.md
backlog/*.md
reviews/review-log.md
docs/codeos-manual.md, if it describes self-dev IDs
README.md, if it describes self-dev workflow
```

Optional later tooling:

```text
scripts/check_feature_threads.sh
scripts/codeos-review.sh packet naming support
```

Do not add mandatory tooling unless the manual convention proves insufficient.

## Migration Plan

1. Assign stable `UPG-####` IDs to every active backlog feature file.
2. Rename backlog files to `backlog/UPG-####-slug.md`, or keep filenames and add front matter if renaming is too disruptive.
3. Update `backlog/features.md` to map each `UPG-####` to one backlog file.
4. Update `status/roadmap.md` to sequence features by `UPG-####`.
5. Update `status/self-development.md` to include both `Feature ID` and `Change ID`.
6. Update `templates/codeos-change.md` with the new trace header and filename convention.
7. Update `prompts/codeos-self-dev.md` so Step 1 creates or selects the feature thread before creating a change record.
8. **New** change records use `changes/UPG-####__CHG-YYYYMMDD-NNN__slug.md`. **Existing**
   historical change records (`changes/0001..0004`) are **not** renamed and receive **no**
   invented `CHG-*` IDs or dates — they are recorded truthfully as historical/piloted (see
   step 10).
9. The `REV__UPG-####__CHG-…__S<N>__R<N>` review id is **documented** as the manual naming
   convention. Actual review-file renaming and `codeos-review.sh` output support are a
   **deferred follow-up `UPG-####`** — reviewer behavior is not changed by this work.
10. Record historical exceptions truthfully instead of pretending old pilot work followed the new scheme.

## Acceptance Criteria

1. Every active backlog feature file contains a unique `feature_id`.
2. Every active backlog feature file has a `## Feature Thread` section.
3. Every non-trivial change filename includes the primary `UPG-####`.
4. Every non-trivial change filename includes a unique `CHG-*`.
5. Every change record trace header lists `feature_id`, `primary_feature_id`, `change_id`, and `implements`.
6. `backlog/features.md` maps each `UPG-####` to exactly one backlog file.
7. `status/roadmap.md` uses `UPG-####` as the primary feature identity.
8. `status/self-development.md` separates `Feature ID` from `Change ID`.
9. Each review identifies both the feature and the change in **at least one** of: the review
   filename, the packet content, a review-log entry, or the change record's Feature Thread.
   (Renaming review files to the `REV__…` pattern and teaching `scripts/codeos-review.sh` to
   emit it is a **deferred follow-up `UPG-####`**, not required by this change.)
10. No in-scope review fix is represented as a new backlog feature unless explicitly triaged OUT-OF-SCOPE BACKLOG.
11. Existing historical/piloted work is represented truthfully without false retroactive sequencing.
12. The user can identify which feature a change file belongs to by filename alone.
13. The user can open the backlog feature file and see all related changes, reviews, tracked findings, and follow-up features.
14. Grep checks find no remaining prose that treats plain `0003`, `0004`, etc. as both feature IDs and change IDs.

## Verification

Run:

```bash
grep -rn "feature_id:" backlog/
grep -rn "UPG-" backlog/ status/ changes/ reviews/ README.md docs/ prompts/ templates/
grep -rn "CHG-" changes/ status/ reviews/ prompts/ templates/
grep -rn "000[0-9]" status/ changes/ backlog/ docs/ README.md prompts/ templates/
```

Manual checks:

* no duplicated `UPG-####`;
* no duplicated `CHG-*`;
* every change filename includes a visible feature ID;
* every dashboard row separates feature and change identity;
* roadmap rows use feature IDs, not change numbers;
* review fixes are recorded inside the current change unless explicitly promoted to backlog;
* the Feature Thread section in each backlog file lists related changes and follow-ups.

## Supported Scenarios

This feature must support:

1. trivial direct edit with no change record;
2. one feature implemented by one change;
3. one feature implemented by multiple changes;
4. one change touching multiple approved related features;
5. review fixes inside the same change;
6. out-of-scope reviewer findings promoted to new backlog features;
7. abandoned changes;
8. superseded features;
9. historical pilot work predating the thread model;
10. roadmap-only planning changes;
11. docs-only normative changes;
12. script/tooling changes;
13. downstream-doctrine changes;
14. self-dev-governance changes.

## Non-Goals

This feature does not:

* implement any roadmap feature beyond the ID/thread system;
* change the downstream DBA doctrine;
* redesign the reviewer pipeline;
* turn the reviewer into an enforcement engine;
* require a PR per feature;
* require generated dashboards;
* require mandatory automation before the convention is proven;
* force historical pilot work into false numbering.

## Suggested Change Record

Use this as the implementation change:

```text
changes/UPG-00XX__CHG-YYYYMMDD-001__feature-thread-traceability.md
```

## Suggested Commit Title

```text
Codeos: add feature-thread traceability and stable IDs
```
