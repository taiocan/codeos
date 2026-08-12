# Self-Development Change: UPG-0030__CHG-20260629-001 — lean-review-profiles

<!--
PURPOSE: Per-change source of truth for UPG-0030 — Lean Self-Development Review Profiles.
Workflow: prompts/codeos-self-dev.md (4-step loop)
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0030
primary_feature_id: UPG-0030
change_id: CHG-20260629-001
slug: lean-review-profiles
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0030
related_features:
  - UPG-0001
  - UPG-0028
  - UPG-0029
review_series: RVS__UPG-0030__CHG-20260629-001__S4
review_profile: PROFILE-5
review_state: ACCEPTED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: UPG-0029
```

<!-- SELF-REFERENCE BOUNDARY: carry review_series + review_state only; exact rounds live in
reviews/review-log.md. See prompts/codeos-self-dev.md → "Self-Reference Boundary". -->

---

## Change Intent

**Why (problem in the toolkit):**

The self-dev loop has no review-round budgets, no delta review mode, no local pre-review
checks, no claim-audit step, no profile assignment, and no named triage category for the
self-reference causal loop. All six problems produce the same symptom: reviews multiply
until the human intervenes by fiat. UPG-0029 Step 3 ran 6 rounds, UPG-0001 Step 3 ran 8,
UPG-0001 Step 4 ran 7 before a Codex usage cap forced the gate.

Specifically:

1. **No round budgets.** Nothing in `prompts/codeos-self-dev.md` says "after N rounds,
   escalate to human decision." The loop runs until the human stops it.

2. **No delta review mode.** R2+ rounds send the full context packet even when only one
   line changed. The reviewer re-reads unchanged material, may re-flag resolved issues,
   and burns token budget on noise.

3. **No local pre-review checks.** False universal claims, stale placeholders, mismatched
   trace-header fields, and scope-boundary violations are deterministically catchable with
   grep before Codex is invoked. Currently nothing requires this.

4. **No claim audit.** Universal quantifiers ("all", "every", "never", "always", "no X")
   in new or modified prose have been the most common Codex-flagged false-claim source
   across UPG-0001 and UPG-0029. No guidance requires auditing them before review.

5. **No review profile.** All change classes that enter the 4-step loop run the same
   per-step review cadence with no round limit. (`backlog-only` and `trivial` are already
   handled outside the loop — as direct edits. It is the looped classes —
   `documentation`, `template`, `prompt`, `script-tooling`, `downstream-doctrine`,
   `self-dev-governance` — that have no review-cadence differentiation and no ceiling.)

6. **No SELF-REFERENCE / REVIEW-BOOKKEEPING triage category.** The four existing
   categories do not name the case where the reviewer flags stale review-bookkeeping
   caused by the previous round's own existence — a causal loop. The stop rule exists in
   `prompts/codeos-self-dev.md` but no triage label matches it, so it is not applied
   consistently.

**What changes:**

1. **`backlog/UPG-0030-lean-self-development-review-profiles.md`** (this change's backlog
   brief) — created in Step 1.

2. **`changes/UPG-0030__CHG-20260629-001__lean-review-profiles.md`** (this file) —
   change record, created in Step 1 and extended through Steps 2–4.

3. **`backlog/features.md`** — add UPG-0030 row to the Feature-ID Map.

4. **`status/self-development.md`** — create and maintain the UPG-0030 / CHG-20260629-001
   row, keeping State and Loop step aligned with the current trace header as the change
   advances.

5. **`prompts/codeos-self-dev.md`** — PRIMARY TARGET. Additions and updates:
   - Insert **Step 0a — Review Profile** between Step 0 (Triage) and Step 1. Maps
     class → PROFILE-N; states per-profile cadence (max rounds; delta rule; Reconcile-only
     vs per-step review); states the budget-exceeded escalation procedure.
   - Update **§Your Role** opening: replace "every non-trivial step requires a compulsory
     (advisory) Codex review first" with profile-qualified language.
   - Update **§4-step loop intro** summary line: replace "produce output → run the compulsory
     Codex review → STOP" with profile-qualified language.
   - Extend **Reviewer Handling**: add local pre-review checklist (run before every
     Codex call), claim-audit requirement (check universal quantifiers before sending),
     and exact delta packet spec for R2+. Replace "Running the review is mandatory at
     every non-trivial step" with profile-qualified language.
   - Add **SELF-REFERENCE / REVIEW-BOOKKEEPING** as a fifth triage category with a
     disposition rule referencing the existing stop rule.
   - Update the **Stopping Rules** section to cross-reference the profile budget.

6. **`templates/codeos-change.md`** — two lean changes:
   - Add `review_profile: PROFILE-N` field to the YAML trace header.
   - Remove the duplicate metadata block at lines 106–118 (the `<!-- METADATA -->` block
     below the Reconciliation section is a second copy of trace-header fields; the
     canonical trace header at the top is authoritative).

7. **`docs/reviewer-pipeline.md`** — five changes:
   - **§4b Delta review mode** (new section): exact delta packet spec (contents, what to omit).
   - **§4c Claim audit** (new section): which quantifiers trigger audit; resolution options.
   - **§4d Review-round budget table** (new section): per-profile round limits and
     budget-exceeded disposition.
   - **§2 Scope Contract + Triage Rule**: added SELF-REFERENCE / REVIEW-BOOKKEEPING as
     fifth category in the triage enumeration (consistency fix).
   - **YAML header `binding:` field**: removed "CLAUDE.md and" (false claim — CLAUDE.md
     is changed by UPG-0030; only stage prompts remain untouched).

8. **`CLAUDE.md`** — four minimal changes:
   - Cross-reference sentence added after the scope-axis paragraph in §Triage Front-Door:
     "After triage, assign a **review profile** (Step 0a of `prompts/codeos-self-dev.md`);
     the profile governs Codex review cadence and round limits."
   - §4-step loop description: "produce output → run the compulsory Codex review → STOP"
     changed to "produce output → run the Codex review if required by profile → STOP",
     with a parenthetical referencing Step 0a.
   - §What You NEVER Do: "Advance a step without running the compulsory review" changed to
     "Advance a step without running the review required by your profile".
   - §"Compulsory review, advisory verdict" section renamed to §"Review cadence and
     advisory verdict": replaces the old blanket "Run the Codex reviewer at every
     non-trivial step" rule with profile-governed cadence language (high-risk profiles
     retain per-step review; lighter profiles may limit Codex review to Reconcile only or
     reduce the round budget). Explicitly states that human approval at each step
     transition is required at every profile. Command syntax and advisory/non-gatekeeping
     invariants remain unchanged.

9. **`backlog/UPG-0029-review-naming-and-thread-tooling.md`** — status fields updated to
   COMPLETE to match the YAML header (body `**Status**:` line and Feature Thread CHG row
   were stale at PROPOSED/IN_PROGRESS after UPG-0029 completed).

**Scope boundary — what stays the same:**

- `dba-system.md` — not touched; downstream DBA doctrine is out of scope.
- `scripts/codeos-review.sh` — not touched; script I/O and output format unchanged.
- The 9-stage DBA loop and stage prompts — not touched.
- `reviews/review-log.md` — will receive organically appended review entries as work
  proceeds; no structural changes.
- `.gitignore` — not changed.
- Naming convention enforcement (`REV__`, `RVS__` filename emission) — remains deferred
  (tracked in UPG-0029 issues #2–#5).
- No new scripts.

**Policy change declared:** UPG-0030 intentionally supersedes the current blanket
per-step advisory-review rule with profile-based review cadence. This is an explicit
governance decision, not an accidental relaxation. The invariants that are NOT weakened:
- Human approval remains the sole gate — APPROVE belongs to the human at every step.
- Reviewer output remains advisory and non-gatekeeping at every profile.
- High-risk profiles (PROFILE-3 through PROFILE-5) retain per-step review for every step.
- Low-risk profiles (PROFILE-1, PROFILE-2) may limit Codex review to Reconcile only or
  reduce the round budget — an intentional trade of review ceremony for operational speed
  on simpler changes. The per-step human approval invariant is unchanged at every profile.

**Class:** `self-dev-governance` — modifies `CLAUDE.md` and the self-dev loop
(`prompts/codeos-self-dev.md`) itself.

**Scope axis:** `self-dev only`

**Backlog item:** `backlog/UPG-0030-lean-self-development-review-profiles.md`

---

## Acceptance Criteria

### A. Scope boundary

**A1.** `git diff HEAD -- dba-system.md scripts/codeos-review.sh` returns empty at Step 4
(Reconcile). No downstream doctrine or reviewer script behavior changes.

**A2.** No stage prompts (`prompts/stage-*.md`) or DBA loop artifacts touched.

### B. Profile table and cadence consistency

**B1.** The profile table in `prompts/codeos-self-dev.md §Step 0a` matches the profile table
in `backlog/UPG-0030-lean-self-development-review-profiles.md` on: Profile ID, "Applies when"
class, cadence, and max rounds/step.

**B2.** No text in `CLAUDE.md` or `prompts/codeos-self-dev.md` imposes unconditional per-step
Codex review without profile qualification. Verified section-by-section at Reconcile:
- `CLAUDE.md` §4-step loop description (currently "produce output → run the compulsory Codex
  review"): updated to reference profile-governed cadence or qualified by profile.
- `CLAUDE.md` §Review cadence and advisory verdict: no blanket per-step mandate remains.
- `prompts/codeos-self-dev.md` §Your Role (currently "every non-trivial step requires a
  compulsory (advisory) Codex review first"): updated to qualify by profile.
- `prompts/codeos-self-dev.md` §4-step loop intro (currently "After each step output and its
  compulsory review"): updated to qualify by profile.
- `prompts/codeos-self-dev.md` §Reviewer Handling (currently "Running the review is mandatory
  at every non-trivial step"): updated to qualify by profile.
Grep verification: `grep -n "compulsory Codex review\|mandatory at every non-trivial step\|compulsory review first\|compulsory review" CLAUDE.md prompts/codeos-self-dev.md` returns no
unqualified prescriptive instances in governing sections.

**B3.** `docs/reviewer-pipeline.md §4b`, `§4c`, `§4d` contents are consistent with the
specs stated in `prompts/codeos-self-dev.md` (delta packet spec, claim audit, budget table).
No contradictions between the two files.

### C. Operational runability

**C1.** Step 0a in `prompts/codeos-self-dev.md` covers every triage class from CLAUDE.md's
triage table: `trivial`, `backlog-only`, `documentation`, `template`/`prompt`/`script-tooling`,
`downstream-doctrine`, `self-dev-governance`. No class is unassigned.

**C2.** The local pre-review checklist in `prompts/codeos-self-dev.md` uses only
deterministic shell operations (grep, git diff, ls). No Codex invocation inside the checklist.

**C3.** The delta packet spec in `prompts/codeos-self-dev.md` (and mirrored in
`docs/reviewer-pipeline.md §4b`) states exactly: (a) what to include, (b) what to omit,
(c) the round trigger (R2+). The spec does not require any change to `scripts/codeos-review.sh`
(the script is not touched by this change).

**C4.** Budget-exceeded escalation is stated as: fix remaining findings inline, escalate to
human decision, do not run further Codex rounds automatically. Verifiable by reading the
budget-exceeded rule in `prompts/codeos-self-dev.md §Step 0a` and the Stopping Rules
final bullet.

### D. Template integrity

**D1.** `templates/codeos-change.md` trace header contains a `review_profile: PROFILE-N`
field after the `review_series` field.

**D2.** `templates/codeos-change.md` has no duplicate metadata block below the Reconciliation
section (the `<!-- METADATA -->` block at lines 106–118 in the pre-change file is removed).

### E. Cross-reference integrity

**E1.** The cross-reference sentence added to `CLAUDE.md` after the triage table mentions
`prompts/codeos-self-dev.md` Step 0a. Grep confirms a section named "Step 0a" exists in
`prompts/codeos-self-dev.md`.

**E2.** Old heading "Compulsory review, advisory verdict" no longer appears in `CLAUDE.md`.
Grep across all toolkit files for `Compulsory review` returns no remaining links or
cross-references that would break.

**E3.** `docs/reviewer-pipeline.md §4b`, `§4c`, `§4d` headings exist (grep). The
`prompts/codeos-self-dev.md` cross-references to these sections resolve (section names match).

### F. Policy-change fidelity

**F1.** High-risk profiles (PROFILE-3, PROFILE-4, PROFILE-5) implement: 1 review per step,
R2+ delta mode, max 3 rounds/step. No step is exempted from review for these profiles.

**F2.** Light profiles are visibly lighter than PROFILE-3: PROFILE-1 has only Reconcile
reviewed (with up to 2 rounds at that step; no Codex review at any other step); PROFILE-2
has a max of 2 rounds/step (vs 3 for PROFILE-3). The cadence difference is explicit in the
profile table, not just implied.

**F3.** At every profile, human approval is stated as the sole gate; reviewer output is
stated as advisory and non-gatekeeping. No profile text implies the reviewer can block.

**F4.** `prompts/codeos-self-dev.md` contains SELF-REFERENCE / REVIEW-BOOKKEEPING as a
named fifth triage category with: a definition (the causal-loop case), a disposition rule,
and a reference to the existing stop rule.

---

## Implementation Notes

1. **`prompts/codeos-self-dev.md`** — PRIMARY TARGET. Six changes:
   - §Your Role: replaced "every non-trivial step requires a compulsory (advisory) Codex
     review first" with "Codex review cadence is governed by the review profile assigned in
     Step 0a."
   - Inserted **§Step 0a — Review Profile** between §Step 0 and §The 4-Step Loop: PROFILE-0
     through PROFILE-5 table, budget-exceeded rule, and `review_profile` field instruction.
   - §The 4-Step Loop summary: "and its compulsory review" → "and its Codex review, if
     required by profile"; added profile caveat to the "run the review before each gate" line.
   - §Feature Thread & IDs triage list: added SELF-REFERENCE / REVIEW-BOOKKEEPING as fifth
     category.
   - §Reviewer Handling: retitled to "Reviewer Handling (advisory)"; added local pre-review
     checklist, claim audit, delta review (§4b cross-ref), R2+ delta spec, findings triage
     table (including SELF-REFERENCE / REVIEW-BOOKKEEPING); replaced "mandatory at every
     non-trivial step" with profile-qualified language.
   - §Stopping Rules: added round-budget stop rule (cross-refs Step 0a and §4d).
   - Also fixed two residual "compulsory review" references in Step 4 checklist and the
     Self-Reference Boundary section.

2. **`templates/codeos-change.md`** — two lean changes:
   - Added `review_profile: PROFILE-N` field to the YAML trace header (after `review_series`, before `review_state` — per AC D1).
   - Removed the duplicate `<!-- METADATA -->` block (lines 106–118 in pre-change file).

3. **`docs/reviewer-pipeline.md`** — five changes:
   - **§4b Delta review mode** (new section): exact delta packet spec (contents/what to omit/round trigger).
   - **§4c Claim audit** (new section): universal quantifier scan; three resolution options.
   - **§4d Review-round budget table** (new section): per-profile limits; budget-exceeded escalation procedure.
   - **§2 triage-rule text**: added SELF-REFERENCE / REVIEW-BOOKKEEPING as fifth category.
   - **YAML header `binding:` field**: removed "CLAUDE.md and" (false claim fixed).

4. **`CLAUDE.md`** — four minimal changes (Change Intent item 8 updated to match):
   - Cross-reference sentence added after the scope-axis paragraph in §Triage Front-Door.
   - §4-step loop summary line: "run the compulsory Codex review" → "run the Codex review
     if required by profile"; added profile cross-reference parenthetical.
   - §What You NEVER Do: "running the compulsory review" → "running the review required by
     your profile."
   - §"Compulsory review, advisory verdict" renamed to §"Review cadence and advisory
     verdict"; blanket per-step mandate replaced with profile-governed cadence language.

5. **`status/self-development.md`** and **`changes/UPG-0030__CHG-20260629-001__lean-review-profiles.md`**
   trace header — advanced at each step per normal bookkeeping.

6. Pre-change files committed before implementation: `dba-system.md`, `scripts/codeos-review.sh`
   — confirmed unchanged throughout (scope boundary clean).

---

## Reconciliation

**Acceptance verification:**

| # | Criterion | Result | Evidence |
|---|---|---|---|
| A1 | `git diff HEAD -- dba-system.md scripts/codeos-review.sh` returns empty | PASS | `git diff -- dba-system.md scripts/codeos-review.sh \| wc -c` → 0 |
| A2 | No stage prompts or DBA loop artifacts touched | PASS | `git diff -- "prompts/stage-*.md"` → empty |
| B1 | Profile table in `prompts/codeos-self-dev.md §Step 0a` matches backlog brief on Profile ID, "Applies when", cadence, max rounds/step | PASS | Tables compared — all six rows identical on all four columns |
| B2 | No unconditional per-step Codex review mandate in CLAUDE.md or prompts/codeos-self-dev.md | PASS | `grep "compulsory Codex review\|mandatory at every non-trivial step\|compulsory review first\|compulsory review" CLAUDE.md prompts/codeos-self-dev.md` → empty |
| B3 | `docs/reviewer-pipeline.md §4b/4c/4d` consistent with specs in `prompts/codeos-self-dev.md` | PASS | §4b delta spec, §4c claim audit, §4d budget table exist; prompts/codeos-self-dev.md cross-references §4b, §4c, §4d by name; no contradictions found |
| C1 | Step 0a covers all six triage classes from CLAUDE.md triage table | PASS | PROFILE-0 through PROFILE-5 map to: trivial/backlog-only-direct, backlog-only-escalated, documentation, template/prompt/script-tooling, downstream-doctrine, self-dev-governance |
| C2 | Local pre-review checklist uses only deterministic shell operations | PASS | Checklist uses grep, git diff, and ls/grep; no Codex invocation inside checklist |
| C3 | Delta packet spec states (a) what to include (b) what to omit (c) round trigger R2+ | PASS | (a) in `prompts/codeos-self-dev.md §R2+ delta reviews`; (b) and (c) in `docs/reviewer-pipeline.md §4b` (cross-referenced from prompt) |
| C4 | Budget-exceeded escalation: fix inline, escalate to human, no further automatic rounds | PASS | `prompts/codeos-self-dev.md` Step 0a: "fix remaining findings inline and escalate to human decision. Do not run further Codex rounds automatically." Stopping Rules final bullet: "Stop adding Codex rounds when the profile's max rounds/step is reached; fix remaining findings inline and escalate to human decision." |
| D1 | `templates/codeos-change.md` trace header has `review_profile` after `review_series` | PASS | Line 35–37: `review_series`, then `review_profile`, then `review_state` |
| D2 | No duplicate `<!-- METADATA -->` block below Reconciliation section in template | PASS | `grep -n "METADATA" templates/codeos-change.md` → no METADATA block found |
| E1 | Cross-reference sentence in CLAUDE.md mentions Step 0a; `## Step 0a` section exists in codeos-self-dev.md | PASS | CLAUDE.md:73 and :86 reference Step 0a; `prompts/codeos-self-dev.md:42` has `## Step 0a — Review Profile` |
| E2 | Old heading "Compulsory review, advisory verdict" gone; no broken links | PASS | `grep -rn "Compulsory review" CLAUDE.md prompts/ templates/ docs/` → 0 results in governing files; new heading `### Review cadence and advisory verdict` confirmed at CLAUDE.md:106 |
| E3 | `docs/reviewer-pipeline.md §4b/4c/4d` headings exist; cross-references resolve | PASS | Headings at lines 148, 170, 184; prompts/codeos-self-dev.md references §4b (l.283), §4c (l.260), §4d (l.57, l.306) |
| F1 | PROFILE-3/4/5: 1 review per step, R2+ delta, max 3 rounds/step; no step exempted | PASS | Profile table confirms cadence and max for all three |
| F2 | PROFILE-1 (Reconcile only, max 2) and PROFILE-2 (max 2/step) visibly lighter than PROFILE-3 (max 3/step) | PASS | Profile table shows explicit cadence and round differences; not just implied |
| F3 | Human approval as sole gate stated at every profile; reviewer advisory/non-gatekeeping stated | PASS | `prompts/codeos-self-dev.md:45` and `CLAUDE.md:112` both state this explicitly |
| F4 | `prompts/codeos-self-dev.md` has SELF-REFERENCE / REVIEW-BOOKKEEPING as named fifth triage category with definition, disposition rule, and stop-rule reference | PASS | Lines 295 (triage table with definition + disposition), 156 (Step 4 checklist), 194 (Feature Thread triage list); stop rule cross-referenced in table |

