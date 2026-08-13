---
component_question: How should shared infrastructure responsibilities be separated from feature modules?
out_of_scope: Feature behavior, universal module layouts, technology selection, and architecture approval mechanics.
---

# Pattern: Shared Infrastructure Boundary

## When This Pattern Applies

Use this pattern when a project has a shared module that multiple feature modules depend
on — a "hub" module that provides common infrastructure.

Common contexts:
- A shared event-emission library depended on by all feature modules
- A shared utility crate (ID generation, logging, DTOs, constants)
- A shared schema or vocabulary loader used across all features
- Any module where adding a dependency on it from a new feature module is trivial

If only one feature module depends on the shared module, the risk is low and this pattern
is not needed yet. When two or more feature modules depend on it, apply this pattern.

---

## The Anti-Pattern: Vertical Drift

Workspace topology can enforce lateral isolation: if module A cannot import module B,
they cannot couple directly. This is a strong structural guarantee.

But it does not protect against vertical drift: both A and B importing the shared hub
and each depositing domain logic there because it is the most convenient place. The hub
accumulates feature-specific rules, status derivations, and vocabulary queries — and
becomes a God module from below. Lateral isolation is preserved on paper while domain
coupling grows through the shared layer.

Vertical drift is slow and invisible. Each addition seems small. The structural guarantee
appears intact. But the hub now encodes behavior specific to multiple features, and a
change to the hub risks breaking any of them.

---

## The Rule

**Permitted in a shared infrastructure module:**
- Event emission adapters (wrappers that bake in fixed fields)
- Append-only log readers and iterators
- Mechanical DTOs that do not encode domain vocabulary or business rules
- Constants (file path strings, timeout values, sentinel strings)
- Re-exports of types from lower-level shared libraries
- ID generation utilities (UUID, correlation ID)

**Not permitted in a shared infrastructure module:**
- Business rules (anything that makes a domain decision)
- Status derivation or computation (`compute_effective_status()` does not belong here)
- Schema validation or vocabulary queries (`find_open_tasks_for_dashboard()` does not
  belong here)
- Domain aggregation (collecting and grouping items by feature-specific criteria)
- Feature-specific DTOs or summaries (a `TaskExportSummary` or
  `RequirementExtractionResult` is not infrastructure, regardless of how generic it
  initially appears)

---

## The Diagnostic Test

Before adding anything to a shared infrastructure module, answer:

> **Would a pure infrastructure module — one with zero knowledge of the domain
> vocabulary — need this?**

If **yes** → the addition is likely legitimate infrastructure. Proceed.

If **no** → the addition encodes domain knowledge and does not belong here. Move it to
the feature module that needs it, or to the vocabulary module that owns the concept.

This test is fast to apply and catches most drift before it happens.

---

## The Justification Gate

Sometimes an addition fails the Diagnostic Test but is still the right architectural
call — for example, a DTO shared by multiple features that would be duplicated
identically without a shared location.

In these cases: **write one sentence of justification** in the commit message or PR
description explaining why the addition cannot live in a feature module.

The requirement to write this justification is the gate. It prevents silent drift
while preserving human judgment for genuine exceptions.

If the justification sentence is hard to write, that is a signal the addition is drift,
not a genuine exception.

---

## Project-Level Instantiation

Every project that has a shared infrastructure module should name it explicitly in the
project's `CLAUDE.md` under a `## Shared Infrastructure` section, listing:

1. The module name
2. The specific permitted additions for this project's hub
3. The specific not-permitted additions
4. The justification requirement

This turns a general architectural principle into an auditable project rule. It also
makes the boundary visible to reviewers who may not know the pattern exists.

---

## LucidPM Reference (`lucid_core`)

`lucid_core` is LucidPM's shared infrastructure module. All 13 feature modules depend
on it.

**Permitted in `lucid_core`:**
- `EventEmitter` — wraps `project_schema::emit_event`, bakes in `events_file` and
  `source_module`
- `open_event_log(path)` — iterator over `events/runtime_events.jsonl`; empty on missing
  file, parse errors as `Err` items
- `EVENTS_FILE` — `"events/runtime_events.jsonl"` constant
- Re-exports: `EventEnvelope`, `SchemaError` from `project_schema`
- `RecordedItem` — see note below

**Not permitted in `lucid_core`:** vocabulary queries, status derivation, domain
aggregation, feature-specific DTOs or summaries.

**Note on `RecordedItem`:** This DTO reconstructs a project record item from events. It
is domain-shaped — it carries fields like `item_type`, `status`, `priority` — and was
added before this pattern was formalized. It is grandfathered as existing shared
architecture. It is **not precedent** for adding further domain records. A proposed
`TaskSummary`, `StakeholderView`, or similar DTO would still require the Diagnostic
Test and written justification.

---

## Applying the Boundary

Apply the Diagnostic Test before adding to shared infrastructure. If the addition establishes or
changes a project-level responsibility boundary for governed features, record it through the
applicable architecture scope. Otherwise treat the work as normal engineering and preserve affected
feature behavior with proportional verification.
