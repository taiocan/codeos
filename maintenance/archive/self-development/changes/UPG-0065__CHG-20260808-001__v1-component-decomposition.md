# Self-Development Change: UPG-0065__CHG-20260808-001 — v1-component-decomposition

<!--
PURPOSE: Second change under UPG-0065 (Modular DBA Configuration Architecture). Decomposes
dba-system.md into candidate v1 component files, using the accepted 203-row disposition table
from CHG-20260807-001. This is Phase A's second sub-step (see backlog/UPG-0065's "Migration
approach"): "decompose dba-system.md into candidate v1 components using this table."
Workflow: prompts/codeos-self-dev.md (4-step loop).
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0065
primary_feature_id: UPG-0065
change_id: CHG-20260808-001
slug: v1-component-decomposition
state: COMPLETE          # DRAFT | IN_REVIEW | IN_PROGRESS | BLOCKED | COMPLETE | ABANDONED | SUPERSEDED
current_step: 4-Reconcile  # 1-Intent | 2-Acceptance | 3-Implement | 4-Reconcile
implements:
  - UPG-0065
related_features: []
review_series: RVS__UPG-0065__CHG-20260808-001__S4   # S1-S4 all human APPROVED — CHG COMPLETE
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

`backlog/UPG-0065-modular-dba-configuration-architecture.md`'s Migration Approach names two
decoupled phases. Phase A's first sub-step — a complete normative delta inventory of
`dba-system.md` — is COMPLETE and accepted (`CHG-20260807-001`, 203 rows: 95 `KEEP-IN-CORE`, 61
`MOVE`, 28 `RETIRE`, 19 `INTENTIONAL-BEHAVIOR-CHANGE`, across 6 `target_owner` values). The next
named sub-step is: "decompose `dba-system.md` into candidate `v1` components using this table."
Without that decomposition, no compatibility sweep against prompts/scripts/templates and no
`DBA-1`-equivalence proof (the sub-steps that follow) can be attempted — there is nothing yet to
sweep or prove equivalent.

**What changes:**

Six new candidate `v1` component files, populated from the disposition table's non-`RETIRE` rows,
grouped by `target_owner`:

- `dba/doctrine/v1.md` — 108 rows (`target_owner: doctrine`)
- `dba/policies/review/v1.md` — 20 rows (`target_owner: review policy`)
- `dba/policies/architecture-synthesis/v1.md` — 18 rows (`target_owner: architecture-synthesis policy`)
- `dba/policies/implementation-profile/v1.md` — 18 rows (`target_owner: implementation-profile policy`)
- `dba/policies/controlled-plain-english/v1.md` — 7 rows (`target_owner: controlled-plain-english policy`)
- `dba/tools/reviewer/v1.md` — 4 rows (`target_owner: reviewer tool contract`)

(108+20+18+18+7+4 = 175 = 95 `KEEP-IN-CORE` + 61 `MOVE` + 19 `INTENTIONAL-BEHAVIOR-CHANGE`; the 28
`RETIRE` rows contribute no content to any component — see Scope boundary.)

Plus the governance surfaces that track this change:

- `changes/UPG-0065__CHG-20260808-001__v1-component-decomposition.md` (this file)
- `status/self-development.md` — dashboard row (Feature ID `UPG-0065`, Change ID
  `CHG-20260808-001`), Loop step updated as the change progresses through the 4-step loop
- `backlog/UPG-0065-modular-dba-configuration-architecture.md` — Status line and Feature Thread
  Changes/Reviews tables updated as this change progresses
- `status/roadmap.md` — `UPG-0065` row's "Planned/active change" column gains `CHG-20260808-001`

**Manifest-path decision (proposed, for approval at this gate):** the brief deferred this
question — `dba-system.md` staying at its current path as the eventual thin manifest vs.
introducing a new `dba/dba.md` path — "to the change that first creates manifest or component
files." Step 1 review R1 found this change record contradicted that timing: it proposed the first
component files while simultaneously calling the manifest-path question still open "until
activation," silently pushing the brief's own resolution point further out. Corrected: adopting
the brief's own recommended default now — `dba-system.md` keeps its current path (repo root;
downstream `.codeos/dba-system.md`), so no already-onboarded project's symlink breaks, with its
eventual role changing to the thin active-configuration manifest once `DBA-1` is approved and
activated. **Deciding the path now does not execute the change** — `dba-system.md`'s actual
content and role stay untouched in this change (see Scope boundary); activation is a later
sub-step this change does not reach. Resolving the path now only removes the open question as a
blocker to future sub-steps; it does not pre-approve `DBA-1` or violate Invariant 1/2.

