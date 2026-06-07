# Architectural Refinement Workflow

## Your Role

You guide a structural change to project infrastructure. You are NOT implementing a
behavioral feature. This workflow has no behavioral contract, no event schema, and no
BDD scenarios. You are a constrained change guide — every step requires human approval.

## When This Workflow Applies

Use this workflow (not the 9-step DBA loop) when the change is:
- Cargo workspace restructuring or module boundary reorganization
- Shared infrastructure extraction (event utilities, test harness, replay helpers)
- Dependency consolidation or version alignment
- Test infrastructure improvements (shared fixtures, test runner changes)
- Build system or CI changes
- File and directory layout normalization
- Naming convention normalization across existing modules

Do **NOT** use this workflow for:
- New observable behavior (use 9-step feature loop)
- Changes to approved event schemas (use Stage 9 refinement)
- New API surfaces visible to end users (use 9-step feature loop)

If you are unsure whether a change is architectural or behavioral — it is behavioral if
it would change any row in a feature's Contract or Event Schema. If yes, use Stage 9
refinement. If no, use this workflow.

---

## The 5-Step Architectural Refinement Pipeline

Each step requires explicit human approval before proceeding to the next.
After each step output, state: **`AWAITING HUMAN APPROVAL TO PROCEED TO STEP [N+1]`**

---

### Step 1 — Scope Intent

**Your task:** Define what is changing and why. Produce the Scope Intent section of
`refinements/arch/[refine_id].md` (using `.codeos/templates/arch-refinement.md`).

**Rules:**
- State the change in terms of structure, not behavior
- Name every module, directory, or artifact that will be created, moved, or removed
- State the motivation — what problem does this solve?
- State what will NOT change — especially: no behavioral contracts change, no event
  schemas change, no observable outputs change

**Scope Intent is complete when:**
- [ ] The structural change is named concisely (one or two sentences)
- [ ] Every affected artifact or module is listed
- [ ] Motivation is stated
- [ ] Scope boundary (what stays the same) is explicit

Output: Scope Intent section + **`AWAITING HUMAN APPROVAL TO PROCEED TO STEP 2`**

---

### Step 2 — Impact Analysis

**Your task:** Analyze which features and artifacts are affected. Produce the Impact
Analysis section of `refinements/arch/[refine_id].md`.

**What to assess:**
1. **Existing features**: which features' code will be moved, renamed, or restructured?
2. **Artifact paths**: which `intents/`, `contracts/`, `events/`, `tests/` paths will change?
3. **Regression risk**: which behavioral tests could break due to the structural change?
4. **Dependency graph**: what does the change assume about load order, import paths, or crate boundaries?
5. **Cross-feature blast radius**: for each hub or module in scope, identify which features
   reference it. Cross-reference `docs/codebase-digest.md` (Module Cluster Map) against
   `features/registry.yaml` (artifact module paths). A hub shared by multiple features
   means a wider blast radius — name those features and their risk level explicitly.

**Rules:**
- Do not assume zero regression risk — state the risk level and evidence for it
- Flag any feature whose test suite references specific file paths or module boundaries
  that will change; these tests need updates as part of Step 3
- If the impact is larger than expected from Step 1, return to Step 1 and narrow scope

**Impact Analysis is complete when:**
- [ ] Every affected module and feature is listed with a risk level
- [ ] Every artifact whose path changes is listed
- [ ] Regression risks are named, not dismissed
- [ ] Cross-feature blast radius assessed: features sharing affected hubs are named

Output: Impact Analysis section + **`AWAITING HUMAN APPROVAL TO PROCEED TO STEP 3`**

---

### Step 3 — Implement

**Your task:** Make the structural change. Update test paths and import statements as
needed. Do not change any behavioral logic.

**Rules:**
- Every file move or rename: update all import/use statements in the same commit
- If a test path changes: update the test reference, not the test assertion
- Do not add new behavioral logic while making structural changes — one concern at a time
- If you discover a needed behavioral change during implementation, stop and flag it:
  it must go through the 9-step feature loop or Stage 9 refinement, not here
- Run the existing test suite after the change; all tests must pass

**Implementation is complete when:**
- [ ] All planned moves/renames completed
- [ ] All import/use statements updated
- [ ] All test references updated (paths, not assertions)
- [ ] Existing test suite passes (or regressions are documented with root cause)

Output: Summary of changes made + **`AWAITING HUMAN APPROVAL TO PROCEED TO STEP 4`**

---

### Step 4 — Verify

**Your task:** Confirm no behavioral regression occurred. Produce the Verification
section of `refinements/arch/[refine_id].md`.

**What to verify:**
1. All existing behavioral tests pass
2. No observable outputs changed (event types, payload shapes, error messages)
3. All artifact paths in `features/registry.yaml` are updated if paths changed

**Verification table:**

| Check | Result | Notes |
|---|---|---|
| All behavioral tests pass | PASS / FAIL | List any failures |
| No new events emitted | CONFIRMED / EXCEPTION | Describe any exception |
| No removed events | CONFIRMED / EXCEPTION | Describe any exception |
| Artifact registry paths updated | COMPLETE / N/A | |

**Verification is complete when:**
- [ ] Verification table is filled in
- [ ] Any FAIL or EXCEPTION is explained and traced to a root cause
- [ ] If a test fails due to behavioral change: escalate to Stage 9 refinement

Output: Verification section + **`AWAITING HUMAN APPROVAL TO PROCEED TO STEP 5`**

---

### Step 5 — Reconcile

**Your task:** Check that existing feature artifacts remain accurate. Produce the
Reconciliation Notes section of `refinements/arch/[refine_id].md`.

**What to check:**
1. Do any `intents/`, `contracts/`, or `events/` files reference file paths or module
   names that changed? If so, these are documentation GAPs — mark them and note the
   minimum correction.
2. Does `features/registry.yaml` need artifact path updates?
3. Are there new shared artifacts (crates, modules) that should be documented in
   `patterns/` or referenced in `CLAUDE.md`?

**Reconciliation is complete when:**
- [ ] Every documentation GAP is named with the minimum correction
- [ ] Feature registry is up to date
- [ ] Any new reusable pattern is noted for documentation

Output: Reconciliation Notes section + **`AWAITING HUMAN APPROVAL — ARCHITECTURAL REFINEMENT COMPLETE`**

Set `status: COMPLETE` in `refinements/arch/[refine_id].md`.
