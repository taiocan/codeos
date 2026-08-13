# Self-Development Change: UPG-0065__CHG-20260808-002 — downstream-consumer-compatibility-sweep

<!--
PURPOSE: Third change under UPG-0065 (Modular DBA Configuration Architecture). Phase A's third
sub-step (see backlog/UPG-0065's "Migration approach"): "compatibility sweep against
prompts/scripts/templates assuming old semantics" — catalog every place in the toolkit's
consumer-facing surfaces (prompts/, scripts/, templates/, patterns/) that references
dba-system.md, and classify whether it survives dba-system.md becoming a thin manifest over
the six dba/ v1 components (CHG-20260808-001), or needs updating at activation time.
Workflow: prompts/codeos-self-dev.md (4-step loop).
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0065
primary_feature_id: UPG-0065
change_id: CHG-20260808-002
slug: downstream-consumer-compatibility-sweep
state: COMPLETE          # DRAFT | IN_REVIEW | IN_PROGRESS | BLOCKED | COMPLETE | ABANDONED | SUPERSEDED
current_step: 4-Reconcile  # 1-Intent | 2-Acceptance | 3-Implement | 4-Reconcile
implements:
  - UPG-0065
related_features: []
review_series: RVS__UPG-0065__CHG-20260808-002__S4   # S1-S4 all human APPROVED — CHG COMPLETE
review_profile: PROFILE-4   # downstream-doctrine (Step 0a)
review_state: ACCEPTED  # DRAFT | IN_REVIEW | REVIEWED | ACCEPTED  (operational; NOT a round; resets per step)
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

`backlog/UPG-0065-modular-dba-configuration-architecture.md`'s Migration Approach names Phase A's
third sub-step, after the normative delta inventory (`CHG-20260807-001`, COMPLETE) and the `v1`
component decomposition (`CHG-20260808-001`, COMPLETE): "compatibility sweep against
prompts/scripts/templates assuming old semantics." A preliminary grep for `dba-system.md` across
`prompts/`, `scripts/`, `templates/`, `patterns/` already surfaces the concrete risk this sub-step
exists to catch: most references are of the form `See .codeos/dba-system.md → "Section Name"` —
a pointer into a *specific section heading* of the current monolith. Once `dba-system.md`
eventually becomes a thin manifest (per the brief's sketch — a later, unstarted sub-step), that
section's content will live in a `dba/` component file instead, and the pointer breaks unless
updated. Finding every such pointer now, while nothing has moved yet, is cheaper than discovering
them one at a time after activation.

**What changes:**

- `changes/UPG-0065__CHG-20260808-002__downstream-consumer-compatibility-sweep.md` (this file) —
  the change record.
- `changes/UPG-0065__CHG-20260808-002__compatibility-report.md` (new, created at Step 3) — the
  per-reference compatibility catalog, kept as a separate evidence file for the same reason the
  delta table was: it is the durable analytical deliverable, independently reviewable.
- `backlog/UPG-0065-modular-dba-configuration-architecture.md`, `status/self-development.md`,
  `status/roadmap.md` — Feature Thread / dashboard / wave-plan change-id column updated as this
  change progresses.

**Preliminary scope (confirmed by grep before writing this Step; the Step 3 report is the
authoritative count):** 21 files reference `dba-system.md`, accounted for as: 18 under `prompts/`,
`scripts/`, `templates/`, `patterns/` that are genuinely downstream-consumer-facing; `prompts/
codeos-self-dev.md` and `templates/codeos-change.md`, which reference it only in a self-development
governance context (scope-boundary checks, "this is not that doctrine" framing); and
`scripts/tests/codeos-implement-tests.sh`, whose one match is a path-traversal-attack test string
(`"doctrine/dba-system.md:outside-stage-area"`), not a real reference to the file at all. All
three of the latter category (18 + 2 + 1 = 21) are expected to be **out of this sweep's scope** —
confirmed or corrected at Step 3, not assumed here.

**Scope boundary — what stays the same:**

- No file under `prompts/`, `scripts/`, `templates/`, or `patterns/` is edited in this change.
  This sweep produces a compatibility *report*, not fixes — updating a consumer file to point at
  its new `dba/` location is meaningful only once activation exists to point at, which this
  change does not create (per the brief's Phase A sequencing: sweep, *then* prove equivalence,
  *then* human-approve `DBA-1`, *then* activate — in that order, not collapsed).
- `dba-system.md` and `dba-system-lean.md` are not edited.
- No `configurations/*.yaml` is created; no `DBA-1`/`DBA-2` is approved or activated.
- The manifest-path and `patterns/`-vs-`policies/` questions already resolved in
  `CHG-20260808-001` are not reopened here.
- No content is moved out of the six existing `dba/*/v1.md` files, and no new component version
  is drafted.

**Class:** downstream-doctrine
**Scope axis:** downstream doctrine only
**Backlog item:** backlog/UPG-0065-modular-dba-configuration-architecture.md

---

## Acceptance Criteria

**Report schema (binding on Step 3).** `changes/UPG-0065__CHG-20260808-002__compatibility-report.md`
carries one row per `dba-system.md` reference found: `file | line | quoted text | kind | detail`.
`kind` is exactly one of:
- **STRUCTURAL-POINTER** — cites a specific piece of `dba-system.md` structure by name, at either
  of two granularities: (a) a **section-level** citation naming a `##`-level heading verbatim
  (e.g. `See .codeos/dba-system.md → "Implementation Profile"`), or (b) a **sub-part** citation
  naming a bolded lead-in phrase from *within* a section — the same phrase the accepted delta
  table uses as that row's own identifying label (e.g. `"Verifying a baseline_version...
  reference"`, which is `ARCH-GATE-13`'s bolded lead-in, not a `##` heading). Every
  `STRUCTURAL-POINTER` row states which granularity it is. `detail` names which `dba/` component
  file(s) that section's or sub-part's content now lives in (per `CHG-20260808-001`'s Source
  Traceability tables), and whether the mapping is clean (all cited content in one `dba/` file) or
  split (spans more than one).
- **WHOLE-FILE-LOAD** — instructs reading/loading `dba-system.md` as a whole (e.g. "Read
  `.codeos/dba-system.md` fully"), with no section-specific dependency. `detail` states this is
  conditionally compatible: correct once and only once the eventual thin-manifest form of
  `dba-system.md` cascades loading of every named `dba/` component — a mechanism the brief
  sketches but this change does not design or build.
- **GENERIC-MENTION** — refers to `dba-system.md` as a concept or path, with no dependency on its
  internal content or structure (e.g. self-dev scope-boundary checks, path literals in test
  fixtures). `detail` states why the reference has no compatibility dependency.

| # | Criterion | How it will be verified |
|---|---|---|
| 1 | **Completeness, per reference, not just per file.** Every individual `dba-system.md` occurrence in the 21 files identified in Step 1 gets its own report row (a file with 3 real occurrences has 3 rows, not 1) — no occurrence is silently skipped or folded into another, including in the 3 files already flagged as likely out-of-scope (`prompts/codeos-self-dev.md`, `templates/codeos-change.md`, `scripts/tests/codeos-implement-tests.sh`) — Step 1's classification of those 3 is confirmed or corrected in the report, not assumed. | For each of the 21 files, run `grep -n "dba-system.md" <file>` at Reconcile to get every line-numbered occurrence. Cross-check that every `(file, line)` pair has exactly one matching report row, and that every report row's `(file, line)` corresponds to a real grep match — both directions checked, not just file-level presence. A file-level-only check (unique file lists matching) is insufficient and does not satisfy this criterion, since it cannot detect a file with N occurrences reported as 1. |
| 2 | **Every `STRUCTURAL-POINTER` row's cited text exists, verified at its own stated granularity.** A **section-level** row's cited name is verified as an actual `dba-system.md` `##`-level heading. A **sub-part** row's cited phrase is verified against the delta table's own bolded lead-in text for the specific `rule_id` it names — not a `##` heading, since sub-parts never are one. A citation that does not match verbatim at its own granularity is itself a pre-existing defect, named as such, not silently corrected (see `templates/architecture-baseline.md`'s two flagged drifts). | For each section-level row, grep `dba-system.md` for the exact quoted name as a `##`-level heading. For each sub-part row, grep `changes/UPG-0065__CHG-20260807-001__delta-table.md` for the named `rule_id`'s `current_rule`/anchor text and compare the quoted phrase against its bolded lead-in. Record PASS or the specific mismatch for every row of both kinds — no row of either granularity is skipped. |
| 3 | **Every `STRUCTURAL-POINTER` row's `dba/` mapping is traceable — all of them, not a sample.** For every one of the report's `STRUCTURAL-POINTER` rows, the `dba/` component file(s) named in `detail` are individually cross-checked against `CHG-20260808-001`'s six Source Traceability tables, not asserted from memory and not spot-checked on a subset. A section-level row's mapping is confirmed clean only if grep confirms every `rule_id` under that `source_section` lands in the named file(s); a sub-part row's mapping is confirmed by grepping the named `rule_id` directly. | For each of the report's `STRUCTURAL-POINTER` rows in turn: section-level rows — grep every `dba/*/v1.md` Source Traceability table for `rule_id`s whose `source_section` (from the delta table) matches the cited section, confirm the resulting file set matches `detail`, exactly, for every row citing that section. Sub-part rows — grep the specific named `rule_id` directly in the one `dba/*/v1.md` file `detail` names. Reconcile records a full per-row result table, not a representative sample. |
| 4 | **No consumer file is edited.** `prompts/`, `scripts/`, `templates/`, `patterns/`, `dba-system.md`, and `dba-system-lean.md` are unchanged by this report-only change. | `git diff -- prompts/ scripts/ templates/ patterns/ dba-system.md dba-system-lean.md` → expect empty. |
| 5 | **Scope-boundary guardrails held.** No `configurations/*.yaml` file exists on disk, in any state — tracked, untracked, or already committed. No file under `dba/` is added, removed, or modified (this change reads the six existing `v1.md` files, never writes to `dba/`). | `find configurations -type f -name '*.yaml' 2>/dev/null` (or equivalent existence check, not a git-status-based one — a `configurations/*.yaml` already committed and clean would not appear in `git status` or `git diff` at all) → expect no output. Separately, both `git diff -- dba/` (catches modifications to the six already-committed `v1.md` files) and `git status --porcelain --untracked-files=all -- dba/` (catches any new tracked-but-uncommitted or untracked file under `dba/`, which `git diff` alone cannot see) → both expect empty. |
| 6 | **Cross-reference consistency.** The change record, the brief's Feature Thread, `status/self-development.md`, `backlog/features.md`, and `status/roadmap.md` agree on this change's current step and state, comparing only the fields each surface actually records. | Grep sweep for `UPG-0065` / `CHG-20260808-002` across all five files at Reconcile; no stale step/state claims (AJ-020 class). |

---

## Implementation Notes

Produced `changes/UPG-0065__CHG-20260808-002__compatibility-report.md`: 49 rows, one per
`dba-system.md` occurrence across the 21 files identified in Step 1 (`grep -n "dba-system.md"`
per file, summed). Breakdown: 34 `STRUCTURAL-POINTER`, 5 `WHOLE-FILE-LOAD`, 10 `GENERIC-MENTION`
(6 of the 10 are the previously-flagged `scripts/tests/codeos-implement-tests.sh` /
`prompts/codeos-self-dev.md` (4 occurrences) / `templates/codeos-change.md`
self-dev-or-non-reference rows, confirmed rather than assumed).

**Method.** Extracted every `(file, line)` pair via `grep -n "dba-system.md" <file>` per file
(not `grep -rln`, which only gives file-level presence — the exact gap Step 2 R1/R2 review found
in the acceptance criteria themselves). Classified each by reading its surrounding context (2-5
lines), not the grep match line alone, since most citations span the cited section name across a
line break. For every `STRUCTURAL-POINTER` row, cross-checked the cited section name against
`dba-system.md`'s actual `##` headings and, for sub-part citations, against the relevant row's
bolded lead-in text in the accepted delta table.

**Finding: three of the five `##`-level sections referenced in this sweep are not 1:1 with a
single `dba/` component file.** Detailed in the report's Part 2:

- **"Default Advisory Review" (Finding A) — a genuine split**, referenced 3 times (rows 9, 18,
  29). Its content divides across all three of `dba/doctrine/v1.md` (`REVIEW-6`),
  `dba/policies/review/v1.md` (the bulk), and `dba/tools/reviewer/v1.md`
  (`REVIEWER-TOOL-1`/`REVIEWER-TOOL-2`). A broad pointer to this whole section cannot be resolved
  to one replacement path at activation time — it needs either a specific sub-pointer (the
  pattern already used elsewhere in this report) or an explicit decision that broad
  section-level citations are no longer supported.
- **"Multi-Feature Architecture Synthesis Gate" (Finding B) — a near-split**, literally cited by
  name in 12 rows (fixed per Step 3 review R3, which found the row list and the stated count
  didn't reconcile, and found one row mislabeled): 11 cite it broadly, 1 (row 19) cites it
  together with a sub-part. None of the 12 depends on `ARCH-GATE-14`'s content (the one row that
  maps elsewhere, to `reviewer tool contract`). Four further rows (36, 40, 41, 45) cite an
  `ARCH-GATE-10`/`13` sub-part directly without the section name appearing on their own matched
  line — same `dba/` file, but not counted in the 12 since they don't literally cite the section.
  Recorded for completeness; not a blocking gap the way Finding A is.
- **"Controlled Plain English Writing Discipline" (Finding E) — a near-split**, literally cited by
  name in 3 rows (1, 33, 49; row 2 cites the `CPE-3a` sub-part alone, with no section-name text on
  its own matched line, and is not counted in this 3 — corrected per Step 3 review R3's finding on
  the same category error in Finding B, applied here proactively). 7 of 8 non-`RETIRE` rows land in
  `dba/policies/controlled-plain-english/v1.md`; the eighth (`CPE-3b`) lands in
  `dba/tools/reviewer/v1.md`. None of the 3 section-citing rows depends on `CPE-3b`'s content.
  Corrected per Step 3 review R2, which found the original wording here (and rows 1/33/49's own
  `detail` text) had claimed this section "maps cleanly," contradicting this near-split status —
  the report rows now say "near-split, not clean" explicitly, matching this note.

**Finding: five `WHOLE-FILE-LOAD` references (rows 11, 12, 31, 47, 48) are conditionally, not
unconditionally, compatible (Finding C).** Most significantly, `templates/project-CLAUDE.md` —
the exact text every new project's `CLAUDE.md` is scaffolded with by `dba-init.sh` — instructs
reading `dba-system.md` "in full." This stays correct only once the eventual thin-manifest form
of `dba-system.md` itself instructs cascading into every named `dba/` component; the brief's
manifest sketch states this requirement but the actual manifest text and cascade mechanism do not
exist yet. Not a defect — a dependency for the (unstarted) activation sub-step.

**Finding: two pre-existing citation-drift defects, unrelated to `v1` decomposition (Finding
D).** Found only because this sweep checked every citation against its actual target, not
introduced by this change and not fixed by it (`prompts/`, `templates/` stay untouched per this
change's scope boundary):
- `prompts/00-session-start.md:14` asks the reader to state "the 3 non-negotiable rules" against
  a doctrine section that has stated 6 since before this sweep began.
- `templates/architecture-baseline.md:33` and `:43` each cite a paraphrased, clause-dropping form
  of `ARCH-GATE-13`'s and `ARCH-GATE-10`'s actual bolded lead-in text.

**Self-verified before requesting review, not merely asserted:**
- `diff` between the report's `(file, line)` pairs and a fresh `grep -n` sweep of all 21 files:
  empty in both directions (49 = 49) — confirms AC1 at the exact granularity the criterion
  requires (per-occurrence, not per-file).
- **AC3, corrected per Step 3 review R1: every one of the 34 `STRUCTURAL-POINTER` rows'
  mappings, not a 5-row sample.** For each of the 25 section-level rows, extracted every
  `rule_id` under its cited `source_section` from the delta table and grepped all six `dba/*/v1.md`
  Source Traceability tables for each — confirming the file(s) named in the report's `detail`
  column exactly:
  - **"Implementation Profile"** (rows 5, 10, 14, 32): 18/18 `rule_id`s land in
    `dba/policies/implementation-profile/v1.md`. Clean, single file.
  - **"Contract-to-Implementation Failure Boundary"** (rows 7, 21, 22, 23): 5/5 `rule_id`s land in
    `dba/doctrine/v1.md`. Clean, single file.
  - **"Controlled Plain English Writing Discipline"** (rows 1, 33, 49): 7 of 8 non-`RETIRE`
    `rule_id`s land in `dba/policies/controlled-plain-english/v1.md`; `CPE-3b` lands in
    `dba/tools/reviewer/v1.md`. Near-split, matching the report's own note — none of these 3 rows
    depends on `CPE-3b`.
  - **"Multi-Feature Architecture Synthesis Gate"** (rows 6, 8, 13, 15, 17, 24, 35, 39, 42, 43,
    44): 17 of 18 `rule_id`s land in `dba/policies/architecture-synthesis/v1.md`; `ARCH-GATE-14`
    lands in `dba/tools/reviewer/v1.md`. Confirms Finding B exactly.
  - **"Default Advisory Review"** (rows 9, 18, 29): 10 `rule_id`s land in
    `dba/policies/review/v1.md`, 1 (`REVIEW-6`) in `dba/doctrine/v1.md`, 2
    (`REVIEWER-TOOL-1`/`REVIEWER-TOOL-2`) in `dba/tools/reviewer/v1.md`. Confirms Finding A
    exactly — a genuine 3-way split.

  For each of the 9 sub-part rows, grepped the named `rule_id` directly in the file `detail`
  names:
  - Rows 19, 36, 40, 45 (`ARCH-GATE-13`) → `dba/policies/architecture-synthesis/v1.md`. Confirmed.
  - Rows 37, 41 (`ARCH-GATE-10`) → `dba/policies/architecture-synthesis/v1.md`. Confirmed.
  - Row 20 (`IMPL-PROFILE-7`) → `dba/policies/implementation-profile/v1.md`. Confirmed.
  - Row 30 (`REVIEW-7`) → `dba/policies/review/v1.md`. Confirmed.
  - Row 2 (`CPE-3a`) → `dba/policies/controlled-plain-english/v1.md`. Confirmed.

  All 34 rows accounted for (25 + 9); every mapping in the report's `detail` column matches what
  grep independently finds. AC3 is now satisfied for the full set, not a sample.
- **AC2, corrected per Step 3 review R1/R2: verified at each row's own stated granularity, by one
  consistent rule.** The 25 section-level rows' 5 distinct cited names confirmed as exact
  `dba-system.md` `##` headings (`grep -c "^## <name>$"` → 1 each, listed above). The 9 sub-part
  rows' cited phrases compared directly against the delta table's bolded lead-in text for their
  named `rule_id` (`ARCH-GATE-13`, `ARCH-GATE-10`, `IMPL-PROFILE-7`, `REVIEW-7`, `CPE-3a`), applying
  one stated rule uniformly (fixed per R2, which found the original wording applied this
  inconsistently — "matches closely" for one row, "exact match... but missing X" for another): a
  citation matches when its quoted **core identifying clause** equals the bolded lead-in's core
  clause verbatim; a clarifying parenthetical (e.g. "(live Stage 4 eligibility)") is not required
  to be quoted for a match. Under this one rule: 7 of 9 rows match (rows 2, 19, 20, 30, 40, 41, 45);
  rows 36 and 37 are the two genuine drifts, each dropping words from the core clause itself, not
  just the parenthetical — named in Finding D, not silently corrected.
- `git diff -- prompts/ scripts/ templates/ patterns/ dba-system.md dba-system-lean.md dba/`:
  empty — confirms AC4/AC5's no-edit guarantee.
- `find configurations -type f -name '*.yaml'`: no output — confirms AC5's guardrail.

No out-of-scope edit made. No consumer file, `dba-system.md`, `dba-system-lean.md`, or `dba/*/v1.md`
touched. No `configurations/*.yaml` created. No `DBA-N` approved or activated.

---

## Reconciliation

**Acceptance verification:**

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Completeness — per-reference, not per-file | PASS | Fresh `grep -n "dba-system.md" <file>` re-run per file at Reconcile against all 21 files; diffed against the report's `(file, line)` pairs — empty in both directions, 49 = 49. Re-verified after all Step 3 review fixes (which only touched the `cited text`/`detail` prose columns, never the `file`/`line` columns) — still clean. |
| 2 | Every `STRUCTURAL-POINTER` row's cited text exists, at its own granularity | PASS | All 5 section-level headings ("Implementation Profile", "Multi-Feature Architecture Synthesis Gate", "Contract-to-Implementation Failure Boundary", "Controlled Plain English Writing Discipline", "Default Advisory Review") confirmed present in `dba-system.md` as `##` headings, exactly once each (`grep -c "^## <name>$"`). All 5 sub-part bolded lead-ins (`ARCH-GATE-13`, `ARCH-GATE-10`, `IMPL-PROFILE-7`, `REVIEW-7`, `CPE-3a`) re-extracted directly from the delta table and compared against the report's quoted phrases — 7 of 9 sub-part rows match verbatim on the core clause; rows 36 and 37 are the two named, un-silenced drifts (Finding D). |
| 3 | Every `STRUCTURAL-POINTER` row's `dba/` mapping is traceable — all 34, not a sample | PASS | Re-ran the full per-section `rule_id`-to-file grep at Reconcile (not re-trusting Step 3's own run): "Implementation Profile" 18/18 → implementation-profile; "Contract-to-Implementation Failure Boundary" 5/5 → doctrine; "Controlled Plain English Writing Discipline" 7 → controlled-plain-english + 1 (`CPE-3b`) → reviewer (near-split, Finding E); "Multi-Feature Architecture Synthesis Gate" 17 → architecture-synthesis + 1 (`ARCH-GATE-14`) → reviewer (near-split, Finding B); "Default Advisory Review" 1 → doctrine + 10 → review + 2 → reviewer (genuine 3-way split, Finding A). Identical to Step 3's recorded results — no drift between Step 3's claim and this independent Reconcile-time re-run. |
| 4 | No consumer file edited | PASS | `git diff -- prompts/ scripts/ templates/ patterns/ dba-system.md dba-system-lean.md dba/` → empty. |
| 5 | Scope-boundary guardrails held | PASS | `find configurations -type f -name '*.yaml'` → no output. `git status --porcelain --untracked-files=all -- dba/` → empty (no new/modified file under `dba/`, tracked or untracked). |
| 6 | Cross-reference consistency | PASS (after 2 self-caught fixes) | At the start of this Reconcile pass, both `status/self-development.md`'s Loop-step column and the brief's Status line still described Step 3 as current, after the human had already approved it — the same AJ-020-class staleness this feature's earlier Reconciles have each caught in themselves; fixed before writing this table, not after. All 5 named surfaces now agree, comparing only the fields each records: change record `current_step: 4-Reconcile`; brief "Steps 1-3 ACCEPTED... Step 4 drafted"; dashboard Loop step `4-Reconcile`; `backlog/features.md:98` and `status/roadmap.md:127` both `IN_PROGRESS` — correct, since neither tracks step-level state and `UPG-0065` the feature stays `IN_PROGRESS` regardless of this CHG's completion. |

**Consistency sweep (grep):** Cross-checked every internal cross-reference inside the report itself
— all "see Finding X" pointers (rows 1/33/49 → Finding E; rows 9/18/29 → Finding A; rows 6/8/13/etc.
→ Finding B) resolve to a Finding section that actually exists and actually covers that row, after
the Step 3 R3 relettering/rewrite. All "same near-split section as row 6" cross-references verified
to point at a row whose own `detail` text is internally consistent with the one they reference. No
orphaned link to `dba-system.md` sections that don't exist — every cited `##` heading and bolded
lead-in re-verified directly against source, not assumed carried over from Step 3. No stage-table
↔ prompt-file drift applicable — this change catalogs references, it does not touch any prompt or
stage-ID mapping.

**Findings scope-triage:**

| Finding | Triage | Action |
|---|---|---|
| (S1 R1, Codex) The "21 files" scope count didn't reconcile against its own stated breakdown — dropped `scripts/tests/codeos-implement-tests.sh` entirely | IN-SCOPE BLOCKER (High) | Fixed — full 21-file breakdown corrected to 18 + 2 + 1, each category confirmed |
| (S1 R1, Codex) `status/roadmap.md` was in the packet diff but not listed in "What changes" | IN-SCOPE BLOCKER (Medium) | Fixed |
| (S1 R2, Codex) NO OBJECTION; 1 Low IN-SCOPE NON-BLOCKER (grep count not independently re-shown in packet) | NON-BLOCKER | Accepted, not fixed (optional per Codex) |
| (S2 R1, Codex) AC1's file-level diff would falsely fail a correct report once any file legitimately has 2+ rows | IN-SCOPE BLOCKER (High) | Fixed by deduplicating both sides — this fix itself over-corrected, caught at R2 |
| (S2 R2, Codex) The dedup fix let a file with 3 real occurrences pass with only 1 reported row | IN-SCOPE BLOCKER (High) | Fixed — AC1 rewritten to check per `(file, line)` pair via `grep -n`, not file-level presence |
| (S2 R2, Codex) AC5's `git status`/`git diff` guardrail checks miss an already-committed `configurations/*.yaml` and a new untracked `dba/` file | IN-SCOPE BLOCKER (High) | Fixed — direct filesystem existence check for `configurations/`; both `git diff` and `git status --untracked-files=all` for `dba/` |
| (S2 R3, Codex) Brief/dashboard/trace-header disagreed on whether Step 2 review had run at all | IN-SCOPE BLOCKER (High) | Fixed — all three synced |
| (S3 R1, Codex) `STRUCTURAL-POINTER` schema/AC2 only covered `##`-heading citations; 9 of 34 rows cite sub-part bolded lead-ins | IN-SCOPE BLOCKER (High) | Fixed — schema/AC2 split into section-level vs. sub-part, both verified |
| (S3 R1, Codex) AC3 spot-checked only 5 of 34 rows | IN-SCOPE BLOCKER (High) | Fixed — full 34-row verification, recorded |
| (S3 R2, Codex) Brief/dashboard/trace-header still said "review not yet run" after R1 | IN-SCOPE BLOCKER (High) | Fixed |
| (S3 R2, Codex) Report rows 1/33/49 claimed the CPE section "maps cleanly," contradicting the near-split already named in Implementation Notes | IN-SCOPE BLOCKER (High) | Fixed — rows corrected, Finding E added |
| (S3 R2, Codex) AC2's verbatim rule applied inconsistently across rows 19/36/40 | IN-SCOPE BLOCKER (Medium) | Fixed — one explicit core-clause rule, applied uniformly |
| (S3 R3, Codex) Finding B's stated row count (12) didn't reconcile with its own two sub-lists (summed to 15, one row mislabeled) | IN-SCOPE BLOCKER (High) | Fixed — root cause found (rows 8/15/24's "cited text" column was truncated with "…", causing the miscount); truncation removed, Finding B rewritten with a mechanically grep-verified list; the identical error proactively found and fixed in Finding E |
| (S4, self-caught before this Reconcile pass, ×2) Dashboard Loop-step and brief Status line each still described Step 3 as current after human approval | IN-SCOPE BLOCKER (self-caught, not a Codex or human finding) | Fixed before writing this Reconciliation table |
| (Report Finding A) "Default Advisory Review" is a genuine 3-way split across `dba/doctrine/v1.md` / `dba/policies/review/v1.md` / `dba/tools/reviewer/v1.md` | OUT-OF-SCOPE BACKLOG — evidence for the next Phase A sub-step (proving `DBA-1` semantically equivalent), not a defect this change fixes | Not resolved here; carried forward as input to the equivalence-proof sub-step |
| (Report Finding C) 5 `WHOLE-FILE-LOAD` references, most significantly `templates/project-CLAUDE.md`, are conditionally compatible pending a manifest-cascade mechanism that doesn't exist yet | OUT-OF-SCOPE BACKLOG — dependency for the (unstarted) activation sub-step | Not resolved here |
| (Report Finding D) Two pre-existing citation-drift defects (`prompts/00-session-start.md:14`'s "3 non-negotiable rules" vs. actual 6; `templates/architecture-baseline.md`'s two paraphrased bolded-lead-in citations) | OUT-OF-SCOPE BACKLOG — pre-existing, unrelated to `v1` decomposition, found only because this sweep checked every citation | Not resolved here; `prompts/00-session-start.md` and `templates/architecture-baseline.md` stay untouched per this change's scope boundary |

No `REJECTED` or `SELF-REFERENCE`/`REVIEW-BOOKKEEPING` findings this Step — every finding across
all three steps' rounds was a real defect in the artifact under review, not a review-process
artifact of reviewing itself.

---
