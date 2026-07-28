# Self-Development Change: UPG-0059__CHG-20260728-001 — wave-gated-batch-review

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0059
primary_feature_id: UPG-0059
change_id: CHG-20260728-001
slug: wave-gated-batch-review
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0059
related_features: [UPG-0051, UPG-0057, UPG-0058]
review_series: RVS__UPG-0059__CHG-20260728-001__S4
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

Multi-feature DBA projects using the existing Multi-Feature Architecture Synthesis Gate
(`UPG-0051`/`UPG-0058`) pay a real efficiency cost from per-feature, per-stage individual review
sessions at cohort scale: Stage 1/2/3 approval fragments into many separate round trips, each its
own context switch. A batched-drafting-and-review approach was proposed to fix this, in a form that
removed the per-feature human approval gate entirely — draft Stage 1-3 for every cohort feature with
no human checkpoint, review only once at the very end. That form was rejected after critical
assessment: it maximizes blast radius, since a bad Stage-1 assumption for one feature could compound
through two more stages and spread to sibling features drafted in the same batch before any human
ever saw it. Four rounds of review converged on a safer alternative that keeps the speed benefit
without that risk. Full design history —
including the rejected original proposal and the reasoning at each refinement round — lives in
`/home/rimo/.claude/plans/calude-consider-this-inputs-steady-pnueli.md`.

**What changes:**

- `dba-system.md` — the Multi-Feature Architecture Synthesis Gate section gets:
  - A **Wave Gate** definition: for a declared cohort, drafting and the human's review session
    batch per stage (one wave = one stage, every member feature, one human sitting), but the
    approval decision stays individual, per feature, per artifact version — Non-Negotiable Rule #1
    unchanged. Partial-wave outcomes are explicit: some features approved, some returned for
    revision, in the same sitting; a Wave Gate is never all-or-nothing.
  - An **independent-justification** requirement: each feature's draft must be justified by its own
    approved upstream inputs; sibling drafts from the same wave may inform comparison but must never
    become unstated authority for a decision the feature's own approved artifacts don't support.
  - A **traceable cohort harmonization** rule: cohort-driven changes must be traceable in the review
    record (no doctrine-level enumeration of exactly where).
  - **Progressive check definitions**, replacing a single uniform "Column Check": Stage 1 has no
    cross-stage check (nothing exists yet to compare); Stage 2 gets an upstream-alignment check
    (Contract vs. that feature's approved Intent); Stage 3 gets the complete Column Check (Event
    Schema vs. that feature's approved Contract and Intent — scope, guarantees, exclusions,
    terminology, lifecycle assumptions, not only actors/event ownership). Sequencing: draft → each
    feature's upstream-alignment/Column Check → the cohort's Row Check (the existing Cohort Check,
    tightened to run as standard practice at the Wave Gate) → only then the Wave Gate decides every
    individual approval. No feature is approved before the cohort-level Row Check has run.
  - **Advisory-but-not-ignorable** wording for both checks: no decision authority of their own, but
    an unresolved material contradiction makes an approval unsupported unless the human explicitly
    overrides it with a recorded rationale.
  - **Targeted reassessment via a lightweight impact assessment**, reusing the section's own
    existing versioned-impact-assessment pattern (today used for cohort baseline/logical-design
    changes) one level down: a revision to one cohort member identifies potentially affected
    already-approved Stage 1-3 artifacts; only those identified are marked for reassessment, every
    other approval remains current at its already-approved version.
  - **Versioned cohort membership for deferred/removed features**, reusing the section's own
    existing membership-versioning pattern: a feature deliberately removed, deferred, split into
    another cohort, or abandoned after a partial Wave Gate requires an explicit membership-version
    revision, never silent exclusion. Architecture Synthesis proceeds once every *current* member of
    the cohort's current approved membership version has passed Stage 3 — so one deferred feature
    cannot indefinitely block synthesis for the rest.
- `01-intent.md` — the existing "Ambiguity Detection" section gets one added requirement: every
  open question is answered as (a) a proposed answer with rationale, (b) bounded alternatives with
  a recommendation, or (c) no defensible proposal yet with the missing decision/evidence named
  explicitly — never a forced, manufactured answer.
- `02-contract.md` — the existing "Ambiguity Detection" section (a hard STOP-and-return-to-Stage-1
  mechanism, not an open-question flow) keeps its STOP unchanged; its existing UNKNOWN-marker
  practice gets the same three-form requirement, so each marker is actionable rather than a bare
  gap.
