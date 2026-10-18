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
- Preserve Feature Workflow v9–v12 behavior. Invariant: a governed lifecycle fact is derived
  exactly once; consumers format or apply it and never reconstruct it.
- If consumers disagree today and resolving it needs a semantic choice, that part stops and becomes
  PROTECTED.

## Acceptance criteria

1. Each shared derivation has one implementation authority and named consumers.
2. Each derivation has one owner test plus cross-consumer contract tests covering a pre-bundle
   acceptance, DBA-27/v11, DBA-28/v12, an accepted-but-stale feature, and a revalidated-current
   feature; they fail when one consumer substitutes a different rule.
3. Reacceptance cannot reorder journey topology.
4. New-format production evidence stays reviewable; the regression test added by `0514192` keeps
   passing.
5. Workflow, reviewer, and compatibility suites pass.

## Inventory (2026-10-18)

| Lifecycle fact | Authority (`codeos_workflow`) | Consumers | Change |
|---|---|---|---|
| Verification-record currency | `checker::verification_bindings_for_mechanic` + `verification::evaluate` | `check`, Candidate Bundles, revalidation, health, review readiness | Readiness had its own rule, which ignored every input except the implementation and the Specification Package; revalidation had its own rule for runtime events, which ignored the result |
| F6 decision currency | `checker::decision_state` (extracted from `check`) | `check`, review readiness, FAI fixtures | Readiness had its own v11/v12 dispatch, which ignored the Specification Package and the selection |
| F6 production-evidence selection | `ReceiptStore::latest`/`latest_as_of`; the revalidation Candidate Bundle's selection | `check`, Candidate Bundles, review readiness, lineage replacement, assessment input correction | Four reviewer parsers of `decisions.jsonl` removed |
| Selected-evidence currency | `checker::validate_production_evidence_selection` | `decide`, review readiness | Readiness had its own per-format rule (`0514192`); that test moved to the owner with the same name |
| Candidate verification set | `checker::candidate_verification_records` | Candidate Bundles, review readiness | The `candidate` checkpoint mapping had been repeated in both |
| Accepted journey topology | `checker::accepted_chain` | `status`, `journey`, record and decision bindings | Already one authority |

The readiness changes restore the Reviewer Tool Contract (v17: `EVIDENCE_STALE`, evidence not bound
to the current implementation **and inputs**). Contract tests: `reviewer/engine/tests/lifecycle_contract.rs`.
Reviewer fixtures that recorded states the workflow never writes now record what `decide` and
`check` write.

## Open: revalidation evidence set (PROTECTED)

`revalidation::required_verifications` (F3 smoke and product journey, F5 behavior and
repeatability, F5g, runtime events in the `events` mode, production evidence) is a second list of
the evidence an accepted feature needs. Unlike `required_verification_records`, it omits F5d
`data_integrity` and F9 `project_qualification`. So for a persistence feature, a revalidation
candidate can be prepared with stale data-integrity evidence, which Stage 8 readiness then
refuses (from reading the code; no test covers it).
Feature Workflow v8 says the checker "derives stale mechanics from current bindings" but does not
name the set. Unchanged pending a human decision.
