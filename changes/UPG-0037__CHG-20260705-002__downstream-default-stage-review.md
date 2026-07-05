---
change_id: CHG-20260705-002
feature_id: UPG-0037
slug: downstream-default-stage-review
triage_class: downstream-doctrine
scope_axis: downstream doctrine only
review_profile: PROFILE-4
review_series: RVS__UPG-0037__CHG-20260705-002__S1
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0037 / CHG-20260705-002 — Default Advisory Review Across the Full Downstream DBA Workflow

## TRACE HEADER

```yaml
feature_id: UPG-0037
primary_feature_id: UPG-0037
change_id: CHG-20260705-002
slug: downstream-default-stage-review
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0037
related_features:
  - UPG-0007
  - UPG-0036
review_series: RVS__UPG-0037__CHG-20260705-002__S1
review_profile: PROFILE-4
review_state: ACCEPTED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

---

## Step 1 — Change Intent

### Problem

Codeos's own self-development (`CLAUDE.md`) makes advisory Codex review mandatory at every
step of the 4-step loop: "Run the Codex reviewer at every non-trivial step... Running the
review is mandatory. The verdict is advisory." The downstream doctrine (`dba-system.md`,
loaded by generated projects via a `.codeos` symlink) has no equivalent default. Its 9-Step
DBA Development Loop states a human-approval gate at every stage but never calls for an
advisory review pass before that gate. The only reviewer integration mentioned anywhere
downstream is `prompts/pipeline-reviewer.md` ("Reviewer Activation Package") — an optional,
manually-pasted prompt covering only Stages 1-9, structurally different (unstructured
Attention-Level/Key-Findings output, no packet, no review log, no acceptance-criteria
framing) from the Rust `codeos-reviewer` pipeline self-dev now relies on by default.

This is the same rigor asymmetry UPG-0036 fixed at the toolkit layer (stack manifest /
`check-drift` dogfooding), now surfacing at the doctrine layer: Codeos requires more of its
own development than it defaults downstream projects into.

**The gap is not capability.** `tools/reviewer/src/packet.rs` already hardcodes complete
`stage_checks()` and `stage_expected()` tables for downstream stages `"1"` through `"9"`
(`packet.rs:605,620`), and its "DBA RULES RELEVANT TO THIS STAGE" packet section is already
generic DBA language, not self-dev-specific (`packet.rs:336-341`: "Human approval is
required for every stage transition... Implementation must trace to approved artifacts...").
The mechanism has simply never been made the *default* in `dba-system.md`'s stated workflow
— and it doesn't yet cover the steps before Stage 1 at all: Solution Discovery
(`00b-solution-discovery.md`, UPG-0007) isn't even referenced in `dba-system.md`'s stage
table today (deliberately deferred by UPG-0007's self-dev-only scope), and Feature Brief /
Onboarding have no reviewer touchpoint either.

**This is not hypothetical.** `/home/rimo/projects/FundFlow` is a real, live downstream
project: `.codeos` is symlinked to this repo, and its `reviewer.toml` is already configured
(`provider = "codex"`, `reasoning_effort = "high"`) — though `reviews/review-log.md` doesn't
exist yet, confirming the reviewer has never actually been invoked there. Because `.codeos`
is a live symlink rather than a versioned copy, any change to `dba-system.md` here is
immediately live for FundFlow on its very next session — there is no sync/migration step to
design around.

### What changes

| File | Change |
|---|---|
| `dba-system.md` | Unified stage-identifier sequence (`discovery`, `brief`, `onboarding`, `1`-`10`) for documentation order and the `codeos-reviewer review <feature_id> <stage>` argument; default advisory-review line at every reviewable gate; Review Waiver practice for when reviewer tooling isn't configured |
| `tools/reviewer/src/packet.rs` | New `stage_checks()`/`stage_expected()` match arms for `"discovery"`, `"brief"`, `"onboarding"`, `"10"` (numeric `"1"`-`"9"` already exist) |
| `prompts/pipeline-reviewer.md` | Reframed opening line: optional supplementary independent-assessor pass, not a replacement for the default review |
| `docs/reviewer-pipeline.md` | New downstream-usage section (today written entirely in self-dev terms — `selfdev-step-N` arguments, `UPG-####` ids assumed) |
| `prompts/00b-solution-discovery.md` | Small addition: output gets reviewed when carried into a Feature Brief or Stage 1 Intent; the session itself stays optional/non-gating |
| `backlog/UPG-0037-downstream-default-stage-review.md` | Feature Thread: CHG-20260705-002 activated (done); a Follow-up Features row added at Step 3 for `UPG-0038` (see below) |
| `backlog/features.md` | Row → IN_PROGRESS (done); a `UPG-0038` row added at Step 3, since discovering and filing a genuinely out-of-scope follow-up during implementation is standard practice (precedent: UPG-0032 filed UPG-0034/UPG-0035 the same way) |
| `status/self-development.md` | Row activated (done) |
| `status/roadmap.md` | UPG-0037 → IN_PROGRESS (done); a `UPG-0038` row added at Step 3, same reason as above |
| `backlog/UPG-0038-review-shim-symlink-resolution.md` | New: follow-up backlog brief for the `codeos-review.sh` shim bug discovered during this change's Step 3 verification (AC-10) — not implemented here, filed as its own future change |

