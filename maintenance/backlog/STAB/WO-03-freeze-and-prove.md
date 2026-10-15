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

### Inconvenience log

- 2026-10-14 (during WO-01): the standalone Codex CLI updated itself from 0.162.0 to 0.162.1. The
  reviewer correctly refused it ("differs from approved release identity 0.162.0", from
  `maintenance/release-lock.json`). The run went ahead only by putting the installed 0.162.0
  binary first in `PATH` for that one command. This is not a defect, since the lock behaved
  correctly, unless the pinned binary stops being reliably available.
