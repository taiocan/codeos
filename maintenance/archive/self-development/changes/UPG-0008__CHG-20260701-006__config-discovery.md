---
change_id: CHG-20260701-006
feature_id: UPG-0008
slug: config-discovery
triage_class: prompt
scope_axis: self-dev only
review_profile: PROFILE-3
review_series: RVS__UPG-0008__CHG-20260701-006__S4
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0008 / CHG-20260701-006 — Configuration Discovery and Configuration Schema Track

## TRACE HEADER

```yaml
feature_id: UPG-0008
primary_feature_id: UPG-0008
change_id: CHG-20260701-006
slug: config-discovery
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0008
related_features:
  - UPG-0007
review_series: RVS__UPG-0008__CHG-20260701-006__S4
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

Configuration requirements often emerge across multiple features during discovery. When
they are not surfaced early, implementations may hardcode behavior, introduce hidden
environment assumptions, or require expensive retrofits. The existing `prompts/00b-solution-discovery.md`
(introduced by UPG-0007) has no dedicated section for configuration discovery.

Without explicit prompting, configuration assumptions remain implicit and untraced until
they cause problems in implementation or deployment.

### What changes

| File | Change |
|---|---|
| `prompts/00b-solution-discovery.md` | Add a "Candidate Configuration Requirements" discovery area as a new numbered section. The section prompts discovery of config names, purpose, affected features, defaults, required/optional, secret/non-secret, environment-specificity, runtime-changeability, validation needs, failure modes, and event impact. Output is clearly marked hypothesis-only — non-authoritative until routed through approved Stage 1–3 or an ADR. |
| `backlog/UPG-0008-config-discovery.md` | Feature Thread: add this change. |
| `status/self-development.md` | Activate row for this change. |
| `status/roadmap.md` | Wave 2 UPG-0008 row: update planned change and state to IN_PROGRESS. |
| `changes/UPG-0008__CHG-20260701-006__config-discovery.md` | This change record. |

### What stays the same (scope boundary)

- `dba-system.md` — NOT in scope. Config discovery output routes through existing DBA
  stages (Stage 1–3 or Stage 10 ADR) — no doctrine change required.
- `prompts/00-session-start.md` — NOT modified.
- Stage prompts `01–09` — NOT modified. Routing guidance in the 00b section simply
  points at existing stages.
- `templates/` — no changes.
- `scripts/` — no changes.
- The section is advisory/hypothesis output only — it must not introduce a new mandatory
  config artifact or approval gate.

### Triage class: `prompt`

Extending one existing prompt file (`prompts/00b-solution-discovery.md`). Class is
`prompt`. 4-step loop with PROFILE-3 review cadence (downstream-facing).

### Scope axis: `self-dev only`

No changes to `dba-system.md`. Toolkit prompt files only.

### Review profile: PROFILE-3

Prompt class, downstream-facing. Codex review before each step gate; human approval at
all four gates; reviewer output is advisory and non-gatekeeping.

### Originating backlog item

`backlog/UPG-0008-config-discovery.md` — Configuration Discovery and Configuration Schema Track.

---

## Step 2 — Acceptance Criteria

### AC-1: Section is present as a numbered discovery area in `prompts/00b-solution-discovery.md`

A "Candidate Configuration Requirements" section (or equivalent heading containing
"Configuration") is added to `prompts/00b-solution-discovery.md` as a **numbered** `###`
discovery area, consistent with the existing `### N.` subsection structure of the prompt.

Verification: `grep -n "^### [0-9].*[Cc]onfig" prompts/00b-solution-discovery.md`
returns at least one hit — proving the section exists as a numbered subsection heading
(the 00b prompt uses `### N.` for discovery areas, not `## N.`).

### AC-2: All eleven discovery fields are present

The section contains all eleven fields from the backlog design:

1. Config name
2. Purpose
3. Feature(s) likely affected
4. Default
5. Required/optional
6. Secret/non-secret
7. Environment-specific
8. Runtime-changeable
9. Validation needed
10. Possible failure mode
11. Possible event impact

