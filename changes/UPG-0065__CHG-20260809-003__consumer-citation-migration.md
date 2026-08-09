# Self-Development Change: UPG-0065__CHG-20260809-003 — consumer-citation-migration

<!--
PURPOSE: Sixth change under UPG-0065 (Modular DBA Configuration Architecture). First half of
Phase A's sixth sub-step ("activate the modular architecture with DBA-1 as the active
configuration"), sequenced before the atomic dba-system.md swap. Updates every downstream consumer
file (`prompts/`, `scripts/`, `templates/`, `patterns/`) that cites `dba-system.md`'s internal
section structure by name (34 STRUCTURAL-POINTER rows, `CHG-20260808-002`'s compatibility report)
so every citation points at the correct `dba/*/v1.md` location BEFORE `dba-system.md` itself
changes. The 5 WHOLE-FILE-LOAD rows are explicitly NOT edited — see Scope boundary for why; an
earlier draft of this change incorrectly included them. `dba-system.md` stays the untouched
monolith throughout this change. See Change Intent for the two specific, checkable properties this
split relies on (plain-revert reversibility; the 12 CHG-20260809-001-waived dba/*/v1.md defects
don't compromise any of the 34 retargeted citations) — stated precisely there, not asserted here.
The atomic `dba-system.md` swap to the thin manifest is a separate, later change
(`dba1-activation`, not yet filed), safe only once this one is COMPLETE.
Workflow: prompts/codeos-self-dev.md (4-step loop).
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0065
primary_feature_id: UPG-0065
change_id: CHG-20260809-003
slug: consumer-citation-migration
state: COMPLETE          # DRAFT | IN_REVIEW | IN_PROGRESS | BLOCKED | COMPLETE | ABANDONED | SUPERSEDED
current_step: 4-Reconcile   # 1-Intent | 2-Acceptance | 3-Implement | 4-Reconcile
implements:
  - UPG-0065
related_features: []
review_series: S4         # S1/S2/S3/S4 all human APPROVED. S4: R1 fixed, R2 NO OBJECTION, 0 findings
review_profile: PROFILE-4   # downstream-doctrine (Step 0a)
review_state: ACCEPTED   # DRAFT | IN_REVIEW | REVIEWED | ACCEPTED  (operational; NOT a round; resets per step)
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

<!-- SELF-REFERENCE BOUNDARY: this artifact is itself reviewed, so it must NOT embed the current
review round. Reference the stable review SERIES (review_series) + review_state; exact rounds live
only in reviews/review-log.md and reviews/codex/*. -->

---

## Change Intent

**Why (problem in the toolkit):**

`CHG-20260808-002`'s compatibility report catalogued every `dba-system.md` reference in
`prompts/`, `scripts/`, `templates/`, `patterns/`: 34 `STRUCTURAL-POINTER` rows cite one of its
section headings by name (e.g. "see the 'Default Advisory Review' section", "see 'Multi-Feature
Architecture Synthesis Gate'"), and 5 `WHOLE-FILE-LOAD` rows instruct reading the file in full.
Human decision (2026-08-09): activation uses the minimal-manifest design — `dba-system.md` becomes
a short pointer to the active `DBA-N`, not a file that keeps its current section structure. Once
that swap happens, every one of those 34 citations names a heading that no longer exists in
`dba-system.md` (the content it named is now in one of six `dba/*/v1.md` files) — this is the
actual problem this change solves. The 5 `WHOLE-FILE-LOAD` rows are not a parallel problem: "read
`dba-system.md`" stays a valid instruction regardless of what the file contains, so they need no
consumer-side edit at all (see Scope boundary).

Fixing all of this atomically with the `dba-system.md` swap itself would land a huge multi-file
diff in the same instant as the doctrine-authority switch, with no way to review the consumer-side
half independently first. This change does the consumer-side half alone, first, while
`dba-system.md` remains completely unchanged. Two specific, checkable properties this split relies
on, stated precisely rather than asserted as a general "safe/reversible" claim:

- **Reversible, concretely:** every edit in this change is a plain prose citation update — no
  schema change, no state, no code, no file deletion. Reverting this change's own commit restores
  every edited file to its pre-change text exactly, the same as reverting any other prose-only
  commit; there is no separate rollback mechanism to design or verify.
- **The 34 retargeted citations do not point at content compromised by the 12 defects
  `CHG-20260809-001` waived.** Checked directly: of the 8 sub-part `STRUCTURAL-POINTER` rows that
  name a specific `rule_id` (rows 19, 20, 30, 36, 37, 40, 41, 45 — targeting `ARCH-GATE-13`,
  `IMPL-PROFILE-7`, `REVIEW-7`, `ARCH-GATE-10`), none names one of the 12 waived rule_ids
  (`FAILURE-BOUNDARY-5`, `HUMAN-NAV-1`, `REVIEW-LOG-1b`, `REVIEW-LOG-1c`, `ARCH-GATE-3b`,
  `IMPL-PROFILE-4a/4b/4c`, `CPE-3a`, `IMPL-PROFILE-8`, `FILE-LAYOUT-5b/5c`). The remaining 26
  section-level rows point at whole component files, 5 of which (all but `dba/tools/reviewer/v1.md`)
  do contain at least one of the 12 waived rows somewhere — but every one of those 12 is a
  citation-precision or wording defect internal to that file's own Source Traceability table (an
  imprecise line-anchor back to `dba-system.md`, or an added editorial parenthetical), never a
  defect in the rule's own substantive content — that is exactly why `CHG-20260809-001`'s AC3
  recorded them as immaterial rather than fixed. A consumer citation pointing at any of these files
  finds correct, working content regardless.

**What changes:**

- `changes/UPG-0065__CHG-20260809-003__consumer-citation-migration.md` (this file) — the change
  record.
- All 34 `STRUCTURAL-POINTER` rows, across 16 files, get their citation updated to name the
  correct `dba/*/v1.md` file (and, for Finding A's genuine split only, every file the section's
  content actually landed in, not just one — Findings B and E are near-splits and keep citing
  their single dominant file, per the Citation format note below Acceptance Criteria):
  `patterns/controlled-plain-english.md`
  (rows 1, 2), `patterns/rust-project-structure.md` (rows 5, 6, 7, 8), `prompts/00a-solution-discovery.md`
  (row 9), `prompts/00c-onboarding.md` (row 10), `prompts/00-session-start.md` (rows 13, 14),
  `prompts/03b-architecture-synthesis.md` (rows 15, 17, 18), `prompts/04-implement.md` (rows 19,
  20, 21, 22), `prompts/05-tests.md` (row 23), `prompts/10-arch-refine.md` (row 24),
  `prompts/pipeline-reviewer.md` (row 29), `prompts/verify-only.md` (row 30), `scripts/dba-init.sh`
  (rows 32, 33), `templates/architecture-baseline.md` (rows 35, 36, 37), `templates/cohort-logical-design.md`
  (rows 39, 40, 41), `templates/feature-registry.yaml` (rows 42, 43, 44, 45), `templates/review-package.md`
  (row 49). (Row numbers per `changes/UPG-0065__CHG-20260808-002__compatibility-report.md`'s Part 1
  table.)
- `backlog/UPG-0065-modular-dba-configuration-architecture.md`, `status/self-development.md`,
  `status/roadmap.md` — Feature Thread / dashboard updated as this change progresses.

**Scope boundary — what stays the same:**

- **The 5 `WHOLE-FILE-LOAD` rows are not edited.** All five just instruct reading `dba-system.md`
  in full (`prompts/00-session-start.md` rows 11, 12; `scripts/dba-init.sh` row 31;
  `templates/project-CLAUDE.md` rows 47, 48). That instruction stays literally true whether
  `dba-system.md` is today's monolith or the future thin manifest — "read the file" doesn't break;
  only *what's in the file* changes. Finding C's manifest-cascade requirement is solved by the
  *manifest's own text* at swap-time (e.g. "all components named here are jointly authoritative and
  must be loaded when applicable"), not by editing these 5 consumer files — an earlier draft of this
  change's own scope incorrectly included these 5 rows for editing; corrected here before Step 2.
  `templates/project-CLAUDE.md` accordingly drops out of this change's file list entirely — it has
  no `STRUCTURAL-POINTER` rows, only these two `WHOLE-FILE-LOAD` ones.
- `dba-system.md` is **not edited** — same content, same role, still the monolithic doctrine every
  project currently loads. The atomic swap to the thin manifest is a separate, later, not-yet-filed
  change (Invariant 1(d) activation itself), which this change is a safe precondition for, not a
  part of.
- No `dba/*/v1.md` file's content is edited. Citations point *at* these files; nothing about their
  own content changes.
- The 10 `GENERIC-MENTION` rows are **not touched** — they name `dba-system.md` as a concept or
  path only, with no structural dependency, so nothing about them breaks when the file's content
  changes.
- **Finding D's two pre-existing citation-drift defects stay unfixed here**, even though this
  change edits the exact files they're in (`prompts/00-session-start.md:14`,
  `templates/architecture-baseline.md:33` and `:43`). They are drift *within* `dba-system.md`'s
  current structure (a stale rule count; a dropped clause), unrelated to the `v1` decomposition or
  this migration — bundling an unrelated correctness fix into a structural migration is exactly the
  scope-creep this project's own discipline avoids. Tracked as separate backlog cleanup, alongside
  the 12 already-waived `dba/*/v1.md` hygiene defects from `CHG-20260809-001`.
- No `DBA-2`, `dba-system-lean.md` decomposition, or any Phase B work.
- `DBA-1` is not re-approved or re-activated by this change — it is already `status: approved`
  (`CHG-20260809-002`); this change doesn't touch `dba/configurations/DBA-1.yaml`.

**Class:** downstream-doctrine
**Scope axis:** downstream doctrine only
**Backlog item:** backlog/UPG-0065-modular-dba-configuration-architecture.md

---

## Acceptance Criteria

**Citation format (binding on Step 3).** Every retargeted `STRUCTURAL-POINTER` citation names the
`dba/*/v1.md` file(s) directly (e.g. "see `dba/policies/implementation-profile/v1.md`"), replacing
the old `dba-system.md` section-name reference. The compatibility report itself distinguishes
**Finding A** (a *genuine* split — "Default Advisory Review", rows 9, 18, 29) from **Findings B and
E** (*near*-splits — "Multi-Feature Architecture Synthesis Gate" and the Controlled Plain English
section), and says explicitly that the near-split rows "cite the section broadly and do not depend
on" the minority file. That distinction carries into this AC: only Finding A's 3 rows name
multiple `dba/*/v1.md` files (`dba/doctrine/v1.md` + `dba/policies/review/v1.md` +
`dba/tools/reviewer/v1.md`); every Finding B/E row keeps citing its single dominant file, per the
report's own analysis — not a contradiction of "target correctness," but the correct application
of it to a row the report already classified as not depending on the minority content.

| # | Criterion | How it will be verified |
|---|---|---|
| 1 | **Completeness and touch-scope, in one check.** All 34 `STRUCTURAL-POINTER` rows from `CHG-20260808-002`'s Part 1 table get their citation updated, and *only* those 34 lines change anywhere in the 16 consumer files — no `GENERIC-MENTION` row, no `WHOLE-FILE-LOAD` row (including the two in `prompts/00-session-start.md`/`scripts/dba-init.sh` that share a file with `STRUCTURAL-POINTER` rows), and no consumer file outside those 16 (in particular `templates/project-CLAUDE.md`, which has no `STRUCTURAL-POINTER` rows at all). | Fresh `grep -c "^| [0-9]* |.*STRUCTURAL-POINTER" changes/UPG-0065__CHG-20260808-002__compatibility-report.md` → 34 (a table-row-only pattern, excluding the legend's own definition line) — re-confirmed at Step 3. `git diff --name-only -- prompts/ scripts/ templates/ patterns/` lists exactly the 16 consumer files (scoped to those four directories, not a repo-wide diff — this change's own record and tracking-surface updates are expected to show up outside that scope and are not part of this check). For each of the 16, the changed line numbers match only that file's `STRUCTURAL-POINTER` row line numbers from the compatibility report — never a `WHOLE-FILE-LOAD` row's line. |
| 2 | **Target correctness.** Each updated citation names the `dba/*/v1.md` file(s) matching that row's own `detail` column in the compatibility report — verified against the report, not memory. Per the Citation format note above, Finding A's 3 rows name all three landing files; every other row (including Findings B and E) names its single dominant file, matching the report's own "does not depend on" analysis for near-splits. | For every one of the 34 rows, read the compatibility report's `detail` column and confirm the new citation text names exactly the file(s) that row's own classification (genuine split vs. near-split vs. clean map) calls for. |
| 3 | **No content change beyond the citation itself.** The surrounding prose is otherwise unchanged — a pointer-retargeting change, not a rewrite. | Diff each edited file; confirm only the `dba-system.md` section name is replaced, nothing else in the line or surrounding paragraph changes. |
| 4 | **`dba-system.md` and `dba/*/v1.md` untouched.** No edit, tracked or untracked, to either. | `git diff -- dba-system.md dba/doctrine/ dba/policies/ dba/tools/` empty; `git diff 6ba113a -- dba-system.md` empty (pinned to the commit that closed `CHG-20260809-002`); `git status --porcelain --untracked-files=all -- dba-system.md dba/doctrine/ dba/policies/ dba/tools/` empty (tracked-only `git diff` cannot see a new untracked file). |
| 5 | **Cross-reference consistency.** The change record, the brief's Feature Thread, and `status/self-development.md` agree on this change's current step and state. | Grep sweep for `UPG-0065` / `CHG-20260809-003` across all three files at Reconcile; no stale step/state claims (AJ-020/AJ-025 class). |

---

## Implementation Notes

All 34 `STRUCTURAL-POINTER` citations retargeted, one file at a time, using the compatibility
report's own `detail` column as the source of truth for each new citation. Every downstream-facing
citation uses the `.codeos/dba/...` form (matching this file's existing `.codeos/patterns/...` /
`.codeos/templates/...` conventions — confirmed the `.codeos` symlink points at the whole toolkit
root, so `.codeos/dba/...` resolves correctly, per `scripts/dba-init.sh`). Finding A's 3 rows
(`prompts/00a-solution-discovery.md:208`, `prompts/03b-architecture-synthesis.md:179`,
`prompts/pipeline-reviewer.md:8`) each now name all three landing files
(`dba/doctrine/v1.md`, `dba/policies/review/v1.md`, `dba/tools/reviewer/v1.md`). Every other row,
including Findings B and E, cites its single dominant file, per the report's own near-split
analysis. The 9 sub-part rows that named a specific bolded lead-in phrase (`ARCH-GATE-13`,
`ARCH-GATE-10`, `IMPL-PROFILE-7`, `REVIEW-7`) kept that exact phrase, only the file reference
changed. Finding D's two pre-existing wording-drift defects (`templates/architecture-baseline.md`
rows 36, 37) were retargeted like every other row but their existing dropped-clause wording was
left exactly as-is, per this change's own scope boundary.

**AC1 (completeness + touch-scope):** `grep -c "^| [0-9]* |.*STRUCTURAL-POINTER"
changes/UPG-0065__CHG-20260808-002__compatibility-report.md` → 34 (table-row-only pattern, matches).
`git diff --name-only -- prompts/ scripts/ templates/ patterns/` → exactly the 16 named files.
`git diff --stat` for those 16 shows small, localized diffs (2-16 lines changed per file) —
consistent with single-citation retargeting, not a rewrite. **PASS.**

**AC2 (target correctness):** verified per-row against the compatibility report's `detail` column
during editing, re-spot-checked after: all Finding A rows carry 3 files, all other rows carry
their single dominant file. **PASS.**

**AC3 (no content change beyond the citation):** every edit is a citation-text substitution in
place; no surrounding sentence was rewritten, reordered, or expanded (two of `scripts/dba-init.sh`'s
edits collapsed a two-line `echo` wrap into one line, since the sentence became short enough — a
formatting consequence of the shorter citation, not a content change). **PASS.**

**AC4 (`dba-system.md`/`dba/*/v1.md` untouched):** `git diff -- dba-system.md dba/doctrine/
dba/policies/ dba/tools/` empty; `git diff 6ba113a -- dba-system.md` empty; `git status --porcelain
--untracked-files=all -- dba-system.md dba/doctrine/ dba/policies/ dba/tools/` empty. **PASS.**

**Residual `dba-system.md` mentions confirmed correctly untouched:** `grep -rn "dba-system.md"
prompts/ scripts/ templates/ patterns/` still finds it in exactly the expected places — the 5
`WHOLE-FILE-LOAD` rows (`scripts/dba-init.sh:194`, `prompts/00-session-start.md:7,14`,
`templates/project-CLAUDE.md:9,11`) and the 10 `GENERIC-MENTION` rows — none of which this change
scopes to edit.

---

## Reconciliation

Each AC re-verified fresh at Reconcile, not re-trusting Step 3's own claims.

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Completeness + touch-scope | **PASS** | Fresh at Reconcile: `grep -c "^| [0-9]* |.*STRUCTURAL-POINTER" changes/UPG-0065__CHG-20260808-002__compatibility-report.md` → 34. `git diff --name-only -- prompts/ scripts/ templates/ patterns/` → 16 files. `git diff -U0` per file, read in full: every changed hunk's old-file line number matches that file's `STRUCTURAL-POINTER` row line number(s) in the compatibility report exactly (e.g. `patterns/controlled-plain-english.md` hunks at 10/16; `prompts/00-session-start.md` at 62/72; `templates/feature-registry.yaml` at 42/52/91/109) — no hunk at a `WHOLE-FILE-LOAD` or `GENERIC-MENTION` line. |
| 2 | Target correctness | **PASS** | Fresh at Reconcile: `git diff -U0 -- prompts/ scripts/ templates/ patterns/ \| grep "^+" \| grep -v "^+++"` — read every one of the 34 new citation lines directly, not re-cited from Step 3. All three Finding A rows carry all three files (`dba/doctrine/v1.md`, `dba/policies/review/v1.md`, `dba/tools/reviewer/v1.md`); every other row names its single dominant file, matching the report's `detail` column. |
| 3 | No content change beyond citation | **PASS** | Fresh at Reconcile: read the full `+`-line diff (not `--stat`) for all 16 files — every added line is the same sentence as the corresponding removed line with only the `dba-system.md` reference replaced; no sentence restructured, no adjacent line touched beyond the two `echo` lines in `scripts/dba-init.sh` that collapsed from two lines to one (noted in Implementation Notes). |
| 4 | `dba-system.md`/`dba/*/v1.md` untouched | **PASS** | `git diff -- dba-system.md dba/doctrine/ dba/policies/ dba/tools/` empty; `git diff 6ba113a -- dba-system.md` empty; `git status --untracked-files=all` for the same paths empty; `git diff -- templates/project-CLAUDE.md` empty (confirms the scope correction held). |
| 5 | Cross-reference consistency | **PASS** | Grep sweep for `UPG-0065`/`CHG-20260809-003` across the change record, brief, and dashboard at Reconcile — dashboard's Loop-step column and narrative both now say `4-Reconcile`; brief and change record agree. |

No findings scope-triage needed beyond what's already logged in Steps 1-3 above — no
`OUT-OF-SCOPE BACKLOG`, `REJECTED`, or unresolved findings this step.

This change does not perform activation (Invariant 1(d)) — `dba-system.md` remains the sole file
downstream projects load, unchanged. The atomic swap to the thin manifest is a separate, later,
not-yet-filed change, now safe to scope since every consumer citation already points at the
correct `dba/*/v1.md` location.
