# Self-Development Change: UPG-0050__CHG-20260716-001 — downstream-feature-id-scheme

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0050
primary_feature_id: UPG-0050
change_id: CHG-20260716-001
slug: downstream-feature-id-scheme
state: COMPLETE         # DRAFT | IN_REVIEW | IN_PROGRESS | BLOCKED | COMPLETE | ABANDONED | SUPERSEDED
current_step: 4-Reconcile  # 1-Intent | 2-Acceptance | 3-Implement | 4-Reconcile
implements:
  - UPG-0050
related_features: [UPG-0001, UPG-0041]
review_series: RVS__UPG-0050__CHG-20260716-001__S4     # ALL Step-4 reviews for this change (stable); Steps 1-3 series S1-S3 ACCEPTED — see review-log.md
review_profile: PROFILE-4  # downstream-doctrine
review_state: ACCEPTED  # DRAFT | IN_REVIEW | REVIEWED | ACCEPTED  (operational; NOT a round)
review_history: reviews/review-log.md   # exact per-round REV__…__R<N> verdicts + human decisions live here, never in this artifact
fixes_findings: []
follow_up_of: null
```

<!-- SELF-REFERENCE BOUNDARY: this artifact is itself reviewed, so it must NOT embed the current
review round (which does not exist until after the packet is built). Reference the stable review
SERIES (review_series) + review_state; exact rounds live only in reviews/review-log.md and
reviews/codex/*. See prompts/codeos-self-dev.md → "Feature Thread & IDs" / "Self-Reference Boundary". -->


## Change Intent

**Why (problem in the toolkit):**
The DBA Feature Brief step (`prompts/00b-feature-brief.md`, Stage `brief`) saves briefs to
`backlog/[feature_id]-[name].md`, but `[feature_id]` has never had a defined format —
`templates/conventions.md`'s "Feature IDs" section only specifies `lowercase_underscore`
(e.g. `add_item_to_cart`), a free-text slug with no counter, no sequencing, and no guaranteed
uniqueness. Separately, `templates/feature-registry.yaml` already declares a structured
`feature_id:` field whose example value is `UPG-0000` — it silently borrowed the
**self-dev-only** `UPG-####` scheme (reserved for this toolkit repo's own backlog, per
`backlog/features.md:87`) without a formalized downstream equivalent ever being defined. Two
downstream-facing artifacts currently imply two different, incompatible feature-id
conventions, and neither is actually specified as a durable identity scheme.

**What changes:**
- `templates/conventions.md` — replace the "Feature IDs: `lowercase_underscore`" section with
  the `F-####` format (4-digit, zero-padded, sequential, permanent, never reused) and an
  id/slug split explanation with example filenames.
- `templates/feature-registry.yaml` — fix the `feature_id: UPG-0000` example (line 39) to
  `feature_id: F-0001`; update the inline comment to point at `conventions.md`'s Feature IDs
  section instead of implying the self-dev scheme.
- `prompts/00b-feature-brief.md` — add explicit id-assignment instructions to the Synthesis
  step (Step 1 of "Synthesis and Completion Check"): scan `features/registry.yaml` (if
  present) + `backlog/F-####-*.md` filenames for the current max, assign next for F-type,
  reuse the parent's id for R-type (no new id minted for refinements). Update the "Brief
  Lifecycle" and "Output Format" sections' `backlog/[feature_id]-[name].md` references to show
  a concrete `F-####` example.
- `templates/feature-brief.md` — update the H1 (`# Feature Brief: [feature_id] — [short
  name]`) and the `**Refines**` field guidance to reference the `F-####` format; add a short
  inline comment noting the id is assigned at Synthesis, not chosen freely by the human.
- `dba-system.md` — File Layout block, Artifact Classification table, and Stage table entries
  that reference `[feature_id]` get one clarifying line each pointing at the `F-####` format
  now defined in `conventions.md`. No change to stage names, gate rules, or 9-stage substance.

**Scope boundary — what stays the same:**
- `scripts/dba-init.sh` — no code change (it already copies `templates/feature-registry.yaml`
  verbatim; the corrected example flows through automatically).
- Feature Brief and Feature Registry remain **Optional** artifacts per `dba-system.md`'s
  Artifact Classification table — this change does not make either required.
