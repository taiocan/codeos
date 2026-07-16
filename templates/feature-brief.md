# Feature Brief: [F-####] — [short name]

<!--
PURPOSE OF THIS FILE:
Pre-Stage-1 discovery artifact. Captures the human's understanding of a feature
before it enters the DBA pipeline. This is NOT a DBA artifact:
  - No actor+outcome form
  - No stable guarantees
  - No scope boundaries in DBA format
Stage 1 (01-intent.md) derives all of that from this brief.

LIFECYCLE: Once stage1_started is filled in, this brief is frozen.
The Intent document (intents/[feature_id].md) supersedes it for all DBA purposes.
No further maintenance of this brief is required or expected.

TYPE:
  F-type = new feature
  R-type = refinement to an existing feature (must name the feature being refined)

FEATURE ID:
  Format: F-#### (4-digit, zero-padded, sequential, never reused).
  Assigned by the interviewer at Synthesis, not chosen by the human — F-type gets the
  next available id; R-type reuses the parent feature's id. See
  .codeos/templates/conventions.md -> Feature IDs.

HOW LONG: 10-15 minutes. If you are writing more than one screen per section,
stop — Stage 1 handles the detail.

TO FILL IN: Replace every [bracket] with your content.
Remove comment blocks before saving.
-->

**Type**: F-type | R-type ← delete one  
**Refines** *(R-type only)*: [F-#### being refined]  
**Tier** *(rough guess, or "unknown")*: [e.g., "3 — Analytics"]  
**Status**: BRIEF-DRAFT

---

## Problem / Need

<!--
WHY does this feature need to exist? 2-4 sentences in plain language.
  - What is broken, missing, or painful today?
  - Who feels it?
  - R-type: what is the observed failure or gap that triggered this?
    A valid R-type trigger is one of:
      recurring failure | reconciliation gap | replay failure |
      observability gap | human-approved evolution

DO NOT describe the solution. Describe the problem only.

Good:  "PMs currently have no way to see all items ranked by urgency
        without manually scanning the full project record."
Bad:   "We need to add a priority sort flag to the view command."
-->

[Describe the problem in plain language.]

---

## Primary Actor

<!--
One sentence: who is the human role that directly uses or benefits from this feature?

Use a role name — not a person name, not "the system":
  "the PM", "the developer", "the reviewer", "the operator"

"The system" is never a valid actor. Systems are mechanisms, not actors.
-->

[One sentence naming the role and their relationship to the problem.]

---

## Core Outcome (informal)

<!--
What becomes possible or true after this feature exists?
1-3 sentences from the actor's perspective.
  - What can they DO or KNOW that they cannot today?
  - Focus on the actor's experience, not the system's behavior.

Do NOT write actor+outcome DBA form — Stage 1 derives that.
Do NOT mention implementation technology, events, APIs, or data structures.

Good:  "The PM can see all items ranked by priority in one view,
        filtered by type or status, without leaving the terminal."
Bad:   "A priority_view command will query the project record and
        sort by the priority field using a descending comparator."

R-type: state what CHANGES compared to current behavior.
-->

[What becomes possible or true after this feature exists.]

---

## Design Tensions and Open Questions

<!--
What are the unresolved decisions that Stage 1 or Stage 2 will need to settle?
List them as questions. Minimum: 1. Aim for 2-4.

Good question forms:
  "Should X apply to both Y and Z, or only Y?"
  "What happens when [edge case] — skip silently, error, or partial result?"
  "Does this feature own the vocabulary for [concept] or consume it from [other]?"
  "Is [scope element] in or out for v1?"

If you have a tentative answer, put it in brackets after the question.
Stage 1 will confirm or surface a better option.

A brief with no open questions is a red flag. There are always tensions.
-->

1. [Question]
2. [Question]
3. [Question — tentative answer: ...]

---

## Suspected Dependencies

<!--
Which features or modules is this brief believed to depend on?
List each with a brief reason.

These are GUESSES — beliefs held before DBA analysis, not architectural decisions.
Stage 1 will verify and may revise them significantly.

If nothing is known, write: "none suspected — Stage 1 to determine."
-->

- [feature_id or module]: [why it is suspected to be needed]
- [feature_id or module]: [why it is suspected to be needed]

---

## Rough Scope Notes

<!--
OPTIONAL. Skip if you have no strong scope intuitions yet.

List things you believe are IN scope and OUT of scope for v1.
These are starting intuitions, not DBA scope boundaries.
Stage 1 will formalize them.

Example:
  In:  basic ranking by priority; filter by type; filter by status
  Out: custom sort orders; saved filters; export
-->

In scope (rough): [...]  
Out of scope (rough): [...]

---

## Readiness Check

<!--
Answer each item before advancing to Stage 1.
If any item is NO, fix the gap first.
-->

- [ ] The problem statement explains WHY, not HOW
- [ ] The primary actor is a human role, not "the system"
- [ ] The core outcome is stated from the actor's perspective
- [ ] At least one open question is listed
- [ ] Suspected dependencies are named (even if marked uncertain)
- [ ] No actor+outcome DBA form appears anywhere in this brief
- [ ] No stable guarantees or DBA scope boundaries appear in this brief
- [ ] The feature can be described without mentioning implementation technology
      (no "dispatcher table", "routing layer", "database schema", "sort comparator", etc.)
- [ ] (R-type only) A valid refinement trigger is identified in the Problem section

**Brief status**: READY FOR STAGE 1 | NEEDS MORE WORK ← delete one

---

<!-- METADATA -->
brief_created:
brief_last_updated:
stage1_started:
