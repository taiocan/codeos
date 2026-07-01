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

## AJ-005 — Template instruction ≠ mechanical enforcement; state the boundary explicitly

**Source:** UPG-0004 / CHG-20260630-001, Step 1 review (2026-06-30)

When a template says "field X must not be empty — write `none` if absent," that is an instruction to practitioners enforced by human review, not by a script. If Step 1 describes the rule without qualifying this, a reviewer will flag it as a false claim (implying validation exists when it does not). Fix: always pair a template rule with an explicit statement — "This CHG does not add script-level enforcement" — so the claim is scoped to what the template actually does. Applies to all future template and prompt CHGs that introduce must/required/not-permitted language.

## AJ-006 — Acceptance criteria referencing loop-step names become stale at every gate transition

**Source:** UPG-0004 / CHG-20260630-001, Steps 3–4 (2026-06-30)

An acceptance criterion written as "status row shows step 2-Acceptance" becomes a false claim the moment the step advances. This is a predictable self-reference trap: the criterion correctly described the state when written, but each gate update invalidates it without any mistake in the implementation.

**Fix pattern:** Criteria for bookkeeping state should be written dynamically — "Loop step reflects the current step at the time of verification" — rather than hardcoding a step name. The same applies to any criterion that asserts the value of a field that is expected to change during the change's own lifecycle (e.g. `current_step`, `state`, `review_state`).

**How to apply:** In Step 2, scan acceptance criteria for any that hardcode a value that will be mutated by the change's own 4-step progression. Replace the hardcoded value with a dynamic description. Catches the same class of bug as [[AJ-003]] (self-reference boundary) but at the AC-level rather than the artifact-content level.

## AJ-007 — A change to a shared function invalidates mode-specific "behavior unchanged" scope claims

**Source:** UPG-0031 / CHG-20260630-002, Steps 3–4 (2026-06-30)

When a fix modifies a function that is called by all review modes (e.g., `run_prechecks`, called regardless of `--mode delta` or `--mode full`), scope claims such as "existing `--mode full` behavior unchanged" are false — even when the intent was to fix only delta-mode behavior. The precheck change (Fix D) affected both modes, making the guardrail and scope boundary wrong before they were reviewed.

**Why:** Mode-specific scope claims are only safe if the changed code is inside a branch that is exclusive to that mode. Shared functions crossed by multiple modes cannot be protected by a single-mode "unchanged" claim.

**How to apply:** Before writing scope boundaries and guardrails for any script change, enumerate every function and code path touched by the change. For each one, check which modes invoke it. If a changed function is called in modes other than the one being targeted, the scope boundary must either exclude those modes from the "unchanged" claim or explicitly state that precheck/shared behavior is intentionally changed. See [[AJ-005]] for the related pattern of template instruction vs. enforcement boundary.

---

## AJ-008 — Pipeline step ordering matters: filter composition can silently hide content

*Origin: UPG-0031 / CHG-20260630-003 (precheck-filter-correction), 2026-06-30.*

When a text filter is implemented as a pipeline of `sed`/`grep` steps, the **order of steps
determines what each subsequent step can see**. A step can silently hide content from all
later steps — even content that would have been caught. Here: `sed '/<!--/,/-->/d'` ran before
inline code span removal. A code span containing `` `<!-- … -->` `` opened an HTML-comment
deletion range at the line containing `` `<!--``, and because GNU sed checks the range-end
pattern only from the NEXT line after the range-start match, the range stayed open until the
next `-->` in the file — silently deleting an entire section (lines 72–113 in the affected
file). The acceptance-criterion smoke test happened to be on a line inside that deleted range,
so it never reached the grep, producing a false PASS.

**Why this is subtle:** The deleted section did not contain a real unfilled placeholder — so
the test's conclusion (no placeholder found) was accidentally correct. The error was invisible
until post-commit verification tried the precheck on the same file and got `exit: 2`.

**How to apply:**
1. When writing a multi-step filter pipeline, write the steps in "narrowest first" order:
   remove inline/code content first, then block-level constructs (HTML comments), then
   line-level constructs (blockquotes). This prevents a block-level pattern inside inline
   content from prematurely opening/closing a deletion range.
2. When writing smoke tests, choose test inputs that cover lines that the filter has NOT
   accidentally hidden. A smoke test that passes because its target line was silently deleted
   is equivalent to no test at all.
3. For precheck-style filters that use sed range deletion: verify that an inline example of
   the range-start pattern (inside a code span) does NOT cause a real placeholder on the
   immediately following line to be skipped. This is the regression test added by AJ-008's
   resolution (C2 in CHG-20260630-003). See [[AJ-003]] for the related self-reference loop
   pattern and [[AJ-007]] for the shared-function scope-claim problem.

---

## AJ-009 — Review packet isolation breaks when multiple UPGs share uncommitted changes to the same status files

*Origin: UPG-0033 / CHG-20260701-001 (review-script-instrumentation), Step 1 R1 review, 2026-07-01.*

When two UPGs are simultaneously in-flight (here: UPG-0007 and UPG-0033), and both have
uncommitted changes to shared bookkeeping files (`status/self-development.md`,
`status/roadmap.md`), a packet review for one UPG captures the other UPG's diff noise.
The reviewer correctly flags this as scope drift (F3 in Step 1 R1).

**Why this matters:** The finding is correct at the signal level — the packet is not clean
— but the root cause is workspace state, not the change under review. If acted on literally
without understanding the cause, it would block valid changes every time two features are
in-flight simultaneously.

**Short-term fix:** Pass `--sha-only` for shared status files when reviewing a single-UPG
change. This excludes their content from the packet diff while still recording their hashes
for integrity.

**Deferred architectural question:** Whether the packet generator should be scoped to only
the file list declared in a change's "What changes" table, rather than the full
working-tree diff. This would make packet isolation a property of the declared scope rather
than a reviewer workaround. Logged as a future backlog candidate. See [[AJ-007]] for the
shared-function scope-claim problem.
