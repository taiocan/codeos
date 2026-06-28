# Self-Development Change: UPG-0001__CHG-20260627-001 — feature-thread-traceability

<!--
PURPOSE: Per-change source of truth for a non-trivial change to the Codeos toolkit itself.
This change is executed through the 4-step Self-Development Loop (prompts/codeos-self-dev.md).
Each step requires explicit human approval and a compulsory (advisory) Codex review.
The live status row lives in status/self-development.md, not here.

This change DOGFOODS the Feature Thread scheme it introduces: its own filename carries the
primary feature ID (UPG-0001) and a unique change ID (CHG-20260627-001), and the trace header
below uses the new convention before the template is formally updated in Step 3.
-->

<!-- TRACE HEADER (new convention introduced by this very change) -->
```yaml
feature_id: UPG-0001
primary_feature_id: UPG-0001
change_id: CHG-20260627-001
slug: feature-thread-traceability
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0001
related_features: []
review_series: RVS__UPG-0001__CHG-20260627-001__S4   # all Step-4 reviews for this change (stable). Exact rounds + final decision live in review-log.md — never pinned here (cf. UPG-0028 self-reference)
review_state: ACCEPTED   # human decision 2026-06-28 (stop rule); confirming advisory review deferred (Codex rate-limited)
review_history: reviews/review-log.md
fixes_findings:
  - "S1/R1 IN-SCOPE BLOCKER: brief AC#9 vs narrowed review-traceability — reconciled in brief"
  - "S1/R1 IN-SCOPE BLOCKER: brief migration step 8 vs historical-not-renamed — reconciled in brief"
  - "S1/R2 IN-SCOPE BLOCKER: brief state-vocabulary case inconsistency — normalized to canonical enum"
  - "S2/R1 IN-SCOPE BLOCKER: C2/E1/E2 unpinned checks under dirty workspace — pinned all git checks to base 89269f1"
  - "S2/R1 IN-SCOPE BLOCKER: D1 verification narrower than criterion — relaxed to 'at least one surface'"
  - "S2/R2 IN-SCOPE BLOCKER: C2 untracked-brief scratch dependency — switched to baseline-commit diff; E3 under-verified — verify by reading brief + E2"
  - "S2/R3 IN-SCOPE BLOCKER: C2 stated two verification methods — unified to baseline-commit diff"
  - "S3/R1 IN-SCOPE BLOCKER: C2 'committed artifacts' overclaim — reworded to baseline + tracked-diff reproducibility"
  - "S3/R1 IN-SCOPE BLOCKER: body **Status**: BACKLOG contradicted header — normalized 24 briefs to canonical enum"
  - "S3/R2 IN-SCOPE BLOCKER: scope boundary didn't permit status-token normalization — amended C2 + Change Intent"
  - "S3/R3 IN-SCOPE BLOCKER: stale backlog/<slug> links in docs/prompts — fixed; 2 script comments re-triaged to UPG-0029"
  - "S3/R4 IN-SCOPE BLOCKER: docs/prompts/journal missing from touched-file inventory — added cross-ref-cleanup group"
  - "S3/R5-R7 IN-SCOPE BLOCKER: E4 verification command/claim imprecise — made resolve-based, exact, internally consistent"
follow_up_of: null
```

## Change Intent

**Why (problem in the toolkit):**
Codeos currently conflates four distinct identities and has no visible traceability spine:
1. the stable backlog feature/upgrade,
2. the concrete self-development change that implements it,
3. review rounds for that change,
4. follow-up fixes or backlog items born from reviewer findings.

The recent roadmap/review-fix work exposed the failure mode: a review fix (`0004-review-fixes`)
was assigned the next numeric ID as if it were the next *feature*, and iterative review rounds
created bookkeeping that drifted from the change record and the dashboard. There is no way to
answer, from filenames and status tables alone, "which feature does this change/review belong
to?" This change introduces a stable Feature Thread model and ID nomenclature so every related
file visibly shows which feature it belongs to.

**What changes (every file touched):**

*Convention / governance definitions*
- `templates/codeos-change.md` — replace the metadata block with the new trace header
  (`feature_id`, `primary_feature_id`, `change_id`, `slug`, `state`, `current_step`,
  `implements`, `related_features`, `review_series`, `review_state`, `review_history`,
  `fixes_findings`, `follow_up_of`); document the `changes/UPG-####__CHG-YYYYMMDD-NNN__slug.md`
  filename convention and the rare `MULTI` case.
