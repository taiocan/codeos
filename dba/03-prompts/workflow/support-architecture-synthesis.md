---
component_question: How should a required project-level architecture scope be synthesized and presented for approval?
out_of_scope: Feature behavior, general lifecycle rules, implementation, reviewer mechanics, and ordinary structural maintenance.
---

# Architecture Synthesis

<!-- DOCTRINE ADAPTER: architecture-entry -->

## Purpose

Resolve the minimum project-level architecture required before governed feature implementation.
This is a conditional support workflow, not a numbered stage.

## Inputs / Prerequisites

Read the selected architecture-synthesis and review policies. Use this prompt only when work would
otherwise establish or change an unresolved project-level boundary. Ordinary behavior-neutral,
architecture-insignificant maintenance uses the project's normal engineering process.

Identify affected governed features and their approved Specification Packages through the
Downstream Project Layout Contract. Run the architecture scope inspector, stop on invalid or
duplicate membership, and reuse a matching scope or draft the current architecture-scope template.

## Task

- Resolve the Platform Baseline for this scope: persistence (PostgreSQL), backend (Rust), webapp
  (Svelte), and runtime (Docker) by default, or the Charter-recorded exception for a tier that does
  not apply. Identify where integrated verification crosses the Postgres/Rust/Svelte boundaries —
  e.g. which component owns the Playwright acceptance surface for a given user journey.
- Review approved feature requirements, Charter System Constraints, and applicable architecture
  together.
- Record only material project-level responsibilities, dependencies, state ownership, integration,
  runtime placement, constraints, and feature obligations.
- Derive decisions from approved requirements or explicit human architectural decisions.
- Treat approved quality requirements and System Constraints as inputs. Record the structural
  consequences they force; never originate a new one here.
- Leave decisions that are local to one component to implementation, even when reversing them would
  be costly; implementation records those in its Stage 4 output.
- Govern project-level structure only; Stage 4 owns local implementation design inside the
  approved architectural boundaries.
- Return a missing requirement to its owning source: a behavioral gap to the affected Specification
  Package, a feature-specific quality requirement to that feature's Contract, and a cross-cutting
  quality requirement or other System Constraint to the Solution Charter.
- Keep `approval: null` while responsibility, dependency direction, data authority, lifecycle, or
  integration conflicts remain unresolved.

## Applicable Checks

Apply the selected policy's synthesis and membership rules. Apply the selected review policy at the
architecture decision boundary.

## Output / Next Action

Create or revise `.codeos/02-architecture/scopes/<scope-id>.md` using the current
architecture-scope template. Keep `approval: null` while architectural issues remain unresolved,
present the complete artifact, and state:

`AWAITING HUMAN APPROVAL OF THE ARCHITECTURE SCOPE`

After explicit approval, record `approval.by` and `approval.at`, then route eligible affected
feature work to `04-implement.md`. If synthesis or reassessment exposes a behavioral gap, return it
to the affected Specification Package as described above; if another prerequisite remains
unresolved, do not route that work to Stage 4. Any later material change first returns approval to
`null` and repeats this boundary; reassess only affected work.
