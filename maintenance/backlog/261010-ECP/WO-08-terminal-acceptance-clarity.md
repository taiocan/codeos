# WO-08 — Terminal Acceptance and Health Clarity

Presentation · NORMAL · from WO-06 gaps A, B, D · Charter: O-1, C-3, C-10

## Goal

The terminal view distinguishes accepted-but-stale features from work never accepted and shows
why each health domain has its status. Only `monitor::render` changes; JSON, derivation
(including `current_feature`), and exit codes are unchanged.

## Rendering rules

- **Project line.** With no derived active feature, features not complete are split into
  `n not yet accepted` and `n accepted, currently STALE`; zero-count parts are omitted.
- **Feature lines.** A historically accepted feature (governance v3+) appends
  `accepted snapshot <date> · current validity <STATUS> (<reasons>)`, using the glossary's
  Accepted Snapshot and Current Validity. The date is the first ten characters of `accepted_at`,
  without timezone conversion. Never-accepted features are unchanged.
- **Health basis.** `Health.basis` is printed verbatim and in order: inline after the status when
  single, indented beneath it when several, omitted when empty.

## Acceptance criteria

1. On JamFree, F-0001–F-0004 read as accepted snapshots with current validity STALE, not
   incomplete.
2. Every terminal health status is accompanied by its basis.
3. `status --json` output is byte-identical.
