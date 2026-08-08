# Self-Development Change: UPG-0065__CHG-20260807-001 — normative-delta-inventory

<!--
PURPOSE: First change under UPG-0065 (Modular DBA Configuration Architecture). Produces the
mandatory Step-1 deliverable the brief itself commits to: a complete, per-rule disposition of
every normative rule in dba-system.md, before any component file is drafted or dba-system.md is
touched. This is Phase A's first sub-step (see backlog/UPG-0065's "Migration approach").
Workflow: prompts/codeos-self-dev.md (4-step loop).
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0065
primary_feature_id: UPG-0065
change_id: CHG-20260807-001
slug: normative-delta-inventory
state: COMPLETE         # DRAFT | IN_REVIEW | IN_PROGRESS | BLOCKED | COMPLETE | ABANDONED | SUPERSEDED
current_step: 4-Reconcile  # 1-Intent | 2-Acceptance | 3-Implement | 4-Reconcile
implements:
  - UPG-0065
related_features: []
review_series: RVS__UPG-0065__CHG-20260807-001__S4   # S1-S4 ACCEPTED — CHG COMPLETE
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

`backlog/UPG-0065-modular-dba-configuration-architecture.md` proposes decomposing `dba-system.md`
into independently versioned components, assembled only through an explicitly approved
configuration (`DBA-N`). The brief itself names the precondition for that decomposition: a
complete, per-rule disposition of every normative rule in `dba-system.md`, classified exactly one
of `KEEP-IN-CORE`, `MOVE` (naming its candidate owner, or `owner: UNRESOLVED` when none clearly
fits), `RETIRE`, or `INTENTIONAL-BEHAVIOR-CHANGE` (requiring a separate, explicit human decision).

