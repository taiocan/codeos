# Codeos Architecture Journal

Optional, cross-cutting institutional memory for the Codeos toolkit. Add an `AJ-NNN` entry only
when an important consequential decision or engineering insight will still matter months from now
and its governing artifact plus Git do not preserve it adequately. PROTECTED work does not require
an entry. Existing entries are historical; supersede rather than rewrite them.

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
doctrine + artifact structure landed here (Self-Reference Boundary in
`maintenance/archive/self-development/retired-process/codeos-self-dev.md`);
teaching the reviewer/packet to honor it is `UPG-0028`. Builds on [[AJ-001]] (singular contract)
and [[AJ-002]] (reproducible-from-committed-artifacts).

---

## AJ-004 — Adding a triage category requires sweeping every prose enumeration, not just the primary table

*Origin: UPG-0030 / CHG-20260629-001 (lean-review-profiles), Steps 3–4, multiple rounds.*

When a new triage category is introduced (here: `SELF-REFERENCE / REVIEW-BOOKKEEPING` as the fifth
category), it is easy to add it to the **primary triage table** and miss every **inline prose
enumeration** scattered across governance docs. UPG-0030 required fixes in four separate locations
across three review rounds before all enumerations were consistent:

- `maintenance/archive/self-development/retired-process/codeos-self-dev.md` — Step 4 checklist and Feature Thread triage list (both missed initially)
- `maintenance/archive/self-development/retired-process/codeos-change.md` — Reconciliation triage table (missed initially)
- `CLAUDE.md` — Step 4 loop summary (caught by R3 budget-exhaustion inline fix)
- `dba/06-reference/reviewer-pipeline.md` — §2 Scope Contract triage rule (caught by R3 budget-exhaustion inline fix)

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
2. Have its coverage semantics explicitly stated in `dba/06-reference/reviewer-pipeline.md` §5.
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

---

## AJ-011 — Reusing a shared function doesn't stop its callers from re-deriving divergent views

*Origin: UPG-0045 / CHG-20260712-002 (review-plan-preview), Step 3 review R1.*

The Step 1 design constraint for `codeos-reviewer plan` was explicit and correct: it must call
the exact same `packet::build()` function `review`/`--print-packet` use, so it can never
describe a packet `review` wouldn't actually build. That constraint was honored — `plan` really
does call the one shared `build()` function, verified by grep (`pub fn build` appears once).

It still shipped a real bug. `packet::build()`'s own oversized-packet warning ranks contributors
from an internal list (`file_contributors`) that deliberately excludes `sha-only` and delta-mode
entries, because those bytes are never counted toward the budget. `plan`'s first implementation,
reading only the *public, final* `ReviewPacket.artifacts` field (which includes every entry,
budget-relevant or not), re-derived its own "largest inputs" ranking from that broader set. The
shared function was reused correctly; a *downstream summary built from its output* silently
recomputed something adjacent-but-different, because the public struct exposed the final answer
but not the specific intermediate the original warning actually used.

**Lesson / how to apply:** "Call the same function" is a necessary but not sufficient guardrail
against parallel-logic drift. When a change adds a *second consumer* of a function that already
does some ranking/filtering/aggregation internally, check whether the second consumer needs the
exact same intermediate the first consumer's diagnostic uses — not just the same top-level
result. If so, expose that intermediate as its own field (as this change did with
`budget_contributors`) rather than letting the second consumer reconstruct a plausible-looking
approximation from whatever public fields already exist. This applies directly to the future
`UPG-0046`/`UPG-0047` (ReviewRun/structured-findings) work: any new surface that summarizes or
re-presents review data must be checked against what the *original* formatter actually consumed,
not just against the final struct's public shape.

**Related:** [[AJ-010]] on structural vs. cosmetic fixes — this is a specific, subtler instance:
the fix here *was* structural (a new shared field), but the bug it fixed looked, at first
glance, like it should already have been prevented by "just reuse the function."

---

## AJ-012 — `Path::exists()` cannot distinguish "absent" from "inaccessible"

