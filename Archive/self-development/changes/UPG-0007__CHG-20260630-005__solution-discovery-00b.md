---
change_id: CHG-20260630-005
feature_id: UPG-0007
slug: solution-discovery-00b
triage_class: prompt
scope_axis: self-dev only
review_profile: PROFILE-3
review_series: RVS__UPG-0007__CHG-20260630-005__S4
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0007 / CHG-20260630-005 — Expanded 00b Solution Discovery

## TRACE HEADER

```yaml
feature_id: UPG-0007
primary_feature_id: UPG-0007
change_id: CHG-20260630-005
slug: solution-discovery-00b
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0007
related_features: []
review_series: RVS__UPG-0007__CHG-20260630-005__S4
review_profile: PROFILE-3
review_state: ACCEPTED
review_history: reviews/review-log.md
triage_class: prompt
scope_axis: self-dev only
corrects: ~
corrected_by: ~
follow_up_of: ~
fixes_findings: []
```

---

## Step 1 — Change Intent

### Problem

`prompts/00b-feature-brief.md` handles individual feature discovery well — one feature at a
time, interview-style, producing a single feature brief. But there is no prompt for the
earlier, broader question: *what features should exist at all?* Before a team can run
00b-feature-brief interviews, they need to understand the problem domain, identify candidate
feature families, surface shared vocabulary, spot architectural pressure points, and decide
what to defer entirely. Without this stage, the first feature brief appears from nowhere and
Stage 1 must implicitly carry the topology reasoning it was never asked to do.

The result is: premature feature entry into the DBA loop, scope creep as undiscovered
dependencies surface late, and architectural decisions embedded in individual intents that
should have been explicit non-decisions at the domain level.

### What changes

| File | Change |
|---|---|
| `prompts/00b-solution-discovery.md` | NEW — solution discovery prompt for pre-feature-brief domain exploration. Covers: domain problem, candidate feature topology, shared vocabulary, event families, configuration hypotheses, architectural risks, and explicit non-decisions. Every output artifact carries the non-authoritative banner. |
| `prompts/00-session-start.md` | Add Session Type E (Solution Discovery) pointing at the new prompt, alongside the existing Session Types A–D. |
| `backlog/UPG-0007-solution-discovery-00b.md` | Feature Thread: add this change. |
| `status/self-development.md` | Activate row for this change. |
| `status/roadmap.md` | Wave 2 UPG-0007 row: update planned change and state to IN_PROGRESS. |
| `changes/UPG-0007__CHG-20260630-005__solution-discovery-00b.md` | This change record. |

### What stays the same (scope boundary)

- `dba-system.md` — NOT in scope. The doctrine's prompt table (`| Feature Brief (pre-Stage 1) | .codeos/prompts/00b-feature-brief.md |`) is intentionally not updated here. The new prompt is usable without a doctrine table entry; the table update is a follow-on `downstream-doctrine` change once the prompt is proven.
- `prompts/00b-feature-brief.md` — NOT modified. Solution discovery is upstream of (and separate from) individual feature briefs; the two prompts work in sequence, not as replacements.
- Stage prompts 01–09 — NOT in scope.
- `templates/` — NOT in scope. The non-authoritative banner lives in the prompt, not in a template.
- `scripts/` — no changes.

### Triage class: `prompt`

Creating one new prompt (`00b-solution-discovery.md`) and updating one existing prompt
(`00-session-start.md` — adding Session Type E). Class is `prompt`. 4-step loop with
PROFILE-3 review cadence.

### Scope axis: `self-dev only`

No changes to `dba-system.md`. We are changing toolkit prompt files only.

### Review profile: PROFILE-3

Prompt class, downstream-facing (used by downstream DBA projects). Codex review before each
step gate; human approval at all four gates; reviewer output is advisory and non-gatekeeping.

### Originating backlog item

`backlog/UPG-0007-solution-discovery-00b.md` — Expanded 00b Solution Discovery / Feature
Topology Stage.

---

## Step 2 — Acceptance Criteria

### AC-1: New prompt file exists and is well-formed

`prompts/00b-solution-discovery.md` is created. It contains:
- A clear purpose statement explaining that this is pre-feature-brief domain exploration.
- Coverage of: domain problem framing, candidate feature topology, shared vocabulary, event
  family hypotheses, configuration hypotheses, architectural risks, and explicit non-decisions.
