# Self-Development Change: UPG-0058__CHG-20260726-002 — cohort-logical-design-artifact

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0058
primary_feature_id: UPG-0058
change_id: CHG-20260726-002
slug: cohort-logical-design-artifact
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0058
related_features: [UPG-0051, UPG-0055]
review_series: RVS__UPG-0058__CHG-20260726-002__S4
review_profile: PROFILE-4
review_state: REVIEWED
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

`UPG-0051`'s Multi-Feature Architecture Synthesis Gate produces exactly one output — the Core
Architecture Baseline — which resolves strategic structural questions (topology, module ownership,
dependency direction, persistence technology, integration style) but not logical design questions
(identity/key strategy, revision/supersession pattern, module interface boundaries, transaction
ownership, event-emission points, read-model ownership). A real downstream project's discovery
found this insufficient: independently-implemented Stage-4 features sharing persistence and
identity can still make conflicting local decisions the gate exists to prevent — one feature using
UUID primary keys for a canonical entity, another identifying the same entity by URL, a third by a
composite key. The baseline is deliberately high-level; nothing in the current gate fixes the
logical detail features actually need shared *before* Stage 4.

**What changes:**

- `dba-system.md` — "Multi-Feature Architecture Synthesis Gate" section: documents the second
  output (Cohort Logical Design), the new **compatibility-only** cohort status `baseline-approved`
  (a fresh cohort going through this UPG's 4-step pipeline does not pass through it — Steps 2-3
  both draft only, and Step 4 approves both artifacts together directly to `approved`;
  `baseline-approved` exists solely to reinterpret a cohort that reached `approved` under the
  pre-this-UPG single-output rule, so its Stage 4 eligibility doesn't silently carry over under the
  new, stricter meaning), the changed meaning of `approved` (now requires both Baseline and Logical
  Design), the compatibility rule itself, the restated structure-not-behavior guardrail for the new
  artifact, one new Artifact Classification row, one new File Layout line, one new History Layout
  line (`architecture/history/cohort-logical-design-v[N].md`).
- `prompts/03b-architecture-synthesis.md` — extends the pipeline from 3 steps to 4: Cohort Evidence
  Review (unchanged) → Draft Baseline (drafts only, no separate gate — unchanged from today) →
  **Draft Cohort Logical Design (new step, drafts only, consuming the draft baseline)** → Approval
  and Activation (renumbered from Step 3 to Step 4; now approves **both** the Baseline and the
  Logical Design together in one human review, writing both artifacts and setting the cohort
  directly to `approved` — unblocking Stage 4 in one combined gate, not two sequential ones).
- `templates/cohort-logical-design.md` (new) — modeled on `templates/architecture-baseline.md`'s
  exact skeleton (Identity and Version / Authoritative Decisions-equivalent sections / Derived-
  Views-equivalent mapping / Open Risks / Revisit Triggers / metadata footer): logical ERD;
  entity/aggregate ownership; identity and key strategy; revision/supersession model; module
  interface map; command/query responsibilities; transaction boundaries; validation ownership;
  event-emission rules; read-model design; indexing/spatial principles; migration strategy;
  integration-test obligations; mapping from each design element to its source feature artifacts.
- `templates/feature-registry.yaml` — new `logical_design_version` field on
  `architecture_cohorts[]` entries (parallel to `baseline_version`, same versioning/staleness
  semantics); `status` enum gains `baseline-approved` between `gate-in-progress` and `approved`.
- `tools/reviewer/src/packet.rs` — small content update to the existing `"architecture-synthesis"`
  checklist (already generic to the whole `03b-architecture-synthesis.md` pipeline per `UPG-0055`)
  to mention the Logical Design step; no new stage id, no new match arm.

**Lifecycle bookkeeping (standard for every non-trivial self-dev change, not substantive scope):**
`backlog/features.md` and `status/roadmap.md` already carry `UPG-0058`'s row (registered before
this Step 1 was drafted); `status/self-development.md` gains this change's row as Step 1 activates.

