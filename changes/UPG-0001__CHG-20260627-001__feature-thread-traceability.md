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
state: IN_PROGRESS
current_step: 2-Acceptance
implements:
  - UPG-0001
related_features: []
latest_review: REV__UPG-0001__CHG-20260627-001__S2__R4   # NO OBJECTION (advisory); review files not renamed (deferred)
fixes_findings:
  - "S1/R1 IN-SCOPE BLOCKER: brief AC#9 vs narrowed review-traceability — reconciled in brief"
  - "S1/R1 IN-SCOPE BLOCKER: brief migration step 8 vs historical-not-renamed — reconciled in brief"
  - "S1/R2 IN-SCOPE BLOCKER: brief state-vocabulary case inconsistency — normalized to canonical enum"
  - "S2/R1 IN-SCOPE BLOCKER: C2/E1/E2 unpinned checks under dirty workspace — pinned all git checks to base 89269f1"
  - "S2/R1 IN-SCOPE BLOCKER: D1 verification narrower than criterion — relaxed to 'at least one surface'"
  - "S2/R2 IN-SCOPE BLOCKER: C2 untracked-brief scratch dependency — switched to baseline-commit diff; E3 under-verified — verify by reading brief + E2"
  - "S2/R3 IN-SCOPE BLOCKER: C2 stated two verification methods — unified to baseline-commit diff"
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
  `implements`, `related_features`, `latest_review`, `fixes_findings`, `follow_up_of`); document
  the `changes/UPG-####__CHG-YYYYMMDD-NNN__slug.md` filename convention and the rare `MULTI` case.
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

**Scope boundary — what stays the same:**
- `dba-system.md` (downstream DBA doctrine) is **not touched** — scope is `self-dev only`. Any
  edit that appears to need a doctrine change stops and re-triages.
- **Mechanical migration only:** briefs receive *only* trace headers, feature IDs, new
  filenames, Feature-Thread sections, and cross-reference updates. No brief-body substance is
  rewritten, re-prioritized, or re-worded. Any substantive brief change discovered mid-migration
  is stopped and re-triaged as its own change.
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
action is a pre-migration baseline commit `<BASELINE_SHA>`** (recorded in Implementation Notes)
that first adds the two briefs untracked at `89269f1` — so **every** brief is tracked at the
baseline and the whole migration is diffable from committed artifacts, with **no session-external
state**. (This reuses the toolkit's own base-pinning model — cf. `codeos-review.sh stage-start
--base`.)

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
| C2 | **[req] Mechanical-only backlog migration:** active briefs are changed **only** by trace header / `feature_id` / new filename / `## Feature Thread` section / cross-reference updates — **no** brief-body substance, prose, or priority rewritten. | Rename-aware diff against the **baseline commit `<BASELINE_SHA>`** (created as the first Step-3 action so *all* briefs — including the two untracked at `89269f1` — are tracked at the baseline): `git diff -M <BASELINE_SHA> -- backlog/` shows, **per active brief**, only added YAML front-matter + an appended `## Feature Thread` section (plus any cross-reference link edits enumerated in Implementation Notes); **no body line modified or deleted**. Fully reproducible from committed artifacts — no session-external state. (`UPG-0001`'s own brief additionally carries its already-approved Step-1 scope reconciliations.) |
| C3 | No plain `000N` is used as **both** a feature ID and a change ID anywhere (brief AC#14). | `grep -rn "000[0-9]" status/ changes/ backlog/ docs/ README.md prompts/ templates/`; every remaining hit is unambiguously a Change ID (historical) — none labelled a feature. |

### D. Review model

| # | Criterion | How verified in Step 4 |
|---|---|---|
| D1 | A review identifies **both** the feature and the change in **at least one** of: review filename, packet content, review-log entry, or the change record's Feature Thread (relaxed brief AC#9). | Confirm **at least one** of the four surfaces carries both IDs — the criterion does **not** require all of them. (In this change two already do: a `grep "UPG-0001__CHG-20260627-001" reviews/review-log.md` hit and the change-record `latest_review`/Feature-Thread id — but a single surface satisfies the contract.) |
| D2 | **[req] Review-fix rule:** an in-scope finding's fix **stays inside the same `CHG-*`**; only an **OUT-OF-SCOPE BACKLOG** finding creates/links a new `UPG-####` — a fix never gets the next feature ID just for happening after a review (brief AC#10). | Read the Review-Fix Rule wording in `prompts/codeos-self-dev.md`; confirm this change's own S1/R1–R2 in-scope fixes stayed inside `CHG-20260627-001` (no new feature minted). |

### E. Scope-drift / governance guards (self-dev-governance)

| # | Criterion | How verified in Step 4 |
|---|---|---|
| E1 | **[req] No `dba-system.md` change** (scope axis = self-dev only). | Against the pinned base: `git diff 89269f1 -- dba-system.md` empty (tracked file; pinned, not workspace-relative). |
| E2 | **[req] No `scripts/codeos-review.sh` behavior change** — the `REV__…` id is documented-only; review-file renaming + script support are deferred. | Against the pinned base: `git diff 89269f1 -- scripts/codeos-review.sh` empty. |
| E3 | The deferred script-tooling work exists as its **own** filed backlog `UPG-####` (not folded into this change). | Read the filed follow-up brief: assert it has its **own** unique `feature_id` (≠ `UPG-0001`) and that its Problem/Goal name the deferred work (`REV__…` review-file renaming + `codeos-review.sh` emission; optionally `scripts/check_feature_threads.sh`). E2 (`git diff 89269f1 -- scripts/codeos-review.sh` empty) independently confirms none of that work is folded into this change. |
| E4 | Cross-reference integrity: `features.md` ↔ briefs ↔ `roadmap.md` ↔ dashboard agree on IDs/states; renames leave **no** dangling links. | Orphaned-link grep sweep (old filenames / `[change_id]` placeholders) clean or fixed. |

---

## Implementation Notes

<!-- Filled in Step 3. -->

---

## Reconciliation

<!-- Filled in Step 4. -->

---

<!-- METADATA -->
status: IN_PROGRESS
change_id: CHG-20260627-001
feature_id: UPG-0001
type: SELF_DEVELOPMENT
class: self-dev-governance
scope: self-dev only
backlog_item: backlog/UPG-0001-feature-thread-traceability.md
step_completed: 1
approved_by: human (Primoz Gorjup) — Step 1 Change Intent
approved_at: 2026-06-27