The brief's own seed comparison — an initial, unsystematic read of `dba-system.md` against
`dba-system-lean.md` — already surfaced eight areas of disagreement that a first pass missed
(authority model, Stage 4 abstraction freedom, Stage 6 execution control, Stage 9 refinement
approval, architecture-governance mechanics, review-persistence requirements, Non-Negotiable Rule
#1's stage-transition scope, and independent-review cadence). An unsystematic comparison already
under-counted the real delta once. Decomposing `dba-system.md` into component files without a
complete inventory first risks silently dropping or altering a rule no one happened to compare.

**What changes:**

- `changes/UPG-0065__CHG-20260807-001__normative-delta-inventory.md` (this file) — the change
  record: Change Intent, Acceptance Criteria, Implementation Notes, Reconciliation.
- `changes/UPG-0065__CHG-20260807-001__delta-table.md` (new, created at Step 3) — the complete
  per-rule disposition table itself, kept as a separate evidence file because it is the durable
  analytical deliverable and should remain independently reviewable, not folded into the change
  record's own governance narrative.
- `backlog/UPG-0065-modular-dba-configuration-architecture.md` — `status` field and Feature Thread
  Changes/Reviews tables updated as this change progresses.
- `status/self-development.md` — dashboard row activated (Feature ID `UPG-0065`, Change ID
  `CHG-20260807-001`, State `IN_PROGRESS`, Loop step `1-Intent`).
- `backlog/features.md`, `status/roadmap.md` — status column updated from `PROPOSED` to
  `IN_PROGRESS`.

**Two independent axes, kept from collapsing into each other:** whether a rule's *meaning* changes,
and which *component* the rule ends up living in. The four dispositions must not silently force a
choice on one axis just because the other is decided. A rule may change meaning **and** move
components at the same time — the current mandatory-every-gate review rule becoming conditional
review, while also relocating from doctrine into a review-policy component, is exactly this case.
It is `INTENTIONAL-BEHAVIOR-CHANGE`, not `MOVE`, precisely because meaning is not preserved — but
it still needs to record *where* the changed rule will live, which `MOVE`'s definition alone can't
carry. Every non-`RETIRE` row therefore carries a `target_owner` field (schema below), and
disposition is chosen by this precedence, checked in order:

1. **Meaning is preserved** *and* the rule is genuinely redundant, duplicated by, or already
   superseded by another surviving rule, such that removing it changes no DBA semantics →
   `RETIRE`. No `target_owner` — the rule ceases to exist as a separate entry.
2. **Meaning changes at all** (an obligation, permission, gate, or authority is added, removed, or
   altered) → `INTENTIONAL-BEHAVIOR-CHANGE`, **regardless of whether the rule's component also
   changes.** This always takes precedence over `KEEP-IN-CORE`/`MOVE` once meaning changes — a
   rule is never coded as "just a move" when its semantics also shift. `target_owner` states where
   the *new* form will live: `doctrine`, a named non-doctrine component, or `UNRESOLVED`. The row
   states the current rule and its proposed new form in old-vs-new terms, and carries
   `requires_human_decision: yes` — never folded silently into `KEEP-IN-CORE` or `MOVE`.
   **Intentional deletion is a distinct sub-case of this disposition**, not `RETIRE` — `RETIRE` is
   reserved for zero-semantic-loss removal (the rule is redundant with a surviving rule); deleting
   an obligation with no successor is itself a meaning change. A row proposing deletion sets
   `proposed_rule: REMOVED` and `target_owner: NONE`, still carrying `requires_human_decision:
   yes` — the rule simply has no new form and nowhere to live, which is different from never
   having existed as a separate row (`RETIRE`) and different from relocating unchanged (`MOVE`).
3. **Meaning is preserved** and the rule stays in the doctrine kernel → `KEEP-IN-CORE`.
   `target_owner` is always `doctrine`.
4. **Meaning is preserved** and the rule relocates to a **non-doctrine** component → `MOVE`:
   `review policy`, `architecture-synthesis policy`, `implementation-profile policy`,
   `controlled-plain-english policy`, `reviewer tool contract`, or a new candidate component the
   inventory itself proposes (justified per Invariant 4: independent authority, independent
   lifecycle, a real reason to vary separately). **`doctrine` is never a valid `MOVE`
   `target_owner`** — a meaning-preserved rule staying in doctrine is `KEEP-IN-CORE`, not `MOVE`;
   this keeps the two mutually exclusive. `target_owner: UNRESOLVED` when no candidate clearly
   fits. The inventory is allowed to challenge the proposed decomposition itself: if the analysis
   finds two candidates should merge, one shouldn't exist, or a rule assumed to `MOVE` should
   actually be `KEEP-IN-CORE` instead, the row records that finding rather than forcing a boundary
   that doesn't fit. `target_owner: UNRESOLVED` rows must be resolved before any component
   drafting begins — this keeps Invariant 4 live during the inventory, rather than treating the
   brief's illustrative tree as already settled before the analysis has tested it.

Non-normative explanatory prose (e.g. rationale, examples) does not need a disposition at all —
see "Rule granularity" below for what counts as a normative rule in the first place.

**Inventory universe:**

Completeness is measured against every normative rule currently in `dba-system.md`, not against
the differences already found comparing it to `dba-system-lean.md`. The lean draft is comparison
evidence, not the inventory's scope boundary — a current rule the lean draft never mentions still
gets its own disposition. This is the failure this change exists to prevent: a rule dropped
silently because no one happened to compare it.

**Rule granularity** (to be pinned precisely as an acceptance criterion at Step 2, stated here so
Step 2 has a starting definition): granularity is semantic, not syntactic. The test is
independence, not sentence structure or the presence of "and"/"or": **a row is one rule if and
only if no part of it could change independently while the rest keeps the same meaning and
disposition.** "Intent, Contract, and Event Schema must be approved before Stage 4" is one atomic
gate condition, not three rows, because none of the three approvals is independently removable
without changing the gate itself — but a sentence combining an approval requirement with an
unrelated logging obligation is two rows, because either could change without touching the other.
A sentence bundling several independently changeable requirements produces one row per
requirement; a requirement artificially split across multiple rows is merged back into one.

**Delta-table row schema** — every row in `changes/UPG-0065__CHG-20260807-001__delta-table.md`
carries these fields, so one-to-one traceability is checkable by field, not by informal
recognition ("I recognize this row as corresponding to that paragraph"):

```text
rule_id             # stable id, e.g. TRUTH-AUTHORITY-2 (section-slug + sequence)
source_section      # the dba-system.md section heading the rule comes from
source_anchor       # pinned line range against commit 77599e9 PLUS a short quoted excerpt —
                     #   both together, not either alone: the line range gives deterministic
                     #   relocation, the excerpt gives human readability if lines drift later
current_rule        # the rule as it exists today, stated precisely
disposition         # KEEP-IN-CORE | MOVE | RETIRE | INTENTIONAL-BEHAVIOR-CHANGE
target_owner        # required for KEEP-IN-CORE (always "doctrine"), MOVE (a named non-doctrine
                     #   component or UNRESOLVED — never "doctrine"), and INTENTIONAL-BEHAVIOR-
                     #   CHANGE (doctrine, a named non-doctrine component, UNRESOLVED, or NONE —
                     #   NONE only when proposed_rule is REMOVED); blank for RETIRE
rationale           # why this disposition — what it duplicates (RETIRE), why this owner (MOVE
                     #   or INTENTIONAL-BEHAVIOR-CHANGE), etc.
```

An `INTENTIONAL-BEHAVIOR-CHANGE` row additionally carries:

```text
proposed_rule            # the rule's new form, or the literal value REMOVED for intentional
                          #   deletion with no successor (pairs with target_owner: NONE)
requires_human_decision  # literal value: yes
```

A row missing any required field for its disposition is incomplete and fails Reconcile.
`target_owner: NONE` is valid only when `proposed_rule: REMOVED`, and vice versa — the two fields
must agree; a row with one but not the other is invalid.

**Scope boundary — what stays the same:**

`dba-system.md` and `dba-system-lean.md` are read only in this change — neither is edited. No
file under a new `dba/` directory is created. No `configurations/*.yaml` is written. No component
file (`doctrine/v1.md`, `policies/*/v1.md`, etc.) is drafted. The manifest-path question and the
`patterns/`-vs-`policies/` directory question (both explicitly left open by the brief) are not
decided here. `DBA-1` and `DBA-2` are not approved or activated — this change produces analysis
only, per Invariant 1 in the brief (no `DBA-N` is pre-approved, including `DBA-1`).

**Class:** downstream-doctrine
**Scope axis:** downstream doctrine only
**Backlog item:** backlog/UPG-0065-modular-dba-configuration-architecture.md

---

## Acceptance Criteria

| # | Criterion | How it will be verified |
|---|---|---|
| 1 | **Completeness.** Every section of `dba-system.md` is explicitly marked reviewed; every normative rule found receives exactly one disposition row, conforming to the delta-table row schema in Change Intent, with a `source_anchor` that lets it be re-located. A section containing no normative rules is marked `NO NORMATIVE RULES` rather than left unaddressed. Individual explanatory sentences within a normative section do not each need their own row. | **Rule-level one-to-one mapping by schema field, not a count comparison or informal recognition**: for each section, independently re-derive its distinct normative rules (applying the granularity test fresh, without looking at the delta table first), then match each derived rule to exactly one row by `source_anchor` and `current_rule` meaning. A pass requires every derived rule matched to exactly one row (none unmatched) and every row's `source_anchor` traceable to exactly one derived rule (no invented or duplicate row); every section is confirmed either fully covered or explicitly `NO NORMATIVE RULES`. Equal row counts alone do not satisfy this. |
| 2 | **Granularity discipline.** No row bundles two or more independently changeable requirements; no single requirement is split across multiple rows, per the semantic independence test in Change Intent's "Rule granularity." | **Full read-through, not a sample, and semantic, not syntactic**: every row in the delta table is checked against the independence test at Reconcile — for each row, ask whether any part of it could change independently while the rest keeps the same meaning and disposition; if yes, split it; if a requirement is split across rows where no part is independently changeable, merge them. The presence of "and"/"or" in the source text is not itself evidence of multiple rules. |
| 3 | **`MOVE` rows are resolvable and valid.** Every `MOVE` row's `target_owner` is one of the five non-doctrine components (`review policy`, `architecture-synthesis policy`, `implementation-profile policy`, `controlled-plain-english policy`, `reviewer tool contract`), a justified new candidate component satisfying **all three** of Invariant 4's tests (independent authority, independent lifecycle, and a real reason to vary separately — not one or two of the three), or explicitly `UNRESOLVED`. `doctrine` is never a valid `MOVE` `target_owner` — meaning-preserving rules that stay in doctrine are `KEEP-IN-CORE` (precedence step 3), never `MOVE`. | Grep the delta table for `MOVE`; for each match, confirm `target_owner` is one of the five named components, `UNRESOLVED`, or a new component whose inline justification explicitly addresses all three Invariant 4 tests individually — reject a new-component owner whose justification addresses only one or two tests, and reject any other value, including `doctrine`, an empty field, or free text. Separately confirm every `KEEP-IN-CORE` row's `target_owner` is exactly `doctrine`. |
| 4 | **`RETIRE` rows justify no semantic change.** Every `RETIRE` row names the specific rule it duplicates or supersedes, and states why removal changes no DBA obligation, permission, gate, or authority. | Read every `RETIRE` row; reject and reclassify any that cannot name what it duplicates/supersedes as `INTENTIONAL-BEHAVIOR-CHANGE` instead. |
| 5 | **`INTENTIONAL-BEHAVIOR-CHANGE` rows are explicit, flagged, located, and deletion-capable.** Every such row states the current rule, carries `requires_human_decision: yes`, and has a valid `(proposed_rule, target_owner)` pair: either a stated new form paired with a `target_owner` that is `doctrine`, one of the five named non-doctrine components, a new component justified against **all three** Invariant 4 tests, or `UNRESOLVED` — or the literal pair `proposed_rule: REMOVED` / `target_owner: NONE` for intentional deletion with no successor. Never folded silently into `KEEP-IN-CORE` or `MOVE`; never forced into inventing a successor form when the actual intent is removal. | Grep the delta table for `INTENTIONAL-BEHAVIOR-CHANGE`; confirm each has an old-state statement, `requires_human_decision: yes`, and one of: (a) `target_owner` ∈ {`doctrine`, the five named components, `UNRESOLVED`} with `proposed_rule` stated, (b) a new-component `target_owner` whose inline justification addresses all three Invariant 4 tests individually, or (c) exactly `proposed_rule: REMOVED` with `target_owner: NONE` together. Reject any row where `target_owner: NONE` and `proposed_rule` is not `REMOVED`, or vice versa; reject a non-empty but otherwise unlisted `target_owner` value; reject a new-component owner whose justification addresses only one or two of the three tests. |
| 6 | **Known-deltas seed coverage.** Every one of the 8 areas in the brief's "Known normative deltas identified so far" table is represented by at least one row in the full inventory (not necessarily identical wording, but not silently dropped). | Cross-check each of the 8 seed-table areas (authority, Stage 4 abstractions, Stage 6 execution, Stage 9 refinement, architecture governance, review persistence, Non-Negotiable Rule #1, independent review) against the delta table. |
| 7 | **Downstream compatibility — source files untouched.** `dba-system.md` and `dba-system-lean.md` are never modified during this change; both stay exactly at their state in the pinned commit `77599e9` (both already clean — neither appears in `git status --short` as of Step 1 acceptance). | `git diff 77599e9 -- dba-system.md dba-system-lean.md` produces empty output at Reconcile — pinned explicitly to the named commit, not to `HEAD` (which could advance past `77599e9` for unrelated reasons during this change and mask a real divergence). `git status --short dba-system.md dba-system-lean.md` must also be empty. |
| 8 | **This change created or modified no out-of-scope artifact, tracked or untracked.** This change's own file list (Change Intent's "What changes") contains no path under `dba/`, `configurations/`, or matching a component pattern (`doctrine/v*.md`, `policies/*/v*.md`, `tools/*/v*.md`). Scoped to what *this change* did, not to the repository's global state — a forbidden-path file created by unrelated concurrent work is not this criterion's concern, but any such file found must be explicitly attributed, not silently waved away. | Two commands, both pinned to baseline `77599e9`: (1) `git diff 77599e9 --name-only -- dba/ configurations/ '**/v*.md'` for tracked changes; (2) `git status --porcelain --untracked-files=all -- dba/ configurations/ '**/v*.md'` for untracked new files, which (1) alone cannot see. If either shows a result, a human confirms at Reconcile whether this change created it (AC8 fails) or it's pre-existing/unrelated concurrent work (AC8 passes, but the file and its attribution are recorded in Reconciliation, not silently ignored). No formal commit- or workspace-binding mechanism is required for this attribution — a stated human judgment is sufficient. |
| 9 | **Cross-reference consistency.** The change record, the brief's Feature Thread, `status/self-development.md`, `backlog/features.md`, and `status/roadmap.md` agree on this change's current step and state, comparing only the fields each surface actually records (not every surface tracks every field). | Grep sweep for `UPG-0065` / `CHG-20260807-001` across all five files at Reconcile; no stale step/state claims (AJ-020). |

