# Self-Development Change: UPG-0044__CHG-20260712-001 — reviewer-pipeline-architecture-refresh

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0044
primary_feature_id: UPG-0044
change_id: CHG-20260712-001
slug: reviewer-pipeline-architecture-refresh
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0044
related_features: [UPG-0042, UPG-0037, UPG-0027, UPG-0001]
review_series: RVS__UPG-0044__CHG-20260712-001__S4
review_profile: PROFILE-2
review_state: REVIEWED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

## Change Intent

**Why (problem in the toolkit):**

`docs/reviewer-pipeline.md` (520 lines, still headed `status: PILOT`) documents the reviewer
system roughly as it stood around UPG-0003/UPG-0006. Since then the system grew a layered
architecture — self-dev `PROFILE-0..5` vs. downstream flat R1/R2/R3 cadence (UPG-0037),
evidence/packet modes full/delta/sha-only refined across UPG-0027's sub-changes, UPG-0031
(delta-mode base-vs-working-tree fix), and UPG-0042 (oversized-packet warnings) — but the doc
narrates these as scattered sections (§4b/§4c/§14) rather than as one coherent model. A human
review of the reviewer system (this session) independently reconstructed a five-layer
architecture (human gate → workflow doctrine → review engine → packet building → durable
records) from first principles because no single section states it. That reconstruction is
worth capturing once, in the doc, instead of being re-derived by every future session that
needs the whole picture.

**What changes:**

- `docs/reviewer-pipeline.md` — restructured/extended in place:
  - Add an architecture-level framing section (four/five-layer model: human gate / workflow
    doctrine / review engine / durable records) near the top, stating the core rule —
    Codex produces advisory evidence, the human gate decides — once, clearly.
  - Add or consolidate a section naming self-dev `PROFILE-0..5` vs. downstream flat
    R1/R2/R3 as two distinct, intentionally non-unified cadences, cross-referencing
    `prompts/codeos-self-dev.md` Step 0a and the UPG-0037 change record rather than
    re-describing their design.
  - Consolidate the existing full/delta/sha-only evidence-mode material (currently in §14
    per UPG-0042) with a use-when / main-risk table; extend rather than duplicate.
  - State the `EMPTY_PACKET` fail-closed behavior and the delta-mode base-vs-working-tree
    fix (UPG-0031) as one coverage-state model (`FULL` / `PARTIAL` / `EMPTY_PACKET` /
    `SECRET_REDACTION`).
  - Add Mermaid diagrams for: the high-level flow, the packet-building/coverage-state flow,
    and the durable-record-ownership split (change record / backlog feature thread / status
    dashboard / review log / raw Codex output — the Self-Reference Boundary from UPG-0001).
  - Explicitly label any "ReviewRun" / control-plane-lite / event-ledger material as
    **future direction, not implemented behavior**, visually and textually separated from
    current-state sections.
  - Revisit the `status: PILOT` badge only if it is factually stale in light of what this
    change documents — flag, don't resolve, if that classification itself looks wrong (out
    of scope for this change; would need its own triage).
- No other files change **implementation-wise** beyond `docs/reviewer-pipeline.md`. Standard
  Step 1 Feature Thread bookkeeping already touched `backlog/features.md` (new `UPG-0044` row)
  and `status/self-development.md` (row activation) — both required by `prompts/codeos-self-dev.md`
  Step 1 ("Feature Thread first" / "Activate the row in `status/self-development.md`") and not
  part of the doc-refresh implementation itself. If cross-references elsewhere turn out to need
  updates (e.g., a stale section-number reference in `prompts/codeos-self-dev.md`), that will be
  caught by the Step 4 grep sweep and either fixed in-scope (pure cross-reference repair) or
  re-triaged as its own change if it's more than that.

**Scope boundary — what stays the same:**

