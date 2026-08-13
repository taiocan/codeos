---
change_id: CHG-20260701-005
feature_id: UPG-0013
slug: stage-4-activation-card
triage_class: template
scope_axis: self-dev only
review_profile: PROFILE-3
review_series: RVS__UPG-0013__CHG-20260701-005__S4
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0013 / CHG-20260701-005 — Stage 4 Activation Card

## TRACE HEADER

```yaml
feature_id: UPG-0013
primary_feature_id: UPG-0013
change_id: CHG-20260701-005
slug: stage-4-activation-card
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0013
related_features: []
review_series: RVS__UPG-0013__CHG-20260701-005__S4
review_profile: PROFILE-3
review_state: ACCEPTED
review_history: reviews/review-log.md
triage_class: template
scope_axis: self-dev only
corrects: ~
corrected_by: ~
follow_up_of: ~
fixes_findings: []
```

---

## Step 1 — Change Intent

### Problem

Stage 4 (Implementation Prep) already has three authoritative behavioral inputs: intent,
contract, and event schema. What is missing is activation metadata — the information Claude
needs to scope and execute implementation correctly: which branch to work on, what the
current repo state is, which files are in or out of scope, and what to report when done.

Without this, practitioners either carry this context informally (risking scope creep or
unreported gaps) or try to create an ad-hoc execution packet that duplicates behavioral
artifacts. The backlog explicitly prohibits large OAP-style packets that restate intent and
contract. What is needed is a small, reference-not-restate card that records metadata only.

### What changes

| File | Change |
|---|---|
| `templates/stage-4-activation-card.md` | NEW — Stage 4 Activation Card template. Records: feature ID, references to approved input artifacts, current repo state, branch policy, implementation scope (in/out), execution constraints, and required output fields. Does not restate behavior. |
| `backlog/UPG-0013-stage-4-activation-card.md` | Feature Thread: add this change. |
| `status/self-development.md` | Activate row for this change. |
| `status/roadmap.md` | Wave 3 UPG-0013 row: update planned change and state to IN_PROGRESS. |
| `changes/UPG-0013__CHG-20260701-005__stage-4-activation-card.md` | This change record. |

### What stays the same (scope boundary)

- `dba-system.md` — NOT in scope. Template usable without doctrine entry; follow-on
  `downstream-doctrine` change once proven.
- `prompts/04-impl-prep.md` — NOT modified. The activation card is a practitioner-filled
  template, not a change to Stage 4 prompt behavior.
- Stage prompts `01–09` — NOT modified.
- `prompts/` — no prompt changes.
- `scripts/` — no changes.
- The card must **not** restate behavioral content from intent, contract, or schema
  artifacts — reference pointers only.

### Triage class: `template`

Creating one new template file (`templates/stage-4-activation-card.md`). Class is
`template`. 4-step loop with PROFILE-3 review cadence (downstream-facing).

### Scope axis: `self-dev only`

No changes to `dba-system.md`. Toolkit template files only.

### Review profile: PROFILE-3

Template class, downstream-facing. Codex review before each step gate; human approval at
all four gates; reviewer output is advisory and non-gatekeeping.

### Originating backlog item

`backlog/UPG-0013-stage-4-activation-card.md` — Stage 4 Activation Card.

---

## Step 2 — Acceptance Criteria

### AC-1: Behavioral artifact references are pointers, not restated content

The section recording approved behavioral artifacts (intent, contract, event schema) uses
reference pointer fields only — file paths or labels pointing to those artifacts. It does
not copy, paraphrase, or restate their behavioral content.

Non-pointer fields elsewhere in the template (repo state, branch, scope, execution
constraints, required output) are operational metadata and are not subject to this
constraint.

Verification: the "Approved input artifacts" section (or equivalent) contains only path/
label fields with no prose behavioral content. No field in that section prompts the
practitioner to write requirements, acceptance criteria, scenarios, or event definitions.
`grep -n "scenario\|acceptance criteria\|event definition" templates/stage-4-activation-card.md`
must return no hits inside the artifact-reference section.

### AC-2: No new authority

The template does not introduce requirements, constraints, or decisions beyond what is
already present in the approved intent, contract, and event schema. In the event of any
conflict between a field on the card and an approved artifact, the approved artifact
takes precedence. The card is an execution aide, not an authority source.