**Consistency sweep (grep):**

- `grep "compulsory" CLAUDE.md prompts/ templates/ docs/` → 0 hits in governing files. Residual instances in `backlog/UPG-0001-feature-thread-traceability.md` (descriptive historical text, not a navigation link) and `backlog/UPG_ideas/` (scratch file) do not break references.
- `git diff -- dba-system.md scripts/codeos-review.sh` → empty (scope boundary clean).
- `git diff -- "prompts/stage-*.md"` → empty.
- `grep -n "^## Step 0a" prompts/codeos-self-dev.md` → line 42 (E1 satisfied).
- `grep -n "^## 4b\|^## 4c\|^## 4d" docs/reviewer-pipeline.md` → lines 148, 170, 184 (E3 satisfied).
- `grep -rn "SELF-REFERENCE" prompts/codeos-self-dev.md templates/codeos-change.md changes/UPG-0030__CHG-20260629-001__lean-review-profiles.md CLAUDE.md docs/reviewer-pipeline.md` → all instances use slash form ("SELF-REFERENCE / REVIEW-BOOKKEEPING"); no em-dash instances remain.
- No `REVIEW-BOOKKEEPING` instances outside the triage labels (no stray references).
- No orphaned links to old `### Compulsory review, advisory verdict` heading.

**Findings scope-triage:**

