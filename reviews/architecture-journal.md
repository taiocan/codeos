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
