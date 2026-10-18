# WO-03 — Isolate Verification and Account for Provider Calls

PROTECTED · after WO-02 · Charter criteria 3

## Goal

Prevent a production-source run from executing deterministic browser work and make provider cost
measurable.

## Work

- Separate deterministic, production-source, and post-production verification execution.
- Order cheap invalidating checks before expensive evidence (STAB WO-04 question A): pre-candidate
  qualification before production runs. Until WO-06 implements the approved stable topology, do
  not run F3 against a frontier truncated by a STALE upstream feature (F-0006's F4 was re-confirmed
  for that reason).
- Stop at an environment boundary and report the exact next command.
- Give each execution mode a deterministic identity that later shared-validation evidence can bind
  without inferring or changing provider mode.
- Extend project production evidence with successful, failed, and total provider-call counts.
- Reconcile recorded totals with the project adapter's counter.

Today only JamFree's Playwright guard (`71adabb`) prevents the F-0005 incident, and call counts
are read from the receipts of successful calls only.

## Protected invariant

CodeOS never changes provider mode on the project's behalf, and production execution never crosses
into a verification that requires deterministic mode.

## Acceptance criteria

1. A production-mode check cannot launch Playwright or a governed fixture journey.
2. Stale F5 and F5g require separate correctly classified executions.
3. The F-0005 incident scenario stops before browser startup with zero extra calls.
4. Every new production-evidence record has reconciled call totals.
5. No provider budget or threshold is invented by CodeOS.
6. A later shared-validation consumer can distinguish deterministic and production executions
   without inspecting an ambient environment variable.
