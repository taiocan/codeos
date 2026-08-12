---
feature_id: UPG-0037
slug: downstream-default-stage-review
title: Default Advisory Review Across the Full Downstream DBA Workflow
status: COMPLETE
priority: P1
depends_on: [UPG-0003, UPG-0032, UPG-0014, UPG-0015]
related_features: [UPG-0007, UPG-0036]
supersedes: []
superseded_by: []
---

# Upgrade: downstream-default-stage-review — Default Advisory Review Across the Full Downstream DBA Workflow

**Priority**: P1
**Status**: COMPLETE
**Type**: downstream-doctrine
**Related**: reviewer-decision-brief (UPG-0003), Rust reviewer engine (UPG-0032), Reviewer
Agent with Full Diff Access (UPG-0014), Bind stage approval to reviewed provenance (UPG-0015),
Solution Discovery (UPG-0007), Stack Manifest Dogfooding (UPG-0036)

## Problem

Codeos's own self-development (`CLAUDE.md`) makes advisory Codex review mandatory at every
step of the 4-step loop. The downstream doctrine (`dba-system.md`) has no equivalent default
— its only reviewer mention is `prompts/pipeline-reviewer.md`, an optional, manually-pasted
prompt covering only Stages 1-9, structurally different from the Rust `codeos-reviewer`
pipeline self-dev now relies on by default. This is the same rigor asymmetry UPG-0036 fixed
at the toolkit layer (stack manifest / `check-drift`), now surfacing at the doctrine layer.

The gap is not capability: `tools/reviewer/src/packet.rs` already hardcodes complete
`stage_checks()`/`stage_expected()` tables for downstream stages `"1"` through `"9"`, and its
"DBA RULES RELEVANT TO THIS STAGE" section is already generic DBA language. The mechanism has
just never been made the default in `dba-system.md`'s stated workflow, and doesn't yet cover
the steps before Stage 1: Solution Discovery, Feature Brief, and Onboarding.

This is not hypothetical: `/home/rimo/projects/FundFlow` is a real, live downstream project
(`.codeos` symlinked to this repo, `reviewer.toml` already configured with `codex`/`high`).
Because `.codeos` is a symlink rather than a copy, any change to `dba-system.md` is
immediately live for FundFlow on its next session.

## Upgrade

Make the Rust `codeos-reviewer` pipeline the default, structured advisory review at every
reviewable gate across the full downstream workflow — not only the numbered Stage 1-9 loop,
but also Solution Discovery, Feature Brief, and Onboarding. Advisory only, never
auto-blocking: Non-Negotiable Rule #1 ("every stage transition requires explicit human
approval") is unchanged.

## Scope

