# Self-Development Change: UPG-0051__CHG-20260719-001 — architecture-synthesis-gate

<!--
PURPOSE: Per-change source of truth for a non-trivial change to the Codeos toolkit
itself (prompts, templates, docs, patterns, scripts).

This is NOT a downstream DBA feature. It has no behavioral contract, no event schema,
and no replay. Trivial changes do not get a record.

Workflow: prompts/codeos-self-dev.md (4-step loop)
Each step requires explicit human approval; Codex review cadence is governed by the assigned review profile (see prompts/codeos-self-dev.md Step 0a).
The live status row lives in status/self-development.md, not here.
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0051
primary_feature_id: UPG-0051
change_id: CHG-20260719-001
slug: architecture-synthesis-gate
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0051
related_features: [UPG-0052]
review_series: RVS__UPG-0051__CHG-20260719-001__S4
review_profile: PROFILE-4
review_state: ACCEPTED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

<!-- SELF-REFERENCE BOUNDARY: this artifact is itself reviewed, so it must NOT embed the current
review round (which does not exist until after the packet is built). Reference the stable review
SERIES (review_series) + review_state; exact rounds live only in reviews/review-log.md and
reviews/codex/*. See prompts/codeos-self-dev.md → "Feature Thread & IDs" / "Self-Reference Boundary". -->


## Change Intent

**Why (problem in the toolkit):**

`dba-system.md` has no mechanism for a multi-feature project to synthesize a shared workspace/
crate/event-transport architecture from cross-feature evidence before implementation begins. Every
feature can independently pass Stages 1–3 and enter Stage 4, so for a project whose features share
canonical artifacts, persistence, or cross-feature events (the motivating case is a downstream
~14-feature project), one feature's Stage 4 implementation risks silently becoming the
architectural precedent the whole cohort inherits, rather than a decision grounded in the complete
approved behavioral and event topology. This was analyzed in depth in `backlog/UPG-0051-multi-
feature-architecture-synthesis-gate.md` (itself refined through three rounds of external review),
which settled the conceptual boundaries this change now carries into doctrine text.

**What changes:**

0. Bookkeeping only, already committed as part of registering `UPG-0051`'s backlog brief (not new
   work introduced by this change): `backlog/features.md` and `status/roadmap.md` gained rows for
   `UPG-0051`..`UPG-0054`; `status/self-development.md` gains the row activating this change
   (`UPG-0051` / `CHG-20260719-001`, below).
1. `dba-system.md`:
   - New **"Multi-Feature Architecture Synthesis Gate"** section: the core cohort test, the
     conditional gate mechanics, the leaner cohort-review model (Intent/Contract Cohort Checks
     recommended, Event Cohort Check required, Architecture Baseline approval as the single new
     mandatory gate), the authoritative-decisions-vs-derived-views distinction for the baseline
     artifact, and the naming rule ("Architecture Synthesis Gate" / "Core Architecture Baseline",
     distinguished from Discovery).
   - One new item appended to the existing **"Truth Authority and Conflict Resolution"** list
     (`dba-system.md:19-28`) — a narrow subordination clause stating the Architecture Baseline is
     authoritative only for structural decisions not fixed by approved behavioral artifacts, never
     overriding Intent/Contract/Event Schema/human correction/safety invariants, with runtime
     evidence conflicts resolved through the existing reconciliation rules (not a new hierarchy).
   - A cross-reference at the Stage 3→4 boundary in the 9-Step DBA Development Loop: Stage 4 entry
     for a feature belonging to a declared core cohort requires that cohort's Architecture Baseline
     to be approved for the applicable version.
   - One clarifying sentence in the Stage 10 (Architectural Refinement) section: structural-only
     corrections to an approved baseline remain Stage-10-eligible only when behavior is unchanged.
   - A new row in the Artifact Requirements table for the Architecture Baseline
     (`architecture/core-baseline.md` — required for core-cohort Stage 4 eligibility, not
     applicable to single-feature or non-cohort projects).
   - File Layout section: add `architecture/core-baseline.md` and `architecture/history/` (holding
     superseded baseline versions once a replacement supersedes them).
2. `prompts/03b-architecture-synthesis.md` (**new file**) — the Architecture Synthesis session
   prompt, sitting between `03-event-schema.md` and `04-implement.md` (naming mirrors the existing
   `00a`/`00b`/`00c` lettered-suffix convention for stage-adjacent, non-numbered activities). Loads
   the cohort's approved Intent/Contract/Event Schema artifacts and relevant Architecture Journal
   entries, separates derived observations from decisions, produces the baseline, returns
   behavioral gaps to the affected feature's earlier stage, and stops for human approval —
   mechanically modeled on Stage 10's own step-gated loop.
3. `templates/architecture-baseline.md` (**new file**) — the baseline artifact template: version/
   identity header, approved cohort membership set, the authoritative-decisions section and the
   derived-views/matrices section (each matrix explicitly marked regenerable with provenance to its
   source artifacts), and a revisit-triggers section.
4. `templates/feature-registry.yaml` — additive schema change (no `schema_version` bump — see scope
   boundary): a new optional per-feature field `architecture_cohort: null` (cohort id or null), and
   a new top-level `architecture_cohorts:` list (paralleling the existing
   `architectural_refinements:` list) with entries carrying `cohort_id`, `member_features`,
   `status: declared | gate-in-progress | approved`, `baseline_version`, `declared_by`,
   `declared_at`. This is where cohort declaration and gate status live pre-baseline-approval — one
   of the questions the backlog brief left open for this step.
5. `prompts/04-implement.md` — add a Stage 4 applicability check: before implementing, check
   `features/registry.yaml` for an `architecture_cohort`; if present, verify the referenced cohort's
   baseline is `approved` for the version covering this feature before proceeding.
6. `prompts/00-session-start.md` — surface architecture cohort/baseline status (declared /
   gate-in-progress / approved) at session start when the Feature Registry declares one, so it
   isn't discovered for the first time at Stage 4.
7. `10-arch-refine.md` — add the same one-sentence clarification as the Stage 10 doctrine text
   change above, kept consistent between the two.

**Scope boundary — what stays the same:**

- The 9-Step DBA Development Loop's existing stage substance (Stages 1–9) is unchanged; **no new
  numbered stage is inserted** — this is a cohort-level gate sitting between cohort Stage 3 and
  cohort Stage 4, not "Stage 3.5" for every feature.
- Stage 10's existing 5-step loop mechanics are unchanged beyond the one clarifying sentence noted
  above.
- **Reviewer/Codex coverage for this new gate is explicitly deferred in this change**, not built.
  Author's direct read of `tools/reviewer/src/packet.rs:661-696` (this file is not part of this
  change and is not attached to this review packet, so this is an author finding, not something
  this packet asks the reviewer to independently confirm) found: `codeos-reviewer` accepts an
  arbitrary `stage` string, but `stage_expected`/`stage_checks` have no match arm for
  `"architecture-synthesis"` — such an invocation would fall through to the generic
  `_ => "(no expected-output template for stage)"` / `"(no stage-specific checklist for stage {})"`
  branches, i.e. it would run but without a tailored checklist. Adding a proper match arm is a
  `script-tooling`-class Rust change, out of scope here. Until that lands, the new `dba-system.md`
  section documents that Architecture Synthesis Gate reviews use the existing Review Waiver
  mechanism (already described under Default Advisory Review) rather than the untailored generic
  fallback. This finding becomes a checkable Step 2 acceptance criterion (re-verified by grep
  against `packet.rs` at Step 4, where the reviewer can independently confirm it against the actual
  file). Native reviewer support (a real match arm) is filed as an out-of-scope-backlog follow-up
  at Reconcile (Step 4), not built now.
- **"Regression tests confirming the gate is actually wired"** (the backlog brief's phrasing) does
  not mean new automated test files — this change touches no Rust/script code, only doctrine
  prose, prompts, and a YAML template. It is satisfied by Step 4's **grep-based cross-reference
  verification**, already mandated for `downstream-doctrine`-class changes by `CLAUDE.md`: e.g.
  grep confirming `prompts/03b-architecture-synthesis.md` is actually referenced from
  `dba-system.md`, `prompts/04-implement.md`'s new cohort check references the exact
  `architecture_cohort` field name added to `templates/feature-registry.yaml`, and
  `prompts/00-session-start.md` references the same field — the check that would have caught
  `patterns/rust-project-structure.md`'s orphaning had it existed. These become explicit,
  independently-verifiable Step 2 acceptance criteria rather than an implicit assumption.
- `templates/feature-registry.yaml`'s `schema_version` stays at `2` — the new fields are additive
  and optional; a registry without them is still valid and simply has no declared cohort.
- No Rust-specific / Implementation Profile content — that is `UPG-0052`, a separate change (an
  independent integrating peer per both UPGs' briefs, not a dependency of this one).
- No downstream project's actual `.codeos/` symlink or generated files are touched — this changes
  the toolkit source; downstream projects pick it up when they resync `.codeos/`.
- No change to `dba-init.sh` — that is `UPG-0053`, gated behind `UPG-0052`.

**Class:** downstream-doctrine
**Scope axis:** downstream doctrine only
**Backlog item:** backlog/UPG-0051-multi-feature-architecture-synthesis-gate.md

---

## Acceptance Criteria

| # | Criterion | How it will be verified |
|---|---|---|
| 1 | **Source-of-truth separation**: `templates/feature-registry.yaml`'s new fields (`architecture_cohort` per feature, `architecture_cohorts:` list) carry only membership, gate status, and a baseline identity/version *reference* — never structural decisions (crate topology, dependency DAG, shared-infrastructure choices). Those remain exclusively in `templates/architecture-baseline.md` / `architecture/core-baseline.md`. | Read-through of the new registry schema comments confirms no structural-decision field was added there; `dba-system.md`'s new section states this separation explicitly in prose. |
| 2a | Stage 4 for a feature with `architecture_cohort: null` proceeds with no cohort check at all. | Read `prompts/04-implement.md`'s new check text: the `null` case is the first branch and short-circuits. |
| 2b | Stage 4 for a cohort member whose cohort has no baseline yet (`architecture_cohorts` entry exists, `status` is not `approved`) is blocked, with the prompt naming the required next action (run Architecture Synthesis). | Read `prompts/04-implement.md`; grep for the literal blocking condition referencing `status: approved`. |
| 2c | A `declared` or `gate-in-progress` cohort status blocks Stage 4 the same way as "no baseline" (2b) — not a distinct, weaker case. | Read-through confirms `prompts/04-implement.md` treats every non-`approved` status identically, not just `declared`. |
| 2d | A registry `baseline_version` that does not equal `architecture/core-baseline.md`'s **current** version — including one that only matches a historical `architecture/history/core-baseline-v<version>.md` file — is treated as **stale, not valid**, and blocks Stage 4 the same as an unapproved status. | `prompts/04-implement.md` states this as an exact equality check against the current version, with historical-file matches explicitly called out as insufficient; `dba-system.md`'s "Verifying a `baseline_version` reference" states the same rule in the same terms. |
| 2e | *(Revised during Step 4 — see Reconciliation)* Live Stage 4 eligibility always requires the **current** baseline version; the "applicable version, not necessarily newest" language applies only to the non-retroactive-invalidation protection for already-completed Stage 4 work (2f), never to a live gating decision for new/re-entering Stage 4 work. `baseline_version` is a single cohort-level field, not tracked per feature. | `dba-system.md`'s "Cohort and baseline versioning" and "Verifying a `baseline_version` reference" subsections state this distinction explicitly; `prompts/04-implement.md`'s check only ever accepts the current version. |
| 2f | A later cohort-membership change (new/removed feature) creates a new cohort/baseline version and **requires an impact assessment**; prior Stage 4 work approved under the earlier version is **not invalidated merely by the membership change itself**, but **must be reconciled when that assessment identifies an actual structural conflict** — this is a conditional protection, not an absolute "never invalidated" guarantee. | `dba-system.md`'s new section states all three parts of this rule (new version required, impact assessment required, conditional reconciliation on actual conflict) in the same terms as `prompts/04-implement.md`'s eligibility check — no contradiction, and no version that drops the "unless an actual conflict is found" qualifier. |
| 2g | **Cohort-overlap semantics**: a feature belongs to at most one active architecture cohort (matching the singular `architecture_cohort` field); a project may declare multiple cohorts, but their feature memberships must not overlap. | `dba-system.md`'s new section states this explicitly; `templates/feature-registry.yaml`'s schema comments document the field as singular-cohort-only, and note that overlapping `architecture_cohorts` entries (same feature listed under two cohort ids) is an invalid registry state. |
| 3 | Reviewer coverage for the new gate is not silently skipped: `dba-system.md`'s new section explicitly names the existing **Review Waiver** mechanism (`dba-system.md:71-78`) as the interim path, with the specific reason ("`architecture-synthesis` stage id has no match arm in `codeos-reviewer`'s `packet.rs` yet") — not a vague "review is optional" statement. This criterion is satisfied by documenting the waiver's use, not by native reviewer support existing (that remains a Step 4 follow-up item and does not block this change's completion). | Grep the new section for "Review Waiver" and confirm the stated reason matches the actual gap; grep `tools/reviewer/src/packet.rs` fresh at Step 4 to re-confirm no `"architecture-synthesis"` match arm exists (re-verifying the Step 1 author finding independently, this time with the file itself available to check). |
| 4 | `templates/feature-registry.yaml` stays `schema_version: 2`; the new fields are optional and a registry omitting them remains valid, keeping its prior meaning unchanged — no existing downstream registry is broken by this change. | Read-through confirms the new fields are documented as optional (not required) and the `schema_version: 2` header comment block is unchanged; **a named backward-compatibility fixture** — the template's own example `feature_id: F-0001` entry, deliberately left without `architecture_cohort` — is checked at Step 4 to confirm it still reads as a complete, valid, cohort-less entry (not as missing a required field). |
| 5 | **Downstream-compatibility** (required for `downstream-doctrine` class): a generated downstream project's `CLAUDE.md` still loads `.codeos/dba-system.md`; every prompt filename referenced from `dba-system.md`'s stage tables still exists after this change (no rename, only additions); the new prompt/template filenames this change introduces are referenced from exactly the `dba-system.md` locations named in Step 1 (Artifact Requirements table, File Layout, new section) and nowhere orphaned. | Grep sweep across `dba-system.md`, `prompts/*.md`, `templates/*` for the exact new filenames (`03b-architecture-synthesis.md`, `architecture-baseline.md`, `architecture_cohort`) confirming each appears in every location Step 1 named — the specific check that would have caught `patterns/rust-project-structure.md`'s orphaning had it existed. |
| 6 | **No internal contradiction**: the Truth Authority subordination clause is *added*, not a restatement — the existing four items in `dba-system.md:19-28`'s Truth Authority and Conflict Resolution list are semantically unchanged, with exactly one new item appended. | `git diff` on that section shows only an appended item, with no wording changes to items 1–4 beyond harmless whitespace normalization (a focused semantic diff, not a strict byte-identical requirement). |
| 7 | The Stage 10 clarification sentence added to `dba-system.md`'s Stage 10 section and the matching sentence added to `10-arch-refine.md` are non-contradictory (need not be verbatim-identical, but must agree on when Stage 10 applies to a baseline correction). | Read both sentences side by side at Step 4; confirm same conditions (behavior unchanged) stated in both. |
| 8 | The new `dba-system.md` section explicitly cross-references `00a-solution-discovery.md` to preserve the Discovery-vs-Synthesis distinction (Discovery is non-authoritative pre-Stage-1; this gate is authoritative, post-Stage-3). | Grep the new section for a reference to Discovery / `00a-solution-discovery.md` and confirm the distinction is stated, not merely implied by naming. |

---

## Implementation Notes

<!-- Summary only — the git diff is the source of truth. -->

All 7 files from Step 1's "What changes" list were edited/created as planned. No scope creep;
no additional files touched beyond the bookkeeping already noted in Step 1 item 0.

**Key decisions made during implementation** (each maps to an acceptance criterion):
- Chosen Stage ID string: `architecture-synthesis` (used consistently in `dba-system.md`'s stage
  table, both prompt files, and the reviewer-deferral note).
- Cohort state lives in `templates/feature-registry.yaml`: `architecture_cohort` (singular, per
  feature) + top-level `architecture_cohorts:` (list) — additive, `schema_version` untouched.
- The new "Multi-Feature Architecture Synthesis Gate" section was placed immediately after the
  9-Step Loop and before "What You Do at Each Stage," since it is stage-adjacent (sits between
  Stage 3 and Stage 4) without being one of the nine.
- `prompts/03b-architecture-synthesis.md` is a 3-step gated pipeline (Cohort Evidence Review →
  Draft Baseline → Approval and Activation), shorter than Stage 10's 5 steps since this workflow
  has a narrower job (produce and approve one baseline) — modeled on Stage 10's step-gate
  mechanics (`AWAITING HUMAN APPROVAL TO PROCEED TO STEP [N+1]`), not its exact step count.
- AC 2c (declared/gate-in-progress treated identically) is satisfied by `04-implement.md`'s check
  testing only for `status: approved` — any other status (including future ones) blocks
  identically, by construction, not by enumerating each non-approved value.
- AC 2g (cohort-overlap) is documented in both `dba-system.md`'s new section and
  `templates/feature-registry.yaml`'s schema comments (the singular-field framing plus an explicit
  "no feature in more than one cohort's `member_features`" rule).
- AC 4's backward-compatibility fixture: the template's own `F-0001` example entry deliberately
  omits `architecture_cohort` (shown commented-out with an explanatory note), demonstrating the
  field's absence is valid and means "no cohort."

**Nothing was deferred or discovered out-of-scope during implementation.**

**Post-Step-3-R1 fixes** (both findings confirmed legitimate, both fixed — not disputed):
- The "version history" check in `prompts/04-implement.md` and AC 2d was genuinely
  underspecified (the baseline template had no version-history mechanism to check against).
  Fixed by defining an exact naming convention: `architecture/core-baseline.md` always holds only
  the current version; superseded versions archive to
  `architecture/history/core-baseline-v<version>.md`, named for their own version. A
  `baseline_version` reference is valid only if it matches one of those two locations — stated in
  the same terms in `dba-system.md` (new "Verifying a `baseline_version` reference" note),
  `prompts/04-implement.md`, `prompts/03b-architecture-synthesis.md`'s Step 3, and
  `templates/architecture-baseline.md`.
- `templates/feature-registry.yaml`'s `architecture_cohorts` example carried an undeclared
  free-form `notes` field (copied by habit from the adjacent `architectural_refinements` example),
  which Step 1 never listed and which weakened AC 1's source-of-truth-separation guarantee.
  Removed; replaced with an explicit comment stating rationale/context belongs in the baseline's
  own Open Architectural Risks / Revisit Triggers sections, never in the registry.

---

## Reconciliation

**Acceptance verification:**

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Source-of-truth separation | PASS | `templates/feature-registry.yaml`'s `architecture_cohorts` schema carries only `cohort_id`, `member_features`, `status`, `baseline_version`, `declared_by`, `declared_at` — no structural-decision field, no `notes`. `dba-system.md`'s "Declaring a cohort" paragraph states the separation explicitly. |
| 2a | `null` cohort → no check | PASS | `prompts/04-implement.md`: "If `architecture_cohort` is absent or `null` — no further check; proceed" is the first branch. |
| 2b | Non-`approved` status blocks | PASS | `prompts/04-implement.md`: "If `status` is not `approved`... **STOP**." |
| 2c | `declared`/`gate-in-progress` treated identically | PASS | The check branches only on `status == approved` vs. not — by construction, every non-`approved` value (including future ones) hits the same STOP, not an enumerated list. |
| 2d | Stale/historical `baseline_version` blocks | PASS | `prompts/04-implement.md` (revised this step): exact-equality check against current version; historical-file match explicitly called "stale, not valid." `dba-system.md`'s "Verifying a `baseline_version` reference" (revised) states the same. |
| 2e | Live eligibility always uses current version; "applicable version" language scoped to 2f only | PASS (revised this step) | `dba-system.md`'s "Cohort and baseline versioning" now states `baseline_version` is "a single, cohort-level field," and "Verifying a `baseline_version` reference" states the live check "only ever accepts the current version." No remaining text implies a historical version can satisfy new Stage 4 entry. |
| 2f | Non-retroactive invalidation, conditional on actual conflict | PASS | `dba-system.md`: "not invalidated merely by the membership change itself — it must be reconciled only when that assessment identifies an actual structural conflict... through Stage 9/10." |
| 2g | Cohort-overlap semantics | PASS | `dba-system.md`: "A feature belongs to at most one active cohort... memberships must not overlap." `templates/feature-registry.yaml` comment: "no feature may appear in more than one cohort's `member_features` list." |
| 3 | Reviewer deferral via named Review Waiver, not silent skip; does not block completion | PASS | `dba-system.md` "Reviewer coverage" paragraph names Review Waiver + specific reason. Fresh grep this step: `tools/reviewer/src/packet.rs` has no match arm for `"architecture-synthesis"` — confirms the Step 1 author finding independently, now with the file itself available. Native support remains an out-of-scope-backlog follow-up (see below), not required for this change to complete. |
| 4 | Schema stays additive, `schema_version: 2`, backward-compat fixture | PASS | `schema_version: 2` line unchanged; `F-0001` example entry has no active `architecture_cohort` field (shown commented-out with rationale) and is otherwise a complete, valid entry — demonstrating omission is valid. |
| 5 | Downstream-compatibility / wiring sweep | PASS | Grep sweep: every `.codeos/prompts/*.md` and `.codeos/templates/*` filename referenced in `dba-system.md`'s two tables resolves to an existing file (17 prompts, 13 templates, including the two new ones). `templates/project-CLAUDE.md` still references `.codeos/dba-system.md` unchanged. New filenames (`03b-architecture-synthesis.md`, `architecture-baseline.md`, `architecture_cohort`/`architecture_cohorts`) each appear in every location Step 1 named (cross-reference table in Step 3 summary). |
| 6 | Truth Authority: only one item appended | PASS | `git diff -- dba-system.md` on that section shows items 1–4 as pure context lines (unmodified), one new line (item 5) added. |
| 7 | Stage 10 consistency | PASS | `dba-system.md`: "Stage-10-eligible only when it does not change any feature's behavior." `prompts/10-arch-refine.md`: "eligible here only when no feature's behavior changes." Same condition, non-contradictory phrasing. |
| 8 | Discovery cross-reference preserved | PASS | `dba-system.md`'s "Naming" paragraph: "Solution Discovery (`.codeos/prompts/00a-solution-discovery.md`) is optional, non-gating, and pre-Stage-1; its output is never approved architecture. This gate is the opposite..." |

All 15 criteria PASS.

**Consistency sweep (grep):**

No stale references, orphaned links, or stage-table↔prompt-file drift found. Every new filename
this change introduces is referenced from every location Step 1 committed to; every pre-existing
stage-table/template-table entry still resolves. No prompt was renamed, so no downstream-compat
risk from a moved reference.

**Findings scope-triage:**

| Finding | Triage | Action |
|---|---|---|
| Step 3 R1: version-history check underspecified (no defined mechanism to check `baseline_version` against) | IN-SCOPE BLOCKER | Fixed in Step 3 (naming convention defined across 4 files) |
| Step 3 R1: undeclared `notes` field in registry `architecture_cohorts` example | IN-SCOPE BLOCKER | Fixed in Step 3 (field removed, rationale documented) |
| Human's pre-Step-4 note: live eligibility could be misread as accepting a historical `baseline_version` as an alternate valid reference, conflating "protect past work" (2f) with "gate new work" | IN-SCOPE BLOCKER | Fixed in Step 4 (this reconciliation) — tightened "Verifying a `baseline_version` reference" to require exact-match against the current version only; historical files reframed as provenance-only, never a live-eligibility alternative; clarified `baseline_version` is a single cohort-level field |
| Native `codeos-reviewer` support for the `architecture-synthesis` stage id (a real match arm in `packet.rs`) | OUT-OF-SCOPE BACKLOG | Not built in this change — deferred via Review Waiver per Step 1's declared scope boundary. Candidate for a future `script-tooling`-class UPG if the human wants to pick it up; not filed as a new UPG yet since no one has requested it — noted here for visibility. |
| Reviewer packet-size warning (109–115 KB, ~3× budget) at Step 3 | REVIEW-BOOKKEEPING | Not an artifact defect — `dba-system.md`'s size plus the change record's own length account for it. Delta mode was available but required tracked files; two new files were untracked at review time. No action needed; did not affect review quality (FULL_COVERAGE both rounds). |

---
