# Self-Development Change: UPG-0054__CHG-20260719-004 — contract-to-implementation-failure-boundary

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
feature_id: UPG-0054
primary_feature_id: UPG-0054
change_id: CHG-20260719-004
slug: contract-to-implementation-failure-boundary
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0054
related_features: [UPG-0052]
review_series: RVS__UPG-0054__CHG-20260719-004__S4
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

Approved Stage 2 failure classifications define observable behavioral outcomes, but there's no
disciplined convention distinguishing rich internal/technical errors from contract-visible
failure classifications during implementation. Without it, Stage 4 work risks silently converting
unexpected failures into misleading business outcomes, leaking internal error detail as a
contract-visible failure, or treating a Contract-approved classification as sufficient on its own
to justify emitting an event — when the approved Stage 3 Event Schema is what actually authorizes
an emission (`dba-system.md` Non-Negotiable Rules #2 and #4 already establish the two halves of
this constraint; this change makes their interaction explicit for failures specifically). This
item was split out of `UPG-0052` during its Step 1 review as scope drift — it's a cross-language
Stage 4/5 concern, not an implementation-profile concern.

**Scope correction from the backlog brief:** the brief's Scope section names only `dba-system.md`
and `patterns/rust-project-structure.md`. This change also touches `prompts/04-implement.md` and
`prompts/05-tests.md` — the actual operative Stage 4/5 prompts. Doctrine text with no wiring into
the prompts that carry it out is exactly the failure mode `UPG-0051` was proposed to react to
(`patterns/rust-project-structure.md` sat orphaned from `dba-system.md` and every prompt for an
unknown period); adding this guidance only to `dba-system.md` and the Rust pattern file, without
touching Stage 4/5's own operative text, would repeat that mistake immediately after fixing it.

**What changes:**

