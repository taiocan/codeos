# Architectural Refinement: [refine_id]

<!--
PURPOSE: Documents a structural change to project infrastructure that is not
driven by behavioral feature requirements.

This is NOT a feature. It has:
  - no behavioral contract
  - no event schema
  - no BDD scenarios

Use when: workspace restructuring, shared library extraction, dependency
consolidation, test infrastructure improvement, naming normalization.

Workflow: .codeos/dba/03-prompts/workflow/10-arch-refine.md (5-step pipeline)
Each step requires explicit human approval.
-->

## Scope Intent

**What is changing:**
[One or two sentences describing the structural change]

**Motivation:**
[What problem this solves or what improvement it makes]

**Scope boundary — what stays the same:**
[Explicit statement of what behavioral contracts, event schemas, and observable outputs
are not changing. Any item not listed here is in scope for change.]

**Artifacts created / moved / removed:**

| Action | Artifact / Path |
|---|---|
| Created | [new path or file] |
| Moved | [old path] → [new path] |
| Removed | [path] |

---

## Impact Analysis

**Affected modules and features:**

| Module / Feature | How affected | Risk |
|---|---|---|
| [module or feature_id] | [restructured / moved / path changed / extracted] | [high / medium / low] |

**Artifact paths changing:**

| Artifact | Old path | New path |
|---|---|---|
| [intent / contract / schema / test] | [old] | [new] |

**Regression risk assessment:**
[Narrative: which tests are most likely to break and why. State "low risk" only if
you can name the evidence — e.g., "tests use relative paths and will update
automatically with import changes."]

---

## Implementation Notes

<!--
Filled in during Step 3. Summary of what was changed.
Do not write a line-by-line change log here — the git diff is the source of truth.
Note only: decisions made, unexpected discoveries, anything that was deferred.
-->

[Summary of structural changes made. Note any behavioral changes discovered and deferred.]

---

## Verification

| Check | Result | Notes |
|---|---|---|
| All behavioral tests pass | PASS / FAIL | [list failures if any] |
| No new events emitted | CONFIRMED / EXCEPTION | [describe exception if any] |
| No removed events | CONFIRMED / EXCEPTION | [describe exception if any] |
| Artifact registry paths updated | COMPLETE / N/A | |

---

## Reconciliation Notes

**Documentation GAPs found:**

| Artifact | GAP | Minimum correction |
|---|---|---|
| [contracts/feature.md] | [references old module path] | [update to new path] |

**New patterns to document:**
[If this refinement introduced a new reusable infrastructure pattern, note it here
for addition to dba/05-guidance/patterns/ or CLAUDE.md.]

---

<!-- METADATA -->
status: DRAFT
refine_id: [refine_id]
type: ARCHITECTURAL_REFINEMENT
step_completed: 0
approved_by:
approved_at:
