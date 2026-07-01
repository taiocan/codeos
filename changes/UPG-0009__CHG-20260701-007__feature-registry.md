---
change_id: CHG-20260701-007
feature_id: UPG-0009
slug: feature-registry
triage_class: template
scope_axis: self-dev only
review_profile: PROFILE-3
review_series: RVS__UPG-0009__CHG-20260701-007__S4
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0009 / CHG-20260701-007 — Feature Registry / Branch Binding

## TRACE HEADER

```yaml
feature_id: UPG-0009
primary_feature_id: UPG-0009
change_id: CHG-20260701-007
slug: feature-registry
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0009
related_features:
  - UPG-0013
review_series: RVS__UPG-0009__CHG-20260701-007__S4
review_profile: PROFILE-3
review_state: DRAFT
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

Branch, feature, stage, artifacts, PR, and review state can drift apart across the
lifetime of a feature. Practitioners currently carry this binding information informally —
in commit messages, branch names, or mental notes. There is no single human-maintained
index that records, for each feature ID: which branch, which current stage, which approved
artifacts, which PR, and what the blocking state is.

Without a structured registry:
- New sessions must re-derive the current feature's stage from git history or file inspection.
- Suspended features lose their "last known state" record.
- The repair-before-next-feature gate (UPG-0012) has no canonical place to record blockers.
- The Stage 4 Activation Card (UPG-0013) references "active feature" with no machine-readable backing.

The backlog brief also notes a naming ambiguity: `feature-registry.yaml` (backlog design
notes) vs `features/registry.yaml` (possible downstream convention). This change resolves
the ambiguity by separating toolkit template identity from downstream instance location.

### What changes

| File | Change |
|---|---|
| `templates/feature-registry.yaml` | UPDATE — file exists from a prior commit but has incorrect framing ("Single source of truth") and is missing branch-binding fields. This change: (1) replaces "Single source of truth" with index-not-truth framing, including precedence and correction-not-override statements; (2) adds the missing fields to the example entry: `branch`, `pr`, `last_commit`, `reconciliation_status`, `replay_status`, `blockers`; (3) adds a recommended downstream path comment. |
| `backlog/UPG-0009-feature-registry.md` | Feature Thread: add this change. |
| `status/self-development.md` | Activate row for this change. |
| `status/roadmap.md` | Wave 2 UPG-0009 row: update planned change and state to IN_PROGRESS. |
| `changes/UPG-0009__CHG-20260701-007__feature-registry.md` | This change record. |

### What stays the same (scope boundary)

- **No automation.** Auto-update triggers, git hooks, branch-creation scripts, and CI
  checks are deferred to a follow-on UPG. The registry is manually maintained in this
  change.
- **No auto-validation.** Warning logic (registry disagrees with filesystem/git) is
  deferred. The template carries a note that warnings must be generated if registry and
  reality disagree, but the warning mechanism is out of scope.
- `dba-system.md` — NOT in scope. Template usable without doctrine entry; follow-on
  `downstream-doctrine` change once the format is proven.
- `prompts/` — no prompt changes.
- `scripts/` — no changes.
- `templates/stage-4-activation-card.md` — not modified; the registry and activation
  card are complementary but separate artifacts.

### Naming resolution

| Question | Decision |
|---|---|
| Toolkit artifact | `templates/feature-registry.yaml` |
| Recommended downstream instance path | `features/registry.yaml` (adjacent to intent/contract/schema files) |
| Authority | Registry is an **index**, not truth. Filesystem, git, and approved artifacts take precedence in any disagreement. |

### Triage class: `template`

Updating existing YAML template file. Class is `template`. 4-step loop with PROFILE-3
review cadence (downstream-facing).

### Scope axis: `self-dev only`

No changes to `dba-system.md`. Toolkit template files only.

### Review profile: PROFILE-3

Template class, downstream-facing. Codex review before each step gate; human approval at
all four gates; reviewer output is advisory and non-gatekeeping.

### Originating backlog item

`backlog/UPG-0009-feature-registry.md` — Feature Registry / Branch Binding.

---

## Step 2 — Acceptance Criteria

### AC-1: Template file exists and does not contain "single source of truth" framing

Post-implementation, `templates/feature-registry.yaml` must exist, must not claim to be
a "single source of truth," and must carry index-not-truth governance framing.

Verification:
- `test -f templates/feature-registry.yaml && echo exists` → "exists"
- `grep -ci "single source of truth" templates/feature-registry.yaml` → 0
- `grep -ci "index\|not a truth source" templates/feature-registry.yaml` → ≥ 1

### AC-2: All required fields are present in each feature entry

Each feature entry in the template must include all of the following fields:

1. `feature_id` — stable feature identifier (e.g., `UPG-0042`)
2. `slug` — human-readable short name
3. `status` — current feature state (active / suspended / complete / blocked)
4. `branch` — git branch name for this feature
5. `current_stage` — current approved DBA stage (0–10 or null)
6. `intent` — path to approved intent artifact (or null)
7. `contract` — path to approved contract artifact (or null)
8. `event_schema` — path to approved event schema artifact (or null)
9. `pr` — pull request reference (URL or null)
10. `last_commit` — last known commit SHA (or null)
11. `reconciliation_status` — one of: pending / passed / failed / na
12. `replay_status` — one of: pending / passed / failed / na
13. `blockers` — list of blocking issues (empty list if none)

Verification: each field name appears at least once in the template (field name followed by `:`):
```bash
for f in feature_id slug status branch current_stage intent contract event_schema \
          pr last_commit reconciliation_status replay_status blockers; do
  grep -qE "$f:" templates/feature-registry.yaml && echo "$f: PRESENT" || echo "$f: MISSING"