### Scope boundary — what stays the same

- **`CLAUDE.md` — not touched, at all.** This is `downstream doctrine only`; no self-dev
  governance changes. Verified as an acceptance criterion (`git diff --stat -- CLAUDE.md`
  stays empty), not just asserted.
- **No new downstream triage/size-classification system.** `dba-system.md` has zero existing
  triage concept (confirmed: no `trivial|triage|class` matches anywhere in it) — review is
  default at every reviewable gate, uniformly, matching the existing "same loop regardless of
  size" philosophy. Inventing a triage taxonomy purely to gate review-mandatoriness would be
  disproportionate scope creep.
- **No porting of self-dev's `PROFILE-N` system into the doctrine.** `PROFILE-N` is
  self-dev-internal tooling (`prompts/codeos-self-dev.md`) and never appears in
  `dba-system.md` or any downstream-facing prompt. Downstream gets a flat, uniform round
  budget described in `dba-system.md`'s own plain language (see Design intent). Note the two
  distinct cadences at play, not to be conflated: *this change itself* (UPG-0037, as a
  self-dev change) is triaged `downstream-doctrine` and reviewed under `PROFILE-4` per
  `prompts/codeos-self-dev.md` — that governs how *this change's own* Step 1-4 gates get
  reviewed during development. What this change *writes into* `dba-system.md` for downstream
  projects is a separate, flat cadence with no `PROFILE-N` vocabulary at all.
- **No new config file or CLI flag for the Review Waiver** in this version. It's a doctrine-
  level practice (a plain review-log/decision-record entry), not new tooling — `reviewer.toml`
  gains no new field.
- **No artifact path, filename, schema, or stage-output changes anywhere.** Verified directly
  against FundFlow's existing `reviewer.toml`, `features/registry.yaml`, and `reviews/`
  layout remaining valid and untouched.
- **No prompt file renames.** `01-intent.md` through `09-refine.md`, `10-arch-refine.md`,
  `00b-feature-brief.md`, `00c-onboarding.md`, `00b-solution-discovery.md` keep their current
  filenames — the unified sequence in Design intent is stage-identifier vocabulary for
  documentation and reviewer-invocation purposes only, not a file-rename.
- **Non-Negotiable Rule #1 is unchanged and unweakened**: every stage transition still
  requires explicit human approval. The Review Waiver (see below) waives only the advisory
  review run — never that approval gate.

### Design intent

**Unified stage-identifier sequence** (vocabulary and documentation order only):
```
discovery -> brief -> onboarding -> 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8 -> 9 -> 10
```
This is explicitly *not* a claim that `onboarding` (Session Type D) is a mandatory linear
step every feature passes through. Onboarding remains structurally what it is today: an
alternate entry point for bootstrapping an existing codebase that lacks DBA artifacts, used
*instead of* `discovery`/`brief` for that scenario, not sequentially after them.
`dba-system.md`'s prose states this explicitly wherever the unified sequence appears, so the
linear-looking list doesn't imply a linear-only reading.

