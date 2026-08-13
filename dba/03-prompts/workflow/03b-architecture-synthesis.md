---
component_question: How should a required project-level architecture scope be synthesized and presented for approval?
out_of_scope: Feature behavior, general lifecycle rules, implementation, reviewer mechanics, and ordinary structural maintenance.
---

# Architecture Synthesis

<!-- DOCTRINE ADAPTER: architecture-entry -->

## Purpose

Resolve the minimum project-level architecture required before governed feature implementation.

## Inputs / Prerequisites

Read the selected architecture-synthesis and review policies. Use this prompt only when work would
otherwise establish or change an unresolved project-level boundary. Ordinary behavior-neutral,
architecture-insignificant maintenance uses the project's normal engineering process.

Identify affected governed features and their approved Specification Packages. Run the architecture
scope inspector, stop on invalid or duplicate membership, and reuse a matching scope or draft the
current architecture-scope template.

## Task

- Review approved feature requirements and applicable architecture together.
- Record only material project-level responsibilities, dependencies, state ownership, integration,
  runtime placement, constraints, and feature obligations.
- Derive decisions from approved requirements or explicit human architectural decisions.
- Return behavioral or new quality requirements to the affected Specification Package.
- Keep `approval: null` while responsibility, dependency direction, data authority, lifecycle, or
  integration conflicts remain unresolved.

## Applicable Checks

Apply the selected policy's synthesis and membership rules. Apply the selected review policy at the
architecture decision boundary.

## Output / Next Action

Present one complete scope artifact and state:

`AWAITING HUMAN APPROVAL OF THE ARCHITECTURE SCOPE`

After explicit approval, record `approval.by` and `approval.at`. Any later material change first
returns approval to `null` and repeats this boundary; reassess only affected work.