1. `dba-system.md`: new **"Contract-to-Implementation Failure Boundary"** section, placed after
   "Implementation Profile" and before "What You Do at Each Stage" (same placement pattern as this
   session's other cross-cutting Stage-adjacent guidance). States:
   - Two boundaries, kept distinct: the **behavioral boundary** (observable business/governance
     outcomes — defined by the Stage 2 Contract's Failure Classifications *and* the Stage 3 Event
     Schema together) and the **technical API boundary** (implementation-internal error
     propagation — a function may legitimately return/propagate storage, serialization, I/O, or
     other internal error types).
   - The rule: only failure classifications approved by the Stage 2 Contract may be exposed as
     classified behavioral outcomes. A failure event may be emitted only when that event is *also*
     present in the approved Stage 3 Event Schema — a Contract-approved classification alone does
     not authorize emitting it as an event (this is Non-Negotiable Rules #2 and #4's existing
     constraint, made explicit for the failure case specifically, not a new rule).
   - Internal/technical errors may propagate through richer implementation error types, but must
     remain distinguishable from contractual outcomes and must never be silently mapped to one.
     Every internal-to-contractual mapping is explicit and reviewable.
   - Stage 5 verifies all four directions: approved contractual failures produce the correct
     observable classification; emitted failure events conform to the approved Stage 3 schema;
     technical failures never masquerade as approved behavioral failures; no unapproved failure
     event is emitted.
   - No universal error library or single canonical enum is prescribed — language-neutral; the
     Rust realization lives in `patterns/rust-project-structure.md`.
2. `patterns/rust-project-structure.md`: new **"Error Boundary Convention"** section — a Rust
   realization: feature-internal error types may be as rich as needed (no crate mandated —
   `thiserror`, `anyhow`, or plain enums are all acceptable); a separate, explicit
   mapping (a function or match arm set) converts internal errors to only the approved failure
   classifications; anything not matched by that mapping propagates unmapped (consistent with
   `04-implement.md`'s existing "No speculative error handling" rule — it does not become a
   misleading approved failure).
3. `prompts/04-implement.md`: tighten the existing **"No speculative error handling"** constraint
   (currently: "Only handle failure modes explicitly listed in the contract's Failure
   Classifications. Other errors propagate as uncaught exceptions.") to state the two-boundary
   distinction and the Contract-classification-alone-is-not-sufficient rule explicitly. Add a
   **Failure Mapping Table** to the Stage 4 output format (alongside the existing Contract
   Satisfaction Table and Event Emission Table): `Internal Error | Contract Failure
   Classification | Emitted Event | Mapping Site` — giving the "explicit and reviewable" mapping
   a concrete output location, the same pattern `UPG-0052` used for provenance recording.
4. `prompts/05-tests.md`: extend the existing **"Failure Mode Tests"** section (currently only
   asserts the correct FAILURE event is emitted for each named contract failure) to also assert
   the negative direction: a technical/internal failure does not produce an approved FAILURE
   event, and no unapproved failure event type appears during failure-path tests.

**Scope boundary — what stays the same:**

- No Non-Negotiable Rule is added or reworded — rules #2 and #4 already establish the underlying
  constraint; this change makes their interaction explicit for failures and builds Stage 4/5
  verification around it, it does not create new authority.
- No specific Rust error-handling crate or single canonical enum is mandated, anywhere.
- No change to Stage 2 (`02-contract.md`, `templates/contract.md`) or Stage 3
  (`03-event-schema.md`) — the Failure Classifications table and its "needs an event in the event
  schema" comment already exist and are sufficient preconditions; this change is purely about
  Stage 4/5 implementation and test discipline.
- No Implementation Profile mechanism content — that's `UPG-0052`, already complete; this shares
  only the same Rust pattern file, no dependency either direction.
- No downstream project's actual `.codeos/` symlink or generated files are touched.

**Class:** downstream-doctrine
**Scope axis:** downstream doctrine only
**Backlog item:** backlog/UPG-0054-contract-to-implementation-failure-boundary.md

---

## Acceptance Criteria

| # | Criterion | How it will be verified |
|---|---|---|
| 1 | No Non-Negotiable Rule is added or reworded. | `git diff -- dba-system.md` shows zero lines changed within "The Non-Negotiable Rules" section. |
| 2 | The two boundaries (behavioral vs. technical API) are stated as distinct in `dba-system.md`. | Read-through of the new section confirms both are named and defined separately. |
| 3 | Classification-alone-insufficient rule: a Contract-approved failure classification does not by itself authorize emitting an event — the event must also be present in the approved Stage 3 Event Schema. | Grep the new section for this exact two-part condition (Contract *and* Schema, not either/or). |
| 4 | Internal/technical errors may propagate through richer implementation error types but must remain distinguishable from contractual outcomes, never silently mapped. | Read-through of the new `dba-system.md` section and the new `patterns/rust-project-structure.md` section state this consistently. |
| 5 | Stage 5 verifies all four directions: approved failures → correct classification; emitted events conform to Stage 3 schema; technical failures never masquerade as approved failures; no unapproved failure event emitted. | `dba-system.md`'s new section lists all four; `prompts/05-tests.md`'s extended Failure Mode Tests section implements at least the negative-direction pair (masquerading, unapproved events) as explicit assertions. |
| 6 | No universal error library or single canonical enum is prescribed anywhere — across all four touched files, not just the two doctrine/pattern files. | Grep all four files (`dba-system.md`, `patterns/rust-project-structure.md`, `prompts/04-implement.md`, `prompts/05-tests.md`) for crate names (`thiserror`, `anyhow`, etc.) or "one enum"/"single enum" phrasing — any mention found in any of the four must be framed as "acceptable" or an example, never as "required." |
| 7 | Rust realization states an explicit internal-error-to-classification mapping, with unmatched errors propagating unmapped (never becoming a misleading approved failure). | Read-through of `patterns/rust-project-structure.md`'s new "Error Boundary Convention" section. |
| 8 | `prompts/04-implement.md`'s "No speculative error handling" rule is tightened to state the two-boundary distinction and the classification-alone-insufficient rule, without contradicting its existing text. | Read the revised rule; confirm the original "other errors propagate as uncaught exceptions" intent is preserved, not replaced. |
| 9 | A Failure Mapping Table (`Internal Error \| Contract Failure Classification \| Emitted Event \| Mapping Site`) is added to Stage 4's output format, alongside the existing Contract Satisfaction Table and Event Emission Table. | Read-through of `prompts/04-implement.md`'s Output Format section confirms the new table's presence and column names. |
| 10 | `prompts/05-tests.md`'s Failure Mode Tests section gains the negative-direction assertions (technical failure does not produce an approved FAILURE event; no unapproved failure event type appears during failure-path tests) without removing the existing positive-direction assertion. | Read-through confirms both the original assertion and the two new ones are present. |
| 11 | No changes to Stage 2 (`02-contract.md`, `templates/contract.md`) or Stage 3 (`03-event-schema.md`). | `git diff` shows zero changes to these three files. |
| 12 | No collision with `UPG-0052`'s existing content in `prompts/04-implement.md`: the tightened error-handling rule and the new Failure Mapping Table are additive to/alongside `UPG-0052`'s Implementation Profile consultation check and provenance-recording line, not a replacement of either. | Read-through confirms `UPG-0052`'s check (lines ~19-32, ~34-57 per its own change record) and Review Package provenance line are unchanged, byte-identical. |
| 13 | **Downstream-compatibility** (required for `downstream-doctrine` class): no prompt/template filename is renamed; the new section is referenced from wherever `dba-system.md`'s structure would naturally point to it (no orphaning — the exact failure mode this change's own scope correction was reacting to). | Grep sweep: confirm `dba-system.md`'s new section doesn't introduce any dangling reference, and that `patterns/rust-project-structure.md`'s realization is discoverable from `dba-system.md`'s new section (a cross-reference, not just parallel unlinked text). |

---

## Implementation Notes

<!-- Summary only — the git diff is the source of truth. -->

All 4 files from Step 1's "What changes" list were edited as planned. No scope creep.

**Key decisions:**
- `dba-system.md`'s new section placed after "Implementation Profile," before "What You Do at
  Each Stage" — same placement pattern as this session's other cross-cutting additions.
- `patterns/rust-project-structure.md`'s "Error Boundary Convention" placed after "Replay Test
  Pattern," before "Recommended Toolchain/Lint Baseline" (both are Rust-realization sections for
  cross-cutting `dba-system.md` guidance, grouped together).