---

## Implementation Notes

Produced `changes/UPG-0065__CHG-20260807-001__delta-table.md`: a complete, section-by-section
disposition of every normative rule in `dba-system.md` @ commit `77599e9`, per the schema and
precedence established in Steps 1-2.

**Scale**: 115 normative rules found across all 19 sections (18 sections with content; `DBA
Vocabulary` correctly produced `NO NORMATIVE RULES` — a glossary of definitions, not obligations).
45 `KEEP-IN-CORE`, 42 `MOVE` (across the five non-doctrine components), 9 `RETIRE`, 19
`INTENTIONAL-BEHAVIOR-CHANGE`.

**Newly discovered deltas beyond the brief's 8-item seed table**: the systematic pass surfaced
11 more real deltas the seed comparison missed — Stage 7's reconciliation-table format (full
ALIGNED/GAP/MISMATCH/MISMATCH table vs. lean's problems-only report), Wave Gate batch approval
(current requires strictly individual per-feature approval; lean explicitly permits one batch
decision), the Architecture Synthesis Gate's versioning/history-file machinery (superseded to
git-history-only under lean), Structural-Only Changes' formal Stage-10 5-step structure (dropped
in lean's prose treatment), Review Logging's mandatory-by-default Decision Log (made conditional
under lean), and — the most significant single finding — a direct tension between NN-1's own
proposed batched Stage 4-8 execution and the standing "never add autonomous planning,
self-direction, or multi-step autonomous execution" prohibition (`NEVER-DO-7`), which lean neither
addresses nor explicitly repeals. This is exactly the kind of gap AC1/AC6 were designed to
surface, and confirms the brief's own warning that the original two-item framing understated the
real delta.

