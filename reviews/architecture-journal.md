# Codeos Architecture Journal

Cross-cutting institutional memory for the Codeos toolkit. One `AJ-NNN` entry per insight that
will still matter months from now to someone who has forgotten the change that produced it.
Append-only — supersede with a new entry, never rewrite. See `CLAUDE.md` → Review Logging.

---

## AJ-001 — Narrowing a change's scope means reconciling its originating brief

*Origin: UPG-0001 / CHG-20260627-001 (feature-thread-traceability), Step 1 review R1→R3.*

When a human narrows a change's scope at planning time, the **originating backlog brief — the
feature's source of truth — must be reconciled to the narrowed scope during Step 1.** Otherwise
the Step 1 change intent and the brief present two contradictory contracts for the same feature,
and the advisory reviewer correctly returns DO NOT ADVANCE (it cannot trace the work to a single
approved intent).

Concretely here: the human narrowed UPG-0001 to "mechanical migration only," "review-file
renaming deferred," and "historical change records not renamed." The change intent captured
those, but the brief still carried the original broader text (AC#9 review-traceability, migration
step 8 rename-existing-changes), and a state-vocabulary case mismatch (`in_progress` vs the
brief's own `IN_PROGRESS` enum). The reviewer flagged each as an in-scope blocker.

**Lesson / how to apply:** Reconciling the originating brief to the approved narrowed scope is
**in-scope Step-1 work, not scope creep.** A Step-1 "Change Intent" is not complete until the
change record and the brief tell exactly one story. Watch specifically for: acceptance criteria
that still demand the broader behavior, migration steps that contradict the narrowed plan, and
enum/vocabulary case drift between front matter and the canonical definition.

---

## AJ-002 — Acceptance criteria must be reproducible from committed artifacts

*Origin: UPG-0001 / CHG-20260627-001 (feature-thread-traceability), Step 2 review R1→R4.*

A Step-2 acceptance criterion is only real if its verification method is **reproducible from
pinned/committed artifacts**. The advisory reviewer runs read-only against a **dirty working tree
with no base pin**, so any check phrased as a plain `git diff` (workspace-relative) or against an
"earlier copy" / session scratchpad is not a stable acceptance test — it can pass or fail
depending on transient state.

**How to apply:** Pin every git-based acceptance check to a **recorded base SHA**, and for
"nothing-but-mechanical-changes" claims, diff against a **baseline commit** created before the
edits (so even previously-untracked files are tracked at the baseline). Reuse the toolkit's own
base-pinning (`codeos-review.sh stage-start --base`). Also: a verification method must actually
test its criterion — `ls <file>` proves existence, not identity/content/ownership; read the file
and assert the specific properties. And state exactly **one** method per criterion. See
[[AJ-001]] for the related rule that the step's contract must be internally singular.

---

## AJ-003 — Don't let an artifact record the review that is reviewing it (self-reference loop)

*Origin: UPG-0001 / CHG-20260627-001 (feature-thread-traceability), Step 4 review rounds R1–R5.*

When a compulsory advisory review assesses artifacts that **also contain the bookkeeping that
tracks the review** (a change record's `latest_review`, the dashboard "latest review" cell, a
Feature Thread review row), Step 4 enters an **infinite self-reference loop**: the artifact cannot
name the review currently assessing it, so every round flags the field as stale, and fixing it
just shifts the lag by one round. (Earlier seen, less acutely, in the `0001–0004` series — see
`UPG-0028`.)

**Fix — separate stable traceability from live review chronology:**
- Reviewed artifacts carry a **stable review-series id** `RVS__UPG-####__CHG-…__S<N>` + a
  `review_state` (DRAFT/IN_REVIEW/REVIEWED/ACCEPTED) — **never** an exact `REV__…__R<N>` round.
- Exact rounds, verdicts, packet hashes, and the human decision live **only** in
  `reviews/review-log.md` and `reviews/codex/*` (the "Surface ownership" rule).
- **Stop rule:** if two consecutive rounds find only stale review-bookkeeping caused by the
  previous round, stop editing the artifact and close by **human decision** — advisory, not
  gatekeeping. Don't chase `NO OBJECTION` on a structurally unsatisfiable field.

**How to apply:** never embed live review chronology in an artifact that is itself reviewed. The
doctrine + artifact structure landed here (Self-Reference Boundary in `prompts/codeos-self-dev.md`);
teaching the reviewer/packet to honor it is `UPG-0028`. Builds on [[AJ-001]] (singular contract)
and [[AJ-002]] (reproducible-from-committed-artifacts).

---

## AJ-004 — Adding a triage category requires sweeping every prose enumeration, not just the primary table

*Origin: UPG-0030 / CHG-20260629-001 (lean-review-profiles), Steps 3–4, multiple rounds.*

When a new triage category is introduced (here: `SELF-REFERENCE / REVIEW-BOOKKEEPING` as the fifth
category), it is easy to add it to the **primary triage table** and miss every **inline prose
enumeration** scattered across governance docs. UPG-0030 required fixes in four separate locations
across three review rounds before all enumerations were consistent:

- `prompts/codeos-self-dev.md` — Step 4 checklist and Feature Thread triage list (both missed initially)
- `templates/codeos-change.md` — Reconciliation triage table (missed initially)
- `CLAUDE.md` — Step 4 loop summary (caught by R3 budget-exhaustion inline fix)
- `docs/reviewer-pipeline.md` — §2 Scope Contract triage rule (caught by R3 budget-exhaustion inline fix)

**Lesson / how to apply:** When any governed taxonomy changes (triage categories, scope-axis values,
profile names, verdict labels), treat it as a **cross-doc refactor**: grep the whole toolkit for
every place the old enumeration appears before declaring the change complete. Do not rely on the
primary table being the only place the list is written. The acceptance criteria for any such change
must explicitly require a grep sweep and name the files expected to contain enumerations, rather
than just asserting "updated everywhere." See [[AJ-003]] for the related self-reference pattern
that makes catching these missed instances even harder inside review rounds.
