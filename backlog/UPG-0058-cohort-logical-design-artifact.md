---
feature_id: UPG-0058
slug: cohort-logical-design-artifact
title: Cohort Logical Design — a Second Architecture Synthesis Output
status: PROPOSED
priority: P1
depends_on: [UPG-0051]
related_features: [UPG-0051, UPG-0055]
supersedes: []
superseded_by: []
---

# Upgrade: cohort-logical-design-artifact — Cohort Logical Design — a Second Architecture Synthesis Output

**Priority**: P1
**Status**: PROPOSED
**Type**: downstream-doctrine

## Problem

`UPG-0051`'s Multi-Feature Architecture Synthesis Gate produces exactly one output — the Core
Architecture Baseline — covering topology, module ownership, dependency direction, persistence
technology, integration style, and shared infrastructure. Real downstream usage (a multi-feature
project with a shared source-intelligence cohort) found this insufficient: the baseline is high-level
enough that independent Stage-4 features can still make conflicting *local* decisions the gate exists
to prevent — one feature using UUID primary keys, another identifying the same canonical entity by
URL, a third by a composite natural key; one feature's revision model diverging from another's;
inconsistent transaction/event-emission ownership across features writing to the same tables. The
baseline resolves *strategic* questions; nothing in the current gate resolves the *logical* design
questions (identity/key strategy, revision/supersession pattern, module interface boundaries,
transaction ownership, event-emission points, read-model ownership) that features sharing persistence
and identity actually need fixed *before* Stage 4, not independently invented *during* it.

## Upgrade

Add a second Architecture Synthesis output, produced after the Baseline and before Stage 4 entry:
**Cohort Logical Design** (`architecture/cohort-logical-design.md`, template
`templates/cohort-logical-design.md`, versioned/superseded exactly like the Baseline is). It covers,
at logical (not implementation) detail: logical ERD; entity/aggregate ownership; identity and key
strategy; revision/supersession model; module interface map; command/query responsibilities;
transaction boundaries; validation ownership; event-emission rules; read-model design; indexing and
spatial principles; migration strategy; integration-test obligations; and an explicit mapping from
each design element back to the approved feature artifacts it derives from.

`prompts/03b-architecture-synthesis.md`'s pipeline extends from 3 steps to 4: Cohort Evidence Review →
Draft Baseline → **Draft Cohort Logical Design (new)** → Approval and Activation. Baseline approval
alone no longer unblocks Stage 4 for cohort members — a new intermediate cohort status,
`baseline-approved`, marks that state; `approved` (the Stage-4-unblocking status) now means **both**
the Baseline and the Logical Design are approved for the applicable versions. The registry gains a
parallel `logical_design_version` field alongside `baseline_version`, versioned and superseded the
same way.

**Same guardrail as the Baseline, restated for this artifact:** the Logical Design may define
structure; it may never invent behavior. Example distinctions (from the discovery): deciding an
`intended_use_id` is a foreign key is architecture; deciding whether one decision may cover multiple
intended uses is behavior and returns upstream. Deciding revisions are append-only is architecture
*if already supported by approved artifacts*; inventing a new "draft"/"expired" status outright is
behavior and returns upstream.

**Definition of success**: a cohort with an approved Baseline can produce and approve a Logical
Design that pins identity/key strategy, revision pattern, module interfaces, and transaction/event
ownership *once*, shared across all member features — so that two independently-implemented Stage-4
features cannot each invent an incompatible local answer to the same cross-feature structural
question.

## Scope

**In scope**: `dba-system.md`'s "Multi-Feature Architecture Synthesis Gate" section (the new output,
the new intermediate status, the restated structure-not-behavior guardrail, File Layout/Artifact
Classification entries); `prompts/03b-architecture-synthesis.md` (new Step 3, renumbered Step 4);
`templates/cohort-logical-design.md` (new); `templates/feature-registry.yaml` (the
`logical_design_version` field and the `baseline-approved` status value); a small update to
`tools/reviewer/src/packet.rs`'s existing `"architecture-synthesis"` checklist content (already
generic to the whole pipeline per `UPG-0055`) to mention the Logical Design.

**Out of scope**: any change to the Baseline's own content or template; any change to the cohort
declaration mechanism itself; any new Stage ID (the Logical Design step lives inside the existing
conditional gate, not the 9-step loop); any new Non-Negotiable Rule; automatic migration of
already-`approved` cohorts under the pre-this-UPG single-output rule (see Guardrail — handled by an
explicit compatibility rule, not silent reinterpretation).

## Value

Closes a real, discovered gap: without this, the Architecture Synthesis Gate can be satisfied while
still allowing exactly the cross-feature structural inconsistency it exists to prevent. Keeps the
Baseline itself stable and concise (per the discovery's own "why keep it separate" reasoning) by not
folding growing logical detail into it.

Trade-off: one more mandatory human-approved artifact per cohort, and downstream cohorts already
`approved` under the old single-output rule need an explicit compatibility decision (see Guardrail),
not just a doctrine text change.

## Risk

Medium. This changes the meaning of an existing status value (`approved`) for any cohort declared
under `UPG-0051`'s original rule — a real downstream project may already have an `approved` baseline
today. Mitigated by introducing `baseline-approved` as the compatibility landing state (see
Guardrail) rather than silently reinterpreting existing `approved` rows, and by keeping the Logical
Design's own guardrail (structure, not behavior) identical in spirit to the Baseline's already-
reviewed one.

## Guardrail

- The Logical Design may never invent behavior; a discovered gap returns to the owning Stage 1, 2, or
  3 artifact, exactly like the Baseline's existing rule — never patched into the Logical Design
  directly.
- **Compatibility rule for existing `approved` cohorts:** a cohort whose registry `status` was
  already `approved` before this UPG lands is treated, on first read after this UPG ships, as
  `baseline-approved` (not `approved`) — its Baseline stays valid, but a Logical Design must still be
  drafted and approved before further Stage 4 entry for that cohort's members. This is a Step 2
  acceptance criterion, not an implementation afterthought.
- `templates/cohort-logical-design.md` stays terse and placeholder-driven, matching
  `templates/architecture-baseline.md`'s existing register — this is a second structural artifact,
  not a license to write a full technical design document inline.
- Do not let the Logical Design duplicate the Baseline's own authoritative decisions (topology,
  dependency direction, persistence technology) — it consumes and elaborates them, it does not
  restate or re-decide them.

## Related

- Extends `UPG-0051` (Multi-Feature Architecture Synthesis Gate), which this UPG's design discovery
  found produces too little detail for independent Stage-4 features sharing persistence/identity.
- `UPG-0055` (reviewer support for the `architecture-synthesis` stage id) already covers this
  pipeline generically; this UPG only updates that checklist's content, not its stage-id wiring.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the change
> records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260726-002 | `changes/UPG-0058__CHG-20260726-002__cohort-logical-design-artifact.md` | Add the Cohort Logical Design artifact and 4-step pipeline | IN_PROGRESS |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
| — | — | — |