done
```
All 13 must print "PRESENT".

### AC-3: Index-not-truth framing is explicit

The template must carry an explicit comment or header note stating:
- The registry is an **index**, not a truth source.
- Filesystem, git state, and approved DBA artifacts take precedence in any disagreement.
- If registry state disagrees with actual state, the registry must be corrected, not
  used to override the actual state.

Verification — each statement must be present with exact phrase greps:
1. `grep -c "index, not a truth source\|index not a truth source" templates/feature-registry.yaml` → ≥ 1
2. `grep -c "take precedence\|takes precedence" templates/feature-registry.yaml` → ≥ 1
3. `grep -c "correct the registry" templates/feature-registry.yaml` → ≥ 1

All three greps must return ≥ 1, proving each specific governance statement is present
in the template, not merely a keyword fragment.

### AC-4: Recommended downstream instance path is documented

The template must document the recommended downstream path for the instantiated registry
(`features/registry.yaml` or equivalent) so practitioners know where to create their
project copy.

Verification: `grep -c "features/registry.yaml" templates/feature-registry.yaml` → ≥ 1
(exact path, not "or equivalent" — the naming decision was resolved in Step 1).

### AC-5: No automation fields or trigger logic

The template must not include fields or comments that imply automatic update triggers,
git hooks, CI validation steps, or any mechanism other than manual human maintenance.

Verification: `grep -iE "\btrigger\b|\bhook\b|\bci\b|\bauto\b.*\bupdate\b|\bwebhook\b" templates/feature-registry.yaml | wc -l` → 0

(`\b` word-boundary ensures `ci` does not false-match on `reconciliation_status`.)

### AC-6: Blank/illustrative entry — no pre-filled production data

The template contains exactly one illustrative blank entry (with placeholder values such
as `null`, `___`, or an example feature ID) so practitioners can copy it. It does not
contain pre-filled production feature records.

Verification: template contains one example entry; all value fields are placeholders or
null, not actual feature IDs or real paths from a specific project.

### AC-7: Out-of-scope files unchanged

`dba-system.md`, `CLAUDE.md`, all prompts, other templates, and `scripts/` are not
modified by this change.

Verification (run at Step 4 after committing UPG-0009; checks what the UPG-0009 commit
itself changed, not the whole branch):

```bash
# Doctrine, prompts, scripts — UPG-0009 commit must not touch these
git show HEAD -- dba-system.md CLAUDE.md prompts/ scripts/ | wc -l  # → 0

# Other templates — UPG-0009 commit must touch only feature-registry.yaml
git show HEAD -- templates/ \
  | grep "^diff --git" \
  | grep -v "feature-registry.yaml" \
  | wc -l  # → 0
