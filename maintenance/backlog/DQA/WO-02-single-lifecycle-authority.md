# WO-02 — One Executable Lifecycle Authority

NORMAL · after WO-01 · Charter criteria 2

## Goal

Remove disagreements caused by workflow, reviewer, and status code deriving the same lifecycle fact
independently.

## Work

- Inventory lifecycle derivations used by more than one component (STAB WO-04 question D).
- Centralize selected evidence, current revalidation bundle, journey frontier, required decision
  bindings, and configuration-dependent evidence requirements.
- Make workflow checks, Candidate Bundles, reviewer readiness, and status consume those results.
- Preserve Feature Workflow v9–v12 behavior.

## Acceptance criteria

1. Each shared derivation has one implementation authority and named consumers.
2. Consumer-agreement tests fail when one consumer substitutes a different rule.
3. Reacceptance cannot reorder journey topology.
4. New-format production evidence stays reviewable; the regression test added by `0514192` keeps
   passing.
5. Workflow, reviewer, and compatibility suites pass.
