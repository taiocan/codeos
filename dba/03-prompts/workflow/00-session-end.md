---
component_question: How should a DBA session handoff be produced from the current authoritative state?
out_of_scope: Changing governed artifacts, granting approvals, executing the next stage, and implementation work.
---

# DBA Session End: Handoff Generation

## Purpose

Produce a navigation aid for the next session. A handoff is not a DBA artifact and does not
override live repository state.

## Inputs

Review the conversation and current repository state for work performed, decisions made, rejected
paths, and unresolved questions.

## Task

Fill `.codeos/dba/05-guidance/templates/handoff.md` without copying governed artifact content.
Summarize approved decisions briefly, record rejected directions, list at most five open questions,
and recommend exactly one next action with a reason.

## Output / Next Action

Present one completed handoff ready to save as
`handoffs/[YYYY-MM-DD]-[short-description].md`. State that live artifacts and the registry must be
verified before acting on it. Do not perform additional stage work from this prompt.