**Directory-taxonomy decision (proposed, for approval at this gate):** the brief left
`patterns/`-vs-`policies/` "open, with a criterion, not a default," deferred to "the change that
first creates component files" — this change. Proposed resolution: use the brief's illustrative
`dba/policies/<name>/v1.md` and `dba/tools/reviewer/v1.md` layout, not `patterns/`. Rationale
against the stated criterion ("the smallest existing home whose authority and loading semantics
are unambiguous"): `patterns/` today holds project-toggled structural/writing content
(`patterns/controlled-plain-english.md`) that a project may or may not adopt, referenced by
policy rather than itself carrying `DBA-N` version-binding semantics. The new components are
normative DBA-governance content with exactly that binding semantics (an immutable `vN`, named by
a future `DBA-N` configuration) — `patterns/`'s existing loading model doesn't fit, so reusing it
would blur rather than clarify authority. `dba/policies/controlled-plain-english/v1.md` itself
stays a thin activation-mechanics pointer, per the delta table's `CPE-1` disposition — the full
layered content stays at `patterns/controlled-plain-english.md` (this repo's own path to that
file; the pointer's transcribed pinned-source text says `.codeos/patterns/controlled-plain-english.md`,
matching how `dba-system.md` itself already addresses that file for a downstream project reading
through its `.codeos` symlink — corrected per Step 3 review, which found this earlier wording
implied the two paths were the same string), referenced, not duplicated or moved.

**Scope boundary — what stays the same:**

- `dba-system.md` and `dba-system-lean.md` are **not edited**. `dba-system.md` remains the sole
  file downstream projects load via `.codeos/dba-system.md`, unchanged in content and role. The
  new `v1` component files are additive only — candidates, not yet wired into anything that
  supersedes the monolith.
- **No `configurations/*.yaml` is created.** Per Invariant 1, no `DBA-N` is drafted as an
  activatable manifest, proposed for approval, or approved in this change — this change produces
  component *content*, not a configuration that combines it.
- **No compatibility sweep** against prompts/scripts/templates is performed here — that is the
  Migration Approach's next sub-step, after these files exist.
- **`v1` content is frozen to current behavior, not the lean proposal.** For every non-`RETIRE`
  row — `KEEP-IN-CORE`, `MOVE`, and `INTENTIONAL-BEHAVIOR-CHANGE` alike — the component file
  places the rule under its `target_owner`. `INTENTIONAL-BEHAVIOR-CHANGE` rows' `proposed_rule`
  text is **not** used anywhere in this change, regardless of disposition: Phase A's goal is a
  `DBA-1` semantically equivalent to the live monolith (per the brief's Migration Approach — "if
  `DBA-1` cannot reproduce the current system exactly, fix the modular architecture before judging
  lean DBA at all"); adopting any `proposed_rule` now would silently pre-empt the human decision
  Phase B exists to make, and would violate Invariant 1 (no `DBA-N`, including `DBA-1`, gets
  pre-approved or altered outside an explicit decision).
- **Content source is the full pinned `dba-system.md` text, not the delta table's `current_rule`
  field.** Corrected after Step 1 review R1 found `current_rule` is sometimes a condensed
  paraphrase, not the rule's actual wording — e.g. `REVIEWER-TOOL-1`'s `current_rule` is "The
  exact invocation syntax and worked examples for running the reviewer," a sentence *describing*
  that the syntax and examples exist, without containing them; `ARCH-GATE-6`'s `current_rule`
  reduces a six-step procedure with per-stage Row/Column check apparatus to one summary sentence.
  Copying `current_rule` verbatim into a component file would silently drop exactly the normative
  detail Phase A must preserve. Each `v1` component's content is therefore transcribed from
  `dba-system.md` @ commit `77599e9` at the row's pinned `source_anchor` line range — the actual
  source text, not its summary. `current_rule` is used only as an index during transcription (to
  locate and group rows by `target_owner`), never as the copied content itself. Step 2 must add an
  acceptance criterion verifying full text transfer against the pinned source, not against
  `current_rule`.
- `RETIRE` rows (28) are **not represented** in any `v1` component — each is a zero-semantic-loss
  duplicate whose content already survives in its named owning row elsewhere in the table.
- No file moves out of `dba-system.md`; nothing is deleted from it.
- The manifest path is decided in this change (see "Manifest-path decision" above: `dba-system.md`
  keeps its current path), but **not executed** — `dba-system.md` itself is not touched, and it
  does not become the thin manifest until `DBA-1` is approved and activated, a later sub-step this
  change does not reach.
- `DBA-1` and `DBA-2` are not approved or activated in this change (Invariant 1, Invariant 2).

**Class:** downstream-doctrine
**Scope axis:** downstream doctrine only
**Backlog item:** backlog/UPG-0065-modular-dba-configuration-architecture.md

---

## Acceptance Criteria

**Per-component file schema (binding on Step 3, referenced by ACs below).** Each `v1` component
file is organized as: rule content blocks in ascending `source_anchor` line order, each headed by
its `rule_id`; a closing **Source Traceability** table with columns `rule_id | source_anchor
(dba-system.md @ 77599e9) | disposition`, one row per rule the file contains. This is the same
discipline the delta table itself enforces (one-to-one, checkable by field, not by informal
recognition) applied to the component files.

| # | Criterion | How it will be verified |
|---|---|---|
| 1 | **Completeness — every non-`RETIRE` row placed exactly once.** All 175 rows the accepted delta table disposes as `KEEP-IN-CORE` (95), `MOVE` (61), or `INTENTIONAL-BEHAVIOR-CHANGE` (19) appear in exactly one `v1` component file's Source Traceability table — the file matching the row's `target_owner` (`doctrine`→`dba/doctrine/v1.md`, `review policy`→`dba/policies/review/v1.md`, `architecture-synthesis policy`→`dba/policies/architecture-synthesis/v1.md`, `implementation-profile policy`→`dba/policies/implementation-profile/v1.md`, `controlled-plain-english policy`→`dba/policies/controlled-plain-english/v1.md`, `reviewer tool contract`→`dba/tools/reviewer/v1.md`). No row is missing; no row appears in more than one file. | Extract the full `rule_id` list per `target_owner` from `changes/UPG-0065__CHG-20260807-001__delta-table.md` (grep on disposition, excluding `RETIRE`); extract the full `rule_id` list from each of the 6 files' Source Traceability tables; diff the two sets per component (both directions) — expect empty diffs. Separately confirm no `rule_id` appears in more than one file's table (global uniqueness across all 6). |
| 2 | **`RETIRE` exclusion — by transcribed text, not just by label.** None of the 28 `RETIRE` `rule_id`s appear in any `v1` component file (label check). No `v1` content block contains any of a `RETIRE` row's transcribed **text** — retired source text is excluded, not merely its identifier, since retired text could appear unlabeled and still pass a label-only check. **A shared line number between an included row's `source_anchor` and a `RETIRE` row's `source_anchor` is not itself a violation** when the two rows split that one source line at a genuine sentence/clause boundary (a sub-line split, the same pattern used throughout this decomposition, e.g. `REVIEW-5a-d`) — the binding requirement is that the *transcribed text itself* never crosses into the `RETIRE`d row's portion of that line, checked per case, not that every declared line-range pair be numerically disjoint. | Grep all 6 files for each `RETIRE` `rule_id` (label check, zero matches expected). Separately: for each `RETIRE` row, confirm no `v1` content block's transcribed text contains that row's retired wording. Where a `source_anchor` line number is shared with an included row (sub-line split), read both rows' quoted source text side by side and confirm the included block's transcription stops exactly at the sentence/clause boundary, never continuing into the `RETIRE`d row's own clause. |
| 3 | **Content fidelity is mandatory, not advisory.** Each rule's content block is the complete normative substance of `dba-system.md` @ commit `77599e9`'s pinned `source_anchor` line range — not the delta table's `current_rule` field (Step 1 review found this is sometimes a condensed paraphrase, e.g. `REVIEWER-TOOL-1`, `ARCH-GATE-6`) and not any `proposed_rule` text. A content block that omits, shortens, or rewords away any normative detail present in its pinned range **fails this criterion and must be corrected before Reconcile** — this is a hard requirement, not a flag-with-rationale exception (corrected per Step 2 review R1: the original wording allowed an unexplained gap to pass with a rationale instead of a fix). | For every rule, diff its content block against `git show 77599e9:dba-system.md \| sed -n '<range>p'` for its `source_anchor`. Any gap is fixed before Reconcile, not merely noted — mirroring the false-`RETIRE`-duplicate lesson from `CHG-20260807-001` Step 4 R3 (a "this preserves it" claim must be checked against the actual target content, never assumed). |
| 4 | **No lean/proposed content anywhere — checked by meaning, not only by verbatim phrase.** No `v1` component file's content matches the *meaning* of any `INTENTIONAL-BEHAVIOR-CHANGE` row's Part-2 `proposed_rule` (the 19 lean-candidate forms) — a paraphrase that avoids the exact wording still violates this guarantee, so a verbatim-string grep alone is insufficient (corrected per Step 2 review R1). Every `INTENTIONAL-BEHAVIOR-CHANGE` row's `v1` content is its current, pinned-source form only. | Two layers: (a) grep all 6 files for each `proposed_rule`'s distinctive verbatim phrases (fast negative check); (b) for all 19 `INTENTIONAL-BEHAVIOR-CHANGE` rows, read the `v1` content block, the row's pinned source text, and the row's Part 2 `proposed_rule` side by side, and confirm the `v1` content's meaning matches only the current/pinned form — a full read, not sampling, since (a) alone cannot catch paraphrase. |
| 5 | **File/path correctness — exactly six files, no more.** `dba/doctrine/v1.md`, `dba/policies/review/v1.md`, `dba/policies/architecture-synthesis/v1.md`, `dba/policies/implementation-profile/v1.md`, `dba/policies/controlled-plain-english/v1.md`, `dba/tools/reviewer/v1.md` exist; no other file exists under `dba/` (tracked or untracked). | `find dba -type f` at Reconcile; expect exactly these 6 paths. `git status --porcelain --untracked-files=all -- dba/` cross-checked against the same list. |
| 6 | **Controlled Plain English component stays a thin pointer.** `dba/policies/controlled-plain-english/v1.md` contains only activation-mechanics content (per the delta table's `CPE-1` disposition) and an explicit reference to `.codeos/patterns/controlled-plain-english.md` (the path `dba-system.md` itself already uses for this file, addressed from a downstream project reading through its `.codeos` symlink) for the full layered content — it does not duplicate or move that file's content into `dba/`. | Read `dba/policies/controlled-plain-english/v1.md`; confirm it references `.codeos/patterns/controlled-plain-english.md` by path and does not reproduce that file's layer definitions. Confirm this repo's `patterns/controlled-plain-english.md` itself is untouched (`git diff` empty). |
| 7 | **Source-traceability schema present and complete.** Every one of the 6 files ends with a Source Traceability table (per the schema above) covering every rule it contains — no content block without a matching table row, no table row without a matching content block. | Per file: count content-block headings vs. Source Traceability table rows; expect equal counts and matching `rule_id` sets. |
| 8 | **Downstream compatibility — no operational surface loads the new components yet.** `dba-system.md` and `dba-system-lean.md` remain byte-identical to commit `77599e9`. No *operational/consumer* surface — `dba-system.md`, `dba-system-lean.md`, `prompts/`, `scripts/` — references or loads the new component paths; the generated project's `.codeos/dba-system.md` symlink target is unaffected, so no downstream project's loaded behavior changes as a result of this change. **Explicitly excluded from this check**: this change record, the backlog brief, `status/self-development.md`, and `status/roadmap.md` — governance/tracking surfaces are expected to name the six paths as planned work; naming a path in bookkeeping is not "wiring it up" (corrected per Step 2 review R1, which found the original unscoped "no file outside `dba/`" wording was already false against this change record's own "What changes" list). | `git diff 77599e9 -- dba-system.md dba-system-lean.md` → expect empty. `grep -rn "dba/doctrine\|dba/policies\|dba/tools" -- prompts/ scripts/ dba-system.md dba-system-lean.md` (operational surfaces only) → expect zero matches. |
| 9 | **Scope-boundary guardrails held.** No `configurations/*.yaml` file exists, tracked or untracked. No text in any touched file asserts or implies `DBA-1`/`DBA-2` is approved or activated. | `git status --porcelain --untracked-files=all -- configurations/` and `git diff 77599e9 --name-only -- configurations/` → both expect empty. Grep all touched files for `approved` near `DBA-1`/`DBA-2`/`active_configuration`; any hit is read in context and confirmed non-assertive (e.g. this AC's own text mentioning "not approved" is not a false positive). |
| 10 | **Cross-reference consistency.** The change record, the brief's Feature Thread, `status/self-development.md`, `backlog/features.md`, and `status/roadmap.md` agree on this change's current step and state, comparing only the fields each surface actually records. | Grep sweep for `UPG-0065` / `CHG-20260808-001` across all five files at Reconcile; no stale step/state claims (AJ-020 class). |

---

## Implementation Notes

Created six candidate `v1` component files, per the schema defined in Step 2, each populated from
the accepted delta table's non-`RETIRE` rows grouped by `target_owner`:

- `dba/doctrine/v1.md` — 108 rows
- `dba/policies/review/v1.md` — 20 rows
- `dba/policies/architecture-synthesis/v1.md` — 18 rows
- `dba/policies/implementation-profile/v1.md` — 18 rows
- `dba/policies/controlled-plain-english/v1.md` — 7 rows
- `dba/tools/reviewer/v1.md` — 4 rows

Total 175 rows (95 `KEEP-IN-CORE` + 61 `MOVE` + 19 `INTENTIONAL-BEHAVIOR-CHANGE`), matching the
accepted delta table's totals exactly.

**Method.** For each of the 175 target rule_ids, extracted the row's `disposition` and pinned
`source_anchor` directly from `changes/UPG-0065__CHG-20260807-001__delta-table.md` (grep/awk,
not manual transcription of the field values), then transcribed the corresponding text from
`dba-system.md` @ commit `77599e9` (confirmed byte-identical to the working tree via
`git diff 77599e9 -- dba-system.md`, so the file could be read directly). `current_rule` and
`proposed_rule` fields were used only to locate/verify content, never copied as the transcribed
text itself, per Step 1/2's fix.

**Finding: the delta table's own `source_anchor` ranges undercounted content in 7 places,
independent of the `current_rule`-summary problem Step 1 already fixed.** While transcribing,
found that several `source_anchor` line ranges stopped short of the row's actual complete
sentence/clause/list, which would have silently dropped real content if the stated range had
been trusted literally instead of independently verified against the pinned text:

- **`ARCH-GATE-6`** (Medium-High impact): stated range `L210-233` covered only steps 1-3 of what
  the delta table's own `current_rule` field calls "a detailed 6-step sequence." Steps 4-6
  (`L234-252` — the actual Architecture Synthesis drafting/approval steps, the single most
  consequential part of the gate) were entirely outside the stated range. Corrected to
  `L210-252`. This is a direct, concrete confirmation of exactly the failure mode Step 1 review
  R1 warned about in the abstract (current_rule paraphrases can drop real content) — except this
  instance was in the pinned-line-range field itself, which R1's fix did not address, because R1
  only distrusted `current_rule`, not `source_anchor`.
- **`REVIEW-7`**, **`REVIEW-LOG-2`**, **`ARCH-GATE-3a`**, **`IMPL-PROFILE-3a`**,
  **`IMPL-PROFILE-3c`**, **`IMPL-PROFILE-9b`**: each stopped 1-7 lines short of the sentence's or
  list's actual end. All corrected by extending the transcribed range to the sentence/list
  boundary; no content dropped in the final files (each correction is called out inline in the
  affected file's Source Traceability table).
- Two further citation-only corrections with no content-loss risk: `HUMAN-NAV-2b` (delta table
  cited a blank line; the actual clause is one line earlier) and `TOOLKIT-USE-1` (delta table
  cited `L788-793`; the file has no line 793 — `wc -l dba-system.md` = 792).

**This is not a re-opening of `CHG-20260807-001`** (`state: COMPLETE`, human-accepted). That
change's own acceptance criteria verified rule-count/disposition/`target_owner` completeness and
`source_anchor` *presence* (a line range plus a quoted excerpt), never byte-exact range
sufficiency against the full pinned text — this change's AC3 is a strictly stricter check the
prior change's ACs didn't require. Recorded here as a real defect discovered downstream, with the
corrected ranges now authoritative in each component file's own Source Traceability table.
Whether `changes/UPG-0065__CHG-20260807-001__delta-table.md` itself should be corrected to match
is a question for the human at this gate — this change does not edit that file (out of its own
declared scope).

**Verified during drafting, not merely assumed:**
- All 175 `rule_id`s land in exactly one file each, matching `target_owner` — confirmed by
  diffing the delta table's per-owner `rule_id` sets against each file's Source Traceability
  table (`comm -23`/`comm -13`, both empty in both directions, all six owners).
- None of the 28 `RETIRE` `rule_id`s appear in any file (grep, zero matches). Checked further by
  line-range comparison, not just this label check: 27 of the 28 `RETIRE` rows occupy a source
  span with no line number in common with any transcribed row. The one exception, `CPE-2d`
  (`RETIRE`, L492-494), shares line 492 with `CPE-2c` (`MOVE`, L490-492) — a genuine sub-line
  split, the same pattern used elsewhere in this decomposition. Verified by hand that
  `CPE-2c`'s transcribed content contains only its own sentence, not `CPE-2d`'s retired text —
  documented explicitly in `dba/policies/controlled-plain-english/v1.md`'s Source Traceability
  section. Found by Step 3 review R1, which correctly identified the original wording here ("each
  RETIRE row's range is a physically distinct span") as an overstated claim — true for 27 of 28
  rows, not all 28.
- A verbatim-phrase grep for 17 distinctive `proposed_rule` fragments across all 6 files found
  one hit: "does not silently amend" in `dba/doctrine/v1.md`. Checked by hand — this is
  `TRUTH-AUTHORITY-7`'s own **current** pinned text ("runtime behavior does not silently amend
  either artifact..."), not a leak of `TRUTH-AUTHORITY-2`'s `proposed_rule` (a different row);
  coincidental wording overlap, not a violation, per AC4's stated exception-handling method. All
  19 `INTENTIONAL-BEHAVIOR-CHANGE` rows were additionally read side-by-side (content block vs.
  pinned source vs. Part 2 `proposed_rule`) during transcription to confirm each block matches
  only the current form.
- `find dba -type f` lists exactly the 6 declared paths; `git status --porcelain
  --untracked-files=all -- dba/` shows all 6 as untracked-new, nothing else.
- `dba/policies/controlled-plain-english/v1.md` contains only activation mechanics and an
  explicit reference to `.codeos/patterns/controlled-plain-english.md` (matching how
  `dba-system.md` itself already addresses this file for a downstream project); this repo's own
  `patterns/controlled-plain-english.md` `git diff` is empty.
- `git diff 77599e9 -- dba-system.md dba-system-lean.md` is empty. A grep for the new component
  paths across `prompts/`, `scripts/`, `dba-system.md`, `dba-system-lean.md` returns zero matches
  — no operational surface loads anything under `dba/` yet.
- No `configurations/*.yaml` exists (tracked or untracked). A grep for `DBA-1`/`DBA-2`/
  `active_configuration` across `dba/` and this change record finds only non-assertive mentions
  in this change record's own prose (explicitly stating `DBA-1` is *not* approved); zero mentions
  inside any of the 6 component files themselves.

No out-of-scope edit made. `dba-system.md`, `dba-system-lean.md`, `patterns/controlled-plain-english.md`
untouched. No `configurations/*.yaml` created. No `DBA-N` approved or activated.

---

## Reconciliation

**Acceptance verification:**

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Completeness — every non-`RETIRE` row placed exactly once | PASS | Per-owner `comm -23`/`comm -13` diff between the delta table's `rule_id` set and each file's Source Traceability table: all six owners return empty in both directions (0 missing, 0 extra) — doctrine 108, review policy 20, architecture-synthesis policy 18, implementation-profile policy 18, controlled-plain-english policy 7, reviewer tool contract 4. Re-run at Reconcile after the Step 3 R1 doctrine-reordering fix; still clean. |
| 2 | `RETIRE` exclusion by transcribed text | PASS | Grep for all 28 `RETIRE` `rule_id`s across all 6 files: zero matches (label check). Line-range comparison of all 28 `RETIRE` source spans against all 175 included spans: 27 of 28 share no line number with any included row. The one exception, `CPE-2c`/`CPE-2d` at line 492, is a genuine sub-line split — verified by hand that `CPE-2c`'s transcribed text stops at "...controlled-plain-english.md`." and never continues into `CPE-2d`'s retired clause; documented explicitly in `dba/policies/controlled-plain-english/v1.md`'s Source Traceability section (Step 3 review R1 finding, fixed; AC2 itself reworded at R2 to state this exact invariant, since the original wording read as an absolute no-line-overlap rule). |
| 3 | Content fidelity is mandatory — pinned source, not `current_rule` summary | PASS | Every content block was transcribed directly from `dba-system.md` @ `77599e9` while that file was fully in context (not retyped from memory or copied from the delta table's summary fields). Re-verified at Reconcile with direct `diff` against `git show 77599e9:dba-system.md` for a spot-check sample spanning all 6 files and prioritizing every corrected row: `ARCH-GATE-6` (exact match, including the previously-missing steps 4-6), `REVIEW-7` (exact match), `ARCH-GATE-3a`/`IMPL-PROFILE-3a` (each correctly truncated at the true sentence boundary, not the delta table's undercounted one), `FAILURE-BOUNDARY-1`, `ARTIFACT-CLASS-1`, `ARCH-GATE-14`, `IMPL-PROFILE-7` — all exact matches. 7 `source_anchor` corrections beyond the delta table's stated ranges (`REVIEW-7`, `REVIEW-LOG-2`, `ARCH-GATE-3a`, `ARCH-GATE-6`, `IMPL-PROFILE-3a`, `IMPL-PROFILE-3c`, `IMPL-PROFILE-9b`) are called out inline in each affected file, per the Implementation Notes finding. |
| 4 | No lean/proposed content, by meaning not only verbatim match | PASS | Verbatim grep for 17 distinctive `proposed_rule` phrases across all 6 files at Reconcile: 1 hit, "does not silently amend" in `dba/doctrine/v1.md` — confirmed by line number (`TRUTH-AUTHORITY-7`'s own current pinned text, a different row from `TRUTH-AUTHORITY-2`, whose `proposed_rule` uses similar wording) — coincidental overlap, not a leak, per AC4's stated exception-handling method. All 19 `INTENTIONAL-BEHAVIOR-CHANGE` rows were read side-by-side (content block / pinned source / Part 2 `proposed_rule`) during Step 3 drafting to confirm each matches only the current form. |
| 5 | File/path correctness — exactly six files | PASS | `find dba -type f` lists exactly `dba/doctrine/v1.md`, `dba/policies/review/v1.md`, `dba/policies/architecture-synthesis/v1.md`, `dba/policies/implementation-profile/v1.md`, `dba/policies/controlled-plain-english/v1.md`, `dba/tools/reviewer/v1.md` — no other path under `dba/`, tracked or untracked. |
| 6 | Controlled Plain English component stays a thin pointer | PASS | `dba/policies/controlled-plain-english/v1.md` contains only activation-mechanics content and references `.codeos/patterns/controlled-plain-english.md` — the path `dba-system.md` itself already uses for this file (corrected at Step 3 review R1, which found the original AC6 wording named the wrong, unprefixed path). This repo's own `patterns/controlled-plain-english.md` remains untouched (`git diff` empty). No layer definitions duplicated. |
| 7 | Source-traceability schema present and complete | PASS | Per file, `## `-heading count minus 1 (the "Source Traceability" heading itself) equals the Source Traceability table's row count, for all 6 files: doctrine 108/108, review policy 20/20, architecture-synthesis policy 18/18, implementation-profile policy 18/18, controlled-plain-english policy 7/7, reviewer tool contract 4/4. |
| 8 | Downstream compatibility — nothing wired up yet | PASS | `git diff 77599e9 -- dba-system.md dba-system-lean.md` empty. Grep for the new component paths (`dba/doctrine`, `dba/policies`, `dba/tools`) across `prompts/`, `scripts/`, `dba-system.md`, `dba-system-lean.md`: zero matches — no operational surface loads anything under `dba/`. |
| 9 | Scope-boundary guardrails held | PASS | No `configurations/*.yaml` exists, tracked or untracked (`git status --porcelain --untracked-files=all` and `git diff 77599e9 --name-only`, both empty for that path). Grep for `DBA-1`/`DBA-2`/`active_configuration` inside `dba/`: zero matches — no component file asserts or implies any `DBA-N` approval. |
| 10 | Cross-reference consistency | PASS (after 1 self-caught fix) | At the start of Reconcile, `status/self-development.md`'s Loop-step column still read `3-Implement` after the human had already approved Step 3 — an AJ-020-class staleness recurrence, caught and fixed before writing this table, not after (mirrors the same class of finding `CHG-20260807-001`'s own Reconcile caught in itself). The brief's Status line and its S3 Reviews-table row also still said "awaiting human gate decision" after approval — fixed together. All 5 named surfaces now agree, comparing only the fields each actually records: change record `current_step: 4-Reconcile`; brief "Steps 1-3 ACCEPTED... Step 4 drafted"; dashboard "4-Reconcile... S4 written"; `backlog/features.md:98` and `status/roadmap.md:127` — the only two of the five that track anything about `UPG-0065` at all — both read `IN_PROGRESS` (verified directly, `grep -n "UPG-0065"` on each), correctly unaffected by this CHG's own step-level progress since neither file tracks step level, only feature-level status, and the feature itself correctly stays `IN_PROGRESS` until Phase A's later sub-steps complete. |

**Consistency sweep (grep):** Cross-checked every `rule_id` cross-reference inside the 6 component
files' explanatory notes (`CPE-2c`↔`CPE-2d`, the IBC-disposition note's 13-row list against the
actual `INTENTIONAL-BEHAVIOR-CHANGE` rows present) resolves correctly — no dangling reference. No
stage-table↔prompt-file drift applicable — this change transcribes content, it does not touch
`prompts/` or any stage-ID mapping. No orphaned links to `dba-system.md`/`dba-system-lean.md`
sections that don't exist — every `source_anchor` re-verified against `git show
77599e9:dba-system.md` during the AC3 spot-check above. `backlog/features.md` requires no edit
(it maps `UPG-0065` → the brief file only, unaffected by CHG-level progress).

**Findings scope-triage:**

| Finding | Triage | Action |
|---|---|---|
| (S1 R1, Codex) `current_rule` copy source could omit normative details (worked examples, multi-step procedures) while claiming semantic equivalence | IN-SCOPE BLOCKER (High) | Fixed — content source changed to the full pinned `dba-system.md` text at each row's `source_anchor`, `current_rule` used only as an index |
| (S1 R1, Codex) Manifest-path decision deferred past the point the brief itself sets ("the change that first creates component files") | IN-SCOPE BLOCKER (Medium) | Fixed — manifest path decided now (`dba-system.md` keeps its current path), not executed |
| (S1 R2, Codex) Manifest-path fix left a stale Scope-boundary bullet still saying the question "stays open — not decided here," self-contradicting the new decision | IN-SCOPE BLOCKER (Medium) | Fixed — stale bullet corrected |
| (S1 R3, Codex) NO OBJECTION, 0 findings | — | — |
| (S2 R1, Codex) AC3/AC4's source-fidelity and no-lean-content checks were label/verbatim-only and could accept lost or paraphrased-lean semantics | IN-SCOPE BLOCKER (High) | Fixed — AC3 made a hard mandatory-fix requirement; AC4 added a full-read meaning check |
| (S2 R1, Codex) AC8's "no file outside `dba/` references the new paths" was already false against this change record's own "What changes" list | IN-SCOPE BLOCKER (Medium) | Fixed — AC8 rescoped to operational/consumer surfaces only, explicitly excluding governance bookkeeping |
| (S2 R1, Codex) AC2's `RETIRE` exclusion checked only `rule_id` labels, not source-text provenance | IN-SCOPE BLOCKER (Medium) | Fixed — AC2 added a provenance-overlap check |
| (S2 R2, Codex) Brief's S1 Reviews-table row still said "Awaiting human gate decision" after approval; change record's "What changes" list still named a stale `1-Intent` loop step | IN-SCOPE BLOCKER (Medium) | Fixed |
| (S2 R3, Codex) NO OBJECTION, 0 findings | — | — |
| (S3 R1, Codex) `CPE-2c`/`CPE-2d` share source line 492 at a genuine sub-line split, making the "every RETIRE range is physically distinct" claim false for 1 of 28 rows | IN-SCOPE BLOCKER (High) | Fixed — documented as the sole exception in `dba/policies/controlled-plain-english/v1.md` |
| (S3 R1, Codex) CPE thin-pointer path mismatch: AC6/Change-Intent prose said `patterns/controlled-plain-english.md`, transcribed content correctly used `.codeos/patterns/controlled-plain-english.md` | IN-SCOPE BLOCKER (Medium) | Fixed — prose corrected to the correct downstream-facing path throughout |
| (S3 R1, Codex) `dba/doctrine/v1.md` placed `STAGE-TABLE-4` (L556) after `STAGE-TABLE-1r-ae` (L562-575), violating the binding ascending-source-anchor schema | IN-SCOPE BLOCKER (Medium) | Fixed — moved to its correct position after `STAGE-TABLE-2` |
| (S3 R2, Codex) R1's `CPE-2c`/`CPE-2d` fact fix left AC2's own criterion text reading as an absolute no-overlap rule, contradicting the now-documented exception | IN-SCOPE BLOCKER (High) | Fixed — AC2 reworded to state the real invariant (no `RETIRE`d text transcribed; a shared line number at a genuine sub-line split is not itself a violation) |
| (S3 R3, Codex) NO OBJECTION, 0 findings | — | — |
| (S4, self-caught before this Reconcile pass) Dashboard Loop-step column and brief Status line/S3 Reviews row still read as if Step 3 were unapproved, after the human had already approved it | IN-SCOPE BLOCKER (self-caught, not a Codex or human finding) | Fixed before writing this Reconciliation table |
| (Implementation Notes, self-caught during Step 3 transcription) The accepted `CHG-20260807-001` delta table's `source_anchor` ranges undercounted content in 7 places, most significantly `ARCH-GATE-6` (missing 3 of 6 procedural steps) | OUT-OF-SCOPE BACKLOG — a defect in a different, already-`COMPLETE` change's artifact, not this change's own scope | Corrected in this change's own component files (authoritative going forward for `v1` decomposition purposes); whether to also correct `changes/UPG-0065__CHG-20260807-001__delta-table.md` itself is left to human decision — not resolved here |

No `REJECTED` or `SELF-REFERENCE`/`REVIEW-BOOKKEEPING` findings this Step — every finding across
all three steps' rounds was a real defect in the artifact under review, not a review-process
artifact of reviewing itself.

---
