---
change_id: CHG-20260630-004
feature_id: UPG-0005
slug: current-verified-state
triage_class: prompt + template
scope_axis: self-dev only
review_profile: PROFILE-3
review_series: RVS__UPG-0005__CHG-20260630-004__S4
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0005 / CHG-20260630-004 — Current Verified State Block

## TRACE HEADER

```yaml
change_id: CHG-20260630-004
feature_id: UPG-0005
slug: current-verified-state
triage_class: prompt + template
scope_axis: self-dev only
review_profile: PROFILE-3
corrects: ~
corrected_by: ~
follow_up_of: ~
```

---

## Step 1 — Change Intent

### Problem

Long-running Codeos work can resume from stale chat context: wrong branch, outdated artifact
content cached in memory, dirty working tree with uncommitted partial changes, or a working
tree that is clean but points to a different commit than the one under review. The current
`prompts/00-session-start.md` asks Claude to read doctrine, identify the session type, and
check the feature registry — but does not ask it to verify the repository's current state
from first principles. Claude can report the wrong branch, the wrong HEAD, and never notice.

### What changes

| File | Change |
|---|---|
| `prompts/00-session-start.md` | Add a new Step 3 (CVS generation) between the codebase-digest step and the session-type step; shift former Steps 3–6 to Steps 4–7. |
| `templates/project-CLAUDE.md` | Add a one-line note to the At Session Start instructions: the session-start prompt includes a state-verification step. |
| `backlog/UPG-0005-current-verified-state.md` | Feature Thread table: add this change. |
| `status/self-development.md` | Activate row for this change. |
| `status/roadmap.md` | Wave 1 UPG-0005 row: update planned change to CHG-20260630-004 and state to IN_PROGRESS. |
| `backlog/features.md` | Added "Future candidates" section with review-runner diagnostics backlog note. |
| `changes/UPG-0005__CHG-20260630-004__current-verified-state.md` | This change record. |

### What stays the same (scope boundary)

- `dba-system.md` — NOT in scope. The CVS block is a session-initialization behavior, not
  DBA architectural doctrine (not about stages, event schemas, or non-negotiable rules). The
  brief says "add the rule to CLAUDE.md" — in downstream projects, the CLAUDE.md is generated
  from `templates/project-CLAUDE.md`. There is no reason to embed this rule in the core
  doctrine.
- Stage prompts 01–09 — NOT in scope for this change. The brief mentions triggering CVS
  before Stage 4, 7, and 9 transitions. That is deferred: it would require modifying each
  stage prompt file (out of scope for a single focused change) and is better done once the
  session-start CVS step is proven.
- `scripts/`, `docs/`, `patterns/` — no changes.

### Triage class: `prompt` + `template`

The change modifies a prompt (session-start step flow) and a template (project-CLAUDE.md).
It does NOT modify `dba-system.md`. Triage class is `prompt + template`, which uses the
4-step loop with PROFILE-3 (Codex review before each step gate).

### Scope axis: `self-dev only`

We are changing toolkit artifacts (`prompts/`, `templates/`). We are not modifying the
downstream DBA doctrine (`dba-system.md`). Scope axis is `self-dev only`.

### Review profile: PROFILE-3

Prompt + template change. PROFILE-3: Codex review before each step gate. Human approval at
all four step gates; reviewer output is advisory and non-gatekeeping.

### Originating backlog item

`backlog/UPG-0005-current-verified-state.md` — Current Verified State Block.

---

## Step 2 — Acceptance Criteria

**A1 — CVS step fires at every downstream DBA session start**
After the change, `prompts/00-session-start.md` contains a numbered step instructing Claude to
run the CVS git commands (`git branch --show-current`, `git rev-parse --short HEAD`,
`git status --short`) and report the results before proceeding to session-type determination.

**A2 — All CVS fields are generated from repo state, not filled manually**
The CVS step specifies shell commands to run; it does not ask Claude to fill in a static
template from memory. The commands are the canonical source (git, filesystem), not chat state.

**A3 — Registry/filesystem disagreement for active work triggers a stop**
If `features/registry.yaml` exists, the CVS step instructs Claude to compare registry entries
for each feature whose status is not `COMPLETE` against the expected artifact presence in the
relevant artifact directories (stage ≥ 1 → `intents/<feature_id>*.md` exists; stage ≥ 2 →
`contracts/<feature_id>*.md` exists; stage ≥ 3 → `events/<feature_id>*.md` exists). If the
registry-reported stage/status contradicts filesystem evidence, Claude must surface the
disagreement and stop for human clarification rather than silently resolving it.

**A4 — Former Steps 3–6 are renumbered correctly with operational meaning unchanged**
Existing Steps 3 (session type), 4 (feature registry), 5 (session context), and 6 (final
confirmation) become Steps 4–7. Operational meaning is unchanged; only renumbering and
cross-reference edits (e.g. "see Step 4" → "see Step 5") are permitted.

**A5 — `templates/project-CLAUDE.md` "At Session Start" instructions reflect the new step**
The template's At Session Start list gains a note that `prompts/00-session-start.md` Step 3
includes repository state verification, so new project CLAUDE.md files communicate this to users.

