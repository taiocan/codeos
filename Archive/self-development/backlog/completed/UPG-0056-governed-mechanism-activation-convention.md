---
feature_id: UPG-0056
slug: governed-mechanism-activation-convention
title: Optional Mechanism Status Convention
status: COMPLETE
priority: P2
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: governed-mechanism-activation-convention — Optional Mechanism Status Convention

**Priority**: P2
**Status**: COMPLETE
**Type**: downstream-doctrine

> **Revision note:** this brief originally described a versioned governed-mechanism framework (a
> Rust resolver, activation modes, provenance stamps, 25 result codes). During planning of the first
> consuming feature (`UPG-0057`, Controlled Plain English), that design was found disproportionate
> for what every known consumer actually needs — a human-readable on/off switch — and was replaced
> with the minimal convention below. The Feature Thread's historical review rows are preserved
> unchanged below them, as an accurate record of what was reviewed at the time; they do not describe
> the current design.

## Problem

A future Codeos feature (the first known case: a "Controlled Plain English" writing-style
discipline, `UPG-0057`, deliberately deferred out of this change) needs a human-controlled, on/off
switch for AI doctrine/generation behavior, in both downstream projects and Codeos's own
self-development. No shared convention for this exists today. Without one, each such feature would
invent its own ad hoc status-file shape and wording, and every prompt consulting it would phrase the
same three-way check ("is it on, off, or broken?") slightly differently.

## Upgrade

Document one minimal convention, once, in `templates/conventions.md`: a status file containing
**exactly one line**, `status: enabled` or `status: disabled`. Four outcomes only:

| File state | Result |
|---|---|
| Absent | Disabled |
| Exact `status: disabled` | Disabled |
| Exact `status: enabled` | Enabled |
| Anything else (unreadable, extra content, any other value) | Stop and report a configuration error |

Missing means disabled, everywhere, with no absence-policy options. Only an explicit human
instruction changes the value; an agent never does so on its own initiative. Git history is the
audit trail — no resolver, no schema version, no activation id, no provenance fields, no versioned
governing artifact.

**Definition of success**: a future feature (starting with `UPG-0057`) can read a status file of
this exact shape directly, with no shared tool, and get one of the four outcomes above,
deterministically, from a one-line check in its own prompt text.

## Scope

**In scope**: `templates/conventions.md` (the convention's full documentation: grammar, the
four-outcome table, whitespace/line-ending pinning, the `architecture/` downstream placement rule),
`templates/optional-mechanism-status.yaml` (a minimal illustrative example file), a short
`dba-system.md` doctrine-pointer section, this brief.

