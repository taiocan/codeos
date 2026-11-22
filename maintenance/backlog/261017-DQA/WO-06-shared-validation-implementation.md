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

## Implementation (2026-10-18)

Semantics: Feature Workflow v13 (`dba/02-policies/workflow-feature/v13.md`), DBA-29 candidate only.

| Approved item | Derivation |
|---|---|
| Validation Evidence Set, execution identity (with Compose build contexts) | `evidence_set.rs`; `checker::run_shared`, `shared_execution` |
| Prospective claims, meaning digest, successor rule | `claims.rs`; `checker::subject_claims`; bundle `claim_digests` |
| Shared production run and per-feature evidence slices | `checker::run_shared_production_check` (`check --subject A --subject B --environment production`) |
| Human Validation Session | `session.rs`; `codeos-workflow session`; `decision_state` attests the session |
| Stable accepted topology; only consumption (inherited claims) needs currentness | `checker::accepted_journey_chain` |
| Status: one acceptance projection per feature per command | `Project::memo_acceptance`, `memo_review_record` (all configurations; output unchanged) |

Evidence (`tests/shared_validation.rs`, `tests/execution_environments.rs`): one execution for two
features; a fixture-emulator change stales its consumers with the implementation state unchanged;
one production run for two named features, distinct slices, an unnamed feature not covered, one
ledger; a feature the shared run does not establish fails alone while the other passes; nothing
starts unless every named feature is ready; a stale producer stays in the topology, a later feature
in the journey is not blocked by position, and a claim inherited from it stays blocked; a successor
claim ID clears the identity block but inherits no evidence (a human claim needs its own outcome, a
`run:` claim a record bound to the current Contract); unattributable, duplicated, foreign-owned and changed-meaning claims fail
closed and a successor ID does not; a session with mixed outcomes, no propagation to an unnamed
feature on the same surface, no later attestation, a not-met human claim; DBA-28 unaffected.

JamFree `172282b` under DBA-28: `status --json` identical (`cf5bfc6f…`), 12.4–13.0 s → 2.2 s,
acceptance derivations 136 → 6, file hashes 4,545 → 458. Replay PASS for a DBA-28-equivalent
candidate and for `dba-29`. DBA-29 closes the execution-identity gap prospectively, for v13
records; a record written before v13 stays governed by the bindings it was written under until it
is regenerated. This is the compatibility boundary, not an open defect.

Glossary: **Verification record** is frozen by DBA-6..DBA-22 (`term:` closure rows), so it is not
edited; v13 uses the forward-only term **Attributed verification record**. The semantic-closure
tests prove that broadening the frozen definition fails the integrity check.

Accepted 2026-10-18, with existing-evidence reuse for successor `run:` claims subject to the
coverage invariant now stated in v13 (semantic change invalidates attribution, not execution
evidence; coverage is judged by Contract approval and Stage 8, not by Codeos).
