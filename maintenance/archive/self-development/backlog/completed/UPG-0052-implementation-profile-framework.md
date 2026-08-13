---
feature_id: UPG-0052
slug: implementation-profile-framework
title: Implementation Profile Framework and Rust-First Default Profile
status: COMPLETE
priority: P2
depends_on: []
related_features: [UPG-0051, UPG-0053, UPG-0054]
supersedes: []
superseded_by: []
---

# Upgrade: implementation-profile-framework — Implementation Profile Framework and Rust-First Default Profile

**Priority**: P2 (effectively P1 for a project like EvidenceAtlas, whose first Architecture
Synthesis should consume an already-approved Rust-first profile rather than retrofit the
preference afterward)
**Status**: COMPLETE
**Type**: downstream-doctrine

## Problem

`dba-system.md` is (correctly) language-neutral through Stage 3, but has no mechanism for a project
to state and enforce a default implementation language before Stage 4. `patterns/rust-project-
structure.md` exists but is confirmed orphaned — zero references from `dba-system.md` or any
`prompts/*.md` file, including `04-implement.md`. Without an explicit profile mechanism, "Rust
first" is either re-argued ad hoc per feature/crate or silently assumed, and the Rust pattern file
is never surfaced at the point where implementation decisions are actually made.

## Upgrade

This brief proposes direction to carry into Step 1; several mechanics are deliberately left open
for Step 1/2 (see "Not decided by this brief" below).

### Proposed direction

- **No hard dependency on `UPG-0051`.** This framework must work for single-feature and
  loosely-coupled projects that never run an Architecture Synthesis Gate. The two integrate as
  peers: synthesis may consume an approved profile; the profile doesn't need synthesis to exist or
  be approved.
- Profile has an explicit **lifecycle** with exactly one non-binding pre-approval state — no
  separate "provisional" state alongside it: identity/version (e.g. `profile_id`,
  `status: proposed|approved|superseded`, `binding: false` while `proposed`, `primary_language`,
  `applies_to`, `exceptions[]`, `approved_by`, `approved_at`, `supersedes`) — exact schema is open,
  but these lifecycle properties, and this single non-binding pre-approval state, are required at
  minimum.
- **Policy choice, explicit:** Codeos *recommends and initially scaffolds a `proposed`, non-binding*
  rust-first profile for new projects, but **no profile becomes binding through absence, defaulting,
  or its mere `proposed` presence** — it requires explicit human approval (moving it to
  `status: approved`) before it governs Stage 4. This rejects both "fully neutral, no visible
  default" and "absence silently means Rust."
- **Binding profile, advisory pattern — kept sharply separate:** Stage 4 must verify that the
  implementation is covered by an *approved* Implementation Profile or by a recorded exception. The
  corresponding technology pattern's recommendations remain advisory unless adopted into an approved
  Architecture Baseline or another project-specific decision. No additional human gate is introduced
  beyond the existing Stage 4 approval. An *approved* profile (its language choice, declared scope,
  any recorded exception) is binding; a merely `proposed` one is not; `rust-project-structure.md`'s
  individual recommendations stay advisory regardless of profile state.
- **Structural separation:** the Implementation Profile *framework* is language-neutral;
  "rust-first" is one *supplied profile* within it, referencing `patterns/rust-project-structure.md`
  as its applicable pattern. This keeps future non-Rust profiles additive rather than requiring
  doctrine redesign.
- **Toolchain/lint responsibility split** (avoids overlap with `UPG-0051`): the profile owns
  preferred-language + applicability only; project-specific crate/workspace topology is the
  Architecture Baseline's job (when one exists); Rust toolchain/edition/MSRV/lint/format/test gates
  live in their own referenced file/section — not crammed into the profile artifact.
- **Session-start and onboarding awareness**, in scope (see Scope below): a `proposed` or
  `approved` profile must be surfaced at session start (`prompts/00-session-start.md`), so it isn't
  discovered only when Stage 4 begins. Existing-project rule: new empty projects may receive a
  scaffolded, `proposed` Rust-first profile; **existing projects must not be presumptively converted
  to rust-first** — `prompts/00c-onboarding.md` may derive a `proposed` profile from the existing
  implementation, or ask the human to declare one, but it must never approve or silently impose the
  profile; only the human may move it to `approved`.
- **Semantic prompt-wiring tests**, not grep-only: rust-first profile → applicable pattern actually
  surfaced; approved non-Rust exception → Rust pattern not treated as mandatory; no approved
  profile → no hidden Rust enforcement; an Architecture Baseline may override the generic pattern
  where explicitly justified; onboarding an existing non-Rust project never silently proposes
  rust-first; a merely `proposed` profile is never treated as binding at Stage 4.

### Removed from this brief (scope drift, moved out)

- The error-boundary/failure-classification convention — a cross-language Stage 4/5 concern, not an
  implementation-profile concern. Spun out as **`UPG-0054`**.
- Detailed Rust toolchain/lint prescription — reduced to "record where these decisions belong," not
  prescribed in doctrine itself.

### Not decided by this brief — questions for the implementer to resolve