Verification: the template includes a note (or the section heading is worded) to make
clear that intent/contract/schema are authoritative and the card is a reference summary.
No field on the card can bind or override an approved artifact.

### AC-3: Stage 4 scoped (optionally Stage 5–6)

The template is scoped to Stage 4 (Implementation Prep) activation. It may note optional
applicability to Stage 5 (Implementation) and Stage 6 (Runtime Verification) as
lightweight reuse, consistent with the backlog brief ("Stage 4 primarily. Optionally
reused by Stage 5 and 6."). It does not embed Stage 1–3 intent/contract/schema definition
steps, Stage 7–10 processes, or present itself as a general all-stages artifact.

Verification: template heading names "Stage 4" or "Implementation" as its primary scope.
`grep -n "^#" templates/stage-4-activation-card.md` — sections cover only:
artifact references, repo state, branch policy, implementation scope, execution
constraints, required output. No section requires Stage 1–3 or Stage 7–10 work.

### AC-4: No behavioral duplication

The template does not duplicate:
- behavioral contracts (what the system must do),
- acceptance criteria (what constitutes correctness),
- event schemas (event names, fields, payloads),
- reconciliation evidence (Stage 7 comparison tables, Stage 8 replay traces).

Verification: template sections contain reference/metadata fields only. No section
prompts the practitioner to re-enter contract clauses, acceptance criteria lists, event
definitions, or Stage 7/8 evidence.

### AC-5: Repo-state metadata is explicitly allowed

The template may include the following operational context fields without violating AC-1
through AC-4:
- Branch name (existing or proposed)
- Commit SHA / working tree status
- Active feature ID
- Files in scope / explicitly out of scope
- Execution constraints (do not change approved artifacts; stop if new behavior required;
  report if contract appears insufficient)
- Required output fields (files changed, risks/blockers, gaps to report)

These are metadata, not behavioral specification. Their presence is correct and expected.

### AC-6: Optional execution aid — no new mandatory gate

The activation card is an optional practitioner tool. Absence of a filled card does not:
- block any DBA stage transition,
- add a new approval gate,
- change any DBA rule or doctrine.

Verification: the template includes a note marking it as optional, or no existing prompt
or doctrine file is modified to require it. `git diff HEAD -- prompts/ dba-system.md CLAUDE.md`
returns no changes (tracked); `git status --short -- prompts/ dba-system.md CLAUDE.md`
returns empty (untracked). Stage prompts do not require the card.

### AC-7: Template is practitioner-filled — no pre-filled behavioral content

The file is a blank template with field labels and placeholder values. It does not contain
pre-filled intent summaries, example contracts, or worked behavioral examples that could be
mistaken for normative content.

Verification: every value field contains a blank, a placeholder (e.g., `___`, `<value>`), or
is explicitly empty. No field contains prose behavioral content.

### AC-8: Out-of-scope files unchanged

`dba-system.md`, `CLAUDE.md`, all stage prompts `01–09`, and `scripts/` are not modified.

Verification (tracked changes):
`git diff HEAD -- dba-system.md CLAUDE.md prompts/00-session-start.md prompts/01-intent.md prompts/02-contract.md prompts/03-event-schema.md prompts/04-impl-prep.md prompts/05-implementation.md prompts/06-runtime-verification.md prompts/07-reconciliation.md prompts/08-stage-replay.md prompts/09-refinement.md scripts/ | wc -l` → 0

Verification (untracked files):
`git status --short -- dba-system.md CLAUDE.md prompts/00-session-start.md prompts/01-intent.md prompts/02-contract.md prompts/03-event-schema.md prompts/04-impl-prep.md prompts/05-implementation.md prompts/06-runtime-verification.md prompts/07-reconciliation.md prompts/08-stage-replay.md prompts/09-refinement.md scripts/ | wc -l` → 0

---

## Step 3 — Implementation

### `templates/stage-4-activation-card.md` (NEW)

Design decisions:

- **Opening notice:** Three-line advisory block makes explicit that the card is optional,
  approved artifacts take precedence, and scope is Stage 4 primary / optionally 5–6.
  Satisfies AC-2 (no new authority), AC-6 (optional, no new gate), AC-3 (scope).
- **Approved input artifacts section:** Three path-only fields (intent, contract, schema)
  with a "reference paths only — do not restate" note. Satisfies AC-1 (pointer-only for
  behavioral references). No prose behavioral content.
- **Operational metadata sections:** Repo state, branch policy, implementation scope,
  execution constraints, required output. These are metadata, not behavioral specification.
  Satisfies AC-5 (metadata explicitly allowed). No behavioral contracts or ACs restated.
- **No pre-filled behavioral content:** Every value field is `___` or a short categorical
  option. No worked examples, no contract clauses, no event definitions. Satisfies AC-7.
- **Execution constraints section:** Fixed constraints re-express the "stop if new behavior"
  principle. These are operational guard-rails derived from DBA philosophy, not new
  requirements not in approved artifacts — the underlying rules already exist in
  `dba-system.md`. Satisfies AC-2 (no new authority beyond approved artifacts).
- **Stage 4 named in heading and scope note.** Satisfies AC-3.
- `dba-system.md`, `CLAUDE.md`, stage prompts `00–09`, `scripts/` not touched. Satisfies AC-8.

---

## Step 4 — Reconcile

### AC Verification

| AC | Verification | Result |
|---|---|---|
| AC-1 | `grep -n "Intent:\|Contract:\|Event schema:" templates/stage-4-activation-card.md` → three `___` path fields; section carries "reference paths only — do not restate" note; no behavioral prose | PASS |
| AC-2 | Opening notice at lines 3–6 explicitly states approved artifacts are authoritative and take precedence; no field introduces new requirements | PASS |
| AC-3 | Heading "# Stage 4 Activation Card" + scope note: "Primary scope: Stage 4 (Implementation Prep). May be reused lightly in Stage 5 and 6." No Stage 1–3 or 7–10 sections | PASS |
| AC-4 | `grep -n "acceptance criteria\|event definition\|Stage 7\|Stage 8\|reconciliation" templates/stage-4-activation-card.md \| wc -l` → 0 | PASS |
| AC-5 | Branch, Commit SHA, Working tree, Files in/out of scope, Execution constraints, Required output — all present as operational metadata fields | PASS |
| AC-6 | Opening notice line 3: "Optional execution aid. Absence of this card does not block any DBA stage transition and does not add a new approval gate." No prompts or doctrine files modified | PASS |
| AC-7 | All value fields contain `___` or short categorical options; no pre-filled behavioral prose | PASS |
| AC-8 | `git diff HEAD -- dba-system.md CLAUDE.md prompts/00-session-start.md prompts/01-intent.md … prompts/09-refinement.md scripts/ \| wc -l` → 0; `git status --short …` → 0 | PASS |

### Cross-reference sweep

| Reference | Target | Status |
|---|---|---|
| Change record → template | `templates/stage-4-activation-card.md` | exists |
| Change record → backlog brief | `backlog/UPG-0013-stage-4-activation-card.md` | OK |
| backlog brief Feature Thread | CHG-20260701-005 row IN_PROGRESS | OK |
| `status/self-development.md` UPG-0013 row | IN_PROGRESS → will mark COMPLETE on acceptance | OK |
| `status/roadmap.md` Wave 3 UPG-0013 row | IN_PROGRESS → will mark COMPLETE on acceptance | OK |

No stage-table↔prompt-file drift. No orphaned links. `dba-system.md` intentionally not
updated (scope boundary declared in Step 1).

### Reviewer scope triage

| Finding | Round | Triage | Disposition |
|---|---|---|---|
| AC-1 said "pointer fields only" across whole template — false for operational metadata sections | Step 2 R1 | IN-SCOPE BLOCKER | Fixed: scoped pointer-only rule to behavioral artifact references section |
| AC-3 said "not a reusable multi-stage artifact" — contradicts backlog scope (Stage 4 primarily, optionally 5–6) | Step 2 R1 | IN-SCOPE BLOCKER | Fixed: aligned to backlog scope language |
| AC-8 verification only checked `prompts/00-session-start.md` but claimed to cover `01–09` | Step 2 R1 | IN-SCOPE BLOCKER | Fixed: extended to list all stage prompts `00–09` explicitly |
