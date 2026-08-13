# DBA Session End: Handoff Generation

<!--
HOW TO USE THIS FILE:
Paste this at the end of a working session to produce a handoff document.
Claude will read the session's approved artifacts and produce a compact handoff file.
-->

---

You are closing a DBA session. Produce a session handoff document that the next session
can use to orient itself without reconstructing context from artifacts alone.

**This is not a DBA artifact.** It does not feed into any stage. It does not require
approval. It is a context-preservation record.

## What You Produce

A single file: `handoffs/[YYYY-MM-DD]-[short-description].md`

Use the template at `.codeos/dba/05-guidance/templates/handoff.md`.

## Instructions

**Step 1 — Identify what happened this session.**

Review the conversation to determine:
- Which features or refinements were worked on
- Which stages and doctrine adapters were completed
- Which artifacts were created or modified

**Step 2 — Summarize approved decisions, not artifact content.**

Do not copy artifact text. If the `specification-approval` adapter ran this session, summarize its
three artifacts in 1–3 lines each:
- Intent: the WHY and actor-outcome core, not implementation details
- Contract: the key scenarios and failure modes that were settled
- Schema: event names and any notable payload decisions

**Step 3 — Record rejected paths explicitly.**

List directions that were considered and ruled out this session. These are the most
valuable part of the handoff — they prevent the next session from re-exploring dead ends.

**Step 4 — State exactly one recommended next step.**

One action, one reason. If multiple options exist, name them and say which is preferred.
Also name the most tempting wrong next move and why it should be avoided.

**Step 5 — List open questions.**

Questions that arose but were not resolved. Keep to ≤5 items. Blocking questions first.

## Output

Present the completed handoff as a fenced markdown block, ready to save.

Then state:

---

**Session Handoff: READY**

Save to `handoffs/[YYYY-MM-DD]-[short-description].md`.

This file is a navigation aid, not a DBA artifact. It does not override approved
artifacts or the feature registry. Verify current state from live artifacts before
acting on this handoff.

---

Do not run any stage work after producing the handoff. If the human wants to continue
working, start a new session using `.codeos/dba/03-prompts/workflow/00-session-start.md`.
