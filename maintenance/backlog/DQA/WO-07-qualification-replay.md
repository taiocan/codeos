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