- **`03-event-schema.md` is explicitly out of scope for this requirement**: unlike the two files
  above, it has no existing "Ambiguity Detection" section to extend (verified by read-through and
  grep) — adding one would be a new section, not an extension, and is left as a candidate follow-up
  rather than done here.
- `backlog/UPG-0059-wave-gated-batch-review.md` (new) — Feature Thread brief; `depends_on:
  [UPG-0051, UPG-0058]`.
- `status/roadmap.md`, `backlog/features.md`, `status/self-development.md` — registration and
  dashboard bookkeeping for this change's own step transitions.

**Scope boundary — what stays the same:**

- `UPG-0057` (Controlled Plain English) is untouched. CPE precision level is explicitly not used as
  a proxy for draft-vs-final rigor anywhere in this change; if a cheap whole-cohort big-picture pass
  is wanted before full Stage 1-3 rigor, `00a-solution-discovery.md` (Solution Discovery) already
  serves that need and is unaffected by this change.
- `tools/reviewer/src` is not touched — review execution stays per-feature-per-stage, a description
  of the current reviewer architecture's packet-budget constraint (documented as operational
  guidance in `docs/reviewer-pipeline.md` if needed, not written into `dba-system.md`'s normative
  text), not a new doctrine principle.
- Architecture Synthesis's own 4-step pipeline (`UPG-0051`/`UPG-0058`: draft baseline, draft logical
  design, combined approval) is unchanged — this change only makes its existing "every cohort member
  passes Stage 3" precondition explicit against versioned membership, it does not touch the
  pipeline's steps themselves.
- No new Stage ID, no new mandatory gate beyond the existing per-feature Stage 1/2/3 gates, no
  Non-Negotiable Rule change.
- No code — pure doctrine and prompt text.

**Class:** downstream-doctrine
**Scope axis:** downstream doctrine only
**Backlog item:** `backlog/UPG-0059-wave-gated-batch-review.md`

---

## Acceptance Criteria

| # | Criterion | How it will be verified |
|---|---|---|
| 1 | `dba-system.md` defines a Wave Gate: batched drafting + one batched human session per stage-wave, with the approval decision itself remaining individual, per feature, per artifact version | Read-through of the new Wave Gate text; confirm it states approval stays per-feature/per-version, not a group decision |
| 2 | The Wave Gate definition explicitly allows partial outcomes (some features approved, some returned) and states a Wave Gate is never all-or-nothing | Read-through of the same section for the partial-outcome sentence |
| 3 | An independent-justification requirement is present: a feature's draft must be justified by its own approved upstream inputs; sibling drafts may inform but never become unstated authority | Read-through; confirm the "informs, never becomes unstated authority" distinction is stated, not just "drafts must be independent" |
| 4 | A traceability requirement for cohort-driven changes is present, without enumerating a specific storage location in doctrine text | Read-through of the harmonization rule; grep confirms no "review notes / cohort-check report / draft history" enumeration appears in `dba-system.md` itself |
| 5 | Progressive check definitions replace a uniform per-stage Column Check: Stage 1 has no cross-stage check; Stage 2 has an upstream-alignment check (Contract vs. that feature's approved Intent); Stage 3 has the complete Column Check (Event Schema vs. that feature's approved Contract and Intent) | Read-through of each stage's check definition; confirm Stage 1 explicitly states no cross-stage check is possible yet |
| 6 | The Stage 3 Column Check's scope covers more than actors/event ownership — explicitly scope, guarantees, exclusions, terminology, and lifecycle assumptions | Read-through of the Stage 3 Column Check definition for all five named concerns |
| 7 | Sequencing is explicit: draft → per-feature check (upstream-alignment or Column) → cohort Row Check → only then Wave Gate approval decisions; no feature is approved before the Row Check has run | Read-through of the sequencing text for the four-step order and the "no early approval" statement |
| 8 | Checks are worded as advisory-but-not-ignorable: no decision authority of their own, but an unresolved material contradiction makes an approval unsupported without an explicit human override + rationale | Read-through for both halves of this statement — advisory framing and the override-required consequence |
| 9 | Targeted reassessment is defined via a lightweight impact assessment, explicitly stated as reusing the existing cohort baseline/logical-design versioned-impact-assessment pattern, applied one level down to individual Stage 1-3 approvals | Read-through; confirm the new text cross-references the existing pattern rather than defining an unrelated new mechanism |
| 10 | Versioned cohort membership for deferred/removed/split/abandoned features is defined, explicitly stated as reusing the existing membership-versioning pattern; Architecture Synthesis proceeds once every *current* member of the current approved membership version has passed Stage 3 | Read-through; confirm the cross-reference to the existing membership-versioning rule and the "current member of current version" precondition wording |
| 11 | `01-intent.md`'s existing "Ambiguity Detection" section, and `02-contract.md`'s existing UNKNOWN-marker practice within its own (differently-shaped) "Ambiguity Detection" section, each gain the three allowed response forms (proposed answer + rationale; bounded alternatives + recommendation; no defensible proposal + named gap), with `02-contract.md`'s existing STOP-and-return-to-Stage-1 behavior unchanged. `03-event-schema.md` has no existing "Ambiguity Detection" section — this criterion does not apply to it, and no new section is added there in this change | Read-through of `01-intent.md` and `02-contract.md`'s sections; grep confirms `03-event-schema.md` has no such section, so nothing is claimed extended there |
| 12 | No new Stage ID, new Non-Negotiable Rule, or new mandatory gate beyond the existing per-feature Stage 1/2/3 gates is introduced | Read-through of the diff against `dba-system.md`'s Non-Negotiable Rules and Stage ID table; confirm neither is touched |
| 13 | Packet-budget mechanics, the CPE-as-wrong-knob argument, and other alternatives-considered reasoning do not appear in `dba-system.md`'s added text | Read-through of the added section; confirm it is purely operative, no discursive justification |
| 14 | `patterns/controlled-plain-english.md` and `UPG-0056`'s status-file convention are unchanged | `git diff --stat -- patterns/controlled-plain-english.md templates/optional-mechanism-status.yaml templates/conventions.md` empty |
| 15 | `tools/reviewer/src/*` is unchanged | `git diff --stat -- tools/reviewer/src` empty; `cargo test --release --manifest-path tools/reviewer/Cargo.toml` passes at the same count as before this change |
| 16 | Architecture Synthesis's own 4-step pipeline (baseline draft, logical design draft, combined approval) is textually unchanged beyond stating its existing Stage-3-across-the-cohort precondition against versioned membership | Read-through of the "gate sequence" numbered steps 4-6; confirm no step's content changed, only the membership-versioning precondition is made explicit |

