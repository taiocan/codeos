---
feature_id: UPG-0050
slug: downstream-feature-id-scheme
title: Downstream Feature-ID Scheme (F-####)
status: COMPLETE
priority: P2
depends_on: []
related_features: [UPG-0001, UPG-0041]
supersedes: []
superseded_by: []
---

# Upgrade: downstream-feature-id-scheme — Downstream Feature-ID Scheme (F-####)

**Priority**: P2
**Status**: COMPLETE
**Type**: downstream-doctrine

## Problem

The DBA Feature Brief step (`prompts/00b-feature-brief.md`, Stage `brief`) saves briefs to
`backlog/[feature_id]-[name].md`, but `[feature_id]` has never had a defined format.
`templates/conventions.md`'s "Feature IDs" section only specifies `lowercase_underscore`
(e.g. `add_item_to_cart`) — a free-text slug with no counter, no sequencing, and no guaranteed
uniqueness across a project's lifetime.

Separately, `templates/feature-registry.yaml` (the downstream feature registry template)
already declares a structured `feature_id:` field whose example value is `UPG-0000` — i.e. it
silently borrowed the **self-dev-only** `UPG-####` scheme (reserved for this toolkit repo's
own backlog, per `backlog/features.md:87`, "UPG-0000 reserved for documentation examples")
without ever formalizing an equivalent scheme for downstream projects. This is a live
inconsistency: two downstream-facing artifacts imply two different, incompatible feature-id
conventions, and neither is actually specified as a durable identity scheme.

## Upgrade

Introduce `F-####` as the downstream feature-id format — same spirit as `UPG-####`
(sequential, zero-padded, permanent, never reused) but with a distinct prefix so the two
namespaces never collide:

- **Format**: `F-####`, 4-digit zero-padded (matches `UPG-####` width for visual/parsing
  consistency). Identity (`F-####`) and human-readable label (`slug`) are separate, exactly as
  `templates/feature-registry.yaml`'s existing `feature_id`/`slug` split already models —
  only the example value needs correcting. Filenames combine both:
  `backlog/F-0001-add-item-to-cart.md`, `intents/F-0001.md`, `contracts/F-0001_contract.md`,
  `events/F-0001_schema.md`.
- **R-type briefs reuse the parent id** — a refinement of an existing feature does not mint a
  new `F-####`; it stays under the original feature's id via the brief's existing
  `**Refines**: [feature_id being refined]` field. Mirrors how Stage 9 (Targeted Refinement)
  keeps refinements inside the same feature rather than spawning a new one.
- **ID minting**: no new required file. At Synthesis in `prompts/00b-feature-brief.md`, the AI
  scans `features/registry.yaml` (if present) and `backlog/F-####-*.md` filenames for the
  current max, assigns next for F-type. Same eyeball-the-existing-set mechanism this toolkit
  already uses for its own `UPG-####` assignment — no counter file, no new required artifact.

## Scope

Files touched:
- `templates/conventions.md` — replace the "Feature IDs: `lowercase_underscore`" section with
  the `F-####` format and the id/slug split.
- `templates/feature-registry.yaml` — fix the `feature_id: UPG-0000` example (line 39) to
  `feature_id: F-0001`; correct the inline comment.
- `prompts/00b-feature-brief.md` — add explicit id-assignment instructions to Synthesis;
  update `backlog/[feature_id]-[name].md` references to a concrete `F-####` example.
- `templates/feature-brief.md` — update the H1 and `**Refines**` guidance to the `F-####`
  format; note where the id comes from.
- `dba-system.md` — one clarifying line per `[feature_id]` placeholder location (File Layout,
  Artifact Classification, Stage table) pointing at the format now defined in
  `conventions.md`. No change to stage names, gates, or 9-stage substance.

Out of scope:
- No change to `scripts/dba-init.sh` code (it already copies the template verbatim; the
  corrected example flows through automatically).
- No new required artifact — Feature Brief and Feature Registry remain **Optional** per
  `dba-system.md`'s Artifact Classification table.
- No downstream equivalent of the self-dev `CHG-YYYYMMDD-NNN` change-id layer — this upgrade
  only formalizes the feature-identity layer, not a change-execution layer.
- No retroactive rename of any existing downstream project's already-assigned slug-style
  `feature_id`s — this is a forward convention, not a migration mandate.
- No change to the self-dev `UPG-####` scheme or `backlog/features.md`'s own numbering.

## Value

Gives downstream DBA projects the same "one id, sequential, never reused" discipline this
toolkit already applies to itself, closes the `UPG-0000`-example inconsistency between
`feature-registry.yaml` and `conventions.md`, and makes `[feature_id]` in `dba-system.md`'s
file layout an unambiguous, checkable format instead of an undefined placeholder.

Trade-offs: doc/template/prompt-only change, no tooling automation for id assignment (the AI
derives the next id by inspection each time, same as self-dev does today) — acceptable given
Feature Brief/Registry are both optional, low-frequency artifacts.

## Risk

Deciding hastily risks:
- Conflating this with the self-dev `UPG-####`/`CHG-*` layered model and accidentally
  proposing a downstream change-id system nobody asked for (explicitly out of scope above).
- Making the id format mandatory in a way that contradicts the Optional classification of
  Feature Brief / Feature Registry in `dba-system.md`.
- Leaving stale `UPG-0000` or `lowercase_underscore` references behind in any of the five
  touched files, recreating the exact inconsistency this change fixes.

## Guardrail

- Preserve the 9-stage loop, stage names, and gate rules in `dba-system.md` untouched —
  this is a format clarification for one placeholder, not a doctrine rewrite.
- Feature Brief and Feature Registry stay Optional artifacts; the `F-####` format is a
  convention layered on top, not a new hard requirement.
- `F-` prefix must never collide with or be confused for `UPG-####` — cross-reference check
  in Reconcile verifies no remaining `UPG-0000` example leakage.

## Related

- **UPG-0001**: Feature Thread Traceability — source of the stable-id / never-reused
  discipline this upgrade extends to the downstream side.
- **UPG-0041**: Feature registry schema v2 — the `features/registry.yaml` structure whose
  `feature_id` example this upgrade corrects.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260716-001 | changes/UPG-0050__CHG-20260716-001__downstream-feature-id-scheme.md | Introduce F-#### downstream feature-id scheme | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|
| RVS__UPG-0050__CHG-20260716-001__S1 | CHG-20260716-001 | 1-Intent | R1→R2 | DO NOT ADVANCE → NO OBJECTION |
| RVS__UPG-0050__CHG-20260716-001__S2 | CHG-20260716-001 | 2-Acceptance | R1 | NO OBJECTION |
| RVS__UPG-0050__CHG-20260716-001__S3 | CHG-20260716-001 | 3-Implement | R1→R2 | DO NOT ADVANCE → NO OBJECTION |
| RVS__UPG-0050__CHG-20260716-001__S4 | CHG-20260716-001 | 4-Reconcile | R1 | NO OBJECTION |

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
