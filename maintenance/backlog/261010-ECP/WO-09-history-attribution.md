# WO-09 — Process History Attribution

ECP-4 · NORMAL · from WO-06 gap F · high priority · Charter: O-3, success criterion 9, C-6

## Goal

Locate where recorded process-friction signals occurred, not only how many. Breakdowns count
events; they measure neither effort, cost, nor cause.

## Semantics

- **Subordinate attribution.** Each observed metric builds one tally over exactly the events
  admitted to its total; `value`, `breakdown`, and `unattributed` derive from it, so
  `sum(breakdown) + unattributed == value` holds by construction. No metric is recomputed for
  attribution.
- **Keys** are the `subject` and, where carried, the `checkpoint` of each counted record, verbatim:
  receipts and verification records (subject, checkpoint); review-log entries (heading subject,
  `Stage n`); Candidate Bundles (`feature`); Operation routes (`target_id`); accepted features
  (subject). Observation records and closures name no subject and are `unattributed`; a later
  route target is not applied to them.
- **No reason key.** The only candidate, the review effective concern, is constant
  (`DO NOT ADVANCE`) once the metric filters on it; reviewer prose is never a key.
- **Review headings** are parsed only by `evidence::review_heading`, the reviewer-owned grammar
  `{timestamp} {KIND} — {feature} — Stage {stage}`. An unparseable heading still counts,
  unattributed.
- **Windows.** `accepted_features` filters acceptance receipts to the window, then counts distinct
  subjects. A receipt is *additional* when an earlier receipt, at any time, exists for the same
  workflow, subject, and checkpoint; it counts when its own timestamp is in the window.
  `candidate_bundles` is all time. Every other metric uses the event's own timestamp.
- **Presentation.** Parts are ordered by key, never by value (C-6). The terminal lists several
  parts under the total, shows a single part inline, and adds nothing for a wholly unattributed
  metric; JSON always carries `breakdown` and `unattributed` for observed metrics. Additive to
  schema v3; plain `status` unchanged.

## Acceptance criteria

1. On JamFree, additional decision receipts and non-progressing assessments are localized to
   subjects and checkpoints.
2. Every breakdown sums to its total or states what is unattributed.
3. Plain `status` output is unchanged.