**Scope boundary — what stays the same:**

Anything not named above is in scope for no change. In particular, this change does **not**: alter
`templates/architecture-baseline.md`'s own authoritative-decision categories or structure — it
receives exactly one minimal, necessary correction (its workflow comment's step count, "3-step" →
"4-step," since the pipeline it names literally changed), and it must **not** mention Cohort
Logical Design by name anywhere, staying a separate, stable artifact per the Guardrail; change the
cohort declaration mechanism (`architecture_cohort` field, cohort test,
declaration procedure) itself; add any new Stage ID (the new step lives inside the existing
conditional gate, not the 9-step loop); add any new Non-Negotiable Rule; silently reinterpret an
already-`approved` cohort as still fully `approved` under the new, stricter meaning (handled by the
explicit `baseline-approved` compatibility landing state, verified as an acceptance criterion); or
write a full technical design document inline in the new template (which stays terse and
placeholder-driven, matching the Baseline template's register).

**Class:** downstream-doctrine
**Scope axis:** downstream doctrine only
**Backlog item:** `backlog/UPG-0058-cohort-logical-design-artifact.md`

---

## Acceptance Criteria

| # | Criterion | How it will be verified |
|---|---|---|
| 1 | `dba-system.md`'s gate section documents Cohort Logical Design as a second output, naming all 14 content areas (logical ERD; entity/aggregate ownership; identity/key strategy; revision/supersession model; module interface map; command/query responsibilities; transaction boundaries; validation ownership; event-emission rules; read-model design; indexing/spatial principles; migration strategy; integration-test obligations; mapping to source feature artifacts) or referencing the template that names them. | Read-through. |
| 2 | The new **compatibility-only** cohort status `baseline-approved` is documented with exact semantics: it is the landing state for a cohort that reached `approved` under the pre-this-UPG single-output rule (Baseline only, Logical Design not yet approved); a fresh cohort going through this UPG's 4-step pipeline does not pass through it — Steps 2-3 draft only, Step 4 approves both artifacts together directly to `approved`. Stage 4 stays blocked in `baseline-approved` (same as `declared`/`gate-in-progress`). | Read-through of `dba-system.md` and `templates/feature-registry.yaml`'s status-enum comment. |
| 3 | `approved`'s meaning is updated **consistently everywhere it gates Stage 4 eligibility** to mean both Baseline and Logical Design are approved for their applicable versions — no file retains the old single-output meaning. | Grep for `architecture_cohorts`/cohort-status language across `dba-system.md`, `prompts/03b-architecture-synthesis.md`, `templates/feature-registry.yaml`, `prompts/04-implement.md`'s cohort eligibility check; confirm consistent wording. |
| 4 | A compatibility rule is explicitly stated for a cohort already `approved` under the pre-this-UPG single-output rule: treated as `baseline-approved` (not `approved`) on first read after this UPG ships — not silently reinterpreted as still fully `approved`. | Read-through of `dba-system.md`'s explicit compatibility paragraph. |
| 5 | The Logical Design's guardrail (may define structure, never invent behavior; a discovered behavioral gap returns to the owning Stage 1, 2, or 3 artifact) is stated explicitly, in the same spirit as the Baseline's existing guardrail. | Read-through comparing wording intent, not literal duplication. |
| 6 | `templates/cohort-logical-design.md` exists, modeled on `templates/architecture-baseline.md`'s skeleton (Identity and Version / structural-decision sections / derived-views-equivalent mapping / Open Risks / Revisit Triggers / metadata footer), with a placeholder section for each of the 14 content areas from AC1. | Direct read + structural side-by-side comparison with `templates/architecture-baseline.md`. |
| 7 | `prompts/03b-architecture-synthesis.md`'s pipeline is exactly 4 steps in order — Cohort Evidence Review → Draft Baseline → Draft Cohort Logical Design (new) → Approval and Activation (renumbered) — each ending in the standard `AWAITING HUMAN APPROVAL TO PROCEED TO STEP [N+1]` gate line. | Read-through. |
| 8 | The new Step 3 (Draft Cohort Logical Design) explicitly consumes the **draft** Baseline from Step 2 (not yet approved — approval for both artifacts happens together at Step 4) plus the original cohort evidence, and restates "do not resolve behavioral gaps here — return upstream," matching Step 2's existing rule. | Read-through. |
| 9 | The renumbered final step versions and writes `architecture/cohort-logical-design.md` the same way the Baseline is versioned (superseded file moves to `architecture/history/cohort-logical-design-v<version>.md` *before* the new version is written), updates the registry's new `logical_design_version` field, and only then may set cohort status to `approved`. | Read-through. |
| 10 | `templates/feature-registry.yaml` documents `logical_design_version` with the same current-version-only / staleness semantics as `baseline_version` (a historical file value is stale, not an alternate valid one, for live Stage-4 eligibility). | Read-through, comparing wording symmetry with the existing `baseline_version` comment block. |
| 11 | `templates/feature-registry.yaml`'s `status` enum comment documents all four values (`declared \| gate-in-progress \| baseline-approved \| approved`). | Read-through. |
| 12 | `tools/reviewer/src/packet.rs`'s existing `"architecture-synthesis"` checklist content mentions the Logical Design without inventing a criterion absent from `dba-system.md`/`prompts/03b-architecture-synthesis.md` (per `UPG-0055`'s own guardrail: the reviewer summarizes doctrine, it is never a second authority). | Read-through comparing checklist text against the doctrine files it summarizes. |
| 13 | No new Stage ID is added — the 9-Step DBA Development Loop / stage table is untouched. | `git diff` on that section shows no change. |
| 14 | No Non-Negotiable Rule is changed. | `git diff` on that section shows no change. |
| 15 | **Downstream-compatibility:** every new cross-reference path this change introduces (to `templates/cohort-logical-design.md`, the renumbered pipeline steps, the new registry fields) resolves to a real file/section — a downstream project's `.codeos` symlink still reaches everything referenced. | Grep/`ls` confirming each new path mentioned actually exists once Step 3 lands; no internal contradiction between `dba-system.md` and `prompts/03b-architecture-synthesis.md`'s step numbering. |
| 16 | **Cross-reference integrity sweep** (required for `downstream-doctrine` class): no file retains a stale description of the old 3-step pipeline or the old single-output `approved` meaning after this change. | Grep across `dba-system.md`, `prompts/03b-architecture-synthesis.md`, `templates/feature-registry.yaml`, `templates/architecture-baseline.md` (which must **not** mention Logical Design — it stays a separate, stable artifact per the Guardrail), `tools/reviewer/src/packet.rs`. |

