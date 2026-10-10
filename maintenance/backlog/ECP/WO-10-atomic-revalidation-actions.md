# WO-10 — Atomic Post-Acceptance Revalidation Actions

ECP-5 conformance repair · NORMAL · from WO-06 gap E · Authority: Workflow Governance v4, WO-05

## Finding

Bounded revalidation already exists: Governance v4 lets a `post_acceptance_revalidation`
Candidate Bundle restore current validity, without an F9 receipt, only when Specification Package
meaning and applicable architecture are unchanged. The defect was ECP's `revalidate_acceptance`
action, which named the multi-step outcome (violating WO-05) and pointed at no mechanism. No
workflow semantics, acceptance rule, or revalidation authority is added.

## Invariant

Status derives the next revalidation action from the same Governance v4 eligibility checks the
revalidation commands apply; it never approximates whether revalidation is allowed. A refusal on
the way is reported, not advertised as a command.

## Steps (`revalidation::next_step`, first match)

| Condition (shared predicate) | Action | Actor | Command |
|---|---|---|---|
| accepted surface has uncommitted changes (`surface_uncommitted`, used by `identified_change`) | `commit_change` | ASSISTED | — |
| trigger = latest surface commit; `preflight` refuses | none — next action states the refusal | ASSISTED | — |
| current revalidation bundle for this acceptance and trigger, assessed and not passed | `resolve_review_finding` | ASSISTED | — |
| current revalidation bundle, not assessed | `run_review` | AUTO | — |
| `stale_revalidation_verifications` (used by `check --revalidation`) non-empty | `run_verification` | AUTO | `check --workflow feature --subject <F> --revalidation --trigger <sha>` |
| `candidate::assemble` (every `prepare-candidate` check, without writing) succeeds | `prepare_candidate` | AUTO | `prepare-candidate --subject <F> --revalidation --trigger <sha>` |
| `candidate::assemble` refuses | none — next action states the refusal | ASSISTED | — |

Commands carry the exact derived trigger SHA. `revalidate_acceptance` stays a reserved schema-v3
value and is never emitted; `commit_change` is added. Legacy configurations without Workflow
Governance v3 keep their previous untyped next action.

## Acceptance criteria

1. On JamFree, each `REVALIDATION_REQUIRED` feature shows its atomic step or the exact refusal.
2. Every emitted command is one the corresponding command's own checks accept.
3. Running revalidation on JamFree is a separate, deliberate project action.