**Consolidation judgment calls, stated transparently in each row's rationale rather than hidden**:
a few tightly-coupled multi-clause paragraphs (the Review Waiver mechanism, Implementation
Profile's "Immutability and the transition path," the File Layout directory tree, the Artifact
Classification table) were kept as one or two rows rather than maximally atomized, on the same
"one coordinated mechanism" reasoning already accepted for `NN-2` (the single Intent+Contract+
Event-Schema gate). Duplicate/redundant restatements (`What You NEVER Do`'s five bullets that
restate Non-Negotiable Rules; `LOOP-SEQ-2`, `STEP4-ACTIVITY`, `STEP6-EVENTS`,
`ARTIFACT-CLASS-2`) were `RETIRE`d with the owning row named, per AC4.

**No file outside declared scope touched.** `dba-system.md` and `dba-system-lean.md` remain
byte-identical to commit `77599e9` (AC7). No `dba/` directory, `configurations/*.yaml`, or
component file exists, tracked or untracked (AC8). All 8 known-deltas seed-table areas are
represented (AC6). No `MOVE` row names `doctrine` as owner; every `KEEP-IN-CORE` row names
exactly `doctrine` (AC3 domain check). Every `INTENTIONAL-BEHAVIOR-CHANGE` row carries
`requires_human_decision: yes` and a stated `proposed_rule` (Part 2) — no row was left as an
unresolved tension without at least a stated candidate resolution or explicit both-options framing
(`NEVER-DO-7`). No deletion case (`target_owner: NONE`) was needed anywhere in this pass.