*Origin: UPG-0046 / CHG-20260713-001 (reviewrun-structured-records), Step 4 review R1.*

`dba/04-tools/reviewer/engine/src/log.rs::compute_review_round` was written with the common pattern:

```rust
if !log_path.exists() { return Ok(1); }
let content = std::fs::read_to_string(log_path)?;
```

The stated contract (`AC-10`) was explicit: a missing log is fine (round 1, not an error), but a
log that exists and cannot be read must fail closed — never silently guess a round. The code
above does not do that. `Path::exists()` is implemented as `fs::metadata(path).is_ok()` — it
collapses **every** reason `metadata` can fail into the same `false`, not just "not found." A
permission error on a containing directory (unsearchable parent, common on locked-down
filesystems or misconfigured CI) also makes `exists()` return `false`, which this code silently
treated as "no log yet" and returned `Ok(1)` — precisely the guessed-round-on-error case the
acceptance criterion forbids.

The existing test at the time (`round_fails_closed_when_log_path_is_unreadable`) did not catch
this: it put a *directory* at the log path, which `exists()` correctly reports as `true` (a
directory is a valid `metadata()` result), so the code fell through to `read_to_string`, which
then failed for an unrelated reason (`IsADirectory`) and was correctly propagated. That test
passed for the wrong reason — it validated a different failure mode than the one the contract
actually needed to cover.

**Fix:** read directly, no `exists()` pre-check, and match the error kind:

```rust
match std::fs::read_to_string(log_path) {
    Ok(c) => c,
    Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(1),
    Err(e) => return Err(e).with_context(|| ...),
}
```

The new test that catches this reproduces the *specific* scenario — an unsearchable parent
directory (`chmod 000` on the containing dir, not the file) — not just "some error occurs."

**Lesson / how to apply:** Whenever a fail-closed contract distinguishes "the thing legitimately
doesn't exist" from "something went wrong trying to find out," `exists()`/`is_ok()`-style
pre-checks are the wrong tool — they discard the *reason* a check failed. Read/stat directly and
match `io::ErrorKind` (or the equivalent typed error) specifically. This applies to any future
Codeos Rust tooling that reads optional-but-sometimes-required files under a fail-closed
contract — `UPG-0047`'s `findings.yaml`, `UPG-0049`'s policy registry files, and any other
"missing is fine, broken is not" read path are the next places this exact bug shape can recur.
Also a testing lesson in its own right: a test that passes for a different reason than the one
the contract cares about (here: `IsADirectory` instead of `PermissionDenied`) gives false
confidence — reproduce the *specific* failure mode the contract names, not just any failure.

**Related:** [[AJ-011]] — both are cases where a seemingly-safe local pattern (reusing a shared
function; checking `exists()` before acting) didn't actually deliver the guarantee it looked
like it delivered, and both were caught only because Codex review checked the acceptance
criterion's exact wording against the code, not just its general shape.

---

## AJ-013 — A one-off grep is a hypothesis; only a permanent corpus test is evidence

*Origin: UPG-0047 / CHG-20260713-002 (structured-finding-lifecycle), Step 1 → Step 3.*

Before proposing the parsing approach for structured findings, Step 1 ran a manual, one-off
validation: `grep -c "^Finding:" reviews/codex/*.md` → 631 blocks across 302 files, and a
blank-line check that found "zero gaps." This read as strong evidence that the reviewer's
`Finding:`/`Evidence:`/`Why:`/`Required action:` output shape was mechanically reliable, and the
Step 1 Codex review accepted it (with a caveat — evidence grade C, "claims not shown directly").

It was wrong in two independent ways, both only discovered once Step 3 built an actual **unit
test** that parsed every real file and asserted an invariant, rather than eyeballing grep output:

1. **The count itself was inflated.** Every assessment file contains a duplicate transcript echo
   (CLI banner + prompt replay + a second copy of the real answer). A naive whole-file grep counts
   real findings twice. The real, deduplicated count was 317, not 631.