**Default review, uniform cadence** (mirrors self-dev's proven mechanism, not its naming):
```
Run the Codex reviewer before each gate's human-approval decision:
  codeos-reviewer review <feature_id> <stage>
Round 1 runs before the gate. Rounds 2-3 are allowed for fixes or material deltas. After 3
rounds, stop and require a human decision. The verdict is advisory — NO OBJECTION / CHANGES
ADVISED / DO NOT ADVANCE inform the human's decision but never auto-block. The reviewer is
independent, read-only, and non-gatekeeping; the human decides at the gate.
```

**Solution Discovery — conditional trigger, not unconditional.** The session itself stays
optional and non-authoritative (UPG-0007's guardrail, unchanged) — running it is never
required, and its output is never an approved architecture. *If* its artifact is carried
into a Feature Brief or Stage 1 Intent, *that handoff* gets the default advisory review pass
(or a Review Waiver). A Discovery session whose output nobody acts on is simply never
reviewed — there's nothing to review yet.

**`prompts/pipeline-reviewer.md`, reframed not replaced.** It stays available as an optional,
independent critical-assessor pass for a second opinion — genuinely different in kind from
the Rust engine's acceptance-criteria-bound, scope-triaged assessment (its own framing is
explicitly "free to conclude DBA itself is inappropriate," a broader charter than the
structured review). Its opening line gets a one-sentence addition stating it's supplementary
to, not a replacement for, the default review log/packet path.

**Review Waiver — precisely scoped.** If reviewer tooling is unavailable or not configured
for a project, the human records an explicit waiver with a reason and may continue — neither
silently skipping the review nor hard-blocking the whole project over missing reviewer
setup. The waiver applies *only* to the advisory review run; it never waives Non-Negotiable
Rule #1's human-approval gate — a waived review still requires the human to explicitly
approve the stage transition, exactly as today. Concretely: a plain entry in that feature's
review log / decision record ("Review waiver: reviewer not configured for this project;
proceeding without advisory review at Stage N. Reason: ..."). `UPG-0015`'s
`--override <RATIONALE>` flag is the closest existing precedent in spirit (human-recorded
rationale overriding a review-adjacent gate) but doesn't fit mechanically — it fires on a
packet that *was* built and found deficient; here there's no packet at all. A future CLI
convenience (e.g. `codeos-reviewer decision ... --waive-review "<reason>"`) is an explicit
possible follow-up, out of scope for this change.

### Triage

- Class: `downstream-doctrine`
- Scope axis: `downstream doctrine only`
- Review profile: `PROFILE-4`
- Originating backlog id: `UPG-0037`

---

## Step 2 — Acceptance Criteria

### Doctrine content

**AC-1 — Unified stage-identifier sequence present, correctly framed**
`dba-system.md` presents the sequence `discovery -> brief -> onboarding -> 1 -> 2 -> 3 -> 4
-> 5 -> 6 -> 7 -> 8 -> 9 -> 10` and explicitly states, adjacent to `onboarding`, that it is an
alternate entry point for bootstrapping an existing codebase lacking DBA artifacts — used
instead of `discovery`/`brief` for that scenario, not sequentially after them. The sequence
is not presented as a single mandatory linear path for every feature.
_Verify in Step 4:_ read the relevant `dba-system.md` section; confirm both the sequence and
the onboarding clarification sentence are present.

**AC-2 — Default review instruction covers every reviewable gate**
`dba-system.md` states the default-review practice once, generically (matching self-dev's
own `CLAUDE.md` precedent, which states its mandatory-review rule once rather than repeating
it per step) — `codeos-reviewer review <feature_id> <stage>` — and the Stage ID table names
the exact identifier for every reviewable gate: Feature Brief (`brief`), Onboarding
(`onboarding`), Stages 1-9 (`1`-`9`), Stage 10 / Architectural Refinement (`10`). The generic
instruction's own text states it applies "at every reviewable gate" (or equivalent), so a
reader cannot mistake it for a Stage-1-only or partial practice. Solution Discovery
(`discovery`) has the separate conditional instruction from AC-3, not this unconditional one.
_Verify in Step 4:_ read the "Default Advisory Review" section and the Stage ID table;
confirm the generic instruction states it applies to every gate, and every non-Discovery
gate above has a Stage ID entry in the table.

**AC-3 — Solution Discovery conditional trigger, correctly worded**
`dba-system.md` (and/or `prompts/00b-solution-discovery.md`) states that the Discovery
session itself remains optional and non-gating, and that review is triggered only when its
output is carried into a Feature Brief or Stage 1 Intent — not unconditionally after every
Discovery session.
_Verify in Step 4:_ read the relevant text in both files; confirm the conditional wording
("if... is carried into...") is present, not an unconditional "every Discovery session is
reviewed."

**AC-4 — Round budget, no `PROFILE-N` leakage into downstream-facing content**
`dba-system.md` states the round budget in its own plain language: round 1 before the gate,
rounds 2-3 for fixes/material deltas, stop after 3 rounds and require a human decision.
`grep -i "PROFILE-" dba-system.md prompts/pipeline-reviewer.md prompts/00b-solution-
discovery.md` returns zero matches — `PROFILE-N` is self-dev-internal vocabulary and must not
appear in anything a downstream project reads. `docs/reviewer-pipeline.md` is a Codeos
contributor-facing doc (not something `dba-init.sh` installs into a downstream project's
`.codeos/`), so its pre-existing §4d self-dev review-round-budget table legitimately
continues to name `PROFILE-0` through `PROFILE-5` — that is explicitly excluded from this
AC's scope. What *is* in scope: the new §12 downstream-usage section added by this change
must itself contain zero `PROFILE-` matches, distinguishing the two cadences without naming
the self-dev one.
_Verify in Step 4:_ run the 3-file grep above (must be empty); separately run
`sed -n '/## 12\. Downstream usage/,/^---$/p' docs/reviewer-pipeline.md | grep -i "PROFILE-"`
(must also be empty) to confirm the new section specifically, not the whole file, is clean.

**AC-5 — Review Waiver, scoped correctly**
`dba-system.md` states the Review Waiver practice: if reviewer tooling is unavailable/not
configured, the human records an explicit waiver with a reason and may continue. The text
explicitly states the waiver applies only to the advisory review run and never waives
Non-Negotiable Rule #1's human-approval requirement — a waived review still requires explicit
human approval to proceed.
_Verify in Step 4:_ read the waiver section; confirm both the "record a reason, don't skip
silently, don't hard-block" framing and the explicit non-waiver of human approval are present.

**AC-6 — `pipeline-reviewer.md` reframed, not contradicted**
`prompts/pipeline-reviewer.md`'s opening states it is an optional, supplementary independent
critical-assessor pass, explicitly not a replacement for the default review log/packet path.
Its existing "independent critical assessor, not a DBA compliance auditor" framing (its
current core content) is unchanged.
_Verify in Step 4:_ diff the file; confirm only the opening framing changed, and the rest of
the file (Stage Summary table, Your Role, Output Format sections) is untouched.

### Downstream compatibility (required for `downstream-doctrine` class)

**AC-7 — No artifact path, filename, or schema changes**
No file under `intents/`, `contracts/`, `events/`, `modules/`, `tests/`, `features/`,
`backlog/`, `refinements/` (the downstream project directory structure per `dba-init.sh`) is
renamed, restructured, or schema-changed by this proposal. No prompt file under `prompts/` is
renamed.
_Verify in Step 4:_ `git diff --stat` for this change touches only the doctrine/tooling/docs
files explicitly named in the "What changes" table above (`dba-system.md`, `packet.rs`,
`pipeline-reviewer.md`, `docs/reviewer-pipeline.md`, `00b-solution-discovery.md`) plus the
listed backlog/status bookkeeping rows — no downstream-project-shaped paths
(`intents/`, `contracts/`, `events/`, etc.) appear anywhere in the diff.

**AC-8 — FundFlow (real downstream adopter) remains valid and untouched**
`/home/rimo/projects/FundFlow`'s `reviewer.toml`, `features/registry.yaml`, and `reviews/`
directory contents are unchanged and remain structurally valid after this change (FundFlow is
not part of this repo and is never edited by it — this AC verifies non-interference, not
migration).
_Verify in Step 4:_ `git -C /home/rimo/projects/FundFlow status --short` shows the identical
output before and after this change (FundFlow has pre-existing untracked files of its own —
the bar is "unchanged by this change," not "empty"); spot-check that `.codeos` still resolves
(`readlink -f /home/rimo/projects/FundFlow/.codeos`)
to this repo post-change.

**AC-9 — `CLAUDE.md` and self-dev governance untouched**
`git diff --stat -- CLAUDE.md` is empty for this change's commits. No file under
`prompts/codeos-self-dev.md`, `templates/codeos-change.md` is modified.
_Verify in Step 4:_ run the diff-stat check; confirm empty.

**AC-10 — No new downstream-side setup burden, no new config field**
Because `.codeos` is a symlink to this repo (not a copy), a downstream project never builds
or maintains its own separate `codeos-reviewer` binary — it always uses whatever binary is
built here, in Codeos. This change's own Step 3/4 rebuilds that shared binary to include the
`packet.rs` match arms (AC-11/AC-12) as part of landing this change — that is ordinary
in-repo implementation work, not an extra step downstream projects must separately perform.
A downstream project that already has `reviewer.toml` configured (UPG-0003/0032/0014's
existing prerequisite) needs **no new configuration, no new config field, and no action of
its own** to start invoking the new stage identifiers once this change lands — it inherits
the rebuilt binary automatically through the existing symlink.
_Verify in Step 4:_ from `/home/rimo/projects/FundFlow`, after this change's binary is
rebuilt in Codeos, invoke the reviewer binary directly (`/home/rimo/projects/Codeos/tools/
reviewer/target/release/codeos-reviewer review <feature> discovery --print-packet ...`)
against an existing FundFlow artifact; confirm it builds a packet without error and without
any edit to FundFlow's own files. **Note**: `.codeos/scripts/codeos-review.sh` itself has a
pre-existing bug unrelated to this change — it resolves `REPO_ROOT` via `git rev-parse
--show-toplevel` from the *calling* repo, so from within FundFlow it looks for the binary
under FundFlow's own (nonexistent) `tools/reviewer/`, not through the `.codeos` symlink to
Codeos. This is filed as an out-of-scope follow-up (see Step 4), not fixed here — direct
binary invocation is the correct verification path for this AC until that shim is fixed.

### Reviewer-engine support

**AC-11 — New stage identifiers produce real checklists, not the generic placeholder**
`tools/reviewer/src/packet.rs`'s `stage_checks()` and `stage_expected()` each gain match arms
for `"discovery"`, `"brief"`, `"onboarding"`, and `"10"`, returning stage-appropriate text
(not the `_ =>` generic placeholder). Numeric `"1"`-`"9"` are unchanged.
_Verify in Step 4:_ unit test (or packet dry-run) for each of the four new identifiers;
confirm the STAGE-SPECIFIC CHECKS / EXPECTED STAGE OUTPUT sections show real text, not
"(no stage-specific checklist for stage X)".

**AC-12 — Build and existing tests remain green**
`cargo build` and `cargo test --test smoke` both pass after the `packet.rs` change, with no
regressions to existing stage `"1"`-`"9"` behavior or any other subcommand.
_Verify in Step 4:_ run both; record pass counts.

### Cross-reference integrity

**AC-13 — No stale stage references left behind**
`grep -rn` across `dba-system.md`, `prompts/pipeline-reviewer.md`, `docs/reviewer-pipeline.md`,
`prompts/00-session-start.md`, `prompts/00b-solution-discovery.md` for any stage-numbering or
naming inconsistent with the unified sequence (e.g., a leftover claim that Discovery isn't
reviewable, or that Stage 10 has no reviewer touchpoint) is clean.
_Verify in Step 4:_ perform the sweep; note any fix made.

---

## Step 3 — Implement

### What was done

| File | Change |
|---|---|
| `dba-system.md` | New "## Default Advisory Review" section (after Non-Negotiable Rules, before the 9-Step Loop): default-review practice, round budget, Solution Discovery's conditional trigger, Review Waiver, and the relationship to `pipeline-reviewer.md`. Added a "Stage ID" column to the "What You Do at Each Stage" table plus a new Solution Discovery row (previously absent); added the onboarding-alternate-entry clarification paragraph. One-line pointer added to the 9-Step Loop's intro. |
| `tools/reviewer/src/packet.rs` | New `stage_checks()`/`stage_expected()` match arms for `"discovery"`, `"brief"`, `"onboarding"`, `"10"`. Added 4 new unit tests confirming these produce real text (not the generic placeholder) and that numeric `"1"`-`"9"` are unchanged. |
| `prompts/pipeline-reviewer.md` | Opening reframed: optional supplementary second opinion, not a replacement for the default review. Stage Summary table, Your Role, and Output Format sections untouched (per AC-6's own scope boundary). |
| `docs/reviewer-pipeline.md` | New "## 12. Downstream usage" section: stage identifiers, the flat cadence (explicitly distinct from `PROFILE-N`), two invocation examples, and a Review Waiver pointer. |
| `prompts/00b-solution-discovery.md` | One paragraph added at the natural handoff point ("After Discovery" section): output gets reviewed when carried into a Feature Brief or Stage 1 Intent; the session itself stays optional/non-gating. |

### Verification (AC-1 through AC-13)

- **AC-1**: `dba-system.md`'s Stage ID table presents the sequence with the explicit
  onboarding-alternate-entry paragraph immediately following it. Confirmed by reading both.
- **AC-2**: the generic instruction states "Advisory review runs by default at every
  reviewable gate across the whole workflow below," and every non-Discovery gate has a Stage
  ID table entry. (AC-2 itself was revised at Step 2 R-review-time to match this DRY,
  self-dev-precedented style rather than requiring a literal repeated command per stage.)
- **AC-3**: the conditional wording ("If its output is actually carried into...") is present
  in both `dba-system.md` and `prompts/00b-solution-discovery.md` — confirmed not
  unconditional.
- **AC-4**: round-budget prose present in `dba-system.md`. `grep -i "PROFILE-"
  dba-system.md prompts/pipeline-reviewer.md docs/reviewer-pipeline.md
  prompts/00b-solution-discovery.md` returns matches **only** from
  `docs/reviewer-pipeline.md`'s pre-existing §4d (Codeos's own self-dev review-round-budget
  table, unrelated to and unchanged by this section) — zero matches in `dba-system.md` or
  either prompt file.
- **AC-5**: Review Waiver section states the "record a reason, don't skip silently, don't
  hard-block" framing and explicitly: "The waiver applies only to the advisory review run. It
  never waives Non-Negotiable Rule #1."
- **AC-6**: `git diff prompts/pipeline-reviewer.md` shows only the opening paragraph added;
  Stage Summary table, Your Role, and Output Format sections are byte-identical to before.
- **AC-7**: `git diff --stat` for this change touches exactly `dba-system.md`,
  `tools/reviewer/src/packet.rs`, `prompts/pipeline-reviewer.md`, `docs/reviewer-pipeline.md`,
  `prompts/00b-solution-discovery.md`, plus the listed backlog/status bookkeeping rows — no
  `intents/`, `contracts/`, `events/`, or other downstream-project-shaped path appears.
- **AC-8**: `git -C /home/rimo/projects/FundFlow status --short` shows the same pre-existing
  untracked entries as before this change (unaffected); `readlink -f
  /home/rimo/projects/FundFlow/.codeos` still resolves to this repo.
- **AC-9**: `git diff --stat -- CLAUDE.md` is empty.
- **AC-10**: verified directly against FundFlow — see "FundFlow dry run" below. **Finding**:
  `.codeos/scripts/codeos-review.sh`, invoked *from* FundFlow, fails with "binary not found"
  — a pre-existing bug where it resolves `REPO_ROOT` via `git rev-parse --show-toplevel` from
  the calling repo (FundFlow) rather than through the `.codeos` symlink to Codeos, so it looks
  for the binary under FundFlow's own nonexistent `tools/reviewer/`. This predates UPG-0037
  (traces to the shim's original design in UPG-0027/UPG-0032) and is **out of scope for this
  change** — filed as follow-up **UPG-0038**. AC-10's verification used
  direct binary invocation instead, which works correctly.
- **AC-11/AC-12**: `cargo build` clean; `cargo test` — **77 smoke tests + 4 new unit tests, 0
  failures**. `stage_expected("1".."9")` confirmed unchanged (new unit test).

**FundFlow dry run** (AC-10/AC-11 concrete evidence): from `/home/rimo/projects/FundFlow`,
invoking `/home/rimo/projects/Codeos/tools/reviewer/target/release/codeos-reviewer review
fundflow-test <stage> --print-packet --skip-prechecks <existing-fundflow-artifact>` for each
of `discovery`, `brief`, `onboarding`, `10` produced the correct new STAGE-SPECIFIC CHECKS /
EXPECTED STAGE OUTPUT text (not the generic placeholder) in every case, against real FundFlow
artifacts (`docs/solution-discovery-fundflow.md`, `backlog/proposal_twin-data-model.md`).
FundFlow's `git status --short` was identical before and after — no side effects.

### Scope check

No edits to `CLAUDE.md`, `templates/stack-manifest.md`/`templates/stack-reconciliation-
report.md`, `tools/reviewer/src/cmd/check_drift.rs`, or any other subcommand's behavior. No
prompt file renamed. `prompts/pipeline-reviewer.md`'s Stage Summary table (covering only
Stages 1-9) intentionally left as-is per AC-6 — it remains the narrower, optional
supplementary tool, not required to mirror the full new Stage ID set.

---

## Step 4 — Reconcile

### Acceptance criteria verification (fresh evidence, raw output)

| AC | Verified by | Result |
|---|---|---|
| AC-1 Stage ID sequence + onboarding clarification | Read `dba-system.md`'s Stage ID table + adjacent paragraph | PASS |
| AC-2 Default review at every gate | Read "Default Advisory Review" section; Stage ID table complete | PASS |
| AC-3 Solution Discovery conditional trigger | Read `dba-system.md` + `00b-solution-discovery.md` conditional wording | PASS |
| AC-4 No `PROFILE-N` leakage (revised scope) | `grep -i "PROFILE-" dba-system.md prompts/pipeline-reviewer.md prompts/00b-solution-discovery.md` → empty (exit 1); §12-isolated grep → empty (exit 1) | PASS |
| AC-5 Review Waiver scoped correctly | Read Review Waiver paragraph; "never waives Non-Negotiable Rule #1" present verbatim | PASS |
| AC-6 `pipeline-reviewer.md` reframed only | `git diff prompts/pipeline-reviewer.md` — only the opening paragraph added, rest untouched | PASS |
| AC-7 No artifact path/schema/filename changes | `git diff --stat` (below) — only doctrine/tooling/docs files + bookkeeping | PASS |
| AC-8 FundFlow unaffected | `git -C .../FundFlow status --short` — identical untracked set as pre-change; symlink resolves correctly | PASS |
| AC-9 `CLAUDE.md`/self-dev governance untouched | `git diff --stat -- CLAUDE.md prompts/codeos-self-dev.md templates/codeos-change.md` → empty | PASS |
| AC-10 No new downstream-side setup; shim limitation disclosed | Docs updated with direct-binary guidance + `UPG-0038` pointer; verified below | PASS |
| AC-11 New stage IDs produce real checklists | `cargo test` → 4 new unit tests pass; live FundFlow dry-run for all 4 identifiers (below) | PASS |
| AC-12 Build/tests green | `cargo build` clean; `cargo test` → **105 passed, 0 failed** (77 smoke + 28 unit, incl. 4 new) | PASS |
| AC-13 No stale cross-references | Swept `00b-solution-discovery.md`'s "Not a reviewer or approval gate" line — still accurate (session stays non-gating; only the *handoff* is reviewed) | PASS |

**Raw evidence:**

```
$ grep -i "PROFILE-" dba-system.md prompts/pipeline-reviewer.md prompts/00b-solution-discovery.md
(no output, exit 1)
$ sed -n '/## 12\. Downstream usage/,/^---$/p' docs/reviewer-pipeline.md | grep -i "PROFILE-"
(no output, exit 1)

$ git diff --stat
 backlog/features.md               |   2 +
 dba-system.md                     |  94 +++++++++++++++++++++++-------
 docs/reviewer-pipeline.md         |  39 +++++++++++++
 prompts/00b-solution-discovery.md |   6 ++
 prompts/pipeline-reviewer.md      |   7 +++
 status/roadmap.md                 |   2 +
 status/self-development.md        |   1 +
 tools/reviewer/src/packet.rs      |  47 +++++++++++++++
 9 files changed, ... (plus reviews/review-log.md, append-only)

$ cargo build && cargo test
    Finished `dev` profile [unoptimized + debuginfo]
test result: ok. 105 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ git -C /home/rimo/projects/FundFlow status --short
?? backlog/
?? docs/solution-discovery-fundflow.md
?? reviews/
?? tenders/
(same 4 pre-existing untracked entries as before this change — no diff)

$ readlink -f /home/rimo/projects/FundFlow/.codeos
/home/rimo/projects/Codeos

# From /home/rimo/projects/FundFlow, direct binary invocation for each new stage:
$ .../codeos-reviewer review fundflow-test discovery --print-packet --skip-prechecks docs/codebase-digest.md
STAGE-SPECIFIC CHECKS
  - every item labeled CANDIDATE/HYPOTHESIZED, not approved; non-authoritative banner present; ...
$ .../codeos-reviewer review fundflow-test brief --print-packet --skip-prechecks docs/codebase-digest.md
STAGE-SPECIFIC CHECKS
  - problem clearly stated; scope explicitly bounded; no implementation detail; ...
$ .../codeos-reviewer review fundflow-test onboarding --print-packet --skip-prechecks docs/codebase-digest.md
STAGE-SPECIFIC CHECKS
  - hypothesized intents clearly labeled as drafts, not approved; ...
$ .../codeos-reviewer review fundflow-test 10 --print-packet --skip-prechecks docs/codebase-digest.md
STAGE-SPECIFIC CHECKS
  - genuinely structural (no contract/schema change); impact assessed before implementing; ...
```

### Cross-reference sweep

- Swept `dba-system.md`, `prompts/pipeline-reviewer.md`, `docs/reviewer-pipeline.md`,
  `prompts/00-session-start.md`, `prompts/00b-solution-discovery.md` for stage-numbering or
  reviewability claims inconsistent with the new default — clean; no fixes needed beyond
  what Step 3 already made.
- Confirmed `prompts/00b-feature-brief.md` and `prompts/00c-onboarding.md` were not touched
  (their reviewer touchpoint is fully described from `dba-system.md`'s side; no internal
  contradiction introduced by leaving them as-is).

### Reviewer scope triage (Step 4 findings)

Step 3 accumulated 3 rounds under `PROFILE-4`'s budget:
- R1 (`CHANGES ADVISED`, IN-SCOPE BLOCKER x3): docs instructing the broken shim path as if it
  worked; `PROFILE-N` literally leaking into the new §12 section; undeclared `UPG-0038`
  bookkeeping. All fixed.
- R2 (`CHANGES ADVISED`, IN-SCOPE BLOCKER x1): AC-4's grep scope was too broad, catching
  `docs/reviewer-pipeline.md`'s own pre-existing self-dev §4d table. Fixed by narrowing AC-4
  to explicitly exclude that pre-existing, legitimately-PROFILE-N-using section while still
  requiring the *new* §12 section be clean.
- R3 (`CHANGES ADVISED`, zero IN-SCOPE BLOCKER findings): verdict driven entirely by a
  `SECRET_REDACTION` coverage flag on a pre-existing, unrelated template field label
  (`Secret / non-secret:`) in `00b-solution-discovery.md`. Budget exhausted at 3 rounds;
  human reviewed and explicitly accepted (`APPROVE_STAGE`, 2026-07-05) rather than requesting
  a 4th round, since no real defect remained.

No Step 4-specific findings — Step 4's own review (below) is the first fresh pass against
the fully reconciled state.

### Outcome

All 13 ACs verified against the final artifacts with raw command/test output (table and
evidence above). No in-scope blockers open. No scope drift — the one addition beyond the
original Step 1 list (`UPG-0038`'s backlog stub and its bookkeeping rows) was explicitly
declared and reviewed as in-scope follow-up filing, not implementation. Step 4 R1 was
accepted human-side despite a `CHANGES ADVISED` verdict, because the sole remaining concern
was a structural `SECRET_REDACTION` false positive (a benign, pre-existing template field in
`00b-solution-discovery.md` that triggers the scanner on any full-content review, not an
in-scope defect) — the same accepted cause as Step 3's final round. Human `APPROVE_STAGE`
recorded (2026-07-05T20:04:19Z), with an explicit note against re-running as `--sha-only`
merely to launder a cleaner packet. Change record, `status/self-development.md`,
`status/roadmap.md`, `backlog/features.md`, and
`backlog/UPG-0037-downstream-default-stage-review.md` updated to COMPLETE in this same pass,
following that approval.