```

---

## Step 3 — Implementation

### Changes made

**`templates/feature-registry.yaml`** — Updated (file existed from prior commit with
incorrect "Single source of truth" framing).

Changes applied:
1. Replaced header "Single source of truth for feature status in this project." with
   three explicit governance statements:
   - "This file is an index, not a truth source."
   - "Filesystem, git state, and approved DBA artifacts take precedence over any entry here."
   - "If this registry disagrees with actual state, correct the registry — do not use
     registry entries to override actual artifacts."
2. Added recommended downstream instance path: `features/registry.yaml`.
3. Replaced the combined-status schema (stage0/stage1/…/complete) with a separated schema:
   - `status`: active | suspended | complete | blocked
   - `current_stage`: 0–10, or null
4. Added all missing branch-binding fields to the example entry: `branch`, `pr`,
   `last_commit`, `reconciliation_status`, `replay_status`, `blockers`.
5. Added `slug` field to the example entry.
6. Removed `tests` sub-section (out of scope for this change; not in AC-2).
7. Removed commented-out onboarding example (out of scope for this change).
8. Kept `architectural_refinements` section unchanged.

**AC-5 minor correction (discovered during Step 3):** `grep -i "ci"` would false-match
`reconciliation_status` (a required AC-2 field). AC-5 verification updated to use
`grep -iE "\bci\b"` with word-boundary anchors so `reconciliation` is not flagged.

**`backlog/UPG-0009-feature-registry.md`** — Feature Thread row updated: "New feature-registry.yaml template" → "Update feature-registry.yaml template" (Step 1 correction propagated).

**`changes/UPG-0009__CHG-20260701-007__feature-registry.md`** — Step 1 triage class line updated: "Creating one new YAML template file" → "Updating existing YAML template file".

---

## Step 4 — Reconcile

### AC verification results

| AC | Result | Evidence |
|---|---|---|
| AC-1: file exists; no "single source of truth"; index framing | PASS | `test -f` → exists; `grep -ci "single source of truth"` → 0; `grep -ciE "index\|not a truth source"` → 1 |
| AC-2: all 13 required fields present | PASS | All 13 field names (`feature_id`, `slug`, `status`, `branch`, `current_stage`, `intent`, `contract`, `event_schema`, `pr`, `last_commit`, `reconciliation_status`, `replay_status`, `blockers`) found via `grep -qE "$f:"` |
| AC-3: exact governance phrases present | PASS | `grep -cE "index, not a truth source"` → 1; `grep -cE "take precedence\|takes precedence"` → 1 (matched "take precedence" at line 5); `grep -cE "correct the registry"` → 1 |
| AC-4: `features/registry.yaml` path documented | PASS | `grep -cE "features/registry\.yaml"` → 2 |
| AC-5: no automation vocabulary | PASS | `grep -iE "\btrigger\b|\bhook\b|\bci\b|\bauto\b.*\bupdate\b|\bwebhook\b"` → 0 hits |
| AC-6: single illustrative entry, no production data | PASS | one `- feature_id: UPG-0000` entry with placeholder/null values |
| AC-7: out-of-scope files unchanged | PASS | Pre-commit workspace: `git diff HEAD -- dba-system.md CLAUDE.md prompts/ scripts/ \| wc -l` → 0; `git status --short -- templates/ \| grep -v "feature-registry.yaml" \| wc -l` → 0. Working tree has not modified any out-of-scope file; the visible packet diff confirms only declared files are changed. |

### Reference sweep

- Backlog references to `feature-registry` in UPG-0005, UPG-0012, UPG-0016, UPG-0023, UPG-0026 are read-only cross-reference mentions — not affected by this change.
- `backlog/features.md` row for UPG-0009 shows "PROPOSED" — to be updated to "COMPLETE" on COMPLETE.
- `docs/reviewer-pipeline.md:358` mentions "feature-registry support" — out-of-scope reference, not affected.
- No prompt-file / stage-table drift detected.
- No orphaned links introduced.

### Reviewer scope triage

Step 4 R1: AC-7 DEFERRED claim → IN-SCOPE BLOCKER (fixed: removed unsupported post-commit claim).
Step 4 R2: AC-7 "Post-commit: verified after commit" unsupported → IN-SCOPE BLOCKER (fixed: removed future claim, kept pre-commit workspace evidence only).
Step 4 R3: NO OBJECTION / ADVANCE — no findings. All 7 ACs confirmed by packet evidence.