- `prompts/04-implement.md`: the existing "No speculative error handling" rule's original
  *intent* is preserved (only handle contract-listed failure modes; other errors propagate
  uncaught) — this is not a verbatim-sentence preservation: the second sentence was generalized
  from "Other errors propagate as uncaught exceptions" to also cover richer error types in
  languages that have them, then the new boundary/mapping guidance was appended (AC8 requires the
  *rule* not be contradicted, not that its exact wording be frozen). The new Failure Mapping Table
  is inserted as output step 4, renumbering the Review Package step (4→5) and the final
  `AWAITING HUMAN APPROVAL` line (5→6).
- `prompts/05-tests.md`: the original Failure Mode Tests assertions (correct FAILURE event,
  unchanged state) are preserved verbatim, with the two negative-direction assertions appended
  (AC10); a matching checklist item added to the Stage 5 completion checklist.

**Discovered but out of scope — flagged, not fixed here:** while inserting into
`dba-system.md`'s "Implementation Profile" section (immediately before this change's insertion
point), noticed its "Codeos's default policy" paragraph says automatic `dba-init.sh` scaffolding
"is tracked separately and not yet built" — this is now stale, since `UPG-0053` (`CHG-20260719-003`)
shipped that scaffolding after `UPG-0052` was written. This is a one-line factual correction
unrelated to this change's declared scope (Step 1 did not include touching the Implementation
Profile section) — flagging for the human to fix as a trivial correction, not bundling it into
this change's diff.

**Nothing else was deferred or discovered out-of-scope during implementation.**

---

## Reconciliation

