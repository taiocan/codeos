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

---

## AJ-010 — Advisory round budgets are not hard gates; the human decides when to advance

*Origin: UPG-0032 / CHG-20260702-001 (Rust reviewer engine), Step 3 review rounds R1–R6.*

PROFILE-3 specifies a 3-round budget per step. Step 3 ran 6 rounds: each returned CHANGES
ADVISED with genuine in-scope blockers (exit-code routing errors, false AC claims about
SECRET_REDACTION and config location, missing fail-closed handling). All findings were
applied. The human approved Step 3 after R6 with the explicit rationale that all blockers
were fixed, 31/31 tests passed, and the round budget is advisory not binding.

**Lesson / how to apply:** A reviewer verdict is advisory. A round budget is a cost-management
guide, not a gatekeeping rule. The "max 3 rounds" limit means: "signal to the human that
the review loop may be chasing diminishing returns." When each round finds new legitimate
blockers (not the same finding restated), the loop is doing its job — the human should
decide whether the cost of another round is worth it, not whether the budget rule triggers.
The protocol enforces human approval at every step gate; it never auto-blocks.

In practice: R1–R6 each found real implementation defects that were worth fixing (incorrect
exit-code mapping, false contract claims in the AC text, unsafe `.ok()` discards). The budget
was exceeded because the implementation had real gaps — not because the reviewer was
misfiring. Once the human confirmed all blockers were closed, they advanced. That is the
correct behavior.

**Corollary:** When the reviewer begins repeating claims or finding style points, that is when
to exercise the advisory override and advance. When it finds new structural correctness issues,
fix them.

---

## AJ-011 — CRITICAL_OMISSION / EMPTY_PACKET are software-enforced stops, not human-non-overridable guarantees

**Origin:** UPG-0015 / CHG-20260702-002 Step 1, F1 (2026-07-02).

These two coverage states mean the reviewer was given incomplete or empty evidence. The
instinct to call them "non-overridable hard stops" is architecturally wrong for Codeos,
because DBA's Rule 1 is: explicit human correction at any gate overrides everything else.
Making a software state non-overridable by the human contradicts the governance model.

**The correct framing:**

> CRITICAL_OMISSION and EMPTY_PACKET are mandatory stop conditions for **automated
> progression**. Advancement requires an explicit human override with recorded rationale.
> The override does not invalidate the finding; it records that the human intentionally
> accepted the associated risk.

This preserves both sides: automated tooling cannot silently proceed past incomplete
evidence (software-enforced stop), and the human remains the final authority
(human-overridable with mandatory rationale and audit trail).

**Distinction to apply when writing gates:** "automated hard stop" ≠ "human-non-overridable."
The former is a code behavior; the latter is a governance claim. Only the former is ever
appropriate in a system that keeps "human approval is required at every gate."

## AJ-012 — Supplementary evidence sections must declare their coverage semantics explicitly

*Origin: UPG-0014 / CHG-20260702-004 (reviewer-full-diff), Step 4 R1–R3.*

When a new section is added to the review packet as **supplementary/informational context** (not
primary evidence), its redaction and error behavior must be explicitly declared as outside the
existing `coverage_state` semantics — or reviewers (human and automated) will interpret the
system's stated coverage floor as applying to it.

The specific failure mode: `coverage_state` is computed from named-artifact evidence before the
supplementary full-diff section is appended. If the full diff has secrets redacted or a git
error, `coverage_state` still says `FULL_COVERAGE`. This is correct — the full diff is
supplementary context, not a named artifact — but without explicit documentation, this looks
like a safety gap.

**Rule:** Any packet section that is supplementary (informational, additive, not part of the
named-artifact evidence set) must:
1. Be labeled "informational" in its section header.
2. Have its coverage semantics explicitly stated in `docs/reviewer-pipeline.md` §5.
3. State what happens on error (silent? explicit marker? coverage escalation?).

