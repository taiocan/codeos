---
change_id: CHG-20260703-001
feature_id: UPG-0017
slug: stack-manifest
triage_class: template
scope_axis: self-dev only
review_profile: PROFILE-3
review_series: RVS__UPG-0017__CHG-20260703-001__S1
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0017 / CHG-20260703-001 — Stack Manifest with Automatic Reconciliation

## TRACE HEADER

```yaml
feature_id: UPG-0017
primary_feature_id: UPG-0017
change_id: CHG-20260703-001
slug: stack-manifest
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0017
related_features:
  - UPG-0019
  - UPG-0020
review_series: RVS__UPG-0017__CHG-20260703-001__S1
review_profile: PROFILE-3
review_state: ACCEPTED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

---

## Step 1 — Change Intent

### Problem

Downstream projects adopting Codeos have no standard place to record stack decisions —
language, runtime, package manager, test framework, dependency policy, config policy. This
knowledge is either absent or in prose that goes stale when a dependency changes. A
diff-triggered reconciliation report — required when dependency/config files change — keeps
the record honest without adding a manual maintenance burden.

### What changes

| File | Change |
|---|---|
| `templates/stack-manifest.md` | New: two-layer template — (1) stable stack decisions, (2) dependency/config policy, with embedded trigger note |
| `templates/stack-reconciliation-report.md` | New: short-form report filled in when a dependency or config file changes |
| `backlog/UPG-0017-stack-manifest.md` | Feature Thread: CHG-20260703-001 activated |
| `status/self-development.md` | Row activated |
| `status/roadmap.md` | UPG-0017 → IN_PROGRESS |

### Scope boundary — what stays the same

- `dba-system.md` — not touched; no downstream doctrine change in this increment
- No existing prompts modified; no stage table changes
- Downstream projects adopt by reading the templates; no forced workflow change
- The templates are *for* downstream projects; Codeos toolkit's own stack is not documented
  here (Codeos has no `package.json`, no database — a separate concern)

### Design intent

**`templates/stack-manifest.md`:** A fill-in-the-blanks template with two sections:
1. *Stable decisions* — facts that change rarely: language, runtime, package manager, test
   framework, deployment target, event log format, allowed/forbidden dependency categories.
2. *Policy* — how the stack evolves: who approves new dependencies, what justification is
   required, where config lives, how secrets are handled.

An embedded **trigger note** states: if any of the listed dependency/config file types change
(e.g. `Cargo.toml`, `package.json`, `Dockerfile`, `.env.example`, `config/*.yaml`), a
`stack-reconciliation-report.md` must be filled in before merge/release.

**`templates/stack-reconciliation-report.md`:** A short report (8–10 fields) capturing:
what dependency/config files changed, what's new/removed/version-bumped, runtime/test/config
impact, and whether the stack manifest itself needs updating — with a required YES/NO + reason
if NO.

### Triage

- Class: `template`
- Scope axis: `self-dev only`
- Review profile: `PROFILE-3` (1 Codex review per step, max 3 rounds)
- Originating backlog id: `UPG-0017`

---

## Step 2 — Acceptance Criteria

### AC-1 — `templates/stack-manifest.md` exists with two layers

The file contains a **Stable Stack Decisions** section (language/runtime, package manager,
test framework, event log format, deployment target, allowed/forbidden dependency categories)
and a **Policy** section (dependency approval, config layout, secret handling). All fields
are clearly marked as fill-in blanks (e.g. `Language/runtime:`).

### AC-2 — Trigger note is embedded in `templates/stack-manifest.md`

The manifest template contains an explicit trigger note stating which dependency/config file
types (e.g. `Cargo.toml`, `Cargo.lock`, `package.json`, `Dockerfile`, `.env.example`,
`config/*.yaml`) require a stack reconciliation report when changed. The trigger note names
`templates/stack-reconciliation-report.md` as the artifact to fill in.

### AC-3 — `templates/stack-reconciliation-report.md` exists with required fields

The file contains at minimum: dependency/config files changed, new/removed/version-bumped
items, runtime impact, test impact, config impact, and a YES/NO field for whether the
stack manifest itself needs updating — with a required reason if NO.

### AC-4 — Reconciliation report aligns with `readiness-checklist.md:30`

`templates/stack-reconciliation-report.md` is the artifact referenced by the existing
readiness-checklist item "Stack / config manifest checked if dependency or config changed."
The report header or title makes this relationship legible to someone reading the checklist.

### AC-5 — No placeholder text left unfilled in templates

Template files contain only structural fill-in fields (e.g. `Language/runtime:`) and
instructional prose. No `TODO`, `FIXME`, `UPG-####`, or draft-marker text.

### AC-6 — No other files touched

`dba-system.md`, all existing prompts, stage tables, and existing templates are unchanged.

### AC-7 — Consistent style with existing templates

Both new templates follow the minimal heading + colon-field style used in
`templates/readiness-checklist.md` and `templates/stage-4-activation-card.md`: no heavy
prose, no nested bullets beyond two levels, no markdown tables for fill-in sections.

---

## Step 3 — Implement

### Changes made

#### `templates/stack-manifest.md`

New file. Four sections:
- Preamble trigger note listing file types that require reconciliation, naming
  `templates/stack-reconciliation-report.md` as the artifact to fill in (AC-2)
- **Stable Stack Decisions** — 10 colon fields (language, package manager, test framework,
  event log format, replay location, persistence, external services, deployment, allowed/
  forbidden dependency categories) (AC-1)
- **Dependency Policy** — 5 colon fields covering approval, justification, documentation,
  required tests (AC-1)
- **Configuration Policy** — 5 colon fields covering config location, schema validation,
  secret handling, environment config, defaulting (AC-1)
- **Last reconciled** — 3 colon fields (date, commit, triggered-by) (AC-1)

No TODO/FIXME/draft markers; all fields are blank colon placeholders (AC-5).

#### `templates/stack-reconciliation-report.md`

New file. Sections:
- Preamble: states the trigger ("Fill in when any dependency or config file changes"),
  links to `templates/stack-manifest.md`, and names the readiness-checklist item it
  satisfies: "Stack / config manifest checked if dependency or config changed" (AC-4)
- **What changed** — files changed, new/removed/version-bumped dependencies
- **Impact** — runtime, test, config, security/supply-chain (AC-3)
- **Stack manifest update** — YES/NO field + required reason if NO (AC-3)
- **Decision** — two checkboxes: manifest updated (or no update needed with reason) +
  readiness-checklist item satisfied (AC-3/AC-4)

### What was NOT changed

- `dba-system.md` — confirmed unchanged (AC-6)
- All existing prompts, stage tables, templates — unchanged (AC-6)
- `readiness-checklist.md:30` — already references the stack manifest concept; no edit
  needed (the new template aligns to it, not the other way)

---

## Step 4 — Reconcile

### AC verification

| AC | Criterion | Verification | Result |
|---|---|---|---|
| AC-1 | `stack-manifest.md` has Stable Stack Decisions + Policy sections; all fields are fill-in blanks | `templates/stack-manifest.md` sections confirmed: Stable Stack Decisions (10 colon fields), Dependency Policy (5), Configuration Policy (5), Last reconciled (3) | PASS |
| AC-2 | Trigger note embedded; names `stack-reconciliation-report.md`; lists trigger file types | `templates/stack-manifest.md` preamble (lines 3–7) lists 13 file types; explicitly names `templates/stack-reconciliation-report.md` | PASS |
| AC-3 | Reconciliation report has all required fields + YES/NO manifest-update field with required reason if NO | `templates/stack-reconciliation-report.md`: What changed (4 fields), Impact (4 fields), Stack manifest update with `(YES / NO)` constraint and If NO reason, Decision checkboxes | PASS |
| AC-4 | Report aligns with `readiness-checklist.md:30`; relationship legible | Preamble quotes exact text "Stack / config manifest checked if dependency or config changed" from `readiness-checklist.md:30`; R2 reviewer confirmed with file in packet | PASS |
| AC-5 | No TODO/FIXME/draft-marker text | grep for TODO, FIXME, and draft-id markers in both template files — no hits | PASS |
| AC-6 | `dba-system.md`, all existing prompts/templates — unchanged | `git diff HEAD` shows only new template files + status/backlog/change record edits; no existing file modified | PASS |
| AC-7 | Style consistent with `readiness-checklist.md` — minimal, colon-field, no heavy prose | Both templates: flat colon fields, max two heading levels, no fill-in tables; R2 confirmed AC-7 met | PASS |

### Reference / orphan sweep

- `templates/stack-manifest.md` references `templates/stack-reconciliation-report.md` — file exists ✓
- `templates/stack-reconciliation-report.md` references `templates/stack-manifest.md` — file exists ✓
- `readiness-checklist.md:30` already contains the aligned item — no edit needed; no stale link created ✓
- `backlog/UPG-0017-stack-manifest.md` previously listed `docs/stack-manifest.md` as the proposed artifact — Step 4 R1 found this was an orphaned reference. Fixed: updated to `templates/stack-manifest.md` and `templates/stack-reconciliation-report.md`.
- No stage table, prompt, or index file references the new templates yet — correct: downstream adoption is opt-in.

### Scope drift check

Only files in the Step 1 "What changes" table were modified, plus the template files. `dba-system.md` untouched. No new files outside `templates/` created.