**Acceptance verification:**

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | No Non-Negotiable Rule changed | PASS | `git diff -- dba-system.md \| grep "^-" \| grep -v "^---" \| wc -l` → `0` (zero deletions anywhere in the file). |
| 2 | Two boundaries stated distinctly | PASS | `dba-system.md:344-346`: "**behavioral boundary**... **technical API boundary**". |
| 3 | Classification-alone-insufficient rule | PASS | `dba-system.md:353`: "not authorize emitting it as an event... Non-Negotiable Rules #2 and #4." |
| 4 | Internal errors may propagate richly, distinguishable | PASS | `dba-system.md:358`: "must remain distinguishable from contractual outcomes and must never be silently mapped to one." |
| 5 | Stage 5 four-direction verification | PASS | `dba-system.md`'s four numbered directions; `prompts/05-tests.md:48,52`: "No masquerading" / "No unapproved events" assertions added. |
| 6 | No crate/enum mandated across all 4 files | PASS | Grep across all four: `dba-system.md:368` and `patterns/rust-project-structure.md:311-312` mention crate names only as *acceptable examples*; `prompts/04-implement.md` and `prompts/05-tests.md` have zero crate-name matches. |
| 7 | Explicit mapping, unmatched propagates unmapped | PASS | `patterns/rust-project-structure.md:329`: "Everything else propagates unmapped." |
| 8 | `04-implement.md` rule tightened correctly, no contradiction with the corrected Rust wording | PASS (after 2 review-round fixes) | Current text: "Two separate approvals gate an emitted failure event... the schema authorizes event types, not classification names" — matches `patterns/rust-project-structure.md`'s corrected wording exactly. |
| 9 | Failure Mapping Table added to Stage 4 output | PASS | `prompts/04-implement.md:154`: "4. Present a **Failure Mapping Table**". |
| 10 | Failure Mode Tests extended, original preserved | PASS | `prompts/05-tests.md:42` (original) + lines 48/52 (new negative-direction assertions), both present. |
| 11 | No Stage 2/3 file changes | PASS | `git diff --stat -- prompts/02-contract.md templates/contract.md prompts/03-event-schema.md` → empty output. |
| 12 | No collision with `UPG-0052`'s content | PASS | `prompts/04-implement.md`'s cohort eligibility check (lines 19-32) and "Implementation Profile applied" line (173) read byte-identical to their state before this change. |
| 13 | Downstream-compatibility | PASS | Full sweep below. |

All 13 criteria PASS.

**AC13 downstream-compatibility sweep — full output:**

```
$ grep -oP '\.codeos/prompts/\K[a-zA-Z0-9_-]+\.md' dba-system.md | sort -u | while read f; do
    test -f "prompts/$f" && echo "OK: $f" || echo "MISSING: $f"; done
OK: 00a-solution-discovery.md   OK: 00b-feature-brief.md   OK: 00c-onboarding.md
OK: 00-session-end.md          OK: 00-session-start.md    OK: 01-intent.md
OK: 02-contract.md              OK: 03b-architecture-synthesis.md
OK: 03-event-schema.md          OK: 04-implement.md        OK: 05-tests.md
OK: 06-observe.md               OK: 07-reconcile.md        OK: 08-replay.md
OK: 09-refine.md                OK: 10-arch-refine.md      OK: pipeline-reviewer.md

$ grep -oP '\.codeos/templates/\K[a-zA-Z0-9_.-]+' dba-system.md | sort -u | while read f; do
    test -f "templates/$f" && echo "OK: $f" || echo "MISSING: $f"; done
OK: architecture-baseline.md   OK: arch-refinement.md   OK: codebase-digest.md
OK: contract.md                 OK: conventions.md        OK: event-schema.md
OK: feature-brief.md            OK: feature-spec.md       OK: handoff.md
OK: implementation-profile.yaml OK: intent.md             OK: refinement.md
OK: review-file.md              OK: review-package.md

$ grep -oP '\.codeos/patterns/\K[a-zA-Z0-9_.-]+' dba-system.md | sort -u | while read f; do
    test -f "patterns/$f" && echo "OK: $f" || echo "MISSING: $f"; done
OK: rust-project-structure.md
```

Zero `MISSING:` lines across all three sweeps.

**Consistency sweep:** no stale references, orphaned links, or drift found. The new
`dba-system.md` section cross-references `patterns/rust-project-structure.md` → "Error Boundary
Convention" and vice versa (bidirectional, confirmed during Step 3).

**Findings scope-triage:**

| Finding | Triage | Action |
|---|---|---|
| Step 3 R1: Rust pattern blurred classification-approval vs. event-schema-authorization into one condition | IN-SCOPE BLOCKER | Fixed — reworded to state two separate, independent approvals |
| Step 3 R1: Implementation Notes falsely claimed "verbatim" sentence preservation | IN-SCOPE BLOCKER | Fixed — reworded to accurately describe intent-preserved-but-wording-generalized |
| Step 3 R2: the same classification/event-schema blur was still present in `prompts/04-implement.md` (fixed in the Rust pattern but not the prompt that inspired it) | IN-SCOPE BLOCKER | Fixed — same two-separate-approvals wording applied to `04-implement.md` |
| Discovered during implementation: `dba-system.md`'s Implementation Profile section says `dba-init.sh` scaffolding is "not yet built," now stale since `UPG-0053` shipped it | OUT-OF-SCOPE BACKLOG | Not fixed here (outside this change's declared scope) — flagged to the human in Implementation Notes as a trivial one-line correction |

---