- The exact three-line non-authoritative banner on every output artifact:
  ```
  This document is non-authoritative planning material.
  It does not approve features, architecture, contracts, schemas, events, or implementation.
  If this document conflicts with later approved DBA artifacts, the approved DBA artifacts win.
  ```

Verification: file exists; the three-line banner appears verbatim (or as a block the practitioner
is instructed to include); all topic areas addressed.

### AC-2: Session Type E added to session-start prompt

`prompts/00-session-start.md` lists Session Type E (Solution Discovery) alongside the
existing Session Types A–D. The entry points to `prompts/00b-solution-discovery.md` and
describes when to use it (exploring a new problem domain before committing to feature briefs).

Verification: `grep -n "E — Solution Discovery" prompts/00-session-start.md` returns a hit;
link to `00b-solution-discovery.md` present; Step 6 label includes E; Step 7 confirmation
includes E.

### AC-3: Session Type E is advisory and does not create a mandatory DBA stage

This is the primary governance constraint for this change. All of the following must hold:

- Session Type E **may** trigger additional solution exploration.
- Session Type E **may** produce recommendations, alternatives, or open questions.
- Session Type E **does not** alter the DBA stage sequence (Stages 1–9 are unchanged).
- Session Type E **does not** become a mandatory step for feature delivery.
- A feature **may** progress through the normal DBA loop without a Session Type E having
  been run.
- The **absence** of Session Type E cannot block feature progression.

Verification: the new prompt and the session-start entry use advisory language throughout
("may", "can", "optional", "candidates"); neither file introduces language that makes
Solution Discovery a prerequisite for Stage 1 or any other stage.

### AC-4: Out-of-scope findings become backlog candidates, not automatic scope expansion

Session Type E may surface valid improvements beyond the active discovery session's scope.
Such findings **must** be recorded as backlog candidates for later evaluation rather than
automatically incorporated into the current work. The prompt instructs practitioners to defer
out-of-scope discoveries explicitly.

This mirrors the principle already in force for reviewer findings: advisory tools gather
evidence; they do not auto-expand scope. The specific backlog tracking mechanism is left to
the downstream project (e.g., a feature list, a to-do, or a backlog file) — the prompt does
not impose Codeos-internal bookkeeping schemes on downstream users.

Verification: the new prompt includes explicit instruction that outputs are candidates for
further evaluation, and that items outside the active discovery session's stated scope should
be recorded for later, not acted on immediately.

### AC-5: `prompts/00b-feature-brief.md` is unchanged

The existing individual-feature-brief prompt is not modified. Solution discovery is upstream
of (and complementary to) individual feature briefs; they work in sequence.

Verification: the file is not listed in the `What changes` table and does not appear in the
Step 3 implementation diff.

### AC-6: `dba-system.md` is unchanged

The downstream doctrine prompt table is not updated in this change (deferring to a follow-on
`downstream-doctrine` change once the prompt is proven). No other `dba-system.md` content
is modified.

Verification: the file is not listed in the `What changes` table and does not appear in the
Step 3 implementation diff.

### AC-7: Stage prompts 01–09 and `templates/` are unchanged

No stage prompts or template files are modified.

Verification: none of these paths are listed in the `What changes` table and none appear in
the Step 3 implementation diff.

---

## Step 3 — Implementation

### `prompts/00b-solution-discovery.md` (NEW)

New prompt file for Session Type E. Key design decisions:

- **Role declaration** opens with an explicit advisory/non-gating statement so the constraint
  is the first thing read, not buried in a later section.
- **"What This Session Is NOT" section** lists five explicit non-gates: not a prerequisite for
  Session A/B, not an approval activity, not a replacement for Stages 1–9, not a reviewer gate.
  Includes a redirect rule: if the human asks for intents/contracts/schemas during this session,
  decline and redirect.
- **Discovery areas** (seven) are presented as optional and exploreable in any order. All
  outputs are labeled CANDIDATE or HYPOTHESIZED inline to keep non-authoritative status visible
  throughout, not only on the banner.
- **Out-of-scope findings** section explicitly instructs the facilitator to surface and defer
  findings outside the session scope as backlog candidates, not incorporate them automatically.
- **Three-line non-authoritative banner** appears verbatim in both the output format section
  (instructed) and the suggested document template — satisfying AC-1.
- **After Discovery** section closes by restating the full DBA path
  (Intent → Contract → Schema → Implement → …) and labeling this prompt as "upstream context,
  not a new stage" — satisfying AC-3.

### `prompts/00-session-start.md` (UPDATED)

Session Type E added after Type D, before the Step 5 separator. Design decisions:

