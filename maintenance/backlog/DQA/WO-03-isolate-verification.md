# WO-03 — Isolate Verification and Account for Provider Calls

PROTECTED · after WO-02 · Charter criteria 3

## Goal

Prevent a production-source run from executing deterministic browser work and make provider cost
measurable.

## Work

- Separate deterministic, production-source, and post-production verification execution.
- Order cheap invalidating checks before expensive evidence (STAB WO-04 question A): pre-candidate
  qualification before production runs. Journey topology during upstream staleness is not
  patched here; stable accepted topology is decided in WO-05 and implemented in WO-06.
- Each verification action declares the execution environments it may use. A production-source
  run executes only production-authorized actions; anything else fails closed before any process
  starts. By default, browser and fixture actions are not production-authorized.
- Stop at an environment boundary and report the exact next command.
- Give each execution mode a deterministic identity that later shared-validation evidence can bind
  without inferring or changing provider mode.
- Account from provider attempts, not evidence receipts: the adapter records attempted,
  successful, failed, and total calls at the network boundary in a form that survives an abort.
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
4. Every new production-evidence record has reconciled call totals; one request that times out
   before the run aborts records attempted=1, failed=1, total=1.
5. No provider budget or threshold is invented by CodeOS.
6. A later shared-validation consumer can distinguish deterministic and production executions
   without inspecting an ambient environment variable.