2. **The "zero gaps" conclusion was wrong outside the small sample actually eyeballed.** Codex
   does not reliably follow the prompt's "combine `Evidence`/`Why`/`Required action` onto one
   line" instruction. A single-shape parser failed on 112 of those 317 real findings — and this
   was not old, resolved history: the exact separate-line shape appeared in this same session's
   own `UPG-0045` review, generated hours earlier.

**Lesson / how to apply:** a manual grep or spot-check against "the corpus" is a *hypothesis*
about the corpus's shape, not evidence of it — especially for text a non-deterministic system
(an LLM) generated, where "the prompt says to do X" does not mean "X is what always happens."
The only thing that counts as evidence is a **permanent, re-runnable test** that actually parses
every real file and asserts the invariant that matters (here: every `Finding:` line is either
parsed or explicitly counted, never silently lost) — not a fixed count, which drifts as the
corpus grows, but a structural invariant plus a bounded-tolerance ceiling for known, understood
residual variance. Build that test *before* trusting a corpus claim enough to design against it,
not after a reviewer asks for "direct evidence." This applies to any future Codeos work that
claims to parse, migrate, or validate real historical or generated text — `UPG-0048`'s eventual
event-log migration and `UPG-0049`'s policy-file parsing are the next obvious candidates.

**Related:** [[AJ-012]] — both are cases where the actual first step (build a test that touches
real data) would have caught the gap immediately, but the natural first move (reason from the
spec, or check a plausible-looking pre-check) did not.

---

## AJ-014 — Step 1's mandatory Feature-Thread bookkeeping must be disclosed in the scope boundary up front

*Origin: UPG-0050 / CHG-20260716-001 (downstream-feature-id-scheme), Step 1 R1 (DO NOT ADVANCE).*

Every non-trivial self-dev change's Step 1 necessarily edits two files that are *not* the
change's actual subject matter: it adds a row to `backlog/features.md`'s Feature-ID Map and
activates a row in `status/self-development.md` — required by
`maintenance/archive/self-development/retired-process/codeos-self-dev.md`'s
"Feature Thread first" rule. If the Step 1 Change Intent's scope boundary states something like
"no change to any self-dev-only file" without carving out these two rows, the reviewer will
correctly flag it: the stated boundary is directly contradicted by the diff it's reviewed
against. This happened on UPG-0050's first Step 1 round and cost a full review cycle to fix.

**Rule:** when drafting a Step 1 scope boundary, explicitly name the two Feature-Thread
bookkeeping edits as expected self-registration (not scope drift) from the start — don't write
a blanket "no self-dev-only file changes" claim and then discover the contradiction via a
DO NOT ADVANCE round.

