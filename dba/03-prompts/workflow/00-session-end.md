---
component_question: How should a DBA session handoff be produced from the current authoritative state?
out_of_scope: Changing governed artifacts, granting approvals, executing the next stage, and implementation work.
---

# DBA Session End: Handoff Generation

## Purpose

Produce a navigation aid for the next session. A handoff is not a DBA artifact and does not
override live repository state.

## Inputs / Prerequisites

Review the conversation and current repository state for work performed, decisions made, rejected
paths, and unresolved questions.

## Task

From live artifacts and Git, summarize the target, current authoritative state, approved decisions
that matter for resumption, unresolved questions, and exactly one next action. Reference governed
artifacts rather than copying their content. Do not create a handoff file unless the human
explicitly asks to persist one.

## Output / Next Action

Present the concise navigation summary inline and state that live artifacts must be verified when
work resumes. This prompt creates no durable project artifact. Do not perform additional stage
work from this prompt.