- No new code: no changes to `scripts/codeos-review.sh`, `tools/reviewer/*`, or any binary.
- No new `ReviewRun` records, no event ledger, no generated `reviews/review-log.md` changes.
- No changes to `CLAUDE.md` or `dba-system.md` — this doc is self-dev only. Evidence:
  `grep -n "reviewer-pipeline" dba-system.md CLAUDE.md` → no matches (exit 1); the only
  in-repo references to `docs/reviewer-pipeline.md` are from `prompts/codeos-self-dev.md`
  and `prompts/reviewer-automated.md` (both self-dev-only prompts) plus several `changes/*`
  records.
- No re-litigation or redesign of self-dev `PROFILE-0..5` or downstream R1/R2/R3 cadence —
  cite the existing decisions, don't reopen them.
- No new standalone `docs/reviewer-architecture.md` file — content lands in the existing
  `docs/reviewer-pipeline.md` to avoid two files describing the same system.
- No behavioral claims are added or removed — every existing normative statement (I/O
  behavior, exit codes, fail-closed guarantees) is preserved; this is a reframing and
  consolidation of existing accurate content plus new diagrams, not a rewrite of what the
  system does.

**Class:** documentation
**Scope axis:** self-dev only
**Backlog item:** backlog/UPG-0044-reviewer-pipeline-architecture-refresh.md

---

## Acceptance Criteria

