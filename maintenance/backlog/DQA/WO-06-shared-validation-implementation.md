# WO-06 — Shared-Validation Implementation

PROTECTED Part B · after WO-05 approval · Charter criteria 4–7

## Goal

Implement exactly the shared-validation semantics approved in WO-05.

## Work

- Add content-addressed Validation Evidence Sets and sealed Human Validation Sessions at canonical
  workflow locations without duplicating their authority in feature receipts.
- Extend Candidate Bundles with claim-attribution references while retaining historical bundle,
  verification, decision, and revalidation readability.
- Add system validation that groups current requirements by compatible execution identity and runs
  each group once. Feature checks consume matching evidence or report the exact isolated command.
- Add one batched human interaction that shows every governed question and records only the
  explicitly selected outcome for each feature, checkpoint, and claim.
- Make workflow status, revalidation, candidate preparation, reviewer readiness, and packet
  construction consume the shared lifecycle derivations established by WO-02.
- Keep unsupported mechanics feature-scoped and fail closed. Enable deterministic system evidence,
  production-source evidence, and human observation only as their approved adapters become ready.
- Implement the stable accepted-journey topology while keeping stale and inherited claims blocked.
- Build an unnumbered candidate composition. Do not alter issued configurations or the active
  default pointer.

## Protected invariant

One execution or observation may have many explicit claim consumers; every semantic judgment and
acceptance remains feature-specific and human authority is never inferred.

## Acceptance criteria

1. One compatible validation identity produces one execution record regardless of consumer count.
2. Every consumer resolves to a current claim meaning, scope, and evidence slice.
3. One human session supports mixed explicit outcomes and cannot gain later implicit outcomes.
4. A stale accepted feature remains in journey topology but cannot satisfy a dependent claim.
5. Legacy projects and DBA-28 behavior remain unchanged.
6. Workflow, reviewer, configuration, boundary, and compatibility suites pass.
7. The protected invariants receive human acceptance before WO-07.