**Step 3 review (Codex R1) found three real defects, all fixed:**

1. **Missing `source_section` field (AC1) and missing quoted excerpts on several anchors (AC1).**
   Added `source_section` as an explicit seventh column to every one of the 131 rows via a scripted
   pass (verified: header/separator rows and all data rows updated consistently, no misalignment).
   Added a quoted excerpt to the 10 rows that previously described their anchor only by line range
   or table-row description.
2. **A `RETIRE` row lost independent semantics (AC4).** `STEP6-EVENTS` ("system emits events to
   `events/runtime_events.jsonl`") had been retired as a duplicate of `NEVER-DO-8` (the modify
   prohibition) — but `NEVER-DO-8` never states that events are emitted there in the first place.
   Corrected to `KEEP-IN-CORE`; the emission-destination fact now has a surviving row.
   `ARTIFACT-CLASS-2`'s Feature Brief classification had the same problem (no other row preserved
   it) — extracted into its own `KEEP-IN-CORE` row.
3. **Bundled independently-changeable rules (AC2).** `ARTIFACT-CLASS-3` (the four/five novel
   classifications), `CPE-2` (activation path + scaffold default + symlink reach + missing-file
   fallback), and `FILE-LAYOUT-1` (the entire directory tree) each combined facts that could
   change independently of each other. Split into 8, 4 (3 `MOVE` + 1 `RETIRE` duplicate of
   `OPT-MECH-1`), and 9 rows respectively (3 `KEEP-IN-CORE` + 6 `RETIRE`, each `RETIRE`d
   individually against its actual owning row elsewhere in the inventory, not as a block).