| # | Criterion | How it will be verified |
|---|---|---|
| 1 | `docs/reviewer-pipeline.md` has a layer-model framing section near the top (before the existing §1 "Roles" content, or replacing/absorbing it) stating the core rule — Codex produces advisory evidence, the human gate decides — once, explicitly. | Read-through of the section; grep for language equivalent to "advisory" + "human" near the top of the file (first ~60 lines). |
| 2 | The doc names self-dev `PROFILE-0..5` vs. downstream flat R1/R2/R3 as two distinct, intentionally non-unified cadences, and cross-references `prompts/codeos-self-dev.md` Step 0a and the UPG-0037 change record rather than redefining either. | Grep the new section for `PROFILE-0` and `R1/R2/R3` (or equivalent) plus a cross-reference to `prompts/codeos-self-dev.md` and `UPG-0037`; confirm no new profile/cadence rules are stated that don't already exist in those sources. |
| 3 | The existing full/delta/sha-only evidence-mode material (current §14, added by UPG-0042) is consolidated with a use-when / main-risk table, not duplicated as a second competing section. | Diff review: confirm §14 content is extended/moved, not copy-pasted into a new section; grep for "full", "delta", "sha-only" — each should appear in exactly one consolidated location. |
| 4 | `EMPTY_PACKET` fail-closed behavior and the delta-mode base-vs-working-tree fix (UPG-0031) are stated as one coverage-state model covering `FULL` / `PARTIAL` / `EMPTY_PACKET` / `SECRET_REDACTION`. | Grep for all four state names co-located in one section; confirm no contradiction with the existing §4b/§4c prose (which stays factually intact per AC 9). |
| 5 | At least three Mermaid diagrams are present: (a) high-level human-gate/workflow/engine/records flow, (b) packet-building/coverage-state flow, (c) durable-record-ownership split. | `grep -c '```mermaid'` ≥ 3 in the file; read each diagram for correspondence to its named topic. |
| 6 | Any "ReviewRun" / control-plane-lite / event-ledger material is labeled "future direction, not implemented" (or equivalent unambiguous wording) and visually/textually separated from current-state sections. | Grep for "ReviewRun" / "control-plane" / "event ledger"; every hit must be inside a clearly labeled future-direction block, none inside a current-state section. |
| 7 | No changes to `CLAUDE.md` or `dba-system.md`. | `git diff --stat` for this change touches no path matching `CLAUDE.md` or `dba-system.md`. |
| 8 | No changes to `scripts/codeos-review.sh`, `tools/reviewer/*`, or any binary/code file. | `git diff --stat` for this change shows only `docs/reviewer-pipeline.md` (plus the already-landed Step 1 bookkeeping files). |
| 9 | Every existing normative statement in `docs/reviewer-pipeline.md` (I/O behavior, exit codes, fail-closed guarantees, CLI usage lines) is preserved — no factual claim is dropped or altered, only reframed/relocated. | Side-by-side diff read: for each normative sentence removed from its old location, confirm an equivalent sentence exists in the new structure. Spot-check at least the exit-code table and the `--mode delta --base <sha>` CLI usage line (both cited by prior change records, e.g. UPG-0027's delta-mode change). |
| 10 | No new standalone doc is created (e.g. no `docs/reviewer-architecture.md`). | `git status --short` for this change shows `docs/reviewer-pipeline.md` as modified, no new file under `docs/`. |
| 11 | The `status: PILOT` badge is either left as-is or, if changed, is changed only because it is factually stale — and if flagged as questionable rather than resolved, that flag is a one-line note, not a reclassification. | Read the badge line before/after; if changed, confirm the change record's Implementation Notes explain why, and that no new classification claim is asserted beyond "flagged." |
| 12 | Toolkit-wide consistency sweep: no stale cross-references to old section numbers (§4b/§4c/§14) left in other files after restructuring, and no orphaned links to the doc. | `grep -rn '§4b\|§4c\|§14' prompts/ changes/ docs/ backlog/ status/` — for each hit outside this change's own new content, confirm the referenced section still exists at that description or update the cross-reference in the same change. |

**Class note:** this is a `documentation` (normative) change, not `downstream-doctrine` or
`script-tooling` — no downstream-compatibility criteria and no I/O/exit-code contract apply to
the change itself (though AC 9 protects the *existing* exit-code documentation from being lost).

---

## Implementation Notes

Only `docs/reviewer-pipeline.md` was edited (116 lines added, 1 line net-split with no content
removed — confirmed by `git diff` showing a single `-` line whose text is fully preserved,
split across two new sentences). No section was renumbered; all new content was inserted as a
new `## 0.` section (before `## 1.`) or as lettered sub-sections following the doc's existing
`4a`/`4b`/`4c`/`4d` convention (`4e`, `4f`), or inline within existing `## 5.` / `## 14.`
without moving their prior prose.

Additions, mapped to acceptance criteria:
- **AC1/AC2** — new `## 0. Architecture at a Glance`: core rule statement, four-layer list,
  one Mermaid flowchart, and the "Two cadences, not one" paragraph cross-referencing `§4d`
  here and `dba-system.md`'s "Default Advisory Review" (via existing `§12`, not redefined).
- **AC3** — `## 14.`: added an "At a glance" use-when/main-risk table and a small Mermaid
  diagram immediately after the existing intro paragraph; the three per-mode subsections
  (Full/Delta/SHA-Only) and their CLI usage lines are untouched.
- **AC4** — `## 5.`: added the five-value `coverage_state` ordering (sourced from
  `reviewer-artifact-schemas.md`, not re-derived), an explicit `EMPTY_PACKET` fail-closed
  statement citing `UPG-0031`, and a Mermaid diagram from evidence mode → coverage state →
  floor. Existing normative prose in `§5` is unchanged below the insertion.
- **AC5** — 4 Mermaid diagrams total (`§0`, `§5`, `§4e`, `§14` "at a glance"), ≥ the required 3.
- **AC6** — new `## 4f. Future direction — not implemented`. Before writing this, searched the
  whole repo for `ReviewRun` / `control-plane` — zero hits outside this change's own new
  backlog/change files. Rather than inventing a plausible-sounding future architecture (which
  would itself become an unverified claim), the section states plainly that no such component
  exists and points at the one backlog item that actually covers this direction today
  (`UPG-0015-reviewer-decision-integrity.md`). This is narrower than the original scope
  language ("document the ReviewRun/control-plane-lite direction as future architecture")
  because there is no approved design to document — flagging this for the human/reviewer
  rather than silently either fabricating detail or dropping the AC.
- **AC7/AC8** — `git diff --stat -- CLAUDE.md dba-system.md` → empty; only
  `docs/reviewer-pipeline.md` changed besides the Step 1 bookkeeping files already covered by
  that step's own scope note.
- **AC9** — spot-checked: the exit-code table lives in `reviewer-artifact-schemas.md` (not
  touched), and the `--mode delta --base <sha>` CLI usage line in `§4b` and the `## 11. Usage`
  examples are byte-identical before/after.
- **AC10** — no new file created; `git status --short docs/` shows only the modified existing
  file.
- **AC11** — the `status: PILOT` badge (top-of-file yaml block) was **left as-is**. It does
  look stale given `§10`'s description of a compiled Rust engine and `§12`'s downstream
  integration, but reclassifying "pilot" vs. "shipped" is a judgment call outside this
  documentation-only change's scope — flagging it here rather than resolving it, per the
  Step 1 guardrail ("flag, don't resolve").
- **AC12** — deferred to Step 4 Reconcile per the workflow (the toolkit-wide consistency
  sweep is a Reconcile-stage task, not an Implement-stage one). A preliminary check already
  run in this step: `grep -rn '§4b\|§4c\|§14' prompts/ changes/ docs/ backlog/ status/` shows
  all hits outside `docs/reviewer-pipeline.md` itself are in already-COMPLETE historical
  change records describing when those sections were first created — none are broken by this
  change, since no section was renumbered. Full AC12 verification is recorded in Step 4.

**R1 fixes:** addressed three Codex findings from the Step 3 review — (1) the "Two cadences"
paragraph in `docs/reviewer-pipeline.md` §0 now names `prompts/codeos-self-dev.md` Step 0a
and the exact `UPG-0037` change record filename rather than only cross-referencing
§4d/§12/`dba-system.md`; (2) this Implementation Notes section no longer claims AC12 is fully
evidenced in a Reconciliation section that didn't exist yet — it now states AC12 is a Step 4
task with a preliminary check shown; (3) §4f's absence claim is narrowed from an unqualified
"does not exist anywhere" to "not found by a repo-wide search of tracked files," with the
exact grep command cited.

**R2 finding (budget exhausted at 2/2 rounds for PROFILE-2):** the R1-fix grep command in §4f
(`"ReviewRun\|control-plane"`) omitted `event ledger`, one of the three terms the sentence's
absence claim covers. **Fixed inline, not re-verified by a further automatic Codex round**
per the §4d budget-exceeded procedure: the command now reads
`grep -rn "ReviewRun\|control-plane\|event ledger" --include="*.md" .`; manually re-run,
confirms zero matches outside this change's own new files. Full round history:
`reviews/review-log.md` (series `RVS__UPG-0044__CHG-20260712-001__S3`).

No out-of-scope changes were introduced.

---

## Reconciliation

**Acceptance verification:**

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Layer-model framing near the top stating advisory/human-gate rule | PASS | `docs/reviewer-pipeline.md:28-33` — "Codex produces advisory evidence; the human gate decides" + four-layer list, inside new `## 0.` before `## 1.` |
| 2 | PROFILE-0..5 vs. R1/R2/R3 cadence named, cross-referencing Step 0a and UPG-0037 | PASS | `docs/reviewer-pipeline.md:59-62` — names `prompts/codeos-self-dev.md`'s Step 0a and `changes/UPG-0037__CHG-20260705-002__downstream-default-stage-review.md` by exact filename |
| 3 | full/delta/sha-only consolidated with use-when/risk table, not duplicated | PASS | New "At a glance" table + diagram inserted once, directly above the pre-existing `### Full Mode` / `### Delta Mode` / `### SHA-Only Mode` subsections, which are otherwise untouched |
| 4 | EMPTY_PACKET + delta-mode fix stated as one coverage-state model | PASS | `## 5.` now states all 5 real `coverage_state` values (`FULL_COVERAGE`, `PARTIAL_COVERAGE`, `SECRET_REDACTION`, `CRITICAL_OMISSION`, `EMPTY_PACKET`, sourced from `reviewer-artifact-schemas.md`) in one ordering + diagram, citing `UPG-0031` for the fail-closed/base-vs-working-tree fix |
| 5 | ≥3 Mermaid diagrams (high-level, packet/coverage, record-ownership) | PASS | `grep -c '```mermaid' docs/reviewer-pipeline.md` → 4 (§0, §5, §4e, §14) |
| 6 | ReviewRun/control-plane/event-ledger material labeled future-direction only | PASS (narrowed, see Implementation Notes) | New `## 4f.`; `grep -rn "ReviewRun\|control-plane\|event ledger" --include="*.md" .` → only matches are `docs/reviewer-pipeline.md` §4f itself (naming the terms to say they're absent) and this change's own `backlog/UPG-0044-*`, `changes/UPG-0044__*`, `reviews/*UPG-0044*` files |
| 7 | No `CLAUDE.md` / `dba-system.md` changes | PASS | `git diff --stat -- CLAUDE.md dba-system.md` → empty |
| 8 | No code/binary changes | PASS | `git status --short scripts/ tools/` → empty |
| 9 | Existing normative statements preserved | PASS | `docs/reviewer-artifact-schemas.md` (exit-code table) untouched (`git diff --stat` empty); `--mode delta --base <sha>` CLI line present verbatim at 4 locations; the one `git diff` `-` line's text is fully preserved, split across two sentences |
| 10 | No new standalone doc created | PASS | `git status --short docs/` → only `M docs/reviewer-pipeline.md` |
| 11 | `status: PILOT` badge left as-is or changed-with-reason | PASS | `git diff` shows no `+`/`-` touching the `status: PILOT` line; flagged as possibly stale in Implementation Notes, not resolved |
| 12 | Toolkit-wide sweep: no stale §-references, no orphaned links | PASS | Only active (non-historical) cross-references to specific sections are in `prompts/codeos-self-dev.md` (§4b/§4c/§4d — all three sections still exist, unrenumbered, at their prior locations); all other hits are frozen historical change/review records describing when those sections were first created, unaffected by this change; no `reviewer-pipeline.md#anchor` links exist anywhere to break |

**Consistency sweep (grep):** Clean. No section was renumbered — all new content added as `## 0.`
(before `## 1.`) or as `4e.`/`4f.` following the doc's existing `4a`/`4b`/`4c`/`4d` lettering
convention, or inserted inline within `## 5.` and `## 14.` without moving their prior prose.
No orphaned links found.

**Findings scope-triage:**

| Finding | Triage | Action |
|---|---|---|
| Step 1 R1: "no other files change" contradicted by `backlog/features.md` / `status/self-development.md` bookkeeping edits | IN-SCOPE BLOCKER | Fixed — scope wording narrowed to "implementation-wise," bookkeeping edits named explicitly |
| Step 1 R1: unsupported "verified" claim about `dba-system.md`/`CLAUDE.md` non-reference | IN-SCOPE BLOCKER | Fixed — exact grep command + result cited |
| Step 3 R1: cadence paragraph didn't name Step 0a / UPG-0037 by exact reference | IN-SCOPE BLOCKER | Fixed |
| Step 3 R1: AC12 claimed evidenced in a Reconciliation section that didn't exist yet | IN-SCOPE BLOCKER | Fixed — reworded to defer AC12 to Step 4 |
| Step 3 R1: §4f absence claim unsupported by shown evidence | IN-SCOPE BLOCKER | Fixed — grep command + scope cited |
| Step 3 R2: §4f cited grep omitted `event ledger` | IN-SCOPE BLOCKER (low severity) | Fixed inline post-budget-exhaustion; re-verified manually, not by a further automatic Codex round (PROFILE-2 budget: 2/2 rounds used at Step 3) |

All findings from all three review steps are resolved. No OUT-OF-SCOPE BACKLOG, REJECTED,
SELF-REFERENCE, or REVIEW-BOOKKEEPING findings arose in this change.

**Stack/dependency reconciliation:** Not applicable — this change touches no watched file
(`Cargo.toml`, `Cargo.lock`; see `status/stack-manifest.md`). No stack-reconciliation report
required.
