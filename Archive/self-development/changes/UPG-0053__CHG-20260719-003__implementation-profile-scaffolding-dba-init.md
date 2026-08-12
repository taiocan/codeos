# Self-Development Change: UPG-0053__CHG-20260719-003 — implementation-profile-scaffolding-dba-init

<!--
PURPOSE: Per-change source of truth for a non-trivial change to the Codeos toolkit
itself (prompts, templates, docs, patterns, scripts).

This is NOT a downstream DBA feature. It has no behavioral contract, no event schema,
and no replay. Trivial changes do not get a record.

Workflow: prompts/codeos-self-dev.md (4-step loop)
Each step requires explicit human approval; Codex review cadence is governed by the assigned review profile (see prompts/codeos-self-dev.md Step 0a).
The live status row lives in status/self-development.md, not here.
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0053
primary_feature_id: UPG-0053
change_id: CHG-20260719-003
slug: implementation-profile-scaffolding-dba-init
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0053
related_features: [UPG-0052]
review_series: RVS__UPG-0053__CHG-20260719-003__S4
review_profile: PROFILE-3
review_state: ACCEPTED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

<!-- SELF-REFERENCE BOUNDARY: this artifact is itself reviewed, so it must NOT embed the current
review round (which does not exist until after the packet is built). Reference the stable review
SERIES (review_series) + review_state; exact rounds live only in reviews/review-log.md and
reviews/codex/*. See prompts/codeos-self-dev.md → "Feature Thread & IDs" / "Self-Reference Boundary". -->


## Change Intent

**Why (problem in the toolkit):**

`scripts/dba-init.sh` has no knowledge of the Implementation Profile artifact `UPG-0052`
introduced (`architecture/implementation-profile.yaml`, template at
`templates/implementation-profile.yaml`). Without scaffolding, every new downstream project needs
this file created by hand before it means anything (per `UPG-0052`'s session-start/onboarding
awareness, now shipped in `CHG-20260719-002`). There is also a real risk of `dba-init.sh` instead
over-reaching — auto-generating Cargo/workspace structure before an Architecture Synthesis Gate
(`UPG-0051`) has approved any crate topology, locking in structure before evidence exists — which
this change must explicitly not do.

**What changes:**

1. `scripts/dba-init.sh` — add a new numbered step, **"8. Implementation Profile"**, inserted
   after the existing step 7 (Codebase digest placeholder) and before the existing step 8 (Git
   init). This renumbers the four subsequent steps: old 8 (Git init) → 9, old 9 (Git remote) → 10,
   old 10 (Reviewer config) → 11, old 11 (Done) → 12. The new step:
   - Creates `architecture/` (via `mkdir -p`) if it doesn't exist — the directory itself is
     optional/independently-created per `UPG-0051`/`UPG-0052`'s doctrine, this is simply where
     `dba-init.sh` first needs it.
   - If `architecture/implementation-profile.yaml` already exists, **skip** (idempotent — matches
     every other step's existing `[skip]`-if-present pattern in this script, e.g. steps 3, 5, 6,
     7, 10).
   - Otherwise, copy `templates/implementation-profile.yaml` verbatim to
     `architecture/implementation-profile.yaml` — **no `sed` substitution needed**, unlike
     `CLAUDE.md`/the codebase digest, since the template's defaults (`status: proposed`,
     `primary_language: rust`) already are the correct scaffolded state; there is no
     `[PROJECT_NAME]`-style placeholder in this template to fill in.
   - Prints an `[ok]`/`[skip]` line matching the script's existing convention.
   - The "Done" step's closing "Next steps" text gains one line mentioning the scaffolded profile
     file, consistent with how the registry/digest/reviewer-config files are already mentioned
     there.

**Scope boundary — what stays the same:**

- No Cargo/workspace generation logic is added anywhere in this script — confirmed by reading the
  full current script: it contains no such logic today, and this change does not introduce any.
- No `dba-system.md` (or any other doctrine/prompt/template) text is touched — `UPG-0052` already
  shipped the doctrine and the profile template this change consumes; this change only wires the
  init script to use them.
- No explicit init CLI modes (e.g. `--implementation-profile none|rust-first`) — the backlog brief
  explicitly flags this as an open, non-essential design question, not committed for v1.
- No change to any of the script's other 11 existing steps beyond renumbering their comment
  headers (8→9, 9→10, 10→11, 11→12) to make room for the new step 8 — their logic is untouched.
- Verification is a **live scratch run** of the script (as `UPG-0050`'s prior `dba-init.sh`-touching
  change did — confirmed no dedicated automated test harness exists for this script), not a new
  permanent test file; "smoke-test coverage" in the backlog brief's Scope refers to this.

**Class:** script-tooling
**Scope axis:** self-dev only
**Backlog item:** backlog/UPG-0053-implementation-profile-scaffolding-dba-init.md

---

## Acceptance Criteria

| # | Criterion | How it will be verified |
|---|---|---|
| 1 | Fresh scratch project: `architecture/implementation-profile.yaml` is created, byte-identical to `templates/implementation-profile.yaml` (`status: proposed`, `primary_language: rust`, no substitution needed). | Live scratch run; `diff` the generated file against the template. |
| 2 | Missing parent directory: if `architecture/` does not exist, it is created (`mkdir -p` semantics — no error if it already exists as a directory). | Scratch run against a project with no pre-existing `architecture/`; confirm the directory and file both exist afterward. |
| 3 | Idempotency — existing file never overwritten: re-running `dba-init.sh` after hand-editing `architecture/implementation-profile.yaml` (e.g. changing `primary_language` or `status: approved`) leaves the file byte-identical to the hand-edited version. | Scratch run: init once, edit the file, init again, `diff` before/after the second run — must be empty. |
| 4 | The scaffolded file never has `status: approved` — it is always written as the template's own default (`proposed`), never pre-approved by the script. | Read the generated file after a fresh scratch run; confirm `status: proposed`. |
| 5 | No Cargo/workspace files are created anywhere by this change. | Scratch run: `find` the scratch project for `Cargo.toml`/`Cargo.lock` before and after running `dba-init.sh` — both empty. |
| 6 | Existing steps' behavior is unchanged beyond (a) comment-header renumbering (8→9, 9→10, 10→11, 11→12) and (b) the one intentional addition to the final "Next steps" echo block (AC8) — no other logic in any other step is altered. | `git diff -- scripts/dba-init.sh` shows only the new step's block added, renumbered comment headers, and the one added "Next steps" line; no other line changed. |
| 7 | The new step introduces no new error-suppression: it relies on the script's existing `set -euo pipefail` (already in effect for every step) rather than adding its own `\|\| true` or ignored exit code, so a failure in `mkdir -p` or `cp` aborts the whole script the same way a failure in any other step already does. This is a fail-closed guarantee, not a claim that `cp` is atomic or interruption-safe — a `cp` killed mid-copy can leave a partial destination file, exactly as it can for any other file this script writes (e.g. the existing `CLAUDE.md`/registry/digest copies), and this change does not change that pre-existing property. | Read-through of the new step's code confirms no `\|\|`, no `2>/dev/null`-style suppression, no subshell that would swallow a non-zero exit; a scratch run's exit code is `0` on a normal, uninterrupted run. |
| 8 | The "Next steps" closing text mentions the scaffolded profile file, consistent with how the registry/digest/reviewer-config files are already mentioned there. | Read-through of the script's final echo block. |

---

## Implementation Notes

<!-- Summary only — the git diff is the source of truth. -->

Single file touched: `scripts/dba-init.sh`. `git diff` confirms exactly the planned change: the
new "8. Implementation Profile" step block, the four renumbered comment headers (8→9, 9→10,
10→11, 11→12 for Git init/Git remote/Reviewer config/Done), and the one added "Next steps" block
— no other line in the script changed.

The new step follows the file's existing `[skip]`-if-present idiom exactly (same shape as steps
3, 5, 6, 7, 11): `mkdir -p` before `cp` so a missing `architecture/` is created transparently, no
`sed` substitution (unlike `CLAUDE.md`/digest, since the template's own defaults are already the
correct scaffolded state), and no new error-suppression added — relies on the script's existing
`set -euo pipefail`.

**Nothing was deferred or discovered out-of-scope during implementation.**

---

## Reconciliation

**Acceptance verification (live scratch run, `mktemp -d` scratch project, exit code 0 throughout):**

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Content matches template verbatim | PASS | `diff templates/implementation-profile.yaml <scratch>/architecture/implementation-profile.yaml` → empty diff. |
| 2 | Missing parent directory created | PASS | `ls -la <scratch>/architecture/` shows the directory and file both present after a run against a project with no pre-existing `architecture/`. |
| 3 | Idempotency — hand-edited file never overwritten | PASS | Edited the scratch file (`status: approved`, `primary_language: python`), re-ran `dba-init.sh`, `diff` before/after the second run → empty; log shows `[skip] architecture/implementation-profile.yaml already exists`. |
| 4 | Never scaffolded as `approved` | PASS | Fresh scratch file: `status: proposed`. |
| 5 | No Cargo/workspace files created | PASS | `find <scratch> -iname "Cargo.toml" -o -iname "Cargo.lock"` → 0 results. |
| 6 | Diff containment (new block + renumbering + one Next-steps addition, nothing else) | PASS | `git diff --stat` → 1 file, 23 insertions/4 deletions, matching exactly the planned insertion + 4 renumbered headers + 5-line Next-steps addition (verified line-by-line in Step 3). |
| 7 | No new error-suppression | PASS | `grep -E '\|\||2>/dev/null'` over the new step's block → no matches. |
| 8 | Next-steps text mentions the profile | PASS | New block present, consistent with the registry/digest/reviewer-config mentions already there. |

All 8 criteria PASS.

**Consistency sweep:** `bash -n scripts/dba-init.sh` (syntax check, Step 3) passed; no other file
was touched, so no cross-reference sweep beyond confirming `templates/implementation-profile.yaml`
(from `UPG-0052`) is the exact file copied — confirmed by AC1's empty diff.

**Findings scope-triage:**

| Finding | Triage | Action |
|---|---|---|
| Step 2 R1: false claim that `cp` is atomic/interruption-safe | IN-SCOPE BLOCKER | Fixed — AC7 reworded to the actual guarantee (no new error-suppression, relies on existing `set -euo pipefail`), false atomicity claim removed |
| Step 2 R1: AC6 ("no other line changed") contradicted AC8 (Next-steps addition) | IN-SCOPE BLOCKER | Fixed — AC6 now explicitly excludes the one intentional Next-steps addition |

---
