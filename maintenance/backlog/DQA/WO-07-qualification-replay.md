# WO-07 — Qualification and JamFree Replay

NORMAL · after WO-06 acceptance · Charter criteria 1 and 4–7

## Goal

Prove the approved shared-validation semantics generically and against the JamFree cases that
motivated them before issuing a supported DBA configuration.

## Work

- Apply WO-01's candidate-configuration qualification without giving the candidate downstream
  authority.
- Run generic workflow, reviewer, configuration, boundary, compatibility, and adversarial suites.
- Replay every WO-04 fixture in disposable JamFree clones. At most one live provider run, only if
  deterministic stubs cannot establish the real-adapter accounting criterion, and only with its
  own call approval.
- Compare actual DBA-28 executions and human actions with unique executions, shared observation
  sessions, explicit feature outcomes, provider calls, and stale claims under the candidate.

## Acceptance criteria

1. JamFree state `7bcc9146` executes each compatible automated validation identity once while all
   required feature claims keep inspectable attribution.
2. Each F-0001/F-0006 identical-surface pair becomes one human observation session with two
   explicit outcomes; a mixed-result replay also passes.
3. A later feature, unchanged pixels with a changed question, and incomplete viewport coverage do
   not reuse an old human judgment.
4. Staling F-0001 leaves the accepted journey topology intact while F-0006's inherited claim stays
   blocked until its upstream evidence is current.
5. Direct production claims execute once per compatible scope; inherited claims make no production
   call of their own; provider-call totals reconcile.
6. Legacy coarse claims and prospective Contract claims consume shared evidence without false
   currency or historical rewriting.
7. The human accepts the generic qualification and JamFree replay before the candidate is issued
   as DBA-29. JamFree adoption and any active-default change remain separate explicit decisions.

## Qualification (2026-10-18)

`qualify-candidate.sh dba-29` at `dqa` `7c133d9`: suites, downstream compatibility matrix and five
replay cases PASS; `qualified_candidate_sha256: 99a0392b2095ccb2de7f66be565adf24068a87ee6b3f729a2841927bb250dcc6`.
No live provider run: the stub verifier establishes the accounting criterion deterministically.

| Criterion | Replay case | DBA-28 actual | DBA-29 candidate |
|---|---|---|---|
| 1 | `jamfree-7bcc9146-shared-validation` (56d28a2), five features | 24 runs of four project-wide commands, 9 journeys, 6 qualifications | 7 executions for 29 attributed records (one per execution identity; F-0003..F-0005 share F-0002's return journey) |
| 2 | `jamfree-f4-pair-{a9a2852f,91451275,89c5edb7}` | 6 separate F4 decisions | 3 sessions, 2 explicit outcomes each; a mixed outcome leaves only F-0006 open |
| 3 | same | — | F8 on the same pixels refused; later feature unchanged; missing viewport reopens F4 |
| 4 | `jamfree-7bcc9146-shared-validation` | accepted frontier drops with staleness | frontier stays `return_departure_decision`; F-0006's inherited claim blocked on F-0001 |
| 5 | same | one 27-call leg per feature | one shared run, 2 legs (outbound, return) for F-0002..F-0005, totals reconciled; F-0006 makes no production run |
| 6 | all; generic tests for prospective claims | — | legacy coarse claims consume shared evidence; 0 history lines removed |

Found and repaired by the replay (`7c133d9`): a mid-chain feature's journey under stable topology,
and the missing shared form of production revalidation.