- Labeled **`*(optional, advisory)*`** in the heading to make its status visible at a glance.
- Opening sentence states the use-case (exploration before Feature Briefs).
- Second paragraph restates **optional and non-gating** and reproduces the full standard DBA
  path inline, so a reader scanning session types sees the primary path without needing to
  look elsewhere.
- Output description explicitly states no output is an approved DBA artifact and that
  out-of-scope findings become backlog candidates — satisfying AC-3 and AC-4.
- Step 6 session context label updated to `[A / B / C / D / E — from Step 4]`.
- Step 7 confirmation updated to include E: "The session type (A, B, C, D, or E)".

---

## Step 4 — Reconcile

### AC Verification

Note on AC-5/6/7: All implementation changes for this change are uncommitted working-tree
edits. `git diff HEAD -- <file>` returning 0 lines at reconcile time confirms the file is
absent from the implementation diff (no uncommitted edits). This is a valid reconcile-time
check, not a contract statement.

| AC | Verification | Result |
|---|---|---|
| AC-1 | `grep -c "non-authoritative planning material" prompts/00b-solution-discovery.md` → 2 (instruction + template) | PASS |
| AC-2 | `grep -n "E — Solution Discovery" prompts/00-session-start.md` → hit at line 91; prompt link at line 94; Step 6 label at line 108; Step 7 at line 135 | PASS |
| AC-3 | Reviewer (R4, evidence A) confirmed advisory/non-gating language throughout both files | PASS |
| AC-4 | `grep -n "backlog candidates" prompts/00b-solution-discovery.md` → hit at line 101 | PASS |
| AC-5 | `git diff HEAD -- prompts/00b-feature-brief.md \| wc -l` → 0 | PASS |
| AC-6 | `git diff HEAD -- dba-system.md \| wc -l` → 0 | PASS |
| AC-7 | `git diff HEAD -- prompts/01-intent.md prompts/02-contract.md prompts/03-schema.md prompts/04-impl-prep.md prompts/05-implementation.md prompts/06-testing.md prompts/07-runtime.md prompts/08-reconcile.md prompts/09-replay.md templates/ \| wc -l` → 0 | PASS |

### Cross-reference sweep

| Reference | Target | Status |
|---|---|---|
| Change record → backlog brief | `backlog/UPG-0007-solution-discovery-00b.md` | OK |
| Change record → new prompt | `prompts/00b-solution-discovery.md` | exists |
| Change record → updated prompt | `prompts/00-session-start.md` | exists |
| session-start Session Type E → prompt | `.codeos/prompts/00b-solution-discovery.md` | link present at line 94 |
| backlog brief originating backlog ref | `backlog/UPG-0007-solution-discovery-00b.md` | OK |
| `status/self-development.md` UPG-0007 row | IN_PROGRESS → will mark COMPLETE on acceptance | OK |
| `status/roadmap.md` Wave 2 UPG-0007 row | IN_PROGRESS → will mark COMPLETE on acceptance | OK |

No stage-table↔prompt-file drift. No orphaned links. `dba-system.md` prompt table intentionally not updated (scope boundary declared in Step 1).

### Reviewer scope triage

| Finding | Round | Triage | Disposition |
|---|---|---|---|
| AC-5/6/7 used `git diff HEAD` as the AC contract text — unsound as a contract statement | Step 2 R1 | IN-SCOPE NON-BLOCKER | Fixed: AC text replaced with "not in What changes table; not in implementation diff"; reconcile verification uses `git diff HEAD` as a reconcile-time check (valid because all implementation edits are uncommitted) |
| AC-1 banner paraphrase instead of verbatim | Step 2 R2 | IN-SCOPE NON-BLOCKER | Fixed: exact three-line banner quoted |
| AC-4 used `UPG-####` — Codeos-internal scheme | Step 2 R2 | IN-SCOPE NON-BLOCKER | Fixed: generic "backlog candidates for later evaluation" |
| Session Type E missing from Step 6/7 | Step 3 R1 | IN-SCOPE NON-BLOCKER | Fixed: Step 6 and Step 7 updated to include E |
| Step 3 notes claimed Step 6 label unchanged | Step 3 R2 | IN-SCOPE NON-BLOCKER | Fixed: notes updated to reflect actual changes |
| AC-2 grep string `"Session Type E"` mismatches heading `"E — Solution Discovery"` | Step 3 R3 | IN-SCOPE NON-BLOCKER | Fixed: grep updated to match actual heading format |
