# Self-Development Change: 0002-doc-consistency-rename

## Change Intent

**Why (problem in the toolkit):**
The `0001-claude-split` change moved the master DBA doctrine from `CLAUDE.md` to
`dba-system.md` and repointed every doctrine-*loading* path, but left descriptive prose in
`docs/` that still attributes the master doctrine (9-step loop, non-negotiable rules, Truth
Authority, DBA vocabulary, Review Logging, Artifact Classification) to a file named
`CLAUDE.md`. Filed at the time as `backlog/doc-consistency-doctrine-rename.md`.

**What changes:**
- `docs/codeos-manual.md` — 24 doctrine-attribution refs `CLAUDE.md` → `dba-system.md`.
- `docs/oap-adoption-candidates.md` — 1 doctrinal-stance ref → `dba-system.md`.

**Scope boundary — what stays the same:**
- Genuine project-level `CLAUDE.md` references are preserved (`codeos-manual.md` lines 336,
  687, 694).
- `docs/oap-codeos-integration.md` and `docs/reviewer-pipeline.md` are **left unchanged**:
  their `CLAUDE.md` references are historical naming-collision analysis and governance
  constraints where a rename does not make sense (renaming would either misstate history —
  "Both claim `CLAUDE.md`" is literally true of OAP — or editorialize an analysis doc). This
  reflects the "rename only where it makes sense" instruction (a deliberate pull-back from an
  earlier "rename everywhere" intent).
- No code, no doctrine substance, no loading paths change.

**Class:** documentation (normative)
**Scope axis:** downstream doctrine only
**Backlog item:** backlog/doc-consistency-doctrine-rename.md

---

## Acceptance Criteria

| # | Criterion | How verified |
|---|---|---|
| 1 | No doctrine-attribution `CLAUDE.md` ref remains in `docs/` | `grep -rn "CLAUDE\.md" docs/` — survivors are only project-level refs (manual 336/687/694) or the deliberately-preserved oap-codeos-integration / reviewer-pipeline analysis refs |
| 2 | No `dba-system.md` misattribution introduced (no line wrongly claims OAP uses it) | `grep -rn "dba-system\.md" docs/` read-through |
| 3 | Renders intact (no broken inline code spans / table cells) | spot read of Source-Map rows 1133–1140 + changed prose |

---

## Implementation Notes

Applied 24 line-addressed substitutions in `codeos-manual.md` (lines 83, 92, 100, 158, 178,
196, 270, 282, 295, 302, 322, 557, 663, 670, 1044, Source-Map rows 1133–1140, 1164) and 1 in
`oap-adoption-candidates.md` (line 82). An earlier exploratory pass also renamed 8 refs in
`oap-codeos-integration.md` and 1 in `reviewer-pipeline.md`; both were **reverted to original**
after the "rename only where it makes sense" steer, since those are historical-analysis /
naming-collision / governance references. Final scope is 25 edits across 2 files.

---

## Reconciliation

**Acceptance verification:**

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | No doctrine-attribution CLAUDE.md left | PASS | `grep` survivors: manual 336/687/694 (project file); oap-codeos-integration 75/173/175 + reviewer-pipeline 20 (deliberately preserved analysis/governance) |
| 2 | No dba-system.md misattribution | PASS | new refs limited to `codeos-manual.md` + `oap-adoption-candidates.md`; none claims OAP uses the doctrine |
| 3 | Renders intact | PASS | Source-Map rows and prose spans intact |

**Consistency sweep (grep):** clean — see acceptance #1/#2.

**Findings scope-triage:**

| Finding | Triage | Action |
|---|---|---|
| oap-codeos-integration.md / reviewer-pipeline.md CLAUDE.md refs | IN-SCOPE NON-BLOCKER → leave | Reverted; preserved as historical/governance refs ("rename only where it makes sense") |

**Codex review:** not run — change executed under direct human supervision (human acting as
in-session reviewer). `codex` CLI is available; automated review deferred by choice, not
silently skipped. Run on request via `codeos-review.sh review 0002-doc-consistency-rename ...`.

---

<!-- METADATA -->
status: COMPLETE
change_id: 0002-doc-consistency-rename
type: SELF_DEVELOPMENT
class: documentation
scope: downstream doctrine only
backlog_item: backlog/doc-consistency-doctrine-rename.md
step_completed: 4
approved_by: human (in-session, 2026-06-27)
approved_at: 2026-06-27