**A6 — Scope boundary held: `dba-system.md` and stage prompts 01–09 are untouched**
`git diff --exit-code -- dba-system.md 'prompts/0[1-9]-*.md'` exits 0 (no changes to those
files).

**A7 — Step 4 verifies that declared artifacts pass precheck**
During Step 4 Reconcile, the review packet command will be run in full print mode over the
declared UPG-0005 artifacts and must exit 0 without `--skip-prechecks`.

**A8 — No files outside the declared set are touched**
The complete set of allowed touched files is exactly:
- `changes/UPG-0005__CHG-20260630-004__current-verified-state.md`
- `prompts/00-session-start.md`
- `templates/project-CLAUDE.md`
- `backlog/UPG-0005-current-verified-state.md`
- `backlog/features.md`
- `status/self-development.md`
- `status/roadmap.md`

No other file is modified.

---

## Step 3 — Implementation

### Changes made

**`prompts/00-session-start.md`**
- Inserted new Step 3 (CVS generation) between the codebase-digest step (Step 2b) and the
  session-type step.
- Step 3 has three sub-steps:
  - 3a: run `git branch --show-current`, `git rev-parse --short HEAD`, `git status --short`
    and report branch, commit, and working-tree cleanliness.
  - 3b: if `features/registry.yaml` exists, compare each in-flight feature's reported stage
    against artifact file presence in `intents/`, `contracts/`, `events/`; stop if mismatch.
    If registry absent, report the Active Features table from `CLAUDE.md` as-is.
  - 3c: `ls intents/ contracts/ events/ tests/` and report each directory's contents.
  - Closes with a required confirmation line: `CURRENT STATE VERIFIED — …`
- Former Steps 3–6 (session type, feature registry, session context, final confirmation)
  renumbered to Steps 4–7. Operational content unchanged; only step numbers and the
  "from Step 3" cross-reference in the Session Context block updated to "from Step 4".

**`templates/project-CLAUDE.md`**
- At Session Start item 2: added parenthetical `(Step 3 generates the Current Verified
  State: branch, commit, working tree, and registry/filesystem agreement)`.

---

## Step 4 — Reconcile

### Acceptance criteria verification

| AC | Result | Evidence |
|---|---|---|
| A1 — CVS step fires at session start | PASS | `prompts/00-session-start.md` Step 3 contains `git branch --show-current`, `git rev-parse --short HEAD`, `git status --short` and the required confirmation line. |
| A2 — CVS fields generated from repo state | PASS | Step 3 specifies shell commands only; no static fill-in template. |
| A3 — Registry/filesystem disagreement for active work triggers a stop | PASS | Prompt Step 3b: "For each feature whose status is not COMPLETE" — checks intents/contracts/events directories; instructs STOP on mismatch. |
| A4 — Former Steps 3–6 renumbered, operational meaning unchanged | PASS | Steps 3→4 (session type), 4→5 (feature registry), 5→6 (session context), 6→7 (final confirm). "from Step 3" cross-reference updated to "from Step 4". No content changes to those steps. |
| A5 — `templates/project-CLAUDE.md` reflects new step | PASS | At Session Start item 2 carries parenthetical: "Step 3 generates the Current Verified State: branch, commit, working tree, and registry/filesystem agreement". |
| A6 — `dba-system.md` and stage prompts 01–09 untouched | PASS | `git diff --exit-code -- dba-system.md 'prompts/0[1-9]-*.md'` → exit 0; unstaged check confirms none. |
| A7 — Step 4 precheck command exits 0 | PASS | `bash scripts/codeos-review.sh review UPG-0005__CHG-20260630-004 selfdev-step-4 --mode full --print-packet <artifacts>` → exit 0 (no bare placeholder tokens). |
| A8 — No files outside declared set touched | PASS | Modified tracked files: `backlog/UPG-0005-current-verified-state.md`, `backlog/features.md`, `prompts/00-session-start.md`, `status/roadmap.md`, `status/self-development.md`, `templates/project-CLAUDE.md`. Untracked new file: `changes/UPG-0005__CHG-20260630-004__current-verified-state.md`. `reviews/review-log.md` also modified as expected operational bookkeeping by the review pipeline — not a scope violation. All within declared set. |

### Reference sweep

- No stale references to old step numbers in `prompts/00-session-start.md` (grepped for "Step 3" in non-CVS context — only the new Step 3 header and the `3a/3b/3c` sub-steps appear).
- No `dba-system.md` or `prompts/0[1-9]-*.md` changes.
- `backlog/UPG-0005-current-verified-state.md` Feature Thread updated with CHG-20260630-004.
- `status/self-development.md` row reflects `3-Implement`.
- `status/roadmap.md` Wave 1 UPG-0005 row shows `CHG-20260630-004 / IN_PROGRESS`.

### Review round summary

| Round | Stage | Verdict | Disposition |
|---|---|---|---|
| R1 | selfdev-step-2 | CHANGES ADVISED | B1 A3-narrowed, B2 A7-reframed, B3 frontmatter fixed |
| R1 | selfdev-step-3 | CHANGES ADVISED | Same three blockers; fixes applied |
| R2 | selfdev-step-3 | NO OBJECTION | Approved by human |