**Out of scope**: any concrete status file (downstream or self-dev — that's each consuming
feature's own job, starting with `UPG-0057`); any change to `scripts/dba-init.sh`, `CLAUDE.md`, or
`prompts/codeos-self-dev.md`; any Controlled Plain English content; any resolver, shell script, Rust
crate, provenance stamp, version field, result-code table, or absence-policy option of any kind; any
Non-Negotiable Rule change; any new Stage ID; any new mandatory human-approval gate; runtime product
feature flags (a different category of thing entirely — this governs AI doctrine/generation
behavior, never application runtime behavior).

## Value

Gives every future toggle-needing feature one documented convention to reuse instead of inventing its
own status-file wording. Deliberately minimal — no behavioral risk from shipping it, since it is
pure documentation plus one illustrative example file; no code exists to have a bug in.

## Risk

Very low. No code, no consumer wired up yet. The only design risk is that the convention proves too
minimal for some future need — acceptable, since the fix is additive (a future feature can still add
its own extra fields if it genuinely needs them; nothing here forecloses that).

## Guardrail

- This UPG documents the convention only. It must not scaffold any concrete status file anywhere.
- Do not let this become a second governance layer parallel to `dba-system.md` — it is a convention a
  future doctrine change may adopt, not doctrine itself. `dba-system.md`'s new section says
  explicitly that no current doctrine rule uses it yet.
- Do not reintroduce resolver/schema/versioning complexity here — if a future feature genuinely needs
  more than "enabled/disabled," that is that feature's own extension, decided when the need is
  concrete, not spec'd speculatively here.

## Related

- Originally proposed together with a "Controlled Plain English" writing-style discipline; narrowed
  to this generic convention only, at the human's explicit direction. Controlled Plain English is
  `UPG-0057`, which depends on this feature and consumes its convention.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the change
> records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260726-001 | `changes/UPG-0056__CHG-20260726-001__governed-mechanism-activation-convention.md` | Document the minimal Optional Mechanism Status Convention | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|
| RVS__UPG-0056__CHG-20260726-001__S1 (original, superseded design) | CHG-20260726-001 | 1-Intent | R1→R3 | R1 DO NOT ADVANCE (scope misclassification, unlisted lifecycle files, overclaimed risk) → R2 DO NOT ADVANCE ("provably authorized" overclaim) → R3 NO OBJECTION — **this round reviewed the original Rust-resolver design, since replaced** |
| RVS__UPG-0056__CHG-20260726-001__S1 (lean revision) | CHG-20260726-001 | 1-Intent | R4 | DO NOT ADVANCE — "one new row" lifecycle claim contradicted by `UPG-0057`'s own row also present in the diff. **PROFILE-4's 3-round/step budget is exceeded (this is round 4 of Step 1 across both design versions)** — fixed inline per CLAUDE.md's budget-exceeded rule; no further automatic Codex round run; **human decision: APPROVE_STAGE** |
| RVS__UPG-0056__CHG-20260726-001__S2 | CHG-20260726-001 | 2-Acceptance | R1→R2 | R1 DO NOT ADVANCE (AC12 overclaimed "no behavioral risk" from "no code" alone — doctrine text is itself behavioral surface) → fixed to ground the claim in AC10 (nothing currently consumes the new section) → R2 NO OBJECTION |
| RVS__UPG-0056__CHG-20260726-001__S3 | CHG-20260726-001 | 3-Implement | R1→R3 | R1 DO NOT ADVANCE (AC5's literal-grep verification method false-flagged negation prose; AC9's self-dev deferral was implicit, not stated) → both fixed → R2 DO NOT ADVANCE (dashboard row itself still said "awaiting approval to proceed to Step 3" despite Step 3 already being under review — a real staleness bug) → fixed → R3 NO OBJECTION |
| RVS__UPG-0056__CHG-20260726-001__S4 | CHG-20260726-001 | 4-Reconcile | R1→R2 | R1 DO NOT ADVANCE — same class of staleness bug recurred: dashboard row said "awaiting approval to proceed to Step 4" while Step 4 was already under review (this time because `status/self-development.md` was omitted from the R1 packet) → fixed → R2 NO OBJECTION |

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|
| (Step 1 R1, original design) Class declared `downstream-doctrine` but the intent added a Rust resolver + shell wrapper + tests — a script-tooling change too | RVS__…__S1 | IN-SCOPE BLOCKER | Fixed at the time; moot now — no code remains in scope after the lean revision |
| (Step 1 R1, original design) `backlog/features.md`/`status/roadmap.md`/`status/self-development.md` changed but not named in "What changes"; dashboard row said "Step 1 not yet drafted" while Step 1 existed | RVS__…__S1 | IN-SCOPE BLOCKER | Fixed — lifecycle bookkeeping still listed explicitly in the revised Change Intent |
| (Step 1 R1, original design) "No behavioral risk from shipping it" — absolute safety claim unsupported by a repo-wide consumer analysis | RVS__…__S1 | IN-SCOPE BLOCKER | Moot after the lean revision — there is no code, so the claim is now straightforwardly true |
| (Step 1 R2, original design) "the change provably authorized" — overclaimed a formal/cryptographic approval-binding guarantee | RVS__…__S1 | IN-SCOPE BLOCKER | Moot after the lean revision — no provenance-authorization claim is made at all now |
| (Planning) The entire original design was disproportionate to the need | — (human direction, not a Codex review finding) | IN-SCOPE BLOCKER | Fixed — replaced with the minimal one-line convention above |
| (Step 1 R4, lean revision) "one new row" lifecycle-bookkeeping claim contradicted by `UPG-0057`'s own row also present in the same working diff | RVS__…__S1 (lean) | IN-SCOPE BLOCKER | Fixed inline — Lifecycle bookkeeping now names `UPG-0057`'s row explicitly as separate, dependent-feature bookkeeping, not this CHG's scope |
| (Step 2 R1) AC12 claimed "no behavioral risk" from "no code" alone, but doctrine text is itself behavioral surface | RVS__…__S2 | IN-SCOPE BLOCKER | Fixed — AC12 now grounds the low-risk claim in AC10 (nothing currently consumes the new section), not merely the absence of code |
| (Step 3 R1) AC5's literal-grep verification method false-flagged accurate negation prose ("No resolver...") as a violation | RVS__…__S3 | IN-SCOPE BLOCKER | Fixed — AC5 reworded to check for an actual field/mechanism, not the literal word |
| (Step 3 R1) AC9 required self-dev placement to be *explicitly* deferred in the doc text, not merely silent on it | RVS__…__S3 | IN-SCOPE BLOCKER | Fixed — added an explicit deferral sentence to `templates/conventions.md`'s "Placement" paragraph |
| (Step 3 R2) `status/self-development.md`'s own UPG-0056 row still said "awaiting approval to proceed to Step 3" while Step 3 was already under review | RVS__…__S3 | IN-SCOPE BLOCKER | Fixed — row updated to reflect the human's actual Step 3 approval and implementation state |
| (Step 4 R1) Same class of staleness bug recurred at Step 4 — dashboard row said "awaiting approval to proceed to Step 4" mid-Step-4-review | RVS__…__S4 | IN-SCOPE BLOCKER | Fixed — row updated; noted as a recurring self-dev-loop discipline lesson (update the dashboard row *before* running each step's review, not just before presenting to the human) |

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
| UPG-0057 | Controlled Plain English — the first consumer of this convention | Deferred by explicit human direction during planning |