---

## Implementation Notes

Six files touched (five as scoped in Step 1, plus one minimal necessary correction caught at
Reconcile — see Post-R2-review fixes below):

- `dba-system.md`: gate sequence renumbered to reflect the 4-step pipeline (Steps 2-3 now draft
  only; Step 4 approves both artifacts together, directly to `approved` — `baseline-approved`
  documented as a compatibility-only landing state, not a state the fresh pipeline passes through,
  to avoid inventing an extra gate beyond what Step 1/2 already scoped); new "What the logical
  design may and may not do" paragraph; "Cohort, baseline, and logical design versioning" section
  (renamed, extended) with the explicit compatibility rule for pre-existing `approved` cohorts;
  "Verifying a ... reference" section extended to both artifacts; Artifact Classification/File
  Layout/templates-table rows added.
- `prompts/03b-architecture-synthesis.md`: new Step 3 (Draft Cohort Logical Design, all 14 content
  areas, same "no behavioral gap resolution here" rule as Step 2); old Step 3 renumbered to Step 4,
  now approving and activating both artifacts together in one human review.
- `templates/cohort-logical-design.md` (new): modeled on `templates/architecture-baseline.md`'s
  skeleton — Identity/Version, Logical Design Decisions (13 placeholder subsections), Mapping to
  Source Feature Artifacts, Open Architectural Risks, Revisit Triggers, metadata footer.