- No downstream equivalent of the self-dev `CHG-YYYYMMDD-NNN` change-id layer is introduced —
  this only formalizes the feature-identity layer (`F-####`), not a change-execution layer.
- No retroactive rename of any existing downstream project's already-assigned slug-style
  `feature_id`s — this is a forward-looking convention, not a migration mandate.
- No change to the self-dev `UPG-####` scheme itself or to `backlog/features.md`'s own
  numbering/sequencing logic. This change *does* touch two self-dev bookkeeping files as
  required by `prompts/codeos-self-dev.md`'s "Feature Thread first" rule: one new row is
  added to `backlog/features.md`'s Feature-ID Map (registering this change's own `UPG-0050`)
  and one row is activated in `status/self-development.md` (this change's own dashboard row).
  These are the standard per-change bookkeeping edits every self-dev change makes to register
  itself, not a substantive change to self-dev doctrine, and are distinct from the five
  downstream-doctrine files under "What changes" above.
- The 9-stage loop's stage names, gates, `[feature_id]` placeholder *positions*, and file-layout
  *structure* are unchanged — only the definition of what `[feature_id]` itself must look like.

**Class:** downstream-doctrine
**Scope axis:** downstream doctrine only
**Backlog item:** backlog/UPG-0050-downstream-feature-id-scheme.md

---

## Acceptance Criteria

<!-- The consistency contracts this change must satisfy. Each must be checkable in Reconcile. -->

| # | Criterion | How it will be verified |
|---|---|---|
| 1 | `templates/conventions.md`'s "Feature IDs" section documents the `F-####` format (4-digit, zero-padded, sequential, permanent, never reused) and the id/slug split, replacing the `lowercase_underscore`-only spec. | Read-through; `grep -n "F-####" templates/conventions.md` hits; `grep -n "lowercase_underscore" templates/conventions.md` no longer describes Feature IDs as slug-only. |
| 2 | `templates/feature-registry.yaml`'s `feature_id` example is `F-0001` (not `UPG-0000`), and its inline comment points at `conventions.md`'s Feature IDs section instead of implying the self-dev scheme. | `grep -n "feature_id:" templates/feature-registry.yaml` shows `F-0001`; `grep -n "UPG-0000" templates/feature-registry.yaml` returns no hits. |
| 3 | `prompts/00b-feature-brief.md`'s Synthesis step contains explicit id-assignment instructions: scan `features/registry.yaml` + `backlog/F-####-*.md` for the current max, assign next for F-type, reuse the parent's id for R-type (no new id minted for refinements). | Read-through of "Synthesis and Completion Check"; `grep -n "F-####" prompts/00b-feature-brief.md` hits inside that section. |
| 4 | `prompts/00b-feature-brief.md`'s "Brief Lifecycle" and "Output Format" sections show a concrete `F-####` example in the `backlog/[feature_id]-[name].md` reference. | `grep -n "backlog/F-" prompts/00b-feature-brief.md` hits. |
| 5 | `templates/feature-brief.md`'s H1 and `**Refines**` field guidance reference the `F-####` format, with a comment noting the id is assigned at Synthesis, not chosen freely by the human. | Read-through of lines 1 and 27-28 (current); `grep -n "F-####" templates/feature-brief.md` hits. |
| 6 | `dba-system.md`'s File Layout block, Artifact Classification table, and Stage table entries referencing `[feature_id]` each carry one clarifying pointer to the `F-####` format in `conventions.md`, without altering stage names, gate wording, or 9-stage substance. | `grep -n "F-####\|feature_id" dba-system.md` shows the new pointer text; diff confirms no stage-table row content, gate text, or stage count changed from the pre-change version. |
| 7 | **Downstream compatibility**: a project freshly scaffolded via `scripts/dba-init.sh` still produces a valid `features/registry.yaml` from the corrected template, and the generated project's `CLAUDE.md` still loads `.codeos/dba-system.md` unchanged. | Run `scripts/dba-init.sh` against a scratch directory; confirm `features/registry.yaml`'s `feature_id` example reads `F-0001`; confirm the generated `CLAUDE.md` still references `.codeos/dba-system.md`. |
| 8 | **No internal contradiction**: no remaining reference to the old `UPG-0000` example or the `lowercase_underscore`-only Feature ID spec exists in any of the five touched files. | `grep -rn "UPG-0000" templates/conventions.md templates/feature-registry.yaml prompts/00b-feature-brief.md templates/feature-brief.md dba-system.md` returns no hits. |
| 9 | **9-stage substance preserved**: `dba-system.md`'s Artifact Classification table still lists Feature Brief and Feature Registry as **Optional**/**Recommended** (unchanged from before this change) — the `F-####` convention is layered on top, not a new hard requirement. | Read-through of the Artifact Classification table; confirm classification column values for those two rows are unchanged from the pre-change version. |

<!-- For downstream-doctrine or both: include downstream-compatibility criteria. -->
<!-- For script-tooling: include I/O behavior, exit-code / fail-closed cases, idempotency. -->

---

## Implementation Notes

<!-- Filled during Step 3. Summary only — the git diff is the source of truth.
Note decisions, discoveries, and anything deferred (and re-triaged as its own change). -->

All five declared files edited, exactly as scoped:

- `templates/conventions.md` — "Feature IDs" section replaced: `F-####` format, id/slug
  split, assignment rule (scan `features/registry.yaml` + `backlog/F-####-*.md`, R-type
  reuses parent), example filenames.
- `templates/feature-registry.yaml` — line 39 `feature_id` example changed
  `UPG-0000` → `F-0001`; comment now points at `conventions.md` → Feature IDs.
- `prompts/00b-feature-brief.md` — Synthesis Step 1 retitled "Assign the feature id, then
  draft the brief" with explicit F-type/R-type assignment instructions; "Brief Lifecycle"
  and "Output Format" sections both gained a concrete `F-0001` example alongside the
  existing `[feature_id]` placeholder.
- `templates/feature-brief.md` — new "FEATURE ID" comment block (format, who assigns it,
  pointer to conventions.md) added next to the existing TYPE block; `**Refines**` field
  changed from generic `[feature_id being refined]` to `[F-#### being refined]`.
- `dba-system.md` — three purely additive one-line-plus-blank pointers added (9-Step Loop
  intro, Artifact Classification intro, File Layout intro), each stating the `F-####`
  format and linking to `conventions.md`. Confirmed via `git diff -- dba-system.md`: no
  stage name, gate wording, or table row content changed — additions only.

**Discovery / scope clarification**: Step 1's Change Intent described the third
`dba-system.md` location as "Stage table entries referencing `[feature_id]`," but the
literal `[feature_id]` placeholders in that file actually live in the **9-Step DBA
Development Loop** code block (STEP 1/2/3 Outputs) and the **File Layout** tree, not in
the Stage ID table (`| Stage | Stage ID | File |`), which contains no `[feature_id]`
text. The Artifact Classification table separately uses a `[id]` shorthand for the same
identity. All three actual locations were covered (9-Step Loop, Artifact Classification,
File Layout); this is a terminology correction from Step 1, not a scope change — same
three locations, same file, same "one clarifying pointer" intent.

No out-of-scope changes were made. `scripts/dba-init.sh` was not edited.

**AC-7 (downstream compatibility) smoke-checked during this step** (formal Reconciliation
recording deferred to Step 4 per the template's own section split, but the underlying run
already happened and is recorded here so the claim is not left dangling):
ran `bash scripts/dba-init.sh smoke-test-project` against a scratch directory. Result:
`features/registry.yaml` line 39 reads
`feature_id: F-0001           # stable feature identifier — F-#### format, see .codeos/templates/conventions.md → Feature IDs`
(the corrected template copied verbatim, no `scripts/dba-init.sh` code change needed);
the generated `CLAUDE.md` still contains `Read `.codeos/dba-system.md`` /
`` `.codeos/dba-system.md` — authoritative DBA doctrine`` (lines 9, 11) unchanged. No
changes to `CLAUDE.md`, stage names, or gate rules.

**AC-5 fix during this step**: `templates/feature-brief.md`'s H1 was changed from
`[feature_id]` to `[F-####]` so the bracket itself states the expected format, matching
the `**Refines**` field's `[F-#### being refined]` phrasing already in place.

---

## Reconciliation

**Acceptance verification:**

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | `conventions.md` documents `F-####` + id/slug split | PASS | `templates/conventions.md` Feature IDs section rewritten; `grep -c "F-####"` = 2; `lowercase_underscore` no longer present |
| 2 | `feature-registry.yaml` example is `F-0001`, comment points to conventions.md | PASS | line 39: `feature_id: F-0001 # ... see .codeos/templates/conventions.md → Feature IDs` |
| 3 | `00b-feature-brief.md` Synthesis has id-assignment instructions | PASS | Step 1 "Assign the feature id, then draft the brief" (lines ~176-185): F-type scan rule, R-type reuse rule |
| 4 | `00b-feature-brief.md` Brief Lifecycle / Output Format show concrete example | PASS | both sections now read `backlog/F-0001-add-item-to-cart.md` |
| 5 | `feature-brief.md` H1 + `**Refines**` reference `F-####` | PASS | H1: `# Feature Brief: [F-####] — [short name]`; Refines: `[F-#### being refined]`; new FEATURE ID comment block |
| 6 | `dba-system.md` carries 3 clarifying pointers, no stage/gate substance changed | PASS | `git diff --stat -- dba-system.md` → `9 insertions(+), 0 deletions(-)`; diff reviewed line-by-line, additive only |
| 7 | Downstream compatibility: fresh `dba-init.sh` scaffold + generated `CLAUDE.md` unaffected | PASS | live scratch run: `features/registry.yaml` line 39 shows `feature_id: F-0001 ...`; generated `CLAUDE.md` lines 9/11 still read `Read .codeos/dba-system.md` / `.codeos/dba-system.md` — authoritative DBA doctrine |
| 8 | No internal contradiction — no leftover `UPG-0000` / `lowercase_underscore` | PASS | `grep -rn "UPG-0000\|lowercase_underscore" templates/ prompts/ dba-system.md` → 0 hits |
| 9 | 9-stage substance preserved; Feature Brief/Registry stay Optional/Recommended | PASS | Artifact Classification table diff shows no classification-column change; `git diff` confirms additive-only edits |

**Consistency sweep (grep):** clean. Ran toolkit-wide checks: (a) `UPG-0000` / `lowercase_underscore` absent from all downstream-facing files — 0 hits; (b) `F-####` present in all 5 touched files (2/1/5/3/3 occurrences respectively) — no file left half-updated; (c) every `conventions.md` cross-reference added by this change resolves to the real file/section (no orphaned links); (d) `dba-system.md`'s Stage ID table (`| Stage | Stage ID | File |`) and the 9-Step Loop's STEP 1-9 headers are byte-identical to pre-change — no stage-table ↔ prompt-file drift introduced; (e) `templates/feature-registry.yaml` re-parsed as YAML post-edit, `feature_id`/`slug` split intact; (f) `scripts/dba-init.sh` and `scripts/codeos-review.sh` — zero diff, confirmed out of scope as declared.

**Findings scope-triage:**

| Finding | Triage | Action |
|---|---|---|
| Step 1 R1: scope boundary claimed "no self-dev-only file changes," contradicted by the required `backlog/features.md` + `status/self-development.md` bookkeeping edits | IN-SCOPE BLOCKER | Fixed — scope boundary reworded to explicitly disclose the two Feature-Thread registration edits as expected bookkeeping, distinct from the five downstream-doctrine files |
| Step 1 R1: empty Acceptance Criteria table flagged as blocker | REJECTED | Acceptance Criteria is Step 2's deliverable, not Step 1's, per `prompts/codeos-self-dev.md`'s 4-step structure; the reviewer packet itself noted "no expected-output template for stage" `selfdev-step-1`. R2 did not re-raise it. |
| Step 1 R2: "six downstream-doctrine files" vs. five actually listed | IN-SCOPE NON-BLOCKER | Fixed inline — wording corrected to "five" |
| Step 3 R1 (AC-5): `feature-brief.md` H1 still read literal `[feature_id]`, not `F-####`-explicit | IN-SCOPE BLOCKER | Fixed — H1 changed to `# Feature Brief: [F-####] — [short name]` |
| Step 3 R1 (AC-7): downstream-compatibility claim pointed to an empty Reconciliation section instead of recording its own evidence | IN-SCOPE BLOCKER | Fixed — `dba-init.sh` smoke-test output recorded inline in Implementation Notes, then reproduced in this Reconciliation's AC-7 row |

---
