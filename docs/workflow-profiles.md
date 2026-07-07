# Workflow Profiles

Optional guidance for branch, PR, and CI discipline in DBA projects. The existing Stage 9
commit-and-push-to-main flow is always valid for small work. Choose a profile when more
structure helps.

---

## Choosing a Profile

| Profile | Use when |
|---|---|
| **A — Simple local** | Solo work; small feature (1–2 stages); low risk; no CI; no external review needed |
| **B — Branch per feature** | Feature spans 3+ stages; other contributors may touch the same code; CI exists; reviewer needs staged visibility |
| **C — Split PRs** | Feature is large or risky; multiple subsystems touched; security / compliance concern; reviewer needs smaller diffs to give a useful signal |

When in doubt, start with B. It adds minimal ceremony and preserves stage history.

---

## Profile A — Simple Local Flow

```
Run Stages 1–9 locally.
Commit after Stage 9.
Push to main.
Optional: run a reviewer pass before the final commit.
```

**Notes:**
- No branch required; work directly on main.
- The Stage 9 commit message becomes the sole audit trail, so write it carefully.
- If the feature grows mid-flight, switch to Profile B rather than continuing on main.

---

## Profile B — Branch per Feature (Recommended Default)

```
1. Create feature/<feature_id> from main.
2. Commit after each approved stage (keeps gate history visible in git log).
3. Open a draft PR after Stage 3 (approved artifacts) or Stage 5 (approved
   implementation), whichever comes first.
4. Run CI as soon as implementation and tests exist (Stage 5 or Stage 6).
5. Merge to main only after Stage 8 (approved; pending runtime) or Stage 9 (complete).
```

**Why one commit per stage:** each gate becomes a checkpoint. If Stage 7 reveals a
problem, you can see exactly what was approved at Stage 5 without reading prose. It also
lets a reviewer diff from any approved gate forward.

**Notes:**
- Use `feature/<feature_id>` as the branch name (e.g. `feature/EVT-0012`).
- Draft PR is optional before Stage 5 if no CI is configured yet.
- Squash-merge is fine at the end if stage history in the PR description is sufficient.

---

## Profile C — Split PRs for Large or Risky Features

Split the feature into up to four sequential PRs, each merged before the next opens.

| PR | What it contains | Merge gate | Branch |
|---|---|---|---|
| PR 1 — Artifacts | Intent, contract, event schema (Stages 1–3) | Stages 1–3 approved | `feature/<feature_id>-artifacts` |
| PR 2 — Implementation | Code, behavioral tests, telemetry tests (Stages 4–6) | Stages 4–6 approved + CI green | `feature/<feature_id>-implementation` |
| PR 3 — Runtime evidence | Sanitized fixtures, replay tests, reconciliation reports (Stages 7–8) | Stages 7–8 approved | `feature/<feature_id>-runtime-replay` |
| PR 4 — Refinement | Only if Stage 9 required a substantive change | Stage 9 approved | `feature/<feature_id>-refinement` |

**Notes:**
- Not all four PRs are always needed. A feature with no replay evidence skips PR 3.
- Each PR should be reviewable in isolation; the reviewer agent gets the diff for that
  PR only, not the full feature history.
- If PR 2 reveals a design issue, go back to PR 1 (already merged) via a follow-up
  change, not by amending the merged branch.

**What Profile C Does Not Require:**
- Automated branch creation — create each branch manually with `git checkout -b`, same as
  Profile B. No script or helper tool is introduced by naming these branches.

---

## Reviewer-Agent Access Model

When running a reviewer pass in a branch or PR context, give the reviewer agent access to
the following evidence (as available):

```
Approved artifacts for the current stage
Full git diff of the branch (or PR diff)
Stage reports (Stage 4 contract, Stage 5 schema, Stage 6 test report, …)
Test output / CI log
Runtime fixtures or replay evidence (Stages 7–8)
```

Suggested checklist for the reviewer to work through:

```
Diff scope:
  - Which files changed?
  - Any files changed that are not in the approved scope?

Approved artifacts:
  - Were any approved Stage 1–3 artifacts modified after approval?

Implementation ↔ contract:
  - Does the implementation match the approved contract and event schema?

Tests ↔ contract:
  - Do tests cover the failure modes named in the contract?
  - Any test that is not traceable to a contract clause?

Runtime evidence:
  - Is runtime / replay evidence present where Stages 7–8 require it?

Risk:
  - Hidden behavior or side effects not mentioned in the contract?
  - CI green?

Recommended decision: NO OBJECTION / CHANGES ADVISED / DO NOT ADVANCE
```

This checklist is guidance for the reviewer prompt; it is not enforced by the pipeline.
Reviewer output is advisory and non-gatekeeping at every profile (see `dba-system.md`).

---

## What Profile B Does Not Require

- A mandatory PR template
- CI configuration (use CI when it exists; profiles work without it)
- Automated branch creation (create the branch manually with `git checkout -b`)
- A specific merge strategy (merge, rebase, or squash — team preference)

---

## Relationship to Existing Workflow

These profiles sit on top of the 9-stage DBA loop. They govern *how commits and PRs are
structured*, not *what is approved at each gate*. The human-approval gates and reviewer
cadence defined in `dba-system.md` are unchanged.

Profile selection is a human decision made at the start of a feature, not enforced by any
tool. If you change your mind mid-feature (e.g. a Profile A feature grows into Profile B
territory), switching is fine — just create the branch from the current commit and keep
going.
