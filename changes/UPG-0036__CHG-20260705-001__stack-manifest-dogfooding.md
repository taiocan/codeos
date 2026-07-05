---
change_id: CHG-20260705-001
feature_id: UPG-0036
slug: stack-manifest-dogfooding
triage_class: self-dev-governance
scope_axis: self-dev only
review_profile: PROFILE-5
review_series: RVS__UPG-0036__CHG-20260705-001__S1
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0036 / CHG-20260705-001 — Stack Manifest & Drift Reconciliation Dogfooding

## TRACE HEADER

```yaml
feature_id: UPG-0036
primary_feature_id: UPG-0036
change_id: CHG-20260705-001
slug: stack-manifest-dogfooding
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0036
related_features:
  - UPG-0017
  - UPG-0020
review_series: RVS__UPG-0036__CHG-20260705-001__S1
review_profile: PROFILE-5
review_state: ACCEPTED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

---

## Step 1 — Change Intent

### Problem

`templates/stack-manifest.md` + `templates/stack-reconciliation-report.md` (UPG-0017) and the
`check-drift` subcommand (UPG-0020) are complete, tested, and COMPLETE in the roadmap — but
only as downstream-facing machinery. Codeos has never instantiated a stack manifest for its
own `tools/reviewer` Rust crate. `tools/reviewer/Cargo.toml` has changed exactly once before
(UPG-0032 / `CHG-20260702-001`, establishing the original 9 dependencies: `anyhow`, `chrono`,
`clap`, `hex`, `regex`, `serde`, `sha2`, `tempfile`, `toml`), with no reconciliation record at
all. If Codeos requires this discipline of downstream projects but never applies it to
itself, the toolkit is performative rather than dogfooded.

### What changes

| File | Change |
|---|---|
| `CLAUDE.md` | New, narrowly-scoped rule: dependency/stack-file changes must be declared in Step 1 or re-triaged; Step 2 adds a verification criterion when a watched file is touched; Step 4 verifies the reconciliation-report instance exists and runs `check-drift`. Expressed in 4-step-loop terms only — no downstream Stage 9/10/readiness-checklist language. |
| `status/stack-manifest.md` | New: live status file recording Codeos's actual stack (Rust/Cargo, `cargo test`, no event log/replay, dependency approval = Step 1/2 human gate), explicitly labeled as evidence/status, not an independent approval authority. |
| `status/stack-reconciliation/CHG-20260702-001-stack-reconciliation-report.md` | New: retroactive, clearly-labeled historical record for UPG-0032's original 9-dependency addition — not a claim that this process existed at the time. |
| `backlog/UPG-0036-stack-manifest-dogfooding.md` | Feature Thread: CHG-20260705-001 activated. Untracked/new — not yet committed at time of this review; shown in full in the review packet. |
| `backlog/features.md` | UPG-0036 row added. Not yet committed at time of this review; shown as a diff in the review packet. |
| `status/self-development.md` | Row activated. **Already committed** in `142bd02` (bundled, by mistake, with UPG-0023's separate Step 1 approval commit) — not visible in this review's diff-vs-HEAD as a result. Verifiable via `git show 142bd02 -- status/self-development.md`. |
| `status/roadmap.md` | UPG-0036 row added, unsequenced. **Already committed** in `142bd02` for the same reason as the row above — verifiable via `git show 142bd02 -- status/roadmap.md`. |

### Scope boundary — what stays the same

- `dba-system.md` — **not touched**. This is self-dev governance, not downstream doctrine.
  If any part of this change were later found to require a downstream-doctrine edit, it would
  be re-triaged as `downstream-doctrine` (or `both`) before proceeding — not folded in here.
- `templates/stack-manifest.md` / `templates/stack-reconciliation-report.md` — **not
  modified**. Codeos reuses their existing field structure for its own instances rather than
  forking the downstream templates.
- `tools/reviewer/src/cmd/check_drift.rs` — **not modified**. Its existing hardcoded
  `WATCHED_EXACT` list already covers `Cargo.toml`/`Cargo.lock`; this change documents which
  of its watched patterns actually apply to this repo (only those two — the rest are
  generic/downstream-oriented and currently match nothing here), it does not change the tool.
- **No new governance layer.** `status/stack-manifest.md` is evidence/status, same tier as
  `status/self-development.md` and `status/roadmap.md` — never authority. Human approval at
  every Step 1–4 gate remains the sole authority; the manifest cannot override or bypass it.
- **UPG-0023 is not backfilled here.** UPG-0023 (the approval-dashboard change, currently
  parked at Step 1 awaiting Step 2 approval) has not yet touched `Cargo.toml` — its
  `serde_yaml` addition hasn't happened yet. Only UPG-0032 is backfilled, because it is the
  one dependency change that has actually already occurred without a reconciliation record.
  UPG-0023's own Step 3/4 becomes the *first live application* of the trigger rule this change
  establishes, not a second backfill.
- No autonomous enforcement. `check-drift` remains a manually-run, advisory, read-only tool;
  running it at Step 4 is a human-followed practice, not a CI gate or hook.

### Design intent

**Trigger rule** (to be added to `CLAUDE.md`, in 4-step-loop terms — not downstream stage
terms):

> Dependency/stack-file changes must be declared in Step 1 "What changes," or explicitly
> re-triaged before implementation if discovered later. If Step 1 declares a watched-file
> change, Step 2 must include a verification criterion for it. Step 4 verifies the
> stack-reconciliation-report instance exists and runs `check-drift` against the change.
> Human approval at each gate remains the authority; the manifest and reconciliation reports
> are evidence, not authority.

**Watched files for this repo:** `Cargo.toml`, `Cargo.lock` — the only two of
`check-drift`'s hardcoded watched patterns (`pyproject.toml`, `package.json`, `Dockerfile`,
`config/*.yaml`, etc.) that exist in this repo today. `status/stack-manifest.md` states this
explicitly rather than re-listing the tool's full generic pattern set as if all of it applied.

**`status/stack-manifest.md` authority disclaimer** (verbatim, per the corrected wording):

> This file records the current observed stack and dependency-policy status for Codeos
> self-development. It is not an independent approval authority. If it conflicts with the
> self-dev workflow, `CLAUDE.md` and the approved change record govern.

**Reconciliation report instance location:** `status/stack-reconciliation/<CHG-id>-stack-
reconciliation-report.md` — one file per change that touches a watched file, named so its
suffix (`stack-reconciliation-report.md`) matches `check-drift`'s existing `ends_with()`
check regardless of directory. The UPG-0032 backfill instance is dated as of *this* change
(2026-07-05), not backdated to UPG-0032's original commit date, and is explicitly labeled
retroactive in its own text.

### Triage

- Class: `self-dev-governance` (changes the self-dev loop trigger rule via `CLAUDE.md`)
- Scope axis: `self-dev only`
- Review profile: `PROFILE-5`
- Originating backlog id: `UPG-0036`

---

## Step 2 — Acceptance Criteria

**AC-1 — Manifest exists and records the real facts**
`status/stack-manifest.md` exists and records: Rust/Cargo as the runtime/build stack,
`cargo test` as primary verification, "not applicable" for event log/replay (self-dev has
none — matches `CLAUDE.md`'s Mode Declaration), and the dependency approval rule (Step 1/2
human gate).
_Verify in Step 4:_ read the file; confirm each of the four facts is present.

**AC-2 — Explicit non-authority disclaimer**
The manifest carries this disclaimer verbatim: "This file records the current observed stack
and dependency-policy status for Codeos self-development. It is not an independent approval
authority. If it conflicts with the self-dev workflow, `CLAUDE.md` and the approved change
record govern."
_Verify in Step 4:_ the sentence's wording (not necessarily on one physical line — prose in
this repo wraps at ~90 chars) must be present in full. Verify with line-wrap collapsed:
`sed 's/^> //' status/stack-manifest.md | tr '\n' ' ' | grep -o "<sentence>"`.

**AC-3 — No downstream stage language in the trigger rule**
The new `CLAUDE.md` section is expressed only in 4-step-loop terms (references Step 1 / Step
2 / Step 4) and contains no "Stage 9", "Stage 10", or "readiness checklist" language.
_Verify in Step 4:_ grep the new section (case-insensitive) for `Stage 9`, `Stage 10`,
`readiness checklist` — zero matches; confirm Step 1/2/4 are each referenced.

**AC-4 — Watched files stated explicitly, scoped to this repo**
`status/stack-manifest.md` lists `Cargo.toml` and `Cargo.lock` as the watched files that
actually apply to this repo, and states that `check-drift`'s other hardcoded patterns
(`pyproject.toml`, `package.json`, `Dockerfile`, `config/*.yaml`, etc.) exist for downstream
repos and currently match nothing here.
_Verify in Step 4:_ read the file; confirm both statements are present.

**AC-5 — Trigger rule's three clauses are all present**
The `CLAUDE.md` addition states, at minimum: (a) a watched-file change must be declared in
Step 1's "What changes," or explicitly re-triaged if discovered later; (b) if declared, Step
2 must include a verification criterion for it; (c) Step 4 verifies the
stack-reconciliation-report instance exists and runs `check-drift`.
_Verify in Step 4:_ read the new section; confirm all three clauses (a)/(b)/(c) are present,
each traceable to a specific sentence.

**AC-6 — UPG-0032 backfill only; UPG-0023 explicitly not backfilled**
`status/stack-reconciliation/CHG-20260702-001-stack-reconciliation-report.md` exists,
documents UPG-0032's original 9 dependencies (`anyhow`, `chrono`, `clap`, `hex`, `regex`,
`serde`, `sha2`, `tempfile`, `toml`) by name, and is explicitly labeled retroactive/historical
— it must not claim this reconciliation process existed at the time of UPG-0032's actual
commit. No `serde_yaml`/UPG-0023 reconciliation-report instance is created by this change —
that is UPG-0023's own Step 3/4 responsibility once it actually touches `Cargo.toml`.

The "one real historical gap" premise itself must be mechanically checked, not merely
asserted: `git log --oneline -- tools/reviewer/Cargo.toml tools/reviewer/Cargo.lock` must
show exactly one commit across *both* watched files (not just `Cargo.toml` alone) before this
change's own commit(s). If that command shows more than one prior commit, the "backfill only
UPG-0032" claim is false and this AC fails — the additional prior change(s) would also need a
retroactive reconciliation-report instance.
_Verify in Step 4:_ (a) run the `git log` command above and record its output in the change
record; confirm it shows exactly one prior commit (`a66bda9`, UPG-0032's `CHG-20260702-001`);
(b) read the backfill instance; confirm all 9 dependencies are named and the retroactive
framing is present; (c) confirm `status/stack-reconciliation/` contains exactly one file (the
UPG-0032 backfill) and nothing referencing `serde_yaml` or UPG-0023.

**AC-7 — `check-drift` actually run against this change**
This change's own Step 4 records the literal `check-drift` invocation run against the Codeos
repo for this change's diff, and its exit code/output — a functional demonstration that the
mechanism works, not just a description of it.
_Verify in Step 4:_ the Step 4 section shows the exact command run and its result.

**AC-8 — No doctrine-specific language leaks into the self-dev artifacts**
Neither `status/stack-manifest.md` nor the new `CLAUDE.md` section contains "Stage 9",
"Stage 10", or "replay" as an *active* concept (a negation such as "no event log/replay" is
fine — that is stating the concept doesn't apply, not importing it as doctrine).
_Verify in Step 4:_ grep both files; manually confirm any "replay" hit is a negation, not an
active reference.

**AC-9 — `dba-system.md` untouched**
This change does not modify `dba-system.md`.
_Verify in Step 4:_ `git diff --stat -- dba-system.md` is empty for this change's commits.

**AC-10 — No autonomous enforcement; human gates preserved**
No hook, CI configuration, or auto-blocking mechanism is added. `check-drift` remains a
manually-invoked, read-only, advisory tool. The `CLAUDE.md` rule's text includes, verbatim,
"Human approval at each gate remains the authority; the manifest and reconciliation reports
are evidence, not authority."
_Verify in Step 4:_ confirm no hook/CI file is added or modified in this change's diff; the
sentence's wording must be present in full (prose wraps at ~90 chars, so a naive single-line
grep is not the check) — verify with line-wrap collapsed:
`tr '\n' ' ' < CLAUDE.md | grep -o "<sentence>"`.

---

## Step 3 — Implement

### What was done

| File | Change |
|---|---|
| `CLAUDE.md` | New "## Stack / Dependency Reconciliation" section (inserted after "### Gate discipline", before "## What You NEVER Do"). States the evidence-not-authority framing, the watched-files pointer, and the three trigger clauses (Step 1 declare/re-triage, Step 2 verification criterion, Step 4 verify + `check-drift`). |
| `status/stack-manifest.md` | New. Records Codeos's actual stack facts, the non-authority disclaimer, the two watched files that apply to this repo, the dependency policy, the reconciliation trigger, and a History table with the UPG-0032 backfill entry. |
| `status/stack-reconciliation/CHG-20260702-001-stack-reconciliation-report.md` | New. Retroactive/historical reconciliation instance for UPG-0032's original 9 dependencies, explicitly labeled as written 2026-07-05 (not backdated), using the existing template's field structure without the downstream "readiness-checklist" framing line. |

### Verification (AC-1 through AC-10)

- **AC-1/AC-4**: `status/stack-manifest.md` states Rust/Cargo, `cargo test`, "not applicable" for event log/replay, the Step 1/2 dependency approval point, and lists `Cargo.toml`/`Cargo.lock` as the two applicable watched files — read directly, present.
- **AC-2**: disclaimer sentence confirmed present verbatim (checked with blockquote markers stripped): "It is not an independent approval authority. If it conflicts with the self-dev workflow, `CLAUDE.md` and the approved change record govern."
- **AC-3/AC-8**: `grep -in "stage 9\|stage 10\|readiness checklist" CLAUDE.md status/stack-manifest.md status/stack-reconciliation/CHG-20260702-001-stack-reconciliation-report.md` → **zero matches**. The only "replay" mentions are negations ("no replay" in `CLAUDE.md`'s pre-existing Mode Declaration; "not applicable ... no runtime events or replay" in the new manifest) — not an active reference.
- **AC-5**: `CLAUDE.md`'s new section states all three clauses: Step 1 declare/re-triage, Step 2 verification criterion, Step 4 verify + run `check-drift`.
- **AC-6**: `git log --oneline -- tools/reviewer/Cargo.toml tools/reviewer/Cargo.lock` → **exactly one commit**, `a66bda9` (UPG-0032 / `CHG-20260702-001`). The backfill instance names all 9 dependencies and is explicitly dated/labeled retroactive. `status/stack-reconciliation/` contains exactly this one file — nothing for `serde_yaml`/UPG-0023.
- **AC-7**: **deferred to Step 4** — not yet run or recorded as of this Step 3 pass. Step 4
  has not been written yet (see below); this AC will be verified there.
- **AC-9**: `git diff --stat -- dba-system.md` → empty; untouched.
- **AC-10**: no hook/CI file added or modified (`git status --short` shows no such paths). `CLAUDE.md`'s new section contains, verbatim: "Human approval at each gate remains the authority; the manifest and reconciliation reports are evidence, not authority."

### Scope check

No edits to `dba-system.md`, `templates/stack-manifest.md`, `templates/stack-reconciliation-report.md`, or `tools/reviewer/src/cmd/check_drift.rs`. No hook, CI configuration, or autonomous enforcement introduced.

---

## Step 4 — Reconcile

### Acceptance criteria verification

| AC | Verified by | Result |
|---|---|---|
| AC-1 Manifest records real facts | Read `status/stack-manifest.md` | PASS |
| AC-2 Non-authority disclaimer, verbatim (wrap-collapsed) | `sed 's/^> //' status/stack-manifest.md \| tr '\n' ' ' \| grep -o "..."` | PASS |
| AC-3 No downstream stage language in trigger rule | `grep -in "stage 9\|stage 10\|readiness checklist"` → zero matches | PASS |
| AC-4 Watched files explicit, scoped to this repo | Read `status/stack-manifest.md` | PASS |
| AC-5 Trigger rule's three clauses present | Read `CLAUDE.md`'s new section | PASS |
| AC-6 UPG-0032 backfill only, mechanically checked | `git log --oneline -- tools/reviewer/Cargo.toml tools/reviewer/Cargo.lock` → exactly one commit (`a66bda9`) | PASS |
| AC-7 `check-drift` actually run | `codeos-reviewer check-drift --base 142bd02` → exit 0, no output (see below) | PASS |
| AC-8 No doctrine-specific language leaks | `grep -in "replay"` → both hits are negations | PASS |
| AC-9 `dba-system.md` untouched | `git diff --stat -- dba-system.md` → empty | PASS |
| AC-10 No autonomous enforcement; human gates preserved | No hook/CI files in diff; human-authority sentence verbatim (wrap-collapsed) in `CLAUDE.md` | PASS |

**AC-7 — actual `check-drift` run for this change:**
```
$ ./tools/reviewer/target/release/codeos-reviewer check-drift --base 142bd02
$ echo $?
0
```
No output, exit 0 — correct, since this change touches neither `Cargo.toml` nor `Cargo.lock`
(it only adds `CLAUDE.md`, `status/stack-manifest.md`, and the reconciliation-report backfill
instance). This demonstrates the tool runs cleanly against Codeos's own repo, not just that it
exists.

**Sanity check (no Rust code touched by this change):** `cargo build` and
`cargo test --test smoke` both re-run clean: 77 passed, 0 failed.

### Cross-reference sweep

- `grep -rln "stack-manifest\|stack-reconciliation"` across the repo (excluding
  `tools/reviewer/target/` and `reviews/codex/` assessment dumps) returns only expected
  references: the historical UPG-0017/UPG-0020 change records and backlog briefs, the
  downstream templates (untouched), the new files themselves, and this change's own record.
  Nothing stale.
- **Gap found and fixed**: `CLAUDE.md`'s "Self-Development File Layout" diagram did not list
  `status/stack-manifest.md` or `status/stack-reconciliation/` — added both under the
  `status/` block, consistent with how `self-development.md`/`roadmap.md` are already listed
  there.
- `dba-system.md`, `templates/stack-manifest.md`, `templates/stack-reconciliation-report.md`,
  `tools/reviewer/src/cmd/check_drift.rs` — confirmed untouched (`git diff --stat` empty for
  each).

### Reviewer scope triage (Step 4 findings)

R1 (DO NOT ADVANCE): AC-2/AC-10 specified verification via a literal single-line grep, but
the actual (intentionally wrapped, house-style) prose broke that literal match — IN-SCOPE
BLOCKER, fixed by rewording both ACs' verification method to be wrap-tolerant.
R2 (DO NOT ADVANCE): the Step 3 Verification section claimed AC-7 was "recorded in Step 4
below" while Step 4 was still a placeholder — an AJ-013-pattern forward-claim. IN-SCOPE
BLOCKER, fixed by rewording to "deferred to Step 4."
R3 (NO OBJECTION): both fixes confirmed; AC-7 explicitly and correctly deferred.

### Outcome

All 10 ACs verified against the final artifacts (table above), including a real
`check-drift` invocation against this repo (AC-7) rather than just a description of the
mechanism. No in-scope blockers open. No scope drift. Step 4 R1 NO OBJECTION; human
APPROVE_STAGE recorded. Change record, `status/self-development.md`, `status/roadmap.md`,
`backlog/features.md`, and `backlog/UPG-0036-stack-manifest-dogfooding.md` updated to
COMPLETE in this same pass, following that approval.