- `prompts/codeos-self-dev.md` — Step 1 creates/selects a Feature Thread + `CHG-*` ID before
  opening the change record; add review-round (`R<N>`) + finding-classification vocabulary;
  document the Review-Fix Rule (in-scope fix stays in the same `CHG-*`; only OUT-OF-SCOPE
  BACKLOG spawns a new `UPG-####`).
- `CLAUDE.md` — consistency pass: update the Self-Development File Layout block and
  `changes/[change_id].md` references to the new naming; add a one-line pointer to the ID model.

*Dashboards / catalog / roadmap*
- `status/self-development.md` — new columns
  `Feature ID | Change ID | Class | Scope | Loop step | Latest review | State | Follow-up`;
  historical rows keep their existing IDs in Change ID with Feature ID mapped or `—` + note.
- `status/roadmap.md` — re-keyed by `UPG-####`
  (`Wave | Feature ID | Title | Priority | Depends on | Planned/active change | State`);
  Current-State entries labelled explicitly as Change IDs.
- `backlog/features.md` — becomes the authoritative `UPG-#### → one file` map.

*Backlog briefs (full but MECHANICAL migration)*
- Rename every active brief to `backlog/UPG-####-slug.md`; prepend the backlog feature header;
  append a `## Feature Thread` section. (This feature's own brief was renamed to
  `backlog/UPG-0001-feature-thread-traceability.md` and given its header in Step 1.)

*Review records*
- `reviews/review-log.md` — header note that new entries identify both feature and change;
  existing append-only entries left intact. The `REV__UPG-####__CHG-…__S<N>__R<N>` id is
  documented as the manual naming — documented, not required (see scope boundary).

*Cross-reference cleanup (consequent on the backlog rename; added in Step 3)*
- `docs/reviewer-pipeline.md`, `docs/reviewer-artifact-schemas.md`, `prompts/reviewer-automated.md`
  — repair stale `backlog/<old-slug>.md` links to the renamed `UPG-####-…` paths.
- `reviews/architecture-journal.md` — institutional-memory entries (AJ-001, AJ-002) from this
  change's reviews.
  (Two comment-only `backlog/…` refs inside `scripts/codeos-review.sh` are **not** edited here —
  re-triaged to `UPG-0029` to keep the script byte-identical; see scope boundary / E2.)

*Step-4 scope amendment — Review Series model (human-approved)*
- Added a 5th id type **`RVS__UPG-####__CHG-…__S<N>`** (review series) and replaced the stale-prone
  `latest_review` round field with **`review_series` + `review_state` + `review_history`** in
  `templates/codeos-change.md`, this change record, and the UPG-0001 brief. Added the **Surface
  ownership** table + **Self-Reference Boundary** (self-reference rule + stop rule) to
  `prompts/codeos-self-dev.md` and the brief; dashboard "Latest review" → review state/outcome.
  Reviewer/packet **enforcement** routed to `UPG-0028`. Reason: Step-4 hit a self-referential
  bookkeeping loop (reviewed artifacts cannot freshly name the review assessing them); this cuts
  the loop by separating stable traceability from live review chronology.

**Scope boundary — what stays the same:**
- `dba-system.md` (downstream DBA doctrine) is **not touched** — scope is `self-dev only`. Any
  edit that appears to need a doctrine change stops and re-triages.
