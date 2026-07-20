---
feature_id: UPG-0055
slug: reviewer-architecture-synthesis-stage-support
title: Reviewer Support for the `architecture-synthesis` Stage ID
status: PROPOSED
priority: P3
depends_on: []
related_features: [UPG-0051, UPG-0049]
supersedes: []
superseded_by: []
---

# Upgrade: reviewer-architecture-synthesis-stage-support — Reviewer Support for the `architecture-synthesis` Stage ID

**Priority**: P3
**Status**: PROPOSED
**Type**: script-tooling

## Problem

`UPG-0051` (Multi-Feature Architecture Synthesis Gate) introduced a new downstream stage id,
`architecture-synthesis`, documented in `dba-system.md` and used by
`prompts/03b-architecture-synthesis.md`. `codeos-reviewer` (`tools/reviewer/src/packet.rs`)
accepts an arbitrary `stage` string, but its `stage_expected`/`stage_checks` functions have no
match arm for `"architecture-synthesis"` — confirmed by direct code read during `UPG-0051`'s
Step 1/4 (`packet.rs:661-696`). An invocation with this stage id falls through to the generic,
untailored fallback branches (`"(no expected-output template for stage)"` /
`"(no stage-specific checklist for stage {})"`) instead of getting a real checklist the way
`discovery`, `brief`, `onboarding`, and `1`-`10` already do. `dba-system.md` currently documents
this gap explicitly and directs use of the Review Waiver mechanism as an interim path — this
brief is what would let that waiver eventually be retired for this stage id.

## Upgrade

Add match arms for `"architecture-synthesis"` to both `stage_expected` and `stage_checks` in
`tools/reviewer/src/packet.rs`, following the exact pattern of the existing entries (e.g. `"10"`,
`"onboarding"`) — terse reviewer reminders, not a restatement of doctrine (see Guardrail). Content
should summarize, not duplicate, `dba-system.md`'s "Multi-Feature Architecture Synthesis Gate"
section and `prompts/03b-architecture-synthesis.md`'s 3-step pipeline (Cohort Evidence Review →
Draft Baseline → Approval and Activation) — e.g.:

- **Expected output**: an approved `architecture/core-baseline.md` distinguishing authoritative
  decisions from derived views, with a stated cohort membership set and version; behavioral gaps
  discovered during synthesis are named and routed back to the affected feature's earlier stage,
  never patched into the baseline.
- **Checklist** — observable review questions only, not deployment/tooling status: cohort declared;
  baseline version declared; every declared cohort member's approved artifacts were evaluated for
  architectural relevance (not necessarily all materially contributing — a member may legitimately
  have none); authoritative decisions vs. derived views kept distinct; no synthesized behavior.

**Definition of success**: after this change, invoking `codeos-reviewer` with
`stage=architecture-synthesis` no longer produces the generic fallback strings
(`"(no expected-output template for stage)"` / `"(no stage-specific checklist for stage {})"`)
and instead renders the dedicated expected-output template and checklist above. This is the
natural starting point for Step 2's acceptance criteria when this is picked up.

## Scope

`tools/reviewer/src/packet.rs` only — the two match arms and their content. Out of scope: a
general external stage-policy mechanism (that already has its own broader, more speculative
backlog item — see Related); any change to `dba-system.md` or `prompts/03b-architecture-synthesis.md`
themselves (this brief only makes the reviewer aware of stage content that already exists);
support for any other future custom stage id beyond `architecture-synthesis` (a genuinely new
stage id would need its own brief or ride along with whichever change introduces it).

## Value

Retires the Review Waiver interim path `UPG-0051` had to adopt for this stage id, giving
Architecture Synthesis Gate reviews a real tailored checklist instead of the generic fallback.
Small, mechanical, low-risk.

Trade-offs: none significant — purely additive to an existing, well-understood match statement.

## Risk

Low. Main risk is `packet.rs` gradually accumulating enough restated doctrine detail that it
becomes a second, driftable authority instead of a thin advisory summary — the same staleness
risk every other stage's checklist already carries, not a new category of risk, but worth
naming explicitly here since this is exactly the failure mode this session's `UPG-0051`/`UPG-0052`
work was reacting to in a different form (orphaned/duplicated doctrine). The guardrail above is
the mitigation: keep the reviewer text terse and clearly subordinate to `dba-system.md`.

## Guardrail

- **`packet.rs` remains a consumer of doctrine, never a second authority.** The reviewer text is
  advisory only and intentionally *summarizes* `dba-system.md`/`prompts/03b-architecture-synthesis.md`
  — it must never introduce a review criterion absent from those files, and it must stay terse
  (matching the existing entries' length and register), not become a parallel normative
  description of Architecture Synthesis. If the two ever appear to disagree, the doctrine files
  are authoritative and the reviewer text is stale and needs updating.
- Match arm content should track `dba-system.md`'s actual "Multi-Feature Architecture Synthesis
  Gate" section, not invent independent criteria.
- Does not change reviewer behavior for any existing stage id.
- Does not attempt to build a general external policy mechanism — that is out of scope here (see
  `UPG-0049`).

## Related

- **UPG-0051**: Multi-Feature Architecture Synthesis Gate — the stage id this brief adds reviewer
  support for; originally shipped with a Review Waiver as an explicit interim path, tracked as a
  named follow-up on `UPG-0051`'s own Feature Thread.
- **UPG-0049**: External Review Policy Registry — the broader, more speculative mechanism for
  externalizing stage-specific review policy generally; this brief is a narrow, immediate
  stopgap, not a substitute for it and not blocked by it.
- Discovered during `UPG-0051` / `CHG-20260719-001`'s Step 1 and Step 4 (direct read of
  `packet.rs:661-696`); noted on both `UPG-0051`'s and `UPG-0052`'s Feature Threads as an
  out-of-scope-backlog finding never previously filed as its own UPG.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| (none yet) | — | — | PROPOSED |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
