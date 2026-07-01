# Stage 4 Activation Card

> **Optional execution aid.** Absence of this card does not block any DBA stage transition
> and does not add a new approval gate. The intent, contract, and event schema artifacts are
> authoritative — this card references them and records operational metadata only. In any
> conflict between this card and an approved artifact, the approved artifact takes precedence.
>
> Primary scope: Stage 4 (Implementation Prep). May be reused lightly in Stage 5
> (Implementation) and Stage 6 (Runtime Verification).

---

## Feature

Feature ID: ___
Change ID: ___

---

## Approved input artifacts

> Reference paths only. Do not restate behavioral content.

- Intent: ___
- Contract: ___
- Event schema: ___

---

## Current repo state

- Branch: ___
- Commit SHA: ___
- Working tree: ___ (clean / dirty — describe any pending changes)
- Active feature: ___
- Current approved stage: ___

---

## Branch policy

- Existing feature branch: ___ (yes / no)
- New branch required: ___ (yes / no)
- Proposed branch name: ___

Branch name convention: `feature/<feature_id>` (default).
For split PRs: `feature/<feature_id>-artifacts`, `-implementation`, `-runtime-replay`, `-refinement`.

Human approval is required before creating a new branch unless project policy explicitly allows automatic branch creation.

---

## Implementation scope

- Files likely in scope: ___
- Files explicitly out of scope: ___

---

## Execution constraints

- Do not change approved artifacts (intent, contract, event schema).
- Do not add events not defined in the event schema.
- Do not add behavior not traceable to the contract.
- Stop and surface to human if implementation requires new behavior not covered by approved artifacts.
- Report if contract or schema appears insufficient — do not silently expand scope.

---

## Required output

- Files changed: ___
- Contract clauses satisfied: ___
- Events emitted: ___
- Tests not yet written: ___
- Runtime evidence not yet captured: ___
- Risks / blockers: ___