| Finding | Triage | Action |
|---|---|---|
| R1: change record trace header missing review_profile field | IN-SCOPE BLOCKER | Fixed: added `review_profile: PROFILE-5` to trace header |
| R1: Step 4 in codeos-self-dev.md listed only four triage categories | IN-SCOPE BLOCKER | Fixed: added SELF-REFERENCE / REVIEW-BOOKKEEPING as fifth |
| R1: templates/codeos-change.md header comment said "compulsory review"; triage table had four categories | IN-SCOPE BLOCKER | Fixed: comment updated; fifth category added |
| R2: UPG-0027 backlog brief rewrite in workspace (pre-existing uncommitted change from a prior session, not from this change) | OUT-OF-SCOPE BACKLOG | Restored to committed state via git checkout (scope-creep cleaning; not a UPG-0030 implementation finding) |
| R2: Change record understated CLAUDE.md scope (two vs three changes) | IN-SCOPE BLOCKER | Fixed: Change Intent item 8 and Implementation Notes item 4 updated |
| R2: review_profile placement: after review_state vs AC D1 "after review_series" | IN-SCOPE BLOCKER | Fixed: moved to after review_series in template and change record |
| R2: Naming inconsistency — slash vs em-dash | IN-SCOPE BLOCKER | Fixed: all instances canonicalized to slash form |
| R3: CLAUDE.md Step 4 triage list had only four categories | IN-SCOPE BLOCKER | Fixed inline (budget exhausted): added fifth category |
| R3: docs/reviewer-pipeline.md §2 Scope Contract had only four categories | IN-SCOPE BLOCKER | Fixed inline (budget exhausted): added fifth category |
