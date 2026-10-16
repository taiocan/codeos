# WO-03 — Freeze and Prove with Normal Features

P3 · NORMAL · after WO-02 · Charter: Success criterion, Phases P3, Measures

## Goal

Deliver qualifying normal features until the total reaches three, at least two of them after
WO-02. The Charter's P3 freeze rule applies throughout.

## Measurement

For each feature, record the Charter's measures:

- CodeOS commits between the feature's first F1 receipt and its F9 receipt (from Git), each
  classified as capability, conformance, or unrelated;
- the diagnostic values from `codeos-workflow status --history`;
- whether the feature qualifies.

Log every inconvenience, extra manual step, or desirable optimization for WO-04 instead of fixing
it.

## Acceptance criteria

Three qualifying features in total, or an explicit human decision to stop.

## Baseline (2026-10-15)

JamFree bootstrap is current (B1–B5; B4 reconfirmed against the amended Charter). F-0001 to
F-0004 are VALID. One known limitation is recorded for P4: the status view's project-qualification
rows read only acceptance-time records (WO-04, question D). No CodeOS correctness blocker is open.
CodeOS is frozen from this baseline onward; STAB backlog records are not CodeOS changes.

During P3, classify every delivery blocker before acting: product defect, project configuration,
expected governed behavior, or CodeOS correctness defect. Only a CodeOS correctness defect breaks
the freeze.

## Results

This section is the authoritative qualifying-feature count.

| Feature | First F1 → F9 | CodeOS commits (class) | Diagnostics | Qualifies |
|---|---|---|---|---|
| JamFree F-0003 | before STAB | — | — | Yes (historical) |
| JamFree F-0005 | F1 2026-10-15 → F9 2026-10-16 | `f5a8f9a` (conformance) | — | No: needed a CodeOS conformance repair at F2 |

### Inconvenience log

- 2026-10-14 (during WO-01): the standalone Codex CLI updated itself from 0.162.0 to 0.162.1. The
  reviewer correctly refused it ("differs from approved release identity 0.162.0", from
  `maintenance/release-lock.json`). The run went ahead only by putting the installed 0.162.0
  binary first in `PATH` for that one command. This is not a defect, since the lock behaved
  correctly, unless the pinned binary stops being reliably available.
- 2026-10-15 (F-0005, F2): F-0005 exposed one CodeOS correctness defect. CodeOS rebuilt the
  accepted journey from the order of acceptance receipts, so F-0001's legitimate re-acceptance
  made a valid chain unreadable. The repair is conformance commit `f5a8f9a`. F-0005 continues
  through F1–F9 but does not count toward the zero-CodeOS-change proof.
- 2026-10-15 (F-0005): `status` and `next` take about 100 s on JamFree, against about 47 s before
  `f5a8f9a`. Plausible, not proven: each frontier derivation now evaluates current acceptance
  validity for all four accepted features, where the old journey error stopped after two. Trigger
  for a performance root-cause trace: latency gets materially worse, blocks normal delivery, or
  grows nonlinearly as features are added. Root cause established 2026-10-15: repeated
  recomputation and subprocess fan-out in the evaluator; see [SPERF](../SPERF/charter.md).
- 2026-10-15 (F-0005, expected): the window must change `web/src/routes/+page.svelte`, an
  implementation input of all four accepted features' bundles. F-0001 to F-0004 will need
  revalidation, about 108 TomTom calls and four Codex assessments (WO-04, question B).
