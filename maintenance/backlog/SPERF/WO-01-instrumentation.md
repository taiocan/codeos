# WO-01 — Instrumentation and Golden Report

NORMAL · Charter: Acceptance criteria 1–5

## Goal

Make the cost measurable and the semantics checkable before anything is optimized.

## Work

- **Counters.** A test-visible count of configuration resolutions, cargo metadata runs, Git
  subprocesses and binding computations per command.
- **Golden report.** A representative fixture project: Bootstrap plus several features, with at
  least one accepted and one stale. Capture its full `status --json` report as the identity
  reference.
- **Baseline.** Record JamFree's counts and wall time.

## Acceptance criteria

The counters and golden report are committed, and the baseline is recorded below.

## Results
