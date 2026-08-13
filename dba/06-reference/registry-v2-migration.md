---
component_question: How should an existing feature registry be migrated to the version 2 schema?
out_of_scope: Feature lifecycle semantics, registry-tool implementation, architecture records, and future schema versions.
---

# Feature Registry Schema v2 Migration Guide

`dba/05-guidance/templates/feature-registry.yaml`'s schema was revised (`UPG-0041`, 2026-07) to add a
`schema_version: 2` marker and represent Onboarding-originated features explicitly, without
adopting a second, incompatible registry shape that had grown independently downstream. This
guide covers what changed and how to update a pre-v2 (unversioned) registry.

Neither the original canonical schema nor any downstream variant was doctrine-authoritative
before this — the active `doctrine` component defines no feature-registry status vocabulary. This is a
deliberate template-level schema decision, not a doctrine change.

---

## 1. Add the `schema_version` marker

Every registry-consuming tool (`generate-approval-dashboard`, `generate-release-evidence`)
now checks for this before doing anything else. Add it once, at the top level of the file,
alongside `features:`:

```yaml
schema_version: 2

features:
  - feature_id: UPG-0001
    ...
```

Without it, `generate-approval-dashboard` refuses to run at all (a hard requirement, not a
warning) — this is intentional: a missing marker means the rest of this guide hasn't been
applied yet, and guessing at an unversioned registry's shape is exactly the kind of silent
tolerance this schema revision was written to avoid.

## 2. Update the `status` vocabulary

**Before (v1):**
```yaml
status: active   # active | suspended | complete | blocked
```

**After (v2):** the same four values, plus a fifth:
```yaml
status: active   # hypothesized | active | suspended | blocked | complete
```

`hypothesized` represents a draft feature registered before Specification Package approval. Its
normal draft Intent must enter a mutually consistent package and receive the package's single
approval before implementation.

**If your registry encodes the DBA stage directly into the status string** (e.g.
`stage0-hypothesized`, `stage1`, `stage4`, `complete`) — this is a different, incompatible
convention that grew independently downstream. It is **not** adopted as canonical by v2.
Split it back into the two separate fields v2 keeps distinct:

| Your value | v2 `status` | v2 `current_stage` |
|---|---|---|
| `stage0-hypothesized` | `hypothesized` | `0` (or `null`) |
| `stage0` | `active` | `0` |
| `stage1` … `stage9` | `active` | `1` … `9` |
| `complete` | `complete` | `9` or `null` |

Do **not** recombine stage and status into one string in v2 — `status` is lifecycle/decision
state; `current_stage` is DBA workflow position. Keeping them independent is what lets a
dashboard query "what needs human attention" (`status`) separately from "how far along is
it" (`current_stage`).

## 3. Keep (or add) `current_stage` as its own field

If your registry already has a separate `current_stage` field, no change needed beyond
possibly re-deriving its value per the table above. If your registry only had a
stage-encoded status string, split it out per step 2.

```yaml
current_stage: 1   # 0 | 1–9 | null
```

## 4. Keep (or add) `slug`

`slug` stays **required** in v2 — it is not optional and does not fall back to `feature_id`.
`feature_id` is stable identity (e.g. `UPG-0042`); `slug` is the human-readable label (e.g.
`checkout-flow`). They solve different problems; a registry entry needs both.

```yaml
feature_id: UPG-0042
slug: checkout-flow
```

If your registry has no `slug` field at all, add one — a short, readable, kebab-case name is
sufficient; it doesn't need to be globally unique the way `feature_id` is.

## 5. Add `notes`, keep `blockers` structured

If your registry uses a single free-form `notes` field in place of a structured `blockers`
list, **add** `blockers: []` back rather than removing `notes` — v2 keeps both:

```yaml
blockers: []   # structured — machine-readable, feeds dashboards/review summaries
notes: ""      # free-form — human context, next action, anything blockers doesn't capture
```

Move any blocking-issue text currently sitting in `notes` into `blockers` as discrete items;
keep genuinely free-form context (rationale, links, "waiting on X's review") in `notes`.

## 6. Full before/after example

**Before (a stage-encoded, no-slug, no-blockers variant):**
```yaml
features:
  - feature_id: example_feature
    description: "Brief description of what this feature does"
    type: F
    status: stage1
    artifacts:
      intent: intents/example_feature.md
      contract: contracts/example_feature_contract.md
      schema: events/example_feature_schema.md
    notes: ""
```

**After (v2):**
```yaml
schema_version: 2

features:
  - feature_id: example_feature
    slug: example-feature
    description: "Brief description of what this feature does"
    type: F
    status: active
    current_stage: 1
    intent: intents/example_feature.md
    contract: contracts/example_feature_contract.md
    event_schema: events/example_feature_schema.md
    blockers: []
    notes: ""
```

(This example also flattens a nested `artifacts:` block into the canonical flat
`intent`/`contract`/`event_schema` fields — v2 does not support a nested `artifacts:` key;
use the flat fields shown in `dba/05-guidance/templates/feature-registry.yaml`.)

## 7. If you don't migrate

`generate-approval-dashboard` will refuse to run against a registry missing
`schema_version: 2`, printing this guide's path. `generate-release-evidence`'s optional
`--registry` lookup degrades gracefully instead (registry-derived fields fall back to
`[FILL]`, with a warning naming the mismatch) — it still produces a usable release-evidence
skeleton, just without registry enrichment, until you migrate.