Failing to state this will cost 2–3 review rounds as the reviewer correctly flags the apparent
gap, and the resolution ("it's intentional, supplementary-only") must be documented anyway.

## AJ-013 — Change records must not narrate future gate outcomes before the gate happens

*Origin: UPG-0021 / CHG-20260703-003, Step 4 R1 (DO NOT ADVANCE).*

A Step 4 Reconcile section was drafted with an "Outcome" claiming the change record,
dashboard, roadmap, and feature registry were "updated to COMPLETE together" — written
*before* the Step 4 Codex review had even run, let alone before the human approved that
gate. Frontmatter still said `IN_PROGRESS` while the prose claimed `COMPLETE`. The reviewer
correctly flagged this as an in-scope blocker: it creates an internally contradictory
artifact (prose vs. frontmatter) and a false human-approval claim, not just a wording slip.

A second, smaller instance of the same failure mode in the same change: the dashboard row's
"human APPROVE_STAGE" note (recorded for the Step 3 gate) read ambiguously once the row had
moved to `4-Reconcile`, as if it were approval for the step the row currently named.

**Rule:** Change-record prose must be strictly backward-looking. Write only what has already
happened and been verified — acceptance-criteria checks against the code as it exists right
now, review verdicts that have actually returned, decisions that have actually been logged.
Never write "will be marked," "updated to X together," or similar forward claims, and never
let a status note about one step's approval read as if it applies to a later step's row.
The actual state-file edits (frontmatter `status`/`state`, dashboard row, roadmap, feature
registry) happen only *after* the human decision is logged, as a separate, later pass — not
narrated in advance inside the same artifact the reviewer is being asked to assess.

**Why it matters beyond this one change:** the reviewer's job is to check claims against
evidence in the packet. A claim about the artifact's own state is trivially falsifiable by
the reviewer (it can just read the frontmatter), so this class of error reliably costs a full
extra review round — cheap to avoid, easy to keep tripping over under time pressure to "just
finish the change record" while the gate is still open.

---

## AJ-010 — Surface-level fixes for structural issues burn review rounds

*Origin: UPG-0043 / CHG-20260711-002 (smoke-test-modularity), Step 4 review R1→R2→R3.*

When a reviewer identifies a **structural issue** — file scope violation, insufficient verification, architecture contradiction — the fix must be **structural**, not cosmetic. Updating doc comments, revising evidence wording, or adjusting narrative does not resolve the underlying problem and will be correctly re-blocked in subsequent rounds.

**Concrete case:** R1 found generate_dashboard.rs violated AC-5 "clear tool-area responsibility" by containing tests for 3 separate commands (generate-report, generate-adr-candidates, generate-approval-dashboard). Initial fix: updated the file's doc comment to say "all generate-approval-dashboard output modes." R2: correctly re-blocked — the commands are separate subcommands, not output modes; file scope still violated. Structural fix: split the file into 3 command-specific files. This resolved F1.

Similarly, R1 found AC-7 evidence contradicted implementation notes. Initial fix: revised evidence wording. R2: correctly re-blocked — wording revision doesn't provide systematic verification. R3: still blocked on documentation contradiction. Structural fix: updated implementation notes to accurately describe the method used (sed line-range extraction), not the method attempted (compilation-driven).

**Lesson / how to apply:** In Step 4 reconcile, if the reviewer identifies:
- **File scope violation** → split or reorganize the file, don't redefine its purpose in a comment
- **Insufficient verification** → provide systematic evidence (full diff, exhaustive check), don't spot-check and claim completeness
- **Documentation contradiction** → update the documentation to match reality, don't paper over it with revised wording

Surface-level fixes are **procrastination** — they defer the real work and burn review rounds. When a reviewer says "this file contains X when it should contain Y," the answer is to change what the file contains, not to change how you describe it. PROFILE-3 max-rounds budget exists to prevent endless cycling; use it by fixing structurally the first time.

**Related:** [[AJ-002]] on reproducible acceptance criteria, [[AJ-001]] on reconciling originating briefs.