4. **`INTENTIONAL-BEHAVIOR-CHANGE` rows didn't carry the literal `requires_human_decision: yes`
   marker per row (AC5)**, and `NEVER-DO-7` proposed two candidate resolutions instead of one.
   Fixed: the marker is now inserted individually after every one of the 19 Part 2 entries
   (scripted, verified by count). `NEVER-DO-7` now states one proposed rule — bounded,
   checkpoint-delimited execution (what `NN-1`'s batching actually is) is not the kind of
   open-ended autonomous planning the prohibition bars — rather than presenting the human with an
   unresolved either/or.

**Net effect on row count**: 115 → 131 (`Artifact Classification` 3→8, `Controlled Plain English`
6→9, `File Layout` 1→9; `The 9-Step DBA Development Loop`'s `RETIRE` count 3→2 as `STEP6-EVENTS`
moved to `KEEP-IN-CORE`). All fixes re-verified against AC1, AC3, AC4, AC5's domain/uniqueness
checks after the corrections (rule_id uniqueness, no `MOVE`-to-`doctrine`, every `KEEP-IN-CORE`
targets exactly `doctrine`, every anchor has a quote, every IBC row has its marker) — see the
Bash verification run in this Step 3 round for the exact commands and their empty/zero output.

**Step 3 review round 2 (Codex R2) found three more real defects, all fixed:**

1. **`ARTIFACT-CLASS-3` and `FILE-LAYOUT-2`/`FILE-LAYOUT-3` still bundled independently-changeable
   facts (AC2), despite sharing one disposition.** A shared disposition does not license bundling
   — only genuine inseparability does. Fully atomized: `Artifact Classification` split from 8 to
   14 rows (one per table entry — Feature Brief through Onboarding artifacts, each independently
   changeable even where several happen to share `RETIRE`); `File Layout` split from 9 to 19 rows
   (`FILE-LAYOUT-2`'s `architecture/` subtree became 8 rows — one top-level framing fact plus 7
   individually-`RETIRE`d sub-paths; `FILE-LAYOUT-3`'s `intents/`/`contracts/`/`events/` block
   became 4 individually-`RETIRE`d rows).
2. **`STAGE-TABLE-4`'s `proposed_rule` presented an open sub-question instead of one complete
   form (AC5).** Fixed: the proposed rule now states definitively that it replaces the formal
   Stage-10 identity, the 5-step structure, and the `refinements/arch/` artifact convention;
   `requires_human_decision: yes` is the human's approve/reject of that one stated form, not a
   choice between alternatives.
3. **The `STEP6-EVENTS` fix from R1 was contradicted by `NEVER-DO-8`'s own rationale text**,
   which still said "see STEP6-EVENTS, retired as its duplicate" — a stale cross-reference left
   over from before the R1 fix. Corrected: both rows now state explicitly why they're distinct
   (emission destination vs. modify prohibition), neither references the other as a duplicate.

**Net effect on row count**: 131 → 147 (`Artifact Classification` 8→14, `File Layout` 9→19).
Re-verified after these fixes: no duplicate `rule_id` across all 147 rows; disposition/target-owner
domain checks still clean; all cross-references between rows checked for staleness given the
renumbering.

**AC7/AC8 verification evidence, embedded directly** (the reviewer is read-only and cannot run
commands itself; both prior rounds noted the commands were described but their output wasn't
shown):

```
$ git diff 77599e9 -- dba-system.md dba-system-lean.md
(no output)

$ git diff 77599e9 --name-only -- dba/ configurations/ '**/v*.md'
(no output)

$ git status --porcelain --untracked-files=all -- dba/ configurations/ '**/v*.md'
(no output)
```

All three commands produced empty output at the time of this Implementation Notes update,
confirming AC7 (source files byte-identical to the pinned baseline) and AC8 (no forbidden
tracked or untracked artifact) directly, not merely by description.

**Step 3 review round 3 (Codex R3) found three more real defects, all fixed. PROFILE-4's 3-round
budget is exhausted at this round; fixed inline per CLAUDE.md's budget-exceeded rule, no automatic
R4.**

1. **Four rows still bundled independently-changeable facts despite the R2 fix (AC2):**
   `FILE-LAYOUT-2g` (3 history paths for 2 different mechanisms), `FILE-LAYOUT-6` (2 review
   destinations), `FILE-LAYOUT-9` (2 test paths), and — the largest — `STAGE-TABLE-1` (the entire
   Stage/Stage-ID/File table and the entire Artifact/Template table, 31 independent mappings,
   bundled into one row from Step 3's very first draft). All fully split: `FILE-LAYOUT-2g` → 3
   rows, `FILE-LAYOUT-6` → 2 rows, `FILE-LAYOUT-9` → 2 rows, `STAGE-TABLE-1` → 31 rows
   (`STAGE-TABLE-1a`-`q` for the 17 Stage mappings, `STAGE-TABLE-1r`-`ae` for the 14 Artifact
   mappings).
2. **`REVIEW-LOG-1` bundled four independent facts under one disposition (AC2/AC5):** the
   preview-before-writing step, the Decision Log write-trigger (the actual `INTENTIONAL-BEHAVIOR-
   CHANGE`), Decision Rationale's own conditionality, and Architecture Journal's own
   conditionality. A human could reasonably approve the write-trigger change while rejecting or
   deferring the other three — the single row couldn't represent that. Split into
   `REVIEW-LOG-1a`-`d`; only `1b` carries the behavior change.
3. **The File Layout section's stated coverage count (16) contradicted its own row listing and
   the summary table (19) (AC1 — a durable, internal contradiction).** Fixed as part of the
   recount below; the section-coverage note is now generated from the same count used everywhere
   else, not restated by hand.

**Net effect on row count**: 147 → 184 (`File Layout` 19→23; `What You Do at Each Stage` 4→34;
`Review Logging` 7→10). Re-verified after these fixes: 184 unique `rule_id`s, zero duplicates;
every `source_anchor` has a quoted excerpt; 19 `INTENTIONAL-BEHAVIOR-CHANGE` rows in Part 1 match
19 `requires_human_decision: yes` markers in Part 2 exactly; AC3's owner-domain check (no `MOVE`
targets `doctrine`, every `KEEP-IN-CORE` targets exactly `doctrine`) still clean; AC7/AC8's file
commands re-run with empty output, confirmed above.

This is now the third consecutive review round finding real defects in the same underlying
principle (one row per independently-changeable fact) that R1 and R2 also found defects in —
worth naming plainly for the human gate decision: this reflects a systematic gap in my own
application of the granularity test during initial drafting, not three unrelated issues.

Full Reconcile-time verification against all 9 ACs is Step 4's job — the checks above are a
self-check during implementation, not a substitute for it.

---

## Reconciliation

**Acceptance verification:**

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Completeness — rule-level one-to-one mapping, every section covered or marked `NO NORMATIVE RULES` | PASS, by cumulative evidence — not a fresh Reconcile-time re-derivation | AC1's stated method is independent re-derivation from `dba-system.md` at Reconcile, without looking at the table first. This Reconcile pass did not redo that from scratch — the evidentiary basis is the mechanical checks (198 unique `rule_id`s, zero duplicates, verified by script; every `**Section coverage**` note kept in sync with the actual row listing) plus five accumulated review rounds (Step 3 R1-R3, Step 4 R1-R2), each of which did read `dba-system.md` against the table and found real gaps. `OPT-MECH-2c` was removed rather than relocated after Step 4 R2 found it stated a historical fact, not a normative rule. This is a real distinction, not a technicality: a full independent re-derivation could still find gaps these five rounds missed, precisely because — as the reviewer has repeatedly and correctly noted — its own packet never includes the full pinned source text. |
| 2 | Granularity — semantic independence, no bundling, no over-splitting | PASS, with a stated limit | Six review rounds found and fixed bundled rows: `ARTIFACT-CLASS-3`/`CPE-2`/`FILE-LAYOUT-1` (Step 3 R1); the same two re-bundled under a shared disposition (Step 3 R2); `FILE-LAYOUT-2g`/`6`/`9`/`STAGE-TABLE-1`/`REVIEW-LOG-1` (Step 3 R3); `ARCH-GATE-3`/`OPT-MECH-2`/`IMPL-PROFILE-9`/`1`/`4` (Step 4 R1); `IMPL-PROFILE-3`/`FILE-LAYOUT-5`/`HUMAN-NAV-2` and the `OPT-MECH-2c` non-normative-row removal (Step 4 R2); `REVIEW-5`/`ARCH-GATE-7`/`IMPL-PROFILE-6` (Step 4 R3). This recurrence across six rounds is named plainly: it reflects a systematic gap in applying the independence test consistently during drafting, not unrelated slips. **Stated limit, unchanged**: the reviewer's packet does not include the full pinned `dba-system.md` text, so it cannot fully independently re-derive completeness — this PASS rests on six rounds of adversarial review, not a formal proof that zero bundled rows remain anywhere in 203 rows. **PROFILE-4's 3-round budget is exhausted at Step 4 R3**; this round's fixes were applied inline per CLAUDE.md's budget-exceeded rule. |
| 3 | `MOVE`/`KEEP-IN-CORE` target-owner domain validity | PASS | `grep "\| MOVE \|" delta-table.md \| grep -c "\| doctrine \|"` → `0`. `KEEP-IN-CORE` rows checked for `target_owner` ≠ `doctrine` → `0` matches. Re-run at Reconcile, same result as during Step 3. |
| 4 | `RETIRE` rows justify zero semantic loss | PASS | Every `RETIRE` row's rationale names the specific owning row it duplicates, and each named target was checked to actually contain the claimed content — not merely assumed to, after Step 4 R3 found `ARTIFACT-CLASS-3`/`4`/`5` and `FILE-LAYOUT-2a`/`2c`/`2d` named real rows that didn't actually preserve the retired content (a scope qualifier; current-version file paths). All six corrected to `KEEP-IN-CORE`. Combined with `STEP6-EVENTS` and the Feature Brief classification from Step 3 R1, eight `RETIRE`-to-`KEEP-IN-CORE` corrections total across this change — each one a case where "duplicate" was asserted without verifying the target row's actual content, now fixed by checking, not assuming. |
| 5 | `INTENTIONAL-BEHAVIOR-CHANGE` rows explicit, flagged, located, deletion-capable | PASS | 19 rows in Part 1 match 19 `**RULE-ID**` entries and 19 literal `requires_human_decision: yes` markers in Part 2, verified by script. Every entry states exactly one `proposed_rule` (`STAGE-TABLE-4`'s two-option draft was resolved to one form in R2). No deletion case occurred in this pass, so `target_owner: NONE`/`proposed_rule: REMOVED` is untested by real data, but the schema and AC5's verification method both support it. |
| 6 | Known-deltas seed coverage (8 areas) | PASS | `TRUTH-AUTHORITY-2` (authority), `NN-3` (Stage 4 abstractions), `STEP6-ACTIVITY` (Stage 6 execution), `STEP9-GATE` (Stage 9 refinement), `ARCH-GATE-5`/`6`/`10` (architecture governance), `REVIEW-LOG-1b` (review persistence), `NN-1` (Non-Negotiable Rule #1), `REVIEW-1` (independent review) — all 8 present, each `grep`-confirmed present exactly once. |
| 7 | Source files untouched (pinned to `77599e9`) | PASS | `git diff 77599e9 -- dba-system.md dba-system-lean.md` → empty. `git status --short dba-system.md dba-system-lean.md` → empty. Both re-run at Reconcile, same empty result as embedded in Implementation Notes. |
| 8 | No out-of-scope artifact, tracked or untracked | PASS | `git diff 77599e9 --name-only -- dba/ configurations/ '**/v*.md'` → empty. `git status --porcelain --untracked-files=all -- dba/ configurations/ '**/v*.md'` → empty. No forbidden-path file exists to attribute. |
| 9 | Cross-reference consistency across the 5 tracking surfaces | PASS (after 1 fix) | At the start of Reconcile, the brief's Status line and the dashboard row still said "Step 3... awaiting human gate decision" after the human had already approved Step 3 — an AJ-020-class staleness recurrence, caught and fixed before writing this table, not after. All 5 surfaces now agree: change record `current_step: 4-Reconcile`; brief "Steps 1-3 ACCEPTED... at Step 4"; dashboard "4-Reconcile... S4 drafted"; `features.md`/`roadmap.md` both `IN_PROGRESS` (correctly — the feature UPG-0065 stays IN_PROGRESS regardless of this CHG's own completion, since Phase A's later sub-steps haven't started). |

**Consistency sweep (grep):** Cross-checked every `rule_id` referenced in another row's rationale
(e.g. `FILE-LAYOUT-7` → `ARTIFACT-CLASS-9`, `FILE-LAYOUT-2a`-`g` → their owning rows, `CPE-2d` →
`OPT-MECH-1`) resolves to a row that actually exists under that exact id — no dangling reference
found after the `ARTIFACT-CLASS` and `FILE-LAYOUT` renumbering in R2/R3. No orphaned links to
`dba-system.md`/`dba-system-lean.md` sections that don't exist (all `source_anchor` line ranges
independently re-checked against `git show 77599e9:dba-system.md` during R3's `STAGE-TABLE-1`
and `Artifact Classification` rewrites). No stage-table↔prompt-file drift applicable — this
change doesn't touch prompt files.

**Findings scope-triage:**

| Finding | Triage | Action |
|---|---|---|
| (S3 R1) Missing `source_section` field; unquoted anchors; `STEP6-EVENTS`/Feature-Brief lost semantics; 3 bundled rows; missing per-row `requires_human_decision` marker | IN-SCOPE BLOCKER (×5, all Codex) | Fixed — see Implementation Notes |
| (S3 R2) 2 rows re-bundled under a shared disposition; `STAGE-TABLE-4` two-option draft; stale `NEVER-DO-8` cross-reference | IN-SCOPE BLOCKER (×3, all Codex) | Fixed — see Implementation Notes |
| (S3 R3) 4 more bundled rows incl. `STAGE-TABLE-1`'s 31-mapping bundle; false File Layout coverage count | IN-SCOPE BLOCKER (×2, Codex; the count claim classified Low severity but still IN-SCOPE BLOCKER since it's a durable internal contradiction) | Fixed inline — PROFILE-4 budget exhausted, per CLAUDE.md's budget-exceeded rule |
| (S4, this Reconcile pass) Brief/dashboard cross-reference staleness (AC9) — Step 3's approval hadn't propagated before Reconcile began | IN-SCOPE BLOCKER (self-caught, not a Codex or human finding) | Fixed before writing the Reconciliation table |
| (S4 R1, Codex) `ARCH-GATE-3` and `OPT-MECH-2` each bundled 3 independently-changeable facts about different subjects under one disposition | IN-SCOPE BLOCKER (×1 High, ×1 Low — both Codex) | Fixed — split into `ARCH-GATE-3a-c` and `OPT-MECH-2a-c` |
| (S4 R1, Codex) File Layout's in-section coverage note still said 16 rules after the Step 3 R3 fix that changed the count to 23 — the Implementation Notes claim that this was "generated from one source" was itself false | IN-SCOPE BLOCKER (Low, Codex) | Fixed — note corrected to 23, with an explicit acknowledgment that the claim of single-source generation was inaccurate |
| (S4 R1, self-initiated) A semicolon-clause scan across all 184 pre-fix rows, run proactively given the four-round recurrence pattern, found `IMPL-PROFILE-1` and `IMPL-PROFILE-4` matched the same bundled-different-subjects pattern Codex had just named in `ARCH-GATE-3`/`OPT-MECH-2`, but Codex had not itself flagged them | IN-SCOPE BLOCKER (self-caught, not a Codex or human finding) | Fixed — split into `IMPL-PROFILE-1a-c` and `IMPL-PROFILE-4a-c` |
| (S4 R2, Codex) `IMPL-PROFILE-3`, `FILE-LAYOUT-5`, `HUMAN-NAV-2` each bundled facts about different subjects; `OPT-MECH-2c` ("CPE is the first adopter") was not a normative rule at all | IN-SCOPE BLOCKER (×4, all Codex) | Fixed — `IMPL-PROFILE-3` split into 3a-c, `FILE-LAYOUT-5` into 5a-c, `HUMAN-NAV-2` into 2a-c; `OPT-MECH-2c` removed (deletion, not relocation) |
| (S4 R2, Codex) Implementation Profile's in-section coverage note said 14 after the Step 4 R1 fix, while the actual row count was 15 | IN-SCOPE BLOCKER (Low, Codex) | Fixed — corrected to the actual count at each recount |
| (S4 R3, Codex) `ARTIFACT-CLASS-3`/`4`/`5` and `FILE-LAYOUT-2a`/`2c`/`2d` claimed to be `RETIRE`-duplicates of rows that did not actually preserve their content (a scope qualifier; current-version file paths) | IN-SCOPE BLOCKER (High, Codex — same content-loss class as `STEP6-EVENTS`) | Fixed — all six corrected to `KEEP-IN-CORE` |
| (S4 R3, Codex) `REVIEW-5`, `ARCH-GATE-7`, `IMPL-PROFILE-6` each bundled facts about different subjects | IN-SCOPE BLOCKER (×3, all Codex) | Fixed — split into `REVIEW-5a-d`, `ARCH-GATE-7a-b`, `IMPL-PROFILE-6a-b` |
| (S4 R3, Codex) Reconciliation's AC1 `PASS` claim rested on counts and review history, not the fresh independent re-derivation from source that AC1's own stated method requires | IN-SCOPE BLOCKER (Medium, Codex) | Fixed — AC1 reworded to state its actual evidentiary basis honestly (cumulative review rounds + mechanical checks) rather than implying a fresh re-derivation that didn't happen |

No `OUT-OF-SCOPE BACKLOG`, `REJECTED`, or `SELF-REFERENCE`/`REVIEW-BOOKKEEPING` findings this
Step — every finding across all rounds was a real defect in the artifact under review, not a
review-process artifact of reviewing itself.

---