Verification: all eleven field labels (or close equivalents) appear within the
"Candidate Configuration" section of `prompts/00b-solution-discovery.md`.

### AC-3: Output is hypothesis-only — non-authoritative framing is explicit within the section

The section must carry explicit advisory/non-authoritative language within the section
itself (heading, opening note, or closing note) — not merely somewhere else in the file.

Verification: the lines immediately following the config section heading contain advisory
framing:
`grep -A 5 "^### [0-9].*[Cc]onfig" prompts/00b-solution-discovery.md | grep -i "candidate\|hypothesis\|advisory\|non-authoritative"`
returns at least one hit — proving the framing is anchored to the section, not a
false-pass from an unrelated occurrence elsewhere in the file.

### AC-4: Routing guidance is present

The section must include routing guidance stating:
- Config that changes observable behavior → Stage 1–3
- Config that is structural or infrastructure-level → Stage 10 / ADR
- Config docs/examples needing update → readiness checklist

Verification: all three routing paths are named.

### AC-5: Section appears exactly once; advisory banner is still present

The section appears as exactly one numbered `###` discovery-area heading containing "Config". The
advisory/non-authoritative banner that opens `00b-solution-discovery.md` is still present
after the change. Non-modification of existing `00b` sections is verified by human review
of the implementation diff at Step 4 reconcile.

Verification — no duplication:
`grep -c "^### [0-9].*[Cc]onfig" prompts/00b-solution-discovery.md` → exactly 1

Verification — advisory banner still present:
`head -10 prompts/00b-solution-discovery.md | grep -i "NOT\|advisory\|non-authoritative\|hypothesis"`
returns at least one hit confirming the banner exists in the opening lines of the file.

### AC-6: No new mandatory gate — section is advisory

The section does not introduce a new mandatory artifact, approval gate, or required output.
The 00b prompt remains advisory throughout. Absence of a filled config section does not
block Stage 1.

Verification: no language in the section makes config discovery mandatory or blocks
proceeding to Stage 1 if the section is left empty.

### AC-7: Out-of-scope files unchanged

`dba-system.md`, `CLAUDE.md`, `prompts/00-session-start.md`, stage prompts `01–09`,
`templates/`, and `scripts/` are not modified.

Verification: `git diff HEAD -- dba-system.md CLAUDE.md prompts/00-session-start.md prompts/01-intent.md prompts/02-contract.md prompts/03-event-schema.md prompts/04-impl-prep.md prompts/05-implementation.md prompts/06-runtime-verification.md prompts/07-reconciliation.md prompts/08-stage-replay.md prompts/09-refinement.md templates/ scripts/ | wc -l` → 0;
`git status --short -- dba-system.md CLAUDE.md prompts/00-session-start.md prompts/01-intent.md prompts/04-impl-prep.md templates/ scripts/ | wc -l` → 0.

---

## Step 3 — Implementation

### `prompts/00b-solution-discovery.md` (UPDATED)

Section 5 expanded from `### 5. Configuration Hypotheses` to
`### 5. Configuration Hypotheses and Candidate Requirements`. Design decisions:

- **Heading retained:** existing section 5 was a brief placeholder treatment; this change
  expands it in place rather than adding a duplicate section. No numbering gaps introduced.
  Satisfies AC-1 (`grep "^### [0-9].*[Cc]onfig"` hits section 5) and AC-5 (single matching
  heading — `grep -c` → 1).
- **Opening advisory note:** `> Output from this section is HYPOTHESIZED / CANDIDATE only`
  anchored directly under the heading. Satisfies AC-3 (advisory framing within the section,
  not a file-level false-pass).
- **Eleven fields** as structured template:
  Config name, Purpose, Feature(s) likely affected, Default, Required/optional,
  Secret/non-secret, Environment-specific, Runtime-changeable, Validation needed,
  Possible failure mode, Possible event impact. Satisfies AC-2.
- **Routing note:** names all three routing paths — behavioral → Stage 1–3; structural →
  Stage 10/ADR; docs → readiness checklist. Satisfies AC-4.