---

## Implementation Notes

**`dba-system.md`.** Replaced the gate sequence's items 1-3 with the Wave-Gated versions: each now
states the Wave Gate applies optionally per stage, defines the progressive check for that stage
(no check at Stage 1; upstream-alignment at Stage 2; complete Column Check at Stage 3), and states
the sequencing/no-early-approval rule at Stage 3. Added a new "Wave Gates" paragraph immediately
before the gate sequence covering: batched drafting/session vs. individual decision,
independent-justification, traceable-harmonization, and the advisory-but-not-ignorable wording for
checks — all in one place since they apply uniformly across all three stages, rather than repeating
them in each numbered item. Added the versioned-cohort-membership sentence for
deferred/removed/split/abandoned features at the end of item 3 (the natural place, since it's the
precondition for item 4's Architecture Synthesis trigger). Added a new "Targeted reassessment of
Stage 1-3 approvals" paragraph directly after the existing "Cohort, baseline, and logical design
versioning" section, explicitly stating it reuses that same impact-assessment pattern one level
down — no new mechanism invented. Confirmed via diff: only the gate-sequence items were replaced
and two new paragraphs added; Non-Negotiable Rules and the Stage ID table are untouched.

**`01-intent.md`.** Extended the existing "Ambiguity Detection" section (which already lists
ambiguity categories to call out) with the three response-form requirement, and updated Step 4's
output-sequence item 4 to reference the same three forms explicitly.

**`02-contract.md`.** The existing "Ambiguity Detection" section here is a *different* mechanism
than 01-intent.md's — it's a hard STOP-and-return-to-Stage-1 when the intent itself is ambiguous,
not an open-question-with-proposed-answer flow. Rather than weaken that stop, extended its existing
UNKNOWN-marker practice: each marker now carries one of the three response forms instead of being a
bare gap, making the clarification request actionable. The STOP itself is explicitly stated as
unchanged.

**`03-event-schema.md` — discovered discrepancy, not implemented.** Change Intent and AC11 assumed
this file has an existing "Ambiguity Detection" section to extend, matching `01-intent.md` and
`02-contract.md`. It does not — grep and read-through confirm no such section exists anywhere in
`03-event-schema.md`; the only related text is a review-package placeholder line ("Known tensions:
from contract ambiguities or coverage gaps, or 'none'") that carries forward context from earlier
stages, not a native ambiguity-detection mechanism of its own. Adding a brand-new section would be
scope beyond "extend the existing," which is what Steps 1-2 were reviewed and approved against.
Left untouched; the gap is a legitimate candidate for a small follow-up UPG, not silently patched
here or silently ignored. AC11 is revised at Reconciliation to reflect this accurately rather than
falsely claimed as a full PASS across all three files.

**Scope-boundary verification (ahead of formal Reconcile):**
- `git diff --stat -- patterns/controlled-plain-english.md templates/optional-mechanism-status.yaml templates/conventions.md config/writing-discipline.yaml` — empty, confirming CPE/`UPG-0056` untouched.
- `git diff --stat -- tools/reviewer/src` — empty.
- `cargo test --release --manifest-path tools/reviewer/Cargo.toml` — 182 passed, 0 failed, same
  count as before this change.
- `git diff dba-system.md` — only the gate-sequence rewrite and the two new paragraphs; no
  Non-Negotiable Rule or Stage ID table line touched.

No other out-of-scope items discovered.

---

## Reconciliation

**Acceptance verification:**

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Wave Gate: batched drafting/session, individual per-feature/per-version approval | PASS | `dba-system.md`'s new "Wave Gates" paragraph: "the approval decision remains individual, per feature, per artifact version, exactly as Non-Negotiable Rule #1 requires" |
| 2 | Partial outcomes explicit; never all-or-nothing | PASS | Same paragraph: "A Wave Gate is never all-or-nothing — some features may be approved while others are returned for revision in the same sitting" |
| 3 | Independent-justification: informs, never unstated authority | PASS | Same paragraph: "sibling drafts... may inform comparison and shared-domain awareness, but must never become unstated authority for a decision the feature's own approved inputs don't support" |
| 4 | Traceability requirement, no storage-location enumeration | PASS | "A change to a draft driven by comparing it against siblings must be traceable in the review record" — no enumerated location; grep confirms no "review notes / cohort-check report / draft history" list in `dba-system.md` |
| 5 | Progressive checks: Stage 1 none, Stage 2 upstream-alignment, Stage 3 complete Column Check | PASS | Gate-sequence item 1: "No cross-stage check is possible yet"; item 2: "an upstream-alignment check against that feature's own approved Intent"; item 3: "the complete Column Check... compared against that feature's own approved Contract *and* Intent" |
| 6 | Stage 3 Column Check covers scope, guarantees, exclusions, terminology, lifecycle | PASS | Item 3: "scope, guarantees, exclusions, terminology, and lifecycle assumptions, not only actors and event ownership" |
| 7 | Sequencing: draft → check → Row Check → approval; no early approval | PASS | Item 3: "Sequence: draft every feature's Stage 3 → each feature's Column Check → the cohort's Event Cohort Check... → only then does the Wave Gate decide every individual Stage 3 approval. No feature is approved before the Event Cohort Check has run." |
| 8 | Advisory-but-not-ignorable wording | PASS | "Wave Gates" paragraph: "Checks... carry no decision authority of their own — the human still decides every approval — but an unresolved material contradiction a check surfaces makes that approval unsupported unless the human explicitly overrides it with a recorded rationale" |
| 9 | Targeted reassessment reuses existing impact-assessment pattern, one level down | PASS | New "Targeted reassessment of Stage 1-3 approvals" paragraph: "The same impact-assessment pattern above applies one level down..." — direct cross-reference, no new mechanism |
| 10 | Versioned cohort membership reuses existing pattern; "current member of current version" wording | PASS | Item 3's new paragraph: "requires an explicit cohort-membership revision (a new membership version, per the versioning rule below)... Architecture Synthesis proceeds once every *current* member of the cohort's current approved membership version has passed Stage 3" |
| 11 | Three response forms in `01-intent.md`/`02-contract.md`; `03-event-schema.md` explicitly out of scope (revised at Step 3 R1) | PASS | `01-intent.md`'s "Ambiguity Detection" section and Step 4 item 4; `02-contract.md`'s UNKNOWN-marker extension with STOP unchanged; `03-event-schema.md` untouched — grep confirms no "Ambiguity Detection" heading exists there, matching Change Intent/AC11's revised, accurate scope |
| 12 | No new Stage ID, Non-Negotiable Rule, or mandatory gate | PASS | Non-Negotiable Rules (5 rules) and Stage ID table both grep-unchanged; the only new mandatory element is the same "single new mandatory gate" combined-approval step this section already had before this change |
| 13 | No packet-budget/CPE-argument/discursive reasoning in `dba-system.md`'s added text | PASS | The one `packet` occurrence in the file predates this change (CPE call-site map, `UPG-0057`); the new text is purely operative — no rationale, no alternatives-considered prose |
| 14 | CPE / `UPG-0056` files unchanged | PASS | `git diff --stat -- patterns/controlled-plain-english.md templates/optional-mechanism-status.yaml templates/conventions.md config/writing-discipline.yaml` empty |
| 15 | `tools/reviewer/src` unchanged; 182 tests | PASS | `git diff --stat -- tools/reviewer/src` empty; `cargo test --release --manifest-path tools/reviewer/Cargo.toml` — 182 passed, 0 failed |
| 16 | Architecture Synthesis's 4-step pipeline textually unchanged beyond the membership precondition | PASS | Gate-sequence items 4-6 (baseline draft, logical design draft, combined approval) are byte-identical to the pre-change text; only item 3 gained the new membership-precondition paragraph |

**Consistency sweep (grep):** `git status --short` on every file named in Change Intent's "What
changes" shows exactly those files touched, nothing else. No stray reference to "03-event-schema.md
extends Ambiguity Detection" remains anywhere in the change record (the Step 3 R1 finding's fix
verified by re-grep). No remaining mention of the rejected original proposal (zero-gate batch
review) anywhere in `dba-system.md`'s new text.

**Findings scope-triage (all Step 1-3 review rounds, this CHG):**

| Finding | Round | Classification | Resolution |
|---|---|---|---|
| Bookkeeping diff included an unrelated `UPG-0057` status promotion, not declared in scope | Step 1 R1 | IN-SCOPE BLOCKER | Fixed — split into standalone commit `28d934f` |
| Change Intent/AC11 still claimed `03-event-schema.md` has an existing Ambiguity Detection section extended by this change, contradicting Implementation Notes' own discovery that it doesn't | Step 3 R1 | IN-SCOPE BLOCKER | Fixed — Change Intent and AC11 rewritten to state the discovery accurately |
| The backlog brief (`backlog/UPG-0059-wave-gated-batch-review.md`) still claimed `03-event-schema.md` was extended, the same false claim already fixed in the change record but never propagated to the backlog artifact | Step 4 R1 | IN-SCOPE BLOCKER | Fixed — backlog brief's "Upgrade" and "Scope" sections corrected to match |
| The backlog brief's Findings Tracked table only recorded the Step 1 finding, missing the Step 3 finding that was fixed | Step 4 R1 | SELF-REFERENCE / REVIEW-BOOKKEEPING | Fixed — Step 3 finding row added |

No OUT-OF-SCOPE BACKLOG, REJECTED findings this CHG. **One follow-up
candidate identified during implementation** (not spawned as a new `UPG-####` — noted here per the
Review-Fix Rule, since it's a discovery, not a finding from a review round): `03-event-schema.md`
has no "Ambiguity Detection" equivalent to `01-intent.md`/`02-contract.md`'s; adding one would be a
legitimate small follow-up but is out of this change's scope (extending an existing section, not
authoring a new one).

**Completion statement.** All 16 Acceptance Criteria pass. `tools/reviewer/src` and CPE
(`patterns/controlled-plain-english.md`, `UPG-0056`'s status-file convention) are unchanged,
confirming the scope boundary held. `dba-system.md`'s Multi-Feature Architecture Synthesis Gate
now defines Wave Gates, progressive Row/Column checks with explicit sequencing, advisory-but-not-
ignorable check wording, targeted reassessment, and versioned-membership handling for deferred/
removed features — all as lean, purely operative doctrine text, with the fuller rationale kept in
this change record and the approved plan
(`/home/rimo/.claude/plans/calude-consider-this-inputs-steady-pnueli.md`), not in `dba-system.md`
itself. `01-intent.md` and `02-contract.md` carry the three ambiguity-response forms;
`03-event-schema.md` is honestly left untouched with the gap noted as a follow-up candidate. With
this change accepted, **`UPG-0059` is complete**.