`dba-system.md`'s full workflow presentation (Solution Discovery through Stage 10), its
supporting reviewer tooling (`tools/reviewer/src/packet.rs`'s stage tables), and the
downstream-facing docs/prompts that describe review usage. No `CLAUDE.md` or self-dev
governance changes of any kind.

## Proposed artifact(s)

- Updated `dba-system.md`: a unified stage-identifier sequence (`discovery`, `brief`,
  `onboarding`, `1` through `10`) used consistently for documentation order and as the
  `<stage>` argument to `codeos-reviewer review <feature_id> <stage>`; a default
  advisory-review line at every reviewable gate; a Review Waiver practice for when reviewer
  tooling isn't available.
- `tools/reviewer/src/packet.rs`: new `stage_checks()`/`stage_expected()` match arms for
  `"discovery"`, `"brief"`, `"onboarding"`, and `"10"` (numeric `"1"`-`"9"` already exist).
- `prompts/pipeline-reviewer.md`: reframed opening line clarifying it's an optional
  supplementary independent-assessor pass, not a replacement for the default review.
- `docs/reviewer-pipeline.md`: a new downstream-usage section (today written entirely in
  self-dev terms — `selfdev-step-N` arguments, `UPG-####` ids assumed).
- `prompts/00b-solution-discovery.md`: a small addition stating that its output gets
  reviewed when carried into a Feature Brief or Stage 1 Intent (the session itself stays
  optional and non-gating).

## Design notes

**Default review, no new triage system.** Every reviewable gate gets advisory review by
default. No size-based `trivial`/`backlog-only` taxonomy is introduced — `dba-system.md` has
no existing triage concept, and inventing one only to gate review-mandatoriness would be
disproportionate scope creep. This matches `dba-system.md`'s existing "same loop regardless
of size" philosophy.

**Round budget** (the mechanism from self-dev's review cadence, not its `PROFILE-N` naming):
round 1 runs before the human gate; rounds 2-3 are allowed for fixes or material deltas;
after 3 rounds, stop and require a human decision. Described entirely in `dba-system.md`'s
own plain language — no `PROFILE-N` vocabulary appears anywhere downstream; that's
self-dev-internal tooling (`prompts/codeos-self-dev.md`).

**Solution Discovery — conditional trigger.** The session itself remains optional and
non-authoritative, unchanged from UPG-0007's original guardrail. If its output is actually
carried into a Feature Brief or Stage 1 Intent, that handoff gets a default advisory review
pass (or an explicit waiver). A Discovery session whose output is never carried forward is
simply never reviewed.

**Unified stage sequence — vocabulary only.** `discovery -> brief -> onboarding -> 1 -> 2 ->
3 -> 4 -> 5 -> 6 -> 7 -> 8 -> 9 -> 10` is identifier vocabulary for the reviewer's `<stage>`
argument and `dba-system.md`'s documentation ordering. It is explicitly not a claim that
Onboarding (Session Type D) is a mandatory linear step every feature passes through --
Onboarding remains an alternate entry point for bootstrapping an existing codebase lacking
DBA artifacts, used instead of Discovery/Brief for that scenario, not sequentially after
them. `dba-system.md`'s prose states this explicitly. Prompt filenames (`01-intent.md`, etc.)
are unchanged -- this is stage-identifier vocabulary, not a file rename.

**`prompts/pipeline-reviewer.md` stays, reframed.** It remains available as an optional,
independent critical-assessor pass for a second opinion -- structurally distinct from the
Rust engine's acceptance-criteria-bound review, and explicitly not a replacement for the
default review log/packet path.

**Review Waiver, precisely scoped.** If reviewer tooling is unavailable or not configured,
the human records an explicit waiver with a reason and may continue -- neither silently
skipping nor hard-blocking the whole project over missing reviewer setup. The waiver applies
only to the advisory review run; it never waives Non-Negotiable Rule #1's human-approval
gate. Concretely a plain review-log/decision-record entry ("Review waiver: reviewer not
configured; proceeding without advisory review at Stage N. Reason: ..."), not a new CLI flag
in this version -- UPG-0015's `--override <RATIONALE>` is the closest existing precedent in
spirit but fires on a built-but-deficient packet, not on "no packet at all," so it doesn't
fit mechanically. A future `--waive-review` convenience is a possible follow-up, out of scope
here.

## Value

High. Closes a real rigor asymmetry between how Codeos treats its own development and what
it defaults downstream projects into, across the entire workflow rather than only the
numbered loop. Directly extends UPG-0036's dogfooding principle from the toolkit layer to
the doctrine layer.

## Risk

- Downstream projects may not have `codeos-reviewer` built/configured -- mitigated by the
  Review Waiver: never a silent skip, never a hard block.
- Scope creep into inventing new downstream governance (triage classes, a profile system)
  beyond "add the review step" -- guarded against explicitly in Design notes above.
- `prompts/pipeline-reviewer.md` could read as redundant with the Rust pipeline if the
  relationship isn't stated clearly -- addressed by the reframed opening line.

## Guardrail

Advisory only, never auto-blocking -- human approval remains the sole gate authority at
every stage, unchanged. No new downstream triage/profile system. No `CLAUDE.md` or
self-dev-governance edits. No artifact path, schema, filename, or stage-output changes --
verified directly against FundFlow's existing `reviewer.toml`, `features/registry.yaml`, and
`reviews/` layout.

## DBA-philosophy note

Directly touches the downstream doctrine's advisory-review posture -- `downstream-doctrine`
scope class in `CLAUDE.md`'s triage table, requiring downstream-compatibility acceptance
criteria, cross-reference grep verification, and reviewer scope-triage in addition to the
normal 4-step loop.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260705-002 | `changes/UPG-0037__CHG-20260705-002__downstream-default-stage-review.md` | Default advisory review across the full downstream DBA workflow | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|
| REV__UPG-0037__CHG-20260705-002__S1__R1 | CHG-20260705-002 | 1-Intent | R1 | NO OBJECTION (evidence C, non-blocking) |
| REV__UPG-0037__CHG-20260705-002__S2__R1 | CHG-20260705-002 | 2-Acceptance | R1 | CHANGES ADVISED (AC-10 false no-build guarantee) |
| REV__UPG-0037__CHG-20260705-002__S2__R2 | CHG-20260705-002 | 2-Acceptance | R2 | NO OBJECTION |
| REV__UPG-0037__CHG-20260705-002__S3__R1 | CHG-20260705-002 | 3-Implement | R1 | CHANGES ADVISED (broken-shim docs; PROFILE-N leak; undeclared UPG-0038 bookkeeping) |
| REV__UPG-0037__CHG-20260705-002__S3__R2 | CHG-20260705-002 | 3-Implement | R2 | CHANGES ADVISED (AC-4 grep scope too broad) |
| REV__UPG-0037__CHG-20260705-002__S3__R3 | CHG-20260705-002 | 3-Implement | R3 | CHANGES ADVISED (0 blockers; SECRET_REDACTION coverage only — accepted at budget) |
| REV__UPG-0037__CHG-20260705-002__S4__R1 | CHG-20260705-002 | 4-Reconcile | R1 | CHANGES ADVISED (0 blockers; same SECRET_REDACTION cause — accepted) |

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
| UPG-0038 | `codeos-review.sh` resolves the binary via the calling repo's git root, not through the `.codeos` symlink — breaks when invoked from any real downstream project (discovered against FundFlow); pre-existing, out of scope for UPG-0037 | Step 3 AC-10 verification |