- Exact profile schema/file format.
- Approval-recording mechanism for projects with no Architecture Synthesis Gate running.
- How a profile change after Stage 4 has started is handled (Stage 10 vs. a new profile version +
  reconciliation).

## Scope

`dba-system.md` (Implementation Profile section + Stage 4 cross-reference), `patterns/rust-project-
structure.md` (add the missing "consumed by Stage 4 / optionally by Architecture Synthesis"
cross-reference), `prompts/04-implement.md` (profile-conformance consultation step),
`prompts/00-session-start.md` (surface a proposed/approved profile), `prompts/00c-onboarding.md`
(existing-project non-conversion rule — the implementer may split this into its own narrowly-scoped
follow-up if the onboarding change turns out larger than expected, but the rule itself is settled
direction, not optional), a new profile template.

Out of scope: failure-boundary conventions (→ `UPG-0054`); `dba-init.sh` scaffolding (→ `UPG-0053`);
Architecture Baseline's own crate-topology content (`UPG-0051`, integrates only).

## Value

Gives Rust real default precedence without hardcoding it as a behavioral truth of DBA; makes the
existing orphaned Rust pattern file actually load-bearing; stops each new crate boundary decision
from re-litigating "why Rust" from scratch, while keeping non-Rust choices fully legitimate with a
recorded rationale.

Trade-offs: another artifact (the profile) to keep in sync with the baseline when both exist;
another Stage 4 conformance check to maintain; touching `00-session-start.md` and
`00c-onboarding.md` widens this beyond a single-file doctrine change.

## Risk

Deciding hastily risks:
- Being misread as making Rust mandatory rather than default — the profile-not-doctrine framing and
  recorded-exception mechanism are the two safeguards against that.
- Conflating profile compliance (binding once approved) with pattern-recommendation compliance
  (always advisory) — the two must stay sharply distinguished in doctrine text.
- Silently converting an existing non-Rust project's onboarding into a Rust-first proposal if the
  onboarding rule isn't honored.

## Guardrail

- Non-Rust implementations remain permitted with recorded rationale.
- The profile document is project-level state, never embedded as a live rule inside `dba-system.md`
  itself.
- Stage 4 conformance check verifies profile/exception coverage only; it never becomes a second
  approval gate beyond existing human sign-off.
- Existing projects are never presumptively converted to rust-first during onboarding.

## Related

- **UPG-0051**: Multi-Feature Architecture Synthesis Gate — optional integration, no hard
  dependency either direction.
- **UPG-0053**: Implementation-Profile Scaffolding in `dba-init.sh` — depends on this.
- **UPG-0054**: Contract-to-Implementation Failure Boundary — failure-boundary convention split out
  from here.
- Proposed by the human during a 2026-07-19 discussion of EvidenceAtlas's architecture sequencing
  (see `reviews/review-log.md`).

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260719-002 | `changes/UPG-0052__CHG-20260719-002__implementation-profile-framework.md` | Add the Implementation Profile framework doctrine, template, and Stage 4/session-start/onboarding wiring | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|
| RVS__UPG-0052__CHG-20260719-002__S1 | CHG-20260719-002 | 1-Intent | R1→R4 | R1 DO NOT ADVANCE (scope-drift, uncommitted UPG-0051) → R2 NO OBJECTION → R3 NO OBJECTION (post 1st human revision) → R4 NO OBJECTION (human-requested beyond budget, post 2nd human revision) |
| RVS__UPG-0052__CHG-20260719-002__S2 | CHG-20260719-002 | 2-Acceptance | R1 | NO OBJECTION |
| RVS__UPG-0052__CHG-20260719-002__S3 | CHG-20260719-002 | 3-Implement | R1 | NO OBJECTION |
| RVS__UPG-0052__CHG-20260719-002__S4 | CHG-20260719-002 | 4-Reconcile | R1→R2 | R1 DO NOT ADVANCE (AC16 unverifiable) → R2 NO OBJECTION — ACCEPTED |

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|
| (Step 1) scope-drift false positive from uncommitted UPG-0051 diff | RVS__…__S1 | IN-SCOPE BLOCKER | Fixed — committed UPG-0051 as two commits, confirmed with human first |
| (Step 1, human round 1) free-text applicability, no immutability/history, no profile-baseline consistency | RVS__…__S1 | IN-SCOPE BLOCKER (×3) | Fixed — resolvable selectors, immutability + history, consistency rule added |
| (Step 1, human round 2) no location for pending proposed replacement, non-deterministic exceptions, no provenance recording | RVS__…__S1 | IN-SCOPE BLOCKER (×3) | Fixed — `proposals/` directory, exception specificity rule, provenance field added |
| (Step 4) AC16 marked PASS with a prose summary, not embedded evidence | RVS__…__S4 | IN-SCOPE BLOCKER | Fixed — full grep sweep output embedded verbatim; journaled as AJ-016 |
| Native `codeos-reviewer` support (from UPG-0051, still open) | — | OUT-OF-SCOPE BACKLOG | Unrelated to this feature; tracked on UPG-0051's own Feature Thread |

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
| — | None spun out — all findings resolved inside this change | — |