A second, smaller pattern from the same change: Step 1's Acceptance Criteria table is
*supposed* to be empty (Acceptance Criteria is Step 2's deliverable per the 4-step loop), but
the reviewer's `selfdev-step-1` packet has no stage-aware "expected output" template and can
flag the empty table as a blocker anyway. This is a structural non-issue, not a defect — reject
the finding with that reasoning rather than prematurely drafting Acceptance Criteria in Step 1
to appease it. In this change, R2 did not re-raise it once the actual scope-boundary blocker was
fixed, suggesting it may not recur consistently, but the reasoning for rejecting it if it does is
worth keeping.

---

## AJ-015 — A "current version governs new work" gate and a "protect already-approved past work" guarantee are different concerns and must not share one validity check

*Origin: UPG-0051 / CHG-20260719-001 (multi-feature-architecture-synthesis-gate), Step 4 Reconcile.*

UPG-0051 introduced a cohort-level `baseline_version` field (`features/registry.yaml`) plus a
superseded-version archive (`architecture/history/core-baseline-v<N>.md`). The first drafted
"valid reference" rule accepted *either* the current baseline's version *or* a historical
archived file as a valid `baseline_version` — reasoning that this was needed to support the
non-retroactive-invalidation guarantee (a feature already approved under an older version
shouldn't be forced to redo Stage 4 merely because a newer version now exists).

That reasoning conflated two genuinely different concerns:
- **Live eligibility** — may this feature enter or re-enter Stage 4 *right now*? This must
  always require the single, current, cohort-level approved version. A registry entry pointing
  at a historical version is stale, not "still valid."
- **Non-retroactive protection** — does a *later* version bump force re-verification of Stage 4
  work *already completed* under an earlier version? No, not unless an impact assessment finds
  an actual conflict — but this is a provenance/audit question about the past, resolved through
  Stage 9/10 if needed, never a live gate a new Stage 4 attempt can satisfy by pointing at an old
  file.

Letting historical files satisfy the live check meant a stale registry entry could silently
"authorize" new work under an outdated architecture, defeating the point of requiring approval
before proceeding.

**Rule:** when a versioned approval-gate schema has a single current-value field plus an archive
of superseded versions, keep exactly one validity condition for "can new work proceed" (current
version only) and treat the archive as read-only provenance for what governed past decisions —
never write a check that accepts both as if they were interchangeable "valid" states.

Related: none.

---

## AJ-016 — A reviewer packet claim is only as verifiable as the evidence actually embedded in it, not the evidence that exists elsewhere

*Origin: UPG-0051 / CHG-20260719-001 Step 1 R1, and UPG-0052 / CHG-20260719-002 Step 4 R1 — same
mistake, twice, in adjacent changes.*

Twice in one session, a change record stated a true claim — "`packet.rs` has no match arm for
this stage id" (UPG-0051), "every referenced prompt/template filename resolves"
(UPG-0052's AC16) — and the reviewer correctly returned DO NOT ADVANCE anyway, because the packet
sent for review didn't contain the evidence, only a prose summary asserting the conclusion. Both
times the underlying claim was already true; both times a review round was spent making it
*shown*, not making it *true*.

The reviewer's own rules state this rationale explicitly: it assesses only what the packet
contains, pinned to that commit, and treats an unverifiable strong claim as a candidate finding
whenever it affects acceptance or scope. A summary sentence — even an accurate one — is not
evidence to a reviewer that cannot see the command that produced it.

**Rule:** when an acceptance criterion is itself "X grep/read-through sweep shows Y," embed the
actual command and its full output in the artifact under review (in the Reconciliation section,
or as an attached file), not a prose summary of the result. This is cheap to do the first time
and costs a full review round to fix after the fact — twice, apparently, before the pattern was
internalized.

Related: none.

---

## AJ-017 — Completed-but-uncommitted self-dev work looks like scope drift on the *next* change's review

*Origin: UPG-0052 / CHG-20260719-002 Step 1 R1.*

`UPG-0051` reached COMPLETE and human-approved, but nothing had been committed yet this session.
When `UPG-0052`'s Step 1 review ran, the packet's diff (computed against the last real commit)
included all of `UPG-0051`'s already-approved changes to `dba-system.md`, `04-implement.md`, and
`dba/05-guidance/templates/feature-registry.yaml` — files `UPG-0052`'s own Step 1 explicitly promised not to
touch. The reviewer correctly flagged this as a scope-boundary contradiction; its own
`HIGHEST-IMPACT UNCERTAINTY` line named the real cause exactly: "if this is accidental carryover,
removing it likely resolves the blocker."

The fix wasn't editing `UPG-0052`'s content at all — it was committing `UPG-0051` first, using
this repo's existing one-commit-per-completed-change convention (`Codeos: UPG-#### CHG-... 
(COMPLETE)`, plus a separate `Backlog: Register UPG-####..` commit for registrations). Once the
working tree was clean except for `UPG-0052`'s own new files, the identical review re-ran clean
with a zero-byte diff.

**Rule:** commit a completed, human-approved self-dev change *before* starting the next change's
Step 1 review — not after, not "whenever convenient." An uncommitted-but-approved change is
invisible to git history but not to `git diff`, and every subsequent change's review packet will
silently inherit its full diff until it's committed, generating false scope-drift findings that
have nothing to do with the new change's actual content.

Related: none.

---

## AJ-018 — Fixing an imprecise rule restatement in one file doesn't fix it everywhere else the same rule is restated

*Origin: UPG-0054 / CHG-20260719-004 (contract-to-implementation-failure-boundary), Step 3 R1→R2.*

`UPG-0054` introduced a rule with two separate, independent approvals: a failure classification
must be named in the approved Contract, and, independently, the event it produces must be present
in the approved Event Schema. The first draft of both `dba/05-guidance/patterns/rust-project-structure.md`'s Rust
realization *and* `dba/03-prompts/workflow/04-implement.md`'s tightened Stage 4 rule blurred these into one
blended condition — reading as if the classification itself had to be "present in the approved
event schema," which is not what the schema authorizes (event types, not classification names).

Step 3 R1 caught this in the Rust pattern file and it was fixed there. R2 caught the *identical*
imprecision still sitting in `dba/03-prompts/workflow/04-implement.md` — the same wrong phrasing, written in a
different file, that the R1 fix never touched because the fix was scoped to the one file the
finding named.

**Rule:** when a reviewer (or anything else) catches an imprecise or incorrect restatement of a
rule, before considering the finding resolved, grep every other file this change touches (or that
already exists) for the same rule being restated in its own words, and fix all of them in the same
pass — not just the location the finding happened to point at. A rule repeated in multiple
prompts/patterns is only as correct as its least-precise restatement.

Related: none.

---

## AJ-019 — `--sha-only` proves a file's identity to the reviewer, not its content

*Origin: UPG-0055 / CHG-20260720-001 (reviewer-architecture-synthesis-stage-support), Step 3 R1→R2.*

An acceptance criterion required proving new reviewer-checklist text traced only to
`dba-system.md` and `dba/03-prompts/workflow/03b-architecture-synthesis.md` — no invented criterion. Step 3 R1
correctly flagged this as unverifiable because neither file was in the packet. To reduce packet
size on the next round, `--sha-only` was used for both files instead of full content. R2 caught
the same finding again, for a subtly different reason: `--sha-only` includes a path and a hash,
not the file's text, so the reviewer could confirm *which version* of each file was being
referenced but still could not read a single clause of either one to check the claim against.

**Rule:** `--sha-only` is for a large, genuinely unrelated context file the reviewer only needs to
confirm exists/is-pinned at a specific version — never for a file an acceptance criterion asks the
reviewer to cross-check content against. If a claim depends on what a file *says*, that file needs
full content in the packet, even at real cost to packet size (accepting the size-budget warning is
cheaper than a wasted review round).

Related: AJ-016 (evidence must be embedded, not summarized) — this is the same principle applied
to a specific tool flag that looks like it solves the size problem while silently reintroducing
the exact evidence gap AJ-016 already named.

## AJ-020 — The dashboard row must be updated *before* the step review runs, not just before the human sees it

*Origin: UPG-0056 / CHG-20260726-001 (governed-mechanism-activation-convention), Step 3 R2 and
Step 4 R1.*

Both times, the review packet included `status/self-development.md` alongside the change record,
and both times the reviewer caught the same contradiction: the change record's trace header said
`current_step: N`, but the dashboard row still said "awaiting human approval to proceed to Step N"
— because the row had only been updated for the *previous* step's outcome, not yet for the fact
that the current step's implementation/reconciliation had already been written and was now under
review. This is a real internal inconsistency, not review noise: a packet claiming to be at Step N
while its own tracked dashboard says Step N hasn't started yet is genuinely contradictory evidence.

**Rule:** update the dashboard row to reflect the *current* step's actual state (implementation
done, review in progress) at the same time the step's own artifact is written — before invoking
the reviewer for that step — not only after the human approves and the loop advances. Treat the
dashboard row as part of the artifact set under review, not a bookkeeping afterthought.

## AJ-021 — Reactive, round-by-round review can over-engineer a simple feature; only an outside proportionality check catches it

*Origin: UPG-0056 (governed-mechanism-activation-convention) / UPG-0057
(controlled-plain-english-writing-discipline), planning phase, rounds 1-7.*

A request for "a human-controlled on/off switch for a writing-style preference" grew, across seven
adversarial planning-review rounds, into a versioned governed-mechanism framework: a Rust resolver
crate, 25 stable result codes, two governance modes, per-artifact provenance stamps, historical-
version review coverage. Each round's fix was individually reasonable — every finding it raised was
real and worth fixing — yet no round ever asked whether the *cumulative* design remained
proportionate to the actual need. It took an explicit, out-of-band human intervention ("configuration
architecture is too complex and is making all new upgs almost impossible") to reset the design to a
one-line status file, after which the same review process converged cleanly in two more rounds.

**Rule:** adversarial round-by-round review is good at finding local defects but structurally blind
to cumulative scope growth, because each round only ever compares the artifact to itself, not to
the original need. A design that has been through several consecutive rounds — especially one that
keeps adding new categories of machinery (a new field, a new code, a new mode) rather than just
refining existing ones — is a signal to explicitly re-ask "is this still proportionate to what was
asked for," from outside the review loop, before continuing to iterate inside it.

## AJ-022 — A rigorous specification is a poor delegation target: Stage 4 delegability falls as contract rigor rises

*Origin: UPG-0060 (deepseek-delegated-implementation), CHG-B gate measurement, 2026-08-03 — a
realistic downstream feature (EvidenceAtlas EA-0003 corpus_construction) run through delegated
Stage 4 implementation and compared against the Claude-only path.*

The premise of delegating Stage 4 to a cheaper model was that implementation is bulk generation:
the approved artifacts already specify the behavior, so a weaker model should be able to satisfy
them and leave the strong model only the approval, reconciliation, and review that already guard
correctness. Measured on a real feature, that premise inverted.

The delegate produced a fluent, plausible, well-organized 466-line module that did not compile, and
that violated eight specific clauses of the approved contract and event schema once repaired enough
to run. The violations were not scattered — seven of the eight landed squarely on the parts of the
specification that exist *because* they are easy to get subtly wrong: the falsification scenarios
(a mirrored source must not inflate coverage), the vocabulary invariants ("Weak" and "weak" must
produce identical outcomes), the three-valued field whose schema text explicitly says which value
may never appear, the boundary scenario distinguishing one stopping reason from another. The
delegate satisfied all of these in *appearance* — the fields were present, the names were right,
the doc comments claimed the checks existed — and violated them in *fact*. Its own notes file even
flagged two of them unprompted, which is to say the draft arrived with an accurate warning that it
was wrong.

The common cause was structural rather than a list of eight mistakes. The candidate implemented the
report as a serializer: the caller computes coverage, decides retention, supplies the derived
fields, and the module writes them to JSON. Every invariant the contract exists to protect was
delegated onward to an unspecified caller. There is no eight-point patch for that — the invariants
have nowhere to live in that design — so the "rework" was a rewrite, and the token saving the whole
mechanism was built to capture never materialized: the delegated arm cost the Claude-only arm plus
the delegate's tokens plus the cost of reading and diagnosing the draft, and saved no generation at
all.

**Rule:** the value of delegating implementation is inversely proportional to how much genuine
specification the approved artifacts contain. DBA's Stage 2 and Stage 3 artifacts are, by design,
concentrated exactly where a fluent-but-shallow drafter fails — so the better a project applies the
methodology, the worse a cheap-model Stage 4 performs against it, and the *less* there is to save.
This does not generalize to every kind of delegation: read-only advisory review delegates cleanly
(a weak review costs nothing because the human still decides at the gate), and mechanical work whose
failures are loud rather than silent — Stage 5 test authoring is the candidate — may still pay off.
The distinction is whether the delegated output is the *primary artifact flowing through the gates*.
When it is, a weaker draft does not remove cost, it relocates it into reconciliation, and the only
honest way to know which happened is to measure one realistic feature before committing any doctrine
text to the mechanism.

Related: AJ-021 (round-by-round review is blind to cumulative disproportion) — the same corrective
shape, an outside empirical check that the review loop itself cannot produce. The two-change split
that made CHG-B contingent on measured evidence is what let this feature stop cleanly at a negative
result instead of shipping doctrine for a saving that was never there.

### Amendment (2026-08-03, same day) — the measurement that produced this entry was confounded

Re-reading the delegation harness after this entry was first written shows the packet handicapped the
delegate in ways the original attribution did not account for. This does not withdraw the rule above,
but a reader must not inherit it as a clean structural verdict.

`dba/03-prompts/delegation/codeos-implementer-task.md` **forbade the build manifest** the candidate needed — "Never emit
a path that is not a source or test file," plus "Add no … files … not traced to the approved
artifacts." The word `Cargo.toml` appears zero times in the 105KB packet. The missing manifest was
therefore a harness defect reported as a model defect. The packet also contained **no layout exemplar**
— the only `modules/` string in it comes from the prompt itself, so the candidate's module naming was a
guess with nothing to guess from. Most consequentially, the prompt's "**add no abstractions**"
minimalism instruction pushes a literal reader away from precisely the invariant-carrying structure
whose absence this entry identified as the root cause: the serializer design may have been partly
induced by the prompt rather than chosen by the model. Output was additionally constrained to
JSON-escaped source in a single shot with no compiler feedback, both known to degrade generated code.

What survives that correction, and is not attributable to the harness: the missing `derive(Hash)` on a
`HashMap` key; a doc comment asserting a validation the function does not perform; a knowingly-stubbed
timestamp shipped with a comment saying a real implementation would differ; the `#[cfg(test)]` module
shipped against an explicit "do not write tests in a Stage 4 candidate" instruction; and the
`scope_fully_examined` violation, whose governing schema sentence — "`null` never means 'examined but
the outcome is unknown'" — was present in the packet twice and still ignored. Seven of the eight
violations landed on invariant-dense contract text that *was* supplied, which is the observation the
rule above rests on and which the confound does not disturb.

A further limit worth naming: the same author wrote the comparator implementation, the violation
suite, and this entry. The individual violations are objective against quoted contract and schema
text, but the count and the framing are not independent of the grader.

**Consequence for the rule:** the structural claim — that verification cost does not compress the way
generation cost does, because the delegated draft is the primary artifact through the gates — is
independent of both the harness and the delegate, and stands. The stronger reading, that a cheap model
*cannot* satisfy a rigorous contract, is **not yet established**: it was measured through a harness
that suppressed manifests, withheld layout, discouraged the needed abstractions, and allowed no repair
iteration. Correcting the harness is therefore a **prerequisite** to any further model comparison, not
an alternative avenue — a re-test that changes the model without fixing the packet is uninterpretable.
See `maintenance/archive/self-development/backlog/completed/UPG-0060-deepseek-delegated-implementation.md`
for the ordered re-test conditions, both of which were discharged on 2026-08-21 with a negative result.

## AJ-023 — A suite tests what its author thought to test; the usage shape a change exists to enable is the one most likely to go untested

*Origin: UPG-0064 / CHG-20260804-002 (delegated Stage-4 envelope alignment), Step 4 R1.*

The change added caller-declared artifact roles so a delegated-implementation pilot could name each
artifact's authority explicitly. Its acceptance criteria recorded a binding precondition on the
dependent work: the pilot must declare **every** governed artifact with a role flag and use no
positional artifacts, so the experiment could not run through the degraded compatibility path.

Forty-five tests passed. The usage check still read `[[ $# -lt 3 ]]`, requiring a positional artifact,
so a role-flags-only call exited 3. **The tool could not be driven the way the dependent work was
required to drive it** — the precondition was unsatisfiable by the implementation written to enable
it. No test caught this, because every test invoked the tool with at least one positional, which was
the pre-existing shape.

The suite was not weak in general: it covered path safety, protocol robustness, secret non-leakage,
idempotency, allowlist drift, and mutation-verified two guards. It simply tested the shapes its author
already had in mind, and the new shape — the one the whole change existed to make possible — was the
one absent from that set.

**Rule:** when a change exists to enable a specific downstream usage, write a test that invokes it in
exactly that shape, before believing the change works. Coverage of what the code *does* is not
coverage of what the change is *for*. The strongest form is a dry run in the dependent work's precise
invocation, executed as part of the enabling change rather than deferred to the first real use.

Related: AJ-016 (evidence must be embedded, not summarised) and the same change's Step 3 incident, in
which filtered test output hid a syntax error and produced a false green. Both are the same underlying
failure — verification aimed at the wrong surface — and both were invisible from inside a passing run.

---

## AJ-024 — A citation that survived adversarial review can still be wrong; verify the pointer, not just the claim

*Origin: UPG-0065 / CHG-20260808-001 (v1 component decomposition), Step 3 self-check.*

`CHG-20260807-001`'s delta table pinned a `source_anchor` line range to every one of 203 rows,
across nine adversarial review rounds, specifically to make content relocation deterministic and
checkable. Step 1 of this follow-on change had already distrusted the table's `current_rule`
*summary* field, after review found it sometimes paraphrased away real content (e.g. "the exact
invocation syntax and worked examples" instead of the syntax and examples themselves) — the fix
was to always transcribe from the pinned line range in the actual source file, never from the
summary.

That fix did not go far enough. While transcribing `ARCH-GATE-6` — a row whose own summary
correctly said "a detailed 6-step sequence" — the stated range `L210-233` turned out to cover only
steps 1-3. Steps 4-6 (`L234-252`, the actual Architecture Synthesis drafting/approval steps, the
single most consequential part of the gate) sat entirely outside the cited range. Six more rows had
the same defect at smaller scale — a sentence or list item continuing one to several lines past
where the citation said it stopped. All were found only because the next change happened to
transcribe against the live file instead of trusting the range.

**Rule:** a line-range citation is itself a claim, not a fact, no matter how many review rounds
produced it — pinning a citation to a commit does not verify that the citation is *complete*, only
that it is *reproducible*. When a citation exists to license copying content later, the consuming
step must re-derive the boundary from the source at the moment of use (read to the actual end of
the sentence/clause/list, not the stated end line), not merely trust the earlier artifact's line
numbers. The failure is invisible from inside the citing document, because the document's own
`current_rule` summary can independently look correct even when the range beneath it is short —
exactly what happened here, and the reason nine rounds of review over the citing artifact alone
never caught it.

Related: AJ-016 and AJ-023, both instances of the same underlying pattern — verification that
checks the artifact in front of it, rather than the thing that artifact points to, can pass cleanly
while the pointed-to fact is wrong.

---

## AJ-025 — Knowing the staleness rule doesn't prevent violating it; the fix has to be mechanical, applied every round

*Origin: UPG-0065 / CHG-20260808-001 and CHG-20260808-002 (v1 decomposition, compatibility
sweep), recurring across roughly six separate review rounds.*

AJ-020 already states the rule: update the dashboard row (and, by the same logic, the brief's
status line and the change record's own trace header) to reflect the current step's actual state
*before* invoking the reviewer for that step. Across two changes in the same session, the same
class of staleness was still caught by Codex repeatedly — not because the rule was unknown, but
because it was applied only at step *boundaries* (write Step N, sync tracking, run review) and not
at *round* boundaries within a step, and not immediately after a human approval either. Every
review round that fixes a finding, and every human "Approved," is itself a state change the
dashboard/brief/trace-header must reflect before the *next* thing happens — not just the first
review of a step. Awareness of a documented rule is not the same as a habit that fires every time.

A second, related pattern surfaced in the same rounds: an acceptance criterion that claims
agreement across several tracking files is not actually verified by a reviewer packet unless
those files' *content* is shown — `--sha-only` proves a file exists at a given hash, not what it
says. Both `CHG-20260808-001` and `CHG-20260808-002` hit the identical Step 4 R1 finding
("`backlog/features.md`/`status/roadmap.md` not shown") for this exact reason, back to back.

**Rule:** (1) re-sync every tracking surface named in a cross-reference-consistency criterion
immediately before *every* review invocation, not once per step — treat "did I just fix something,
or did the human just approve something" as the trigger, not "did I just start a new step." (2)
Any acceptance criterion that claims agreement across N named files must include *all* of them as
full content (not `--sha-only`) in the packet that is meant to prove it — `--sha-only` is for
unchanged large context, never for a file whose content is itself the evidence.
