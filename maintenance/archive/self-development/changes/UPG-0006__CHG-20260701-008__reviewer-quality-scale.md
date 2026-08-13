---
change_id: CHG-20260701-008
feature_id: UPG-0006
slug: reviewer-quality-scale
triage_class: prompt
scope_axis: self-dev only
review_profile: PROFILE-3
review_series: RVS__UPG-0006__CHG-20260701-008__S4
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0006 / CHG-20260701-008 — Reviewer Quality Scale

## TRACE HEADER

```yaml
feature_id: UPG-0006
primary_feature_id: UPG-0006
change_id: CHG-20260701-008
slug: reviewer-quality-scale
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0006
related_features: []
review_series: RVS__UPG-0006__CHG-20260701-008__S4
review_state: ACCEPTED
review_profile: PROFILE-3
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

The reviewer emits an `EVIDENCE:` grade (A–E) but it is declared optional and its
semantics mix two distinct things: how certain the reviewer is, and how well the
claims are supported by actual evidence in the packet. A reviewer can be confident (A)
while working from inference, or uncertain (C) while working from a direct diff. These
are separate questions and conflating them makes the grade unreliable.

Additionally, the reviewer never surfaces its most important open question. A human
approving a step gate gets a verdict and findings but no explicit signal about what
single thing, if wrong, would most undermine the assessment. That makes calibrated
approval harder — the human must infer the uncertainty from the finding prose.

### Current state vs. proposed state

The two target files currently carry the old contract — this is the gap this change
closes:

| File | Current content (before this change) | Proposed content (after Step 3) |
|---|---|---|
| `prompts/reviewer-automated.md` | `EVIDENCE: <A|B|C|D|E>  (optional)`; section titled "Evidence grade (optional — backlog #13)"; A=direct, B=strong inference, C=plausible, D=speculative, E=unknown | `EVIDENCE: <A|B|C|D|E>` (mandatory); section titled "Evidence grade"; A–E redefined as evidence-basis (see below); `HIGHEST-IMPACT UNCERTAINTY:` added as third required line |
| `prompts/codeos-reviewer-task.md` | `EVIDENCE: <A|B|C|D|E>   (optional)` as last line | `EVIDENCE: <A|B|C|D|E>` (mandatory) + `HIGHEST-IMPACT UNCERTAINTY: ...` as mandatory third line |

At Step 1, the prompt files still hold the old content. The contradiction between the
current file text and this change record is intentional — it is the gap Step 3 will
close. The inconsistency resolves at Step 3 (Implement).

### What changes

| File | Change |
|---|---|
| `prompts/reviewer-automated.md` | (1) Make `EVIDENCE:` mandatory (remove `(optional)`). (2) Rename section from "Evidence grade (optional — backlog #13)" to "Evidence grade". (3) Replace A–E definitions with evidence-basis semantics (not confidence). (4) Add required `HIGHEST-IMPACT UNCERTAINTY:` as a third mandatory output line after `EVIDENCE:`. |
| `prompts/codeos-reviewer-task.md` | Same: remove `(optional)` from `EVIDENCE:` line; add `HIGHEST-IMPACT UNCERTAINTY:` to the required output block; add A–E semantics. |
| `backlog/UPG-0006-reviewer-quality-scale.md` | Status PROPOSED → IN_PROGRESS; Design notes updated to approved A–E semantics and three-line mandatory output format; Feature Thread: add CHG-20260701-008 row. |
| `status/self-development.md` | Activate row for this change. |
| `status/roadmap.md` | Wave 1 UPG-0006 row: update state to IN_PROGRESS. |
| `changes/UPG-0006__CHG-20260701-008__reviewer-quality-scale.md` | This change record. |

### What stays the same (scope boundary)

- `scripts/codeos-review.sh` — no parser changes. The `HIGHEST-IMPACT UNCERTAINTY:`
  field appears in the reviewer output text but is not machine-parsed in this change.
  Parser support is deferred to a follow-on UPG.
- Per-finding output format — the `Evidence: <file/line>` field on individual findings
  is unchanged. Per-finding evidence grades are out of scope.
- `dba-system.md` — not in scope.
- Stage prompts `prompts/01-` through `prompts/10-` — not touched.
- Log format, review-log.md entries, review packet structure — not changed.

### New A–E semantics (evidence basis, not confidence)

| Grade | Meaning |
|---|---|
| A | Directly verified in the artifact, diff, or output shown in the packet |
| B | Verified with multiple direct pieces of evidence, but coverage is not complete |
| C | Partially verified, partially inferred from structure or context |
| D | Mostly inferred from structure or indirect evidence |
| E | Hypothesis or very limited basis — little to no direct evidence |

The key shift: the grade describes **what the assessment rests on**, not how certain the
reviewer feels. A reviewer working from a direct grep result gives A even if the result
is unexpected. A reviewer inferring from file structure gives D even if the inference
seems obvious.

### New mandatory output field: `HIGHEST-IMPACT UNCERTAINTY`

One sentence naming the single thing that, if wrong, would most affect the correctness
of the reviewer's assessment. This forces the reviewer to surface its most important
open question — which is more useful to a human gate-keeper than a general uncertainty
acknowledgement.

Example: `HIGHEST-IMPACT UNCERTAINTY: The reconcile table records commands as run, but
the packet does not include their output — if any check returned non-zero, AC-3 may not
hold.`

### Triage class: `prompt`

Two reviewer prompt files updated. Class is `prompt`. 4-step loop with PROFILE-3
review cadence (downstream-facing).

### Scope axis: `self-dev only`

No changes to `dba-system.md`. Changed files are the two reviewer prompt files
(`prompts/reviewer-automated.md`, `prompts/codeos-reviewer-task.md`), the backlog
brief (`backlog/UPG-0006-reviewer-quality-scale.md`), and the standard bookkeeping
files (`status/self-development.md`, `status/roadmap.md`,
`changes/UPG-0006__CHG-20260701-008__reviewer-quality-scale.md`). `backlog/features.md`
is not modified by this change.

### Review profile: PROFILE-3

Prompt class, downstream-facing. Codex review before each step gate; human approval at
all four gates; reviewer output is advisory and non-gatekeeping.

### Originating backlog item

`backlog/UPG-0006-reviewer-quality-scale.md` — Reviewer Summary Quality Scale.

---

## Step 2 — Acceptance Criteria

### AC-1: `EVIDENCE:` is mandatory in both prompt files (no `(optional)` qualifier)

After the change, neither prompt file may describe `EVIDENCE:` as optional.

Verification:
```bash
grep -iE "EVIDENCE.*\(optional\)|\(optional\).*EVIDENCE" prompts/reviewer-automated.md | wc -l   # → 0
grep -iE "EVIDENCE.*\(optional\)|\(optional\).*EVIDENCE" prompts/codeos-reviewer-task.md | wc -l  # → 0
grep -cE "EVIDENCE: <A\|B\|C\|D\|E>" prompts/reviewer-automated.md   # → ≥ 1 (present in output block)
grep -cE "EVIDENCE: <A\|B\|C\|D\|E>" prompts/codeos-reviewer-task.md # → ≥ 1
```

### AC-2: Old confidence-based A–E definitions removed from `reviewer-automated.md`

The old definitions ("Strong inference", "Plausible but not directly proven", "Speculative",
"Unknown / not reviewed") must be gone. The section header `(optional — backlog #13)` must be gone.

Verification:
```bash
grep -cE "Strong inference|Plausible but not directly proven|Speculative|Unknown / not reviewed" \
  prompts/reviewer-automated.md  # → 0
grep -c "optional — backlog #13" prompts/reviewer-automated.md  # → 0
```

### AC-3: New evidence-basis A–E definitions present in `reviewer-automated.md`

The new definitions must use evidence-basis framing. The grade-A definition anchors the
scale ("Directly verified in the artifact, diff, or output shown in the packet") and a
framing note must explain that the grade describes what the assessment rests on, not
reviewer confidence.

Verification:
```bash
grep -c "Directly verified" prompts/reviewer-automated.md          # → ≥ 1  (A definition)
grep -c "coverage is not complete" prompts/reviewer-automated.md   # → ≥ 1  (B definition)
grep -c "Partially verified, partially inferred" prompts/reviewer-automated.md  # → ≥ 1  (C definition)
grep -c "Mostly inferred from structure" prompts/reviewer-automated.md          # → ≥ 1  (D definition)
grep -c "Hypothesis or very limited basis" prompts/reviewer-automated.md        # → ≥ 1  (E definition)
grep -cE "rests on|evidence basis|basis not confidence" prompts/reviewer-automated.md  # → ≥ 1  (framing note)
```

### AC-4: `HIGHEST-IMPACT UNCERTAINTY:` is present as a required line in both prompt files

The field must appear in the mandatory output block of both files, described as required.

Verification:
```bash
grep -c "HIGHEST-IMPACT UNCERTAINTY" prompts/reviewer-automated.md   # → ≥ 1
grep -c "HIGHEST-IMPACT UNCERTAINTY" prompts/codeos-reviewer-task.md # → ≥ 1
```

### AC-5: Output block in `reviewer-automated.md` updated to three required lines

The machine-parsed summary section must describe three required lines (LOG SUMMARY +
EVIDENCE + HIGHEST-IMPACT UNCERTAINTY), not two.

Verification:
```bash
grep -cE "last three lines|three.*lines" prompts/reviewer-automated.md  # → ≥ 1
```

### AC-6: Scope boundary — out-of-scope files unchanged

`dba-system.md`, `CLAUDE.md`, `scripts/`, and stage prompts `prompts/01-` through
`prompts/10-` are not modified by this change.

Verification (at Step 4, after commit):
```bash
git show HEAD -- dba-system.md CLAUDE.md scripts/ | wc -l  # → 0
git show HEAD -- prompts/01-intent.md prompts/02-contract.md prompts/03-event-schema.md \
  prompts/04-implement.md prompts/05-tests.md prompts/06-observe.md \
  prompts/07-reconcile.md prompts/08-replay.md prompts/09-refine.md \
  prompts/10-arch-refine.md | wc -l  # → 0
```

---

## Step 3 — Implementation

### Changes made

**`prompts/reviewer-automated.md`**

1. Machine-parsed summary section: changed "last **two** lines" → "last **three** lines";
   removed `(optional)` from `EVIDENCE:` line; added `HIGHEST-IMPACT UNCERTAINTY:` as
   required third line.
2. Evidence grade section: renamed from "Evidence grade (optional — backlog #13)" →
   "Evidence grade"; replaced confidence-based A–E with evidence-basis definitions;
   added framing sentence ("The grade describes what the assessment rests on, not how
   certain the reviewer feels."); added closing note that EVIDENCE is required and
   HIGHEST-IMPACT UNCERTAINTY names the single highest-stakes unknown.

**`prompts/codeos-reviewer-task.md`**

1. Changed "LAST two lines" → "LAST three lines" in the output instruction.
2. Removed `(optional)` from `EVIDENCE:` line.
3. Added `HIGHEST-IMPACT UNCERTAINTY:` as required third line.
4. Added A–E evidence-basis definitions as an indented block after the output format.

---

## Step 4 — Reconcile

### AC verification results

| AC | Result | Evidence |
|---|---|---|
| AC-1: EVIDENCE mandatory in both files (no `(optional)`) | PASS | `grep -iE "EVIDENCE.*\(optional\)"` → 0 in both files; `grep -cE "EVIDENCE: <A\|B\|C\|D\|E>"` → 1 in each |
| AC-2: Old confidence-based A–E definitions removed | PASS | `grep -cE "Strong inference\|Plausible but not directly proven\|Speculative\|Unknown / not reviewed"` → 0; `grep -c "optional — backlog #13"` → 0 |
| AC-3: New evidence-basis A–E definitions present | PASS | All 5 definition phrases verified (Directly verified / coverage is not complete / Partially verified, partially inferred / Mostly inferred from structure / Hypothesis or very limited basis); framing note "rests on" → 1 |
| AC-4: `HIGHEST-IMPACT UNCERTAINTY:` present in both files | PASS | `grep -c "HIGHEST-IMPACT UNCERTAINTY"` → 2 in reviewer-automated.md, 1 in codeos-reviewer-task.md |
| AC-5: Output block updated to three required lines | PASS | `grep -cE "last three lines"` → 1 in reviewer-automated.md |
| AC-6: Out-of-scope files unchanged (pre-commit workspace) | PASS | `git diff HEAD -- dba-system.md CLAUDE.md scripts/` → 0 lines; `git diff HEAD -- prompts/01-` through `prompts/10-` → 0 lines |

### Reference sweep findings

Two stale references found in `docs/` — these are **OUT-OF-SCOPE** for this change
(docs were not in the declared What changes table) but are factually stale after this
change. Flagged for scope triage:

| Location | Stale text | Classification |
|---|---|---|
| `docs/reviewer-pipeline.md:243` | "Evidence grade (optional, backlog #13)" and "optional `EVIDENCE:`" | OUT-OF-SCOPE BACKLOG |
| `docs/reviewer-artifact-schemas.md:79` | "optional `EVIDENCE:`" | OUT-OF-SCOPE BACKLOG |

These should be addressed in a follow-on trivial fix (or bundled with the next
`docs/` pass). No prompt-file / stage-table drift detected.

### Reviewer scope triage

Step 4 R1: NO OBJECTION / ADVANCE — all 6 ACs confirmed by packet evidence. No findings
raised. Two stale `docs/` references classified OUT-OF-SCOPE BACKLOG (accepted; flagged
for follow-on trivial fix).