- `templates/feature-registry.yaml`: `logical_design_version` field added parallel to
  `baseline_version`; `status` enum documents `baseline-approved` explicitly as the compatibility
  landing state, not a normal pipeline step.
- `tools/reviewer/src/packet.rs`: `"architecture-synthesis"` checklist content updated to mention
  the Logical Design and the 4-step pipeline — no new match arm, no new stage id, per `UPG-0055`'s
  guardrail (summarizes doctrine, never invents a criterion).

`prompts/04-implement.md`'s cohort eligibility check was also updated (not separately named in Step
1's file list, but directly required by AC3's "everywhere it gates Stage 4 eligibility" — this was
an oversight in Step 1's file enumeration, caught during implementation; noted here rather than
silently expanding scope, since it is a direct, necessary consequence of AC3, not a new capability).

Full `cargo test --release` suite re-run after the `packet.rs` change: 182 tests, 0 failures — no
regression from the checklist-content edit.

No out-of-scope items discovered beyond the `04-implement.md` note above.

**Post-R1-review fixes:** the first Step 3 review round found three real issues, all fixed in
place: (1) the Change Intent's own prose described `baseline-approved` as a normal intermediate
state a fresh pipeline reaches, contradicting AC7's single combined "Approval and Activation" step
and the actual implementation — corrected throughout the Change Intent and ACs 2/8 to describe it
as compatibility-only, matching what was actually built; (2) `dba-system.md`'s "The rule." paragraph
(and Truth Authority rule #5) still said Stage 4 requires only the Architecture Baseline —
corrected to require both artifacts; (3) the Step 3 prompt's content-area count was internally
inconsistent ("13" vs. an implied 14) — the two merged bullets were split so all 14 items from AC1
are enumerated 1:1, matching the template's already-correct 13-subsection-plus-mapping structure.

**Post-R2-review fixes:** two more stale sentences, both leftover from the single-output model,
found and fixed: (1) "Declaring a cohort" still described the registry as holding "a baseline
version reference" (singular) — corrected to "baseline and logical design version references";
(2) the Artifact Classification row and File Layout entry for the Cohort Logical Design both said
it exists "after the Baseline is approved," implying sequential approval — corrected to "approved
together with the Baseline at Architecture Synthesis Step 4," matching the combined-approval model.

---

## Reconciliation

**Acceptance verification:**

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | 14 content areas named | PASS | `dba-system.md` references the template; `templates/cohort-logical-design.md`'s "Logical Design Decisions" (13 subsections) + "Mapping to Source Feature Artifacts" name all 14. |
| 2 | `baseline-approved` documented as compatibility-only, not a normal pipeline state | PASS | `dba-system.md:206-214`; `templates/feature-registry.yaml:49-58` — consistent wording confirmed by direct grep. |
| 3 | `approved` requires both artifacts, everywhere | PASS | `dba-system.md`'s "The rule." (180-185) and Truth Authority #5 (27); `prompts/04-implement.md:22-35`; `prompts/03b-architecture-synthesis.md`'s Step 4. |
| 4 | Compatibility rule explicit | PASS | `dba-system.md:261-269`. |
| 5 | Logical design guardrail restated | PASS | `dba-system.md:222-234`; `prompts/03b-architecture-synthesis.md:122-127`; `templates/cohort-logical-design.md:15-17`. |
| 6 | Template structural parity with Baseline template | PASS | Direct side-by-side: both have Identity/Version, membership table, a main decisions section, a mapping/derived-views section, Open Risks, Revisit Triggers, metadata footer. |
| 7 | 4-step pipeline, correct order, standard gate lines | PASS | `prompts/03b-architecture-synthesis.md` Steps 1-4; each ends in an `AWAITING HUMAN APPROVAL` line (Step 4's is a named terminal line, consistent with the old Step 3's own terminal-line precedent). |
| 8 | Step 3 consumes draft Baseline + evidence, restates no-behavioral-gap rule | PASS | `prompts/03b-architecture-synthesis.md:93-131`. |
| 9 | Step 4 versions/writes both artifacts + registry, unblocks Stage 4 | PASS | `prompts/03b-architecture-synthesis.md:145-159`. |
| 10 | `logical_design_version` mirrors `baseline_version` semantics | PASS | `templates/feature-registry.yaml:101-117`. |
| 11 | All four status values documented | PASS | `templates/feature-registry.yaml:45-58`. |
| 12 | Reviewer checklist mentions Logical Design, invents nothing | PASS | `tools/reviewer/src/packet.rs:676,696`; wording traced directly to `dba-system.md`/`prompts/03b-architecture-synthesis.md`. |
| 13 | No new Stage ID | PASS | `git diff` on the stage table (dba-system.md:440-455 area) shows no change. |
| 14 | No Non-Negotiable Rule changed | PASS | `diff` of the full "## The Non-Negotiable Rules" section against `HEAD` is byte-identical. |
| 15 | Downstream-compatibility: new cross-references resolve | PASS | Every new path (`templates/cohort-logical-design.md`, `architecture/cohort-logical-design.md`, `architecture/history/cohort-logical-design-v[N].md`) exists or is correctly documented as created at runtime by the pipeline. |
| 16 | Cross-reference integrity sweep | PASS | All three stale-reference findings from Step 3 R1/R2 are fixed; `templates/architecture-baseline.md`'s stale "3-step pipeline" (found at Reconcile R1) is now "4-step pipeline" with **no** mention of Logical Design (the R1 fix attempt wrongly added one — caught and corrected at R2): `grep -c "Logical Design" templates/architecture-baseline.md` → 0. |

**Consistency sweep:** full-repo grep for `after the Baseline is approved`, `a baseline version reference` (singular), and any remaining single-output `approved` framing returns no matches outside historical review artifacts. `git diff` on `dba-system.md` touches only the Multi-Feature Architecture Synthesis Gate section, Truth Authority rule 5, Artifact Classification, File Layout, and the templates table — no stage table, Non-Negotiable Rule, or unrelated section changed. Full `cargo test --release --manifest-path tools/reviewer/Cargo.toml` suite re-run at Reconcile: **182 tests, 0 failures** (10 test binaries: 49+5+16+14+24+18+21+9+21+5).

**Post-Reconcile-review fixes (Step 4 R1 and R2):** the first Reconcile review round caught a real
gap the Step 3 rounds missed — `templates/architecture-baseline.md`'s own workflow comment still
said "3-step pipeline," which AC16's sweep set explicitly includes. Fixed by adding the word
"Cohort Logical Design" to explain the new step... which the **second** Reconcile round correctly
caught as a *new* violation of the Guardrail's "must not mention Logical Design" rule and of the
Scope boundary's "does not alter this file" claim. Resolved properly on the second attempt: the
workflow comment now says only "4-step pipeline" — the step count is corrected without naming
Logical Design at all, and `templates/architecture-baseline.md` is added to this change's touched-
file list (six files, not five) with that one-word/number correction disclosed explicitly rather
than the change record continuing to claim the file was untouched.

**Findings scope-triage:** all 5 findings raised across Step 3's two review rounds were IN-SCOPE BLOCKER and fixed inline within this same CHG (see backlog Feature Thread). None was OUT-OF-SCOPE BACKLOG, REJECTED, or SELF-REFERENCE/REVIEW-BOOKKEEPING. No new findings surface at Reconcile.
