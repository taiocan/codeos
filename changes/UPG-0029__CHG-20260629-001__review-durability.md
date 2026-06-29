# Self-Development Change: UPG-0029__CHG-20260629-001 — review-durability

<!--
PURPOSE: Per-change source of truth for UPG-0029 — Review artifact durability policy.
Workflow: prompts/codeos-self-dev.md (4-step loop)
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0029
primary_feature_id: UPG-0029
change_id: CHG-20260629-001
slug: review-durability
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0029
related_features:
  - UPG-0001
  - UPG-0003
review_series: RVS__UPG-0029__CHG-20260629-001__S4
review_state: ACCEPTED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: UPG-0001
```

<!-- SELF-REFERENCE BOUNDARY: carry review_series + review_state only; exact rounds live in
reviews/review-log.md. See prompts/codeos-self-dev.md → "Self-Reference Boundary". -->

---

## Change Intent

**Why (problem in the toolkit):**

After UPG-0001, `reviews/review-log.md` contains 30+ entries of the form:

```
Full assessment: reviews/codex/<timestamp>-<slug>.md (sha256:<hash>)
Reviewed packet: reviews/codex/packets/<timestamp>-<slug>.packet.txt (sha256:<hash>)
```

Almost all of those files are **untracked** — not committed to the repo. One assessment
(`2026-06-27T044240Z-reviewer-pipeline-stage-0-5e01520.md`) is already committed; the
remaining 27 assessment files and all packet files in `reviews/codex/packets/` are
untracked (27 of 28 `Full assessment:` entries reference uncommitted files). The `.gitignore` carves out `reviews/codex/_scratch/` for scratch/test
assessments, but the referenced root-level assessment files in `reviews/codex/` are not
gitignored — they are simply not being committed. A path+sha pointer in the log to an
uncommitted file is not reproducible from the committed tree: a fresh checkout cannot
verify the assessment, and another reviewer cannot read it. Without a policy, the log
claims traceability that a checkout cannot always satisfy.

There is no documented policy — anywhere in `docs/`, `CLAUDE.md`, or the review log
itself — stating which review artifacts should be committed vs. kept local-only vs.
treated as scratch. UPG-0001 explicitly deferred this to UPG-0029 as a concrete
OUT-OF-SCOPE BACKLOG finding.

**What changes:**

1. **`changes/UPG-0029__CHG-20260629-001__review-durability.md`** (this file) —
   the change record for this self-dev execution (created in Step 1, extended in Steps 2–4).

2. **`docs/reviewer-pipeline.md`** — add a "Review artifact durability" section
   documenting the policy: which artifacts are committed (durable), which are scratch
   (local-only, never committed), and the rule for `review-log.md` references.

3. **`reviews/review-log.md`** — add a header note establishing the policy effective point
   (the commit that lands UPG-0029) as the durability boundary. All log entries created before
   that commit are pre-policy. With one exception, pre-policy path+sha references to review
   artifacts point to local-only files (assessment files existed on disk but were not committed).
   The exception — `2026-06-27T044240Z-reviewer-pipeline-stage-0-5e01520.md` — was already
   committed and is durable. This is documented retroactively, not an error. Entries created
   after the commit that lands UPG-0029 must either reference committed review artifacts or
   explicitly mark the reference `[local-only]` / non-durable.

4. **`backlog/UPG-0029-review-naming-and-thread-tooling.md`** — Feature Thread:
   populate the Changes table row for CHG-20260629-001.

5. **`status/self-development.md`** — activate the UPG-0029 / CHG-20260629-001 row
   (State: IN_PROGRESS) and advance the Loop step column at each step.

**Scope boundary — what stays the same:**

- `scripts/codeos-review.sh` — not touched; script I/O behavior, filenames, and
  output format unchanged. Emitting `REV__`-shaped filenames is deferred (UPG-0029
  issues #2–#5).
- `dba-system.md` — not touched.
- `CLAUDE.md` — not touched.
- The untracked `reviews/codex/*` assessment files — not committed, not deleted.
  The policy is doc-only; committing 30+ historical assessments retroactively is
  out of scope. The retroactive log header note (added in Step 3) will document
  their local-only/pre-policy status.
- `.gitignore` — not changed. The existing `reviews/codex/_scratch/` rule is correct;
  the root `reviews/codex/` is intentionally not gitignored (durable files committed
  there going forward will be tracked automatically).
- Naming convention enforcement (`REV__`, `RVS__` filename emission) — deferred.
- No new scripts.

**Class:** `documentation` (normative) — defines a policy in `docs/reviewer-pipeline.md`
and aligns the review log with it.

**Scope axis:** `self-dev only`

**Backlog item:** `backlog/UPG-0029-review-naming-and-thread-tooling.md`

---

## Acceptance Criteria

| # | Criterion | How it will be verified |
|---|---|---|
| A1 | **Policy section present in docs.** `docs/reviewer-pipeline.md` contains a "Review artifact durability" section that defines all three categories (committed/durable, scratch/local-only, and the rule for `review-log.md` path+sha references), and states the going-forward rule: a log entry that references a full assessment by path+sha must either reference a committed file or be explicitly marked local-only/non-durable. | Read `docs/reviewer-pipeline.md`; confirm the section exists and all three categories + the rule are stated. |
| A2 | **Policy is doc-only — no script behavior changed.** `scripts/codeos-review.sh` is byte-identical to its state at the start of this change (base commit `9f2a87d8bb54834b07836e2abd8eb33626549b30`). | `git diff 9f2a87d8bb54834b07836e2abd8eb33626549b30 -- scripts/codeos-review.sh` returns empty. |
| A3 | **No downstream doctrine changed.** `dba-system.md` is untouched since base commit `9f2a87d8bb54834b07836e2abd8eb33626549b30`. | `git diff 9f2a87d8bb54834b07836e2abd8eb33626549b30 -- dba-system.md` returns empty. |
| B1 | **Retroactive log header note present.** `reviews/review-log.md` has a header note (above all review entries) that: (a) identifies pre-policy entries as referencing local-only files; (b) names the one already-committed exception; (c) states the going-forward rule. | Read the header section of `reviews/review-log.md`; confirm all three elements are present. |
| B2 | **Append-only invariant preserved.** The log header note is an addition only — no existing review entry lines are modified, deleted, or reordered. | Run `git diff 9f2a87d8bb54834b07836e2abd8eb33626549b30 -- reviews/review-log.md` and confirm: (1) the output contains no lines beginning with `-` other than the `---` diff header; (2) all existing entry content is intact. |
| C1 | **Cross-reference integrity.** Any file paths or section links introduced in the new `docs/reviewer-pipeline.md` section resolve to existing files or sections in the committed tree. | Manually inspect each link/path in the new section; `ls`/`grep` to confirm each target exists. |
| C2 | **Change record and dashboard consistent.** The change record trace header matches the dashboard row (feature_id, change_id, current_step, state). | Read both files and compare. |
| D1 | **Feature Thread populated.** `backlog/UPG-0029-review-naming-and-thread-tooling.md` Feature Thread / Changes table has a row for CHG-20260629-001. | `grep "CHG-20260629-001" backlog/UPG-0029-review-naming-and-thread-tooling.md` returns a hit in the Feature Thread section. |

---

## Implementation Notes

**Edits made (Step 3):**

1. **`docs/reviewer-pipeline.md`** — added new section `## 4a. Review artifact durability
   policy` between sections 4 and 5 (no existing section renumbered). Defines committed/durable,
   scratch/local-only, and the rule for `review-log.md` path+sha references. Refers to the
   `reviews/review-log.md` header for the retroactive classification of pre-policy entries.

2. **`reviews/review-log.md`** — added a second blockquote to the header (after the existing
   Feature Thread identification note, before the first review entry). With one exception,
   identifies pre-policy path+sha references as local-only; names the one committed exception
   (`2026-06-27T044240Z-reviewer-pipeline-stage-0-5e01520.md`); and states the going-forward rule.
   Append-only: no existing entry lines were modified.

3. **`backlog/UPG-0029-review-naming-and-thread-tooling.md`** — Feature Thread Changes table
   populated in Step 1; problem statement corrected in Step 1 (accurate counts). No further
   body edits needed in Step 3.

4. **`status/self-development.md`** — dashboard row advanced to 3-Implement (Step 3); advanced
   to 4-Reconcile (Step 4).

5. **`changes/UPG-0029__CHG-20260629-001__review-durability.md`** (this file) — trace header
   advanced at each step; Implementation Notes filled (Step 3); Reconciliation section added (Step 4).

**Cross-references checked:** the new `docs/reviewer-pipeline.md §4a` refers to
`reviews/review-log.md` (exists); the `reviews/review-log.md` header note refers back to
`docs/reviewer-pipeline.md §4a` (exists). `scripts/codeos-review.sh` and `dba-system.md`
not touched.

---

## Reconciliation

**Verification of acceptance criteria (Step 4):**

| # | Criterion | Evidence | Result |
|---|---|---|---|
| A1 | Policy section present in docs | `grep -n "Review artifact durability" docs/reviewer-pipeline.md` → line 124: `## 4a. Review artifact durability policy`. All three categories confirmed: `grep -n "Committed / durable\|Scratch / local-only\|The rule for" docs/reviewer-pipeline.md` → lines 130, 136, 140. Going-forward rule stated at line 140. | PASS |
| A2 | No script behavior changed | `git diff 9f2a87d8bb54834b07836e2abd8eb33626549b30 -- scripts/codeos-review.sh` → empty (0 bytes) | PASS |
| A3 | No downstream doctrine changed | `git diff 9f2a87d8bb54834b07836e2abd8eb33626549b30 -- dba-system.md` → empty (0 bytes) | PASS |
| B1 | Retroactive log header note present | Header contains (a) `Entries created before that commit are **pre-policy**` + pre-policy reference classification; (b) `**Exception:** reviews/codex/2026-06-27T044240Z-reviewer-pipeline-stage-0-5e01520.md was committed before this policy and is durable`; (c) `**Going forward:** entries created after the commit that lands UPG-0029 must either reference committed review artifacts or explicitly mark the reference [local-only] / non-durable`. All three elements confirmed at lines 15–25 of `reviews/review-log.md`. | PASS |
| B2 | Append-only invariant preserved | `git diff 9f2a87d8bb54834b07836e2abd8eb33626549b30 -- reviews/review-log.md \| grep "^-" \| grep -v "^---" \| wc -l` → 0. No existing entry lines removed or modified. | PASS |
| C1 | Cross-reference integrity | All paths in §4a and surrounding §4: `reviews/review-log.md` (exists), `docs/reviewer-artifact-schemas.md` (exists), `reviews/codex/_scratch/` (exists), `backlog/UPG-0015-reviewer-decision-integrity.md` (exists). All confirmed by `ls`. | PASS |
| C2 | Change record and dashboard consistent | Change record: `feature_id: UPG-0029`, `change_id: CHG-20260629-001`, `current_step: 4-Reconcile`, `state: IN_PROGRESS`. Dashboard row: `UPG-0029 \| CHG-20260629-001 \| documentation \| self-dev only \| 4-Reconcile \| PENDING … \| IN_PROGRESS`. Fields match. | PASS |
| D1 | Feature Thread populated | `grep "CHG-20260629-001" backlog/UPG-0029-review-naming-and-thread-tooling.md` → line 103 in Feature Thread Changes table. | PASS |

**Scope sweep:** `git diff 9f2a87d8... -- dba-system.md scripts/codeos-review.sh` both empty. CLAUDE.md untouched. `.gitignore` untouched. No new scripts. No retroactive commits of historical assessments. No orphaned links found — §4a cross-references all resolve. Stage-table and prompt-file drift not applicable (no stage tables touched). Naming convention enforcement (REV__/RVS__ emission) confirmed deferred.

**All criteria: PASS. Ready for Step 4 Codex review.**