- **Advisory throughout:** framing questions retained; no mandatory language; section is
  optional. Satisfies AC-6. "Do not carry config hypotheses into implementation without
  explicit routing" reinforces non-authoritative status.
- **AC-1 grep correction noted in Step 2:** the existing 00b structure uses `### N.`
  subsections, so AC-1 verification uses `grep "^### [0-9].*[Cc]onfig"` not `## [0-9]`.
  Fixed in AC text during Step 3.
- `dba-system.md`, `CLAUDE.md`, `prompts/00-session-start.md`, stage prompts `01–09`,
  `templates/`, `scripts/` not touched. Satisfies AC-7.

---

## Step 4 — Reconcile

### AC Verification

| AC | Verification | Result |
|---|---|---|
| AC-1 | `grep -n "^### [0-9].*[Cc]onfig" prompts/00b-solution-discovery.md` → hit at line 76: `### 5. Configuration Hypotheses and Candidate Requirements` | PASS |
| AC-2 | `grep -c "Config name:\|Purpose:\|Feature.s. likely\|Default:\|Required / optional\|Secret / non-secret\|Environment-specific\|Runtime-changeable\|Validation needed\|Possible failure mode\|Possible event impact"` → 11 | PASS |
| AC-3 | `grep -A 5 "^### [0-9].*[Cc]onfig" … \| grep -i "candidate\|hypothesis\|advisory\|non-authoritative"` → "> Output from this section is HYPOTHESIZED / CANDIDATE only" | PASS |
| AC-4 | Lines 104–106: Stage 1–3 (behavioral), Stage 10/ADR (structural), Readiness checklist (docs) | PASS |
| AC-5 | `grep -c "^### [0-9].*[Cc]onfig"` → 1; `head -10 \| grep advisory` → "This session is optional and non-gating" | PASS |
| AC-6 | `grep -n "must\|mandatory\|required" prompts/00b-solution-discovery.md \| grep -i "config"` → 0 hits | PASS |
| AC-7 | `git diff HEAD -- dba-system.md CLAUDE.md prompts/00-session-start.md prompts/01-intent.md … scripts/ \| wc -l` → 0; `git status --short …` → 0 | PASS |

### Reviewer scope triage

| Finding | Step | Round | Triage | Disposition |
|---|---|---|---|---|
| AC-2 said "twelve fields" but listed eleven — contradictory | Step 2 | R1 | IN-SCOPE BLOCKER | Fixed: removed "twelve", consistently says eleven |
| AC-1 verification grep could false-pass from non-section hit | Step 2 | R1 | IN-SCOPE BLOCKER | Fixed: required numbered heading pattern |
| AC-5 verification couldn't prove banner preservation or no-conflict | Step 2 | R1 | IN-SCOPE BLOCKER | Fixed: split into grep-c=1 and head-10 banner check |
| AC-3 full-file grep could false-pass without framing in section | Step 2 | R2 | IN-SCOPE BLOCKER | Fixed: anchored with grep -A 5 on section heading |
| AC-5 claimed "preserved verbatim" and "no conflict" — unverifiable | Step 2 | R2 | IN-SCOPE BLOCKER | Fixed: dropped overclaims; non-modification deferred to diff review |
| AC-5 false scope claim — said AC-7 covers existing 00b sections | Step 2 | R3 | IN-SCOPE BLOCKER | Fixed: removed false rationale; used honest "human diff review" |
| AC-1 body text still said `##` section despite grep fix | Step 3 | R1 | IN-SCOPE BLOCKER | Fixed: body text updated to `###` |
| AC-3 grep still used `^## [0-9]` — would false-fail on `### 5.` | Step 3 | R1 | IN-SCOPE BLOCKER | Fixed: grep pattern updated to `^### [0-9]` |
| AC-5 body said `##` heading — contradicted `###` implementation | Step 3 | R1 | IN-SCOPE BLOCKER | Fixed: body text updated to `###` |

Human review of implementation diff — existing `00b` sections unmodified: section 5 heading retained; only body content expanded. No other 00b sections changed.
