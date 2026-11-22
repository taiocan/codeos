# WO-01 — Freeze the Terminal Output Contract

NORMAL · Charter criteria 1–5

## Goal

Define the compact view before changing the renderer.

## Work

- Capture JamFree and fixture reports as before/after golden text.
- Specify section order: header, summary, health, features, attention, details.
- Define columns, truncation, grouped reasons, and narrow-terminal wrapping.
- Map every current text fact to its new location or explicit omission rule.

## Acceptance criteria

1. Every state, reason, evidence basis, and next-action command has a mapping.
2. Common rows contain labels and fragments rather than full sentences.
3. Widths of 80 and 120 columns have deterministic output.
4. No colour is required to interpret state.

## Contract (2026-10-17)

Owned by `monitor::render_width` and its tests. It covers project-wide text `status` only;
`status --json`, targeted `status --subject`, and `status --history` are unchanged.

- **Order:** header (project · configuration · Release state), SUMMARY, HEALTH, FEATURES,
  inventory warnings, ATTENTION, DETAILS, COMMANDS, current-feature CHECKPOINTS, open observations.
- **Width:** `COLUMNS` when it is a number of at least 40, otherwise 100. Only text columns shrink:
  first by word-wrapping down to their longest word, then, only if still needed, by splitting words,
  preferably after `/`, `-`, or `_`. Identifier columns keep their full width; nothing is truncated.
  COMMANDS lines are printed whole so they can be copied.
- **Grouping:** health bases that differ only by a leading feature ID are listed once after their
  IDs; a domain whose bases equal an earlier domain's reads "same as <domain>". DETAILS lists each
  issue, unchanged-surface set, and next action once with every subject it applies to.
- **Plain text:** state is always a word (no colour); markers are ASCII except existing dashes,
  arrows and `·`, each one column wide.

| Previous fact | New location |
|---|---|
| Project name, configuration | Header |
| Bootstrap checkpoint marks and count | SUMMARY `Bootstrap`: count, plus every checkpoint not COMPLETE with its state |
| Active feature line | SUMMARY `Active feature`, same wording |
| Health domain, state, each basis | HEALTH row; bases grouped or referenced as above, text verbatim |
| Feature ID, name, state, accepted date, current validity and reasons | FEATURES row |
| Feature checkpoint (POINT) | ATTENTION row (every incomplete feature is an attention item) |
| `◀ CURRENT` marker | `(current)` after the name, and SUMMARY `Active feature` |
| Planned candidates | FEATURES rows `CANDIDATE (NOT STARTED)` |
| Inventory warnings | Unchanged section |
| Attention class, subject, actor, changed surfaces | ATTENTION row |
| Attention issue (+ reason code), unchanged surfaces, next action | DETAILS, grouped |
| Executable command (previously JSON only) | COMMANDS, one per subject |
| Current feature checkpoints, issue, additional-issue count, next action | CHECKPOINTS table; DETAILS `issue`, `also`, `next` |
| Open observations | Unchanged section |
