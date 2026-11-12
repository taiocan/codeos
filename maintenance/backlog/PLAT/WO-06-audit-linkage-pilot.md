# WO-06 — Selective Audit-Linkage Pilot

PROTECTED · deferred until an approved project requirement names the need and privacy boundary

## Goal

Determine whether one real GUI action must be traceable through its backend outcome to a committed
database effect, and whether CodeOS should retain reusable guidance.

## Work

- Select one project action with an approved audit, incident, or regulatory requirement.
- Propagate a bounded correlation identity across only the necessary tiers.
- Prove the expected semantic outcome and database commit without recording sensitive payloads.
- Record the generalization decision; keep project-specific semantics in the project.

## Acceptance criteria

1. One valid chain and broken GUI-to-backend and backend-to-database links are tested.
2. The correlation identity does not become proof of correctness by itself.
3. No generic click logging, universal database trigger, or mandatory frontend Event Spine is added.
4. Generic CodeOS guidance is retained only if the pilot demonstrates repeatable value.
