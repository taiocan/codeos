# WO-10 — Atomic Post-Acceptance Revalidation Actions

ECP-5 conformance repair · NORMAL · from WO-06 gap E · Authority: Workflow Governance v4, WO-05

## Goal

For an accepted feature stale by `REVALIDATION_REQUIRED`, report the one next revalidation
operation, with an exact command when one fully performs it.

## Finding

Governance v4 already provides bounded revalidation: a `post_acceptance_revalidation` Candidate
Bundle restores current validity, without an F9 receipt, when meaning and architecture are
unchanged. The defect was ECP's non-atomic `revalidate_acceptance` action.

## Invariant

Status derives each step from the revalidation commands' own checks and reports their refusals; it
never advertises a command those checks would refuse.

## Steps (`revalidation::next_step`, first match)

| Condition | Action | Actor | Command |
|---|---|---|---|
| uncommitted surface change | `commit_change` | ASSISTED | — |
| `preflight` on the latest surface commit refuses | refusal | ASSISTED | — |
| current revalidation bundle, assessed, not passed | `resolve_review_finding` | ASSISTED | — |
| current revalidation bundle, not assessed | `run_review` | AUTO | — |
| stale revalidation verifications | `run_verification` | AUTO | `check … --revalidation --trigger <sha>` |
| `candidate::assemble` succeeds / refuses | `prepare_candidate` / refusal | AUTO / ASSISTED | `prepare-candidate … --revalidation --trigger <sha>` / — |

`revalidate_acceptance` stays a reserved schema-v3 value, never emitted. Running revalidation on
JamFree is a separate project action.