- **Mechanical migration only:** briefs receive *only* trace headers, feature IDs, new
  filenames, Feature-Thread sections, cross-reference updates, and **canonical state-token
  normalization** (body `**Status**: BACKLOG` → the header enum `PROPOSED`/`PILOTED`, so body and
  header agree — a consistency fix, not new content). No brief-body prose, substance, or priority
  is rewritten or re-worded. Any substantive brief change discovered mid-migration is stopped and
  re-triaged as its own change. (Scope-boundary clarified in Step 3 per review finding S3/R1-2;
  still honors the human's "no substance rewrite" constraint.)
- **Review-file renaming is deferred.** This change renames no existing review file and does not
  change `scripts/codeos-review.sh` behavior/output naming. The acceptance bar (relaxing the
  brief's strict AC #9): a review must identify its feature and change in *at least one* of
  filename, packet content, review-log entry, or the change record's Feature Thread. Actual
  `REV__…` file renaming + `codeos-review.sh` emission is filed as a follow-up `UPG-####`.
- Historical change records `changes/0001..0004` are recorded as historical/piloted — **not
  renamed**, with no invented CHG IDs or dates.

**Class:** self-dev-governance
**Scope axis:** self-dev only
**Backlog item:** backlog/UPG-0001-feature-thread-traceability.md

---

## Acceptance Criteria

Consistency contracts this change must satisfy. Each is checkable in Step 4 by the named method.
"Active brief" = a non-historical backlog feature file (`backlog/UPG-####-slug.md`). Criteria are
grouped; the seven contracts the human required in Step 2 are tagged **[req]**.

**Pinned base for all `git`-based checks.** Every diff/scope-guard check below is computed
against the recorded pre-change base commit **`89269f1`** (the `HEAD` before this change; all of
this change's edits are uncommitted working-tree changes), with rename detection (`-M`) — **not**
against the dirty working tree. This makes the acceptance tests reproducible despite the reviewer
running with no base pin and a dirty workspace. The literal SHA `89269f1` (not `HEAD`) remains the
base for scope-guard checks (E1/E2). For the mechanical-migration check (C2), the **first Step-3
action is a pre-migration baseline commit `b835016`** (recorded in Implementation Notes)
that first adds the two briefs untracked at `89269f1` — so **every** brief is tracked at the
baseline. The migration is then diffable as **this change's tracked working-tree changes against the
pinned baseline `b835016`** (and, once this change is committed, its change commit). The base is
committed and pinned and the migrated side is ordinary tracked-file diffs — **no ephemeral /
session-external state** (unlike a scratch snapshot). (Reuses the toolkit's base-pinning model —
cf. `codeos-review.sh stage-start --base`.)

### A. Identity & numbering

| # | Criterion | How verified in Step 4 |
|---|---|---|
| A1 | Every active backlog brief has exactly one `feature_id` (brief AC#1). | `grep -rn "^feature_id:" backlog/UPG-*.md`; count == number of active briefs; no duplicate value (`… \| sort \| uniq -d` empty). |
| A2 | **[req] UPG numbering rule:** `UPG-0001` = this feature; remaining active briefs numbered in **roadmap order** (Current-State/piloted first, Wave 1→5, then the two loose briefs). IDs assigned once, never reused; piloted/complete briefs get IDs marked truthfully (no false sequencing). | Read `backlog/features.md` map; assert `UPG-0001` == feature-thread-traceability; assert assignment order matches `status/roadmap.md`; `uniq -d` on the UPG list is empty. |
| A3 | `backlog/features.md` maps each `UPG-####` to **exactly one** existing backlog file (brief AC#6). | For each map row, `test -f` the target; assert one-to-one (no UPG mapped twice; no file mapped twice). |
| A4 | Change-record trace header lists `feature_id`, `primary_feature_id`, `change_id`, `implements` (brief AC#5). | Read this change record's trace header. |
| A5 | Every **non-trivial change** filename carries the primary `UPG-####` **and** a unique `CHG-*` (brief AC#3/AC#4). | `ls changes/UPG-*__CHG-*` includes this change; `CHG-*` values unique. Historical `0001..0004` are exempt by C1. |

### B. Visibility ("at all times, in all related files")

| # | Criterion | How verified in Step 4 |
|---|---|---|
| B1 | **[req] Feature ID + Change ID visibility:** the feature owning a change is identifiable **from the filename alone** (brief AC#12); opening a brief's `## Feature Thread` shows its changes, reviews, tracked findings, and follow-ups (brief AC#13). | Inspect this change's filename (`UPG-0001__CHG-…`); inspect `backlog/UPG-0001-…` Feature Thread → lists `CHG-20260627-001`. |
| B2 | Every active backlog brief has a `## Feature Thread` section (brief AC#2). | `grep -L "## Feature Thread" backlog/UPG-*.md` returns empty. |
| B3 | `status/self-development.md` separates **Feature ID** from **Change ID** columns (brief AC#8). | Read dashboard header; every active row has distinct, populated Feature ID and Change ID cells. |
| B4 | `status/roadmap.md` uses `UPG-####` as the primary key with the required columns; Current-State entries are explicitly labelled **Change IDs**, not feature IDs (brief AC#7). | Read roadmap header row; wave rows lead with `UPG-`; Current-State table column header says Change ID. |

### C. Migration boundaries

| # | Criterion | How verified in Step 4 |
|---|---|---|
| C1 | **[req] Historical 0001–0004 mapping rule:** existing change records `changes/0001..0004` are **not** renamed, receive **no** invented `CHG-*` IDs or dates, and appear truthfully as historical/piloted in the dashboard (Change ID = existing stem; Feature ID = mapped UPG where a brief exists, else `—` + note). No false retroactive sequencing (brief AC#11). | `ls changes/` shows `0001..0004` unchanged; `grep -E "CHG-2026" ` finds none attached to them; read dashboard historical rows. |
| C2 | **[req] Mechanical-only backlog migration:** active briefs are changed **only** by trace header / `feature_id` / new filename / `## Feature Thread` section / cross-reference updates / **canonical state-token normalization** (e.g. body `**Status**: BACKLOG` → the header enum `PROPOSED`/`PILOTED`, so body and header agree) — **no** brief-body prose, substance, or priority rewritten. | Rename-aware diff against the **baseline commit `b835016`** (created as the first Step-3 action so *all* briefs — incl. the two untracked at `89269f1` — are tracked at the baseline): `git diff -M b835016 -- backlog/` shows, **per active brief**, only: added YAML front-matter, an appended `## Feature Thread`, at most one `**Status**:` token normalized to the header enum, and any enumerated cross-reference edits; **no prose/substance/priority line modified or deleted**. Reproducible by diffing the pinned baseline `b835016` against this change's tracked changes (its change commit once landed) — no ephemeral/scratch state. (`UPG-0001`'s own brief additionally carries its approved Step-1 scope reconciliations and the example-id `UPG-0025`→`UPG-0000` substitution.) |
| C3 | No plain `000N` is used as **both** a feature ID and a change ID anywhere (brief AC#14). | `grep -rn "000[0-9]" status/ changes/ backlog/ docs/ README.md prompts/ templates/`; every remaining hit is unambiguously a Change ID (historical) — none labelled a feature. |

### D. Review model

| # | Criterion | How verified in Step 4 |
|---|---|---|
| D1 | A review identifies **both** the feature and the change in **at least one** of: review filename, packet content, review-log entry, or the change record's Feature Thread (relaxed brief AC#9). | Confirm **at least one** of the four surfaces carries both IDs — the criterion does **not** require all of them. (In this change two already do: a `grep "UPG-0001__CHG-20260627-001" reviews/review-log.md` hit and the change-record `review_series`/Feature-Thread id — but a single surface satisfies the contract.) |
| D2 | **[req] Review-fix rule:** an in-scope finding's fix **stays inside the same `CHG-*`**; only an **OUT-OF-SCOPE BACKLOG** finding creates/links a new `UPG-####` — a fix never gets the next feature ID just for happening after a review (brief AC#10). | Read the Review-Fix Rule wording in `prompts/codeos-self-dev.md`; confirm this change's own S1/R1–R2 in-scope fixes stayed inside `CHG-20260627-001` (no new feature minted). |

### E. Scope-drift / governance guards (self-dev-governance)

| # | Criterion | How verified in Step 4 |
|---|---|---|
| E1 | **[req] No `dba-system.md` change** (scope axis = self-dev only). | Against the pinned base: `git diff 89269f1 -- dba-system.md` empty (tracked file; pinned, not workspace-relative). |
| E2 | **[req] No `scripts/codeos-review.sh` behavior change** — the `REV__…` id is documented-only; review-file renaming + script support are deferred. | Against the pinned base: `git diff 89269f1 -- scripts/codeos-review.sh` empty. |
| E3 | The deferred script-tooling work exists as its **own** filed backlog `UPG-####` (not folded into this change). | Read the filed follow-up brief: assert it has its **own** unique `feature_id` (≠ `UPG-0001`) and that its Problem/Goal name the deferred work (`REV__…` review-file renaming + `codeos-review.sh` emission; optionally `scripts/check_feature_threads.sh`). E2 (`git diff 89269f1 -- scripts/codeos-review.sh` empty) independently confirms none of that work is folded into this change. |
| E4 | Cross-reference integrity: `features.md` ↔ briefs ↔ `roadmap.md` ↔ dashboard ↔ docs/prompts agree on IDs/states; renames leave **no** dangling navigational links. | Every `backlog/*.md` path referenced in tracked files (excl. `changes/`, `reviews/`) resolves to an existing file **except the two intentional textual mentions noted below**. The de-duplicating verify command — `git grep -hoE 'backlog/[A-Za-z0-9._-]+\.md' -- ':!changes' ':!reviews' \| sort -u \| while read p; do [ -f "$p" ] \|\| echo MISSING $p; done` — lists exactly **two** unique non-resolving paths, both *intentional textual* mentions (not navigational links): (a) `backlog/UPG-0000-…` — the reserved **example** id in UPG-0001's spec; (b) `backlog/reviewer-decision-integrity.md` — which occurs in two `scripts/codeos-review.sh` comments **plus** one UPG-0029 line describing them, all deferred to `UPG-0029` (script kept byte-identical, E2). |

### F. Self-reference boundary (added in Step 4 — review-series model)

| # | Criterion | How verified in Step 4 |
|---|---|---|
| F1 | **No reviewed artifact embeds live review chronology.** Change records / briefs / dashboard carry a stable `review_series` (`RVS__…__S<N>`) + `review_state` only — never a round-specific "latest review". Exact `REV__…__R<N>` rounds + human decisions live **only** in `reviews/review-log.md` + `reviews/codex/*`. | (1) the YAML *field* is gone: `grep -rnE '^[[:space:]]*latest_review:' changes/ backlog/ status/ templates/` → none (prose mentions of the word in this change record's own narrative don't count). (2) No round-specific latest claim in the **live bookkeeping** surfaces: `grep -rnE 'REV__[A-Za-z0-9_]+__R[0-9]' status/ changes/` → none (round ids live under `reviews/`; the UPG-0001 brief shows `REV__…__R<N>` only as an ID-format *example*, not a live claim). (3) `grep -rn 'review_series\|review_state' changes/ templates/` → present. |

---

## Implementation Notes

**Pre-migration baseline commit:** `b835016` (`b835016183f078f1a567e7bca7157c32c5f082ca`) — the
required C2 base. It tracks the two briefs untracked at `89269f1`
(`UPG-0001-feature-thread-traceability`, `replacing-review-scripts`) plus the Step 1/2 change
record, dashboard, review log, and architecture journal. Per the human's constraints it excludes
non-scratch codex assessments, `dba-system.md`, and `scripts/codeos-review.sh`.

**Mechanical backlog migration (27 briefs):** `git mv` to `backlog/UPG-####-slug.md` + prepended
trace-header front matter + appended empty `## Feature Thread` section, via a scripted prepend/
append (bodies cat'd verbatim). UPG ids assigned in roadmap order: `UPG-0002` doc-consistency
(COMPLETE), `UPG-0003` reviewer-decision-brief (PILOTED), `UPG-0004…0026` Wave 1→5, `UPG-0027`
replacing-review-scripts, `UPG-0028` reviewer-self-reference-recursion. `UPG-0001`'s own brief got
its trace header (Step 1) and a populated `## Feature Thread` (this step).

**C2 evidence:** against baseline `b835016`, each migrated brief's diff is: added front-matter +
appended `## Feature Thread` + (for the 24 "Upgrade:" briefs) one `**Status**: BACKLOG` → header-enum
(`PROPOSED`/`PILOTED`) token normalization so body and header agree (added per Step-3 review finding 2;
permitted by the amended C2 criterion). Non-pure-addition exceptions, all accounted for: `UPG-0027`
gained a trailing newline (baseline lacked one; body text verbatim); `UPG-0001`'s own brief carries its
approved Step-1 scope reconciliations + the example-id `UPG-0025`→`UPG-0000` substitution (≈20 lines,
to avoid colliding with the real UPG-0025). `backlog/features.md` is the **index** (not a migrated
brief) and is intentionally rewritten into the authoritative UPG map. **No brief prose, substance, or
priority was rewritten.**

**Convention decision (recorded for the reviewer):** backlog feature headers carry
`feature_id/slug/title/status/priority` + thread-linkage fields; **`class`/`scope` are declared per
change** (Step 1 + dashboard Class/Scope columns) and are optional in a backlog header. UPG-0001's
spec example was annotated to match, so spec and implementation agree. `BACKLOG` status mapped to the
new enum `PROPOSED`.

**Other files updated:** `backlog/features.md` (now the authoritative `UPG-#### → file` map; ~28
broken slug links repaired to `UPG-####-…`); `status/roadmap.md` (re-keyed by `UPG-####`,
Current-State labelled as Change IDs); `status/self-development.md` (new columns Feature ID | Change
ID | Class | Scope | Loop step | Latest review | State | Follow-up; historical `0001..0004` kept as
Change IDs, unrenamed; the `changes/[change_id].md` prose non-blocker fixed); `templates/codeos-change.md`
(trace header + filename convention); `prompts/codeos-self-dev.md` (Step 1 feature-thread-first,
nomenclature table, Review-Fix Rule, review-round `R<N>` command); `CLAUDE.md` (file-layout block,
naming refs, ID-model pointer); `reviews/review-log.md` (identification note, append-only entries
intact).

**Cross-reference cleanup (E4):** after the rename, stale `backlog/<old-slug>.md` links were
repaired in `docs/reviewer-pipeline.md`, `docs/reviewer-artifact-schemas.md`, and
`prompts/reviewer-automated.md` (→ `UPG-0003`/`UPG-0015`/`UPG-0018`). Two *comment-only* path
references inside `scripts/codeos-review.sh` are deliberately left and re-triaged to `UPG-0029`
(editing the script would violate E2). The resolve-check then reports exactly two unique
non-resolving paths, both intentional: the reserved `backlog/UPG-0000-…` example in UPG-0001's
spec, and `backlog/reviewer-decision-integrity.md` (the two script comments + the UPG-0029 line
describing them). No accidental dangling links remain.

**Deferred follow-up filed (E3):** `backlog/UPG-0029-review-naming-and-thread-tooling.md` — the
`REV__…` review-file renaming + `codeos-review.sh` emission + optional `check_feature_threads.sh`
+ the two stale script comment-path refs above. `scripts/codeos-review.sh` behavior unchanged;
`dba-system.md` untouched.

---

## Reconciliation

**Acceptance verification** (against Step-2 criteria; all `git`-based checks pinned to base
`89269f1` / baseline `b835016`):

| # | Criterion | Result | Evidence |
|---|---|---|---|
| A1 | unique `feature_id` per active brief | PASS | 29 briefs, 29 unique front-matter ids, no dups |
| A2 | UPG numbering rule (UPG-0001=this; roadmap order) | PASS | UPG-0001=feature-thread; ids contiguous UPG-0001..0029; order matches `roadmap.md` |
| A3 | `features.md` maps each UPG → exactly one file | PASS | 29 unique map rows, all `test -f` resolve |
| A4 | change-record trace header complete | PASS | `feature_id`/`primary_feature_id`/`change_id`/`implements` present |
| A5 | non-trivial change filename has UPG + unique CHG | PASS | `changes/UPG-0001__CHG-20260627-001__…` |
| B1 | feature visible from filename; brief thread shows changes | PASS | filename carries UPG-0001; UPG-0001 thread lists CHG-20260627-001 |
| B2 | every active brief has `## Feature Thread` | PASS | `grep -L` empty over 29 briefs |
| B3 | dashboard separates Feature ID / Change ID | PASS | header `| Feature ID | Change ID | …` |
| B4 | roadmap keyed by UPG; Current-State labelled Change IDs | PASS | `| Wave | Feature ID |` + `| Change ID | Feature ID | State |` |
| C1 | historical 0001–0004 unrenamed, truthful, no fabricated CHG | PASS | files present at original names; no `CHG-2026` on them; dashboard rows historical |
| C2 | mechanical-only migration | PASS | `git diff -M b835016 -- backlog/`: only added header+thread + 24 status-token normalizations + UPG-0027 EOL; no prose/substance/priority deletions (features.md index + UPG-0001 excluded as intended) |
| C3 | no plain `000N` conflated as feature/change/review id | PASS | scan finds only labelled historical change-stems |
| D1 | review identifies feature + change (≥1 surface) | PASS | `review-log` entries name `UPG-0001__CHG-20260627-001` |
| D2 | Review-Fix Rule; in-scope fixes stay in CHG | PASS | rule in `prompts/codeos-self-dev.md`; all S1–S3 fixes stayed in CHG-20260627-001; only new feature is UPG-0029 (OUT-OF-SCOPE follow-up, not a fix) |
| E1 | no `dba-system.md` change | PASS | `git diff 89269f1 -- dba-system.md` empty |
| E2 | no `codeos-review.sh` behavior change | PASS | `git diff 89269f1 -- scripts/codeos-review.sh` empty (byte-identical) |
| E3 | deferred tooling filed as own UPG | PASS | `UPG-0029` exists, own `feature_id`, names `codeos-review.sh`/`REV__`/`check_feature_threads.sh` |
| E4 | cross-reference integrity (no dangling links) | PASS | resolve-check lists only 2 intentional non-resolving paths (UPG-0000 example; deferred `reviewer-decision-integrity` in 2 script comments + 1 UPG-0029 line) |
| F1 | no reviewed artifact embeds live review chronology | PASS | no `latest_review:` field anywhere; no `REV__…__R<N>` in `status/`+`changes/`; `review_series`/`review_state` present; rounds live only in `reviews/` |

**Consistency sweep (grep):** clean. Old `backlog/<slug>.md` links repaired in docs/prompts; the
only non-resolving `backlog/*.md` mentions are the two intentional ones in E4. No stale
`changes/[change_id].md` placeholders remain in normative files (template/prompt/CLAUDE/dashboard
updated to the new naming).

**Tooling smoke:** `scripts/codeos-review.sh` exercised repeatedly through the loop — `review`
entries recorded in `review-log.md` for `selfdev-step-1..4` and `decision` entries for steps 1–3
(the step-4 decision is the final human gate, appended on approval) — with valid packets, the
review subcommand completing normally (no fail-closed abort on its inline v0 field/enum checks),
and behavior unchanged from base. (Formal JSON-Schema validation remains **deferred** per
`docs/reviewer-pipeline.md` and is **not** claimed here.)

**Self-reference note (cf. `UPG-0028`) — RESOLVED by the Review Series model:** this change's
artifacts include the bookkeeping that tracks its own review, so the Step-4 gate review could not
observe its own logged verdict at packet-build time — earlier rounds kept flagging a stale
"latest review" round. This is now **fixed by design** (F1): reviewed artifacts carry a stable
`review_series` (`RVS__…__S<N>`) + `review_state`, never a live round, so the lag can no longer
manifest. Exact `REV__…__R<N>` rounds + the human decision live only in `reviews/review-log.md` /
`reviews/codex/*`. Reviewer-side enforcement of the Self-Reference Boundary is tracked in `UPG-0028`.

**Findings scope-triage:**

| Finding | Triage | Action |
|---|---|---|
| `backlog/UPG_ideas/review_optimization.md` (user-renamed from `user_needs/` mid-Step-4) — an untracked ideas note on reducing reviewer packet size; not a feature brief, not in baseline | OUT-OF-SCOPE BACKLOG | Left untouched (Step-4 "no new scope"); its ideas (diff-only C2 appraisal, header-only index for E4) are candidate input to `UPG-0029` |
| 2 comment-only `backlog/reviewer-decision-integrity.md` refs in `scripts/codeos-review.sh` | OUT-OF-SCOPE BACKLOG | Deferred to `UPG-0029` (script kept byte-identical, E2) |
| `REV__…` review-file renaming + `codeos-review.sh` emission | OUT-OF-SCOPE BACKLOG | Deferred to `UPG-0029` (documented manual convention only) |
| Step-4 self-referential bookkeeping lag — reviewed artifacts cannot embed the in-flight Step-4 verdict | RESOLVED (was IN-SCOPE NON-BLOCKER) | **Fixed by design:** introduced the `RVS` review-series id + `review_series`/`review_state` (F1) + Self-Reference Boundary; reviewed artifacts no longer name a live round, so the lag can no longer manifest. Reviewer-side enforcement → `UPG-0028`. |

---

<!-- METADATA -->
status: COMPLETE
change_id: CHG-20260627-001
feature_id: UPG-0001
type: SELF_DEVELOPMENT
class: self-dev-governance
scope: self-dev only
backlog_item: backlog/UPG-0001-feature-thread-traceability.md
step_completed: 4
approved_by: human (Primoz Gorjup) — Steps 1-4 (Step 4 accepted by decision; confirming review Codex-rate-limited)
approved_at: 2026-06-28
