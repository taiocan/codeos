# Self-Development Change: UPG-0031__CHG-20260630-003 — precheck-filter-correction

<!--
PURPOSE: Corrective CHG for UPG-0031 / CHG-20260630-002. Fixes an in-scope
verification defect: B8b was falsely recorded as PASS in Step 4 of CHG-20260630-002.
Post-commit verification revealed two script defects and one artifact formatting defect.
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0031
primary_feature_id: UPG-0031
change_id: CHG-20260630-003
slug: precheck-filter-correction
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0031
related_features:
  - UPG-0027
corrects: CHG-20260630-002
review_series: null
review_profile: PROFILE-1
review_state: DRAFT
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: CHG-20260630-002
```

<!-- SELF-REFERENCE BOUNDARY -->

## Change Intent

**Why (problem in the toolkit):**

Post-commit verification of UPG-0031 / CHG-20260630-002 found that B8b
("precheck on the change record itself passes") was falsely recorded as PASS in
the Step 4 reconciliation. Actual result: `exit: 2`.

Three root causes were identified (see CHG-20260630-002 reconciliation note):

1. The precheck's HTML comment stripping (`sed '/<!--/,/-->/d'`) ran BEFORE inline
   code span removal. A code span containing `` `<!-- … -->` `` on line 72 of the change
   record opened a sed deletion range that silently swallowed lines 73–113, hiding the
   scope-boundary prose (including line 100) from the grep. The original B8b "PASS" was
   an artifact of this accidental deletion.

2. After correcting the filter ordering, line 100 of the change record
   (`produced false positives (UPG-#### in blockquotes…)`) contained a bare placeholder
   token without a code span, which would correctly trigger a false-positive.

3. The B3c smoke test transcript (line 241) was inside a fenced code block and contained
   `'UPG-####'` in single quotes. The backtick-span filter does not handle fenced code
   blocks, so the token survived to the grep.

**What changes:**

1. `scripts/codeos-review.sh` — fix `run_prechecks`: strip inline code spans (`sed
   's/\`[^`]*\`//g'`) as the FIRST step, before HTML comment block deletion. Both the
   `UPG-####` and `CHG-YYYYMMDD-NNN` checks updated. Comment updated to document the
   correct ordering rationale.

2. `changes/UPG-0031__CHG-20260630-002__review-delta-working-tree.md` — three artifact fixes:
   - Line 100: wrap bare `UPG-####` in backticks.
   - Lines 239–243: convert B3c fenced code transcript to blockquote format (blockquote
     lines are explicitly filtered by the precheck).
   - Lines 270–274: replace false B8b transcript with corrected transcript; add
     post-completion defect-and-correction note documenting all three root causes.

3. `changes/UPG-0031__CHG-20260630-003__precheck-filter-correction.md` — this file.

4. `status/self-development.md` — CHG-20260630-003 row added; CHG-20260630-002 row
   updated to reflect the corrective CHG.

**What stays the same:**

- `dba-system.md`, `prompts/`, `templates/`, `docs/` — not touched.
- Packet format, review log format — unchanged.
- Advisory/read-only/non-gatekeeping guarantees — unchanged.
- Fail-closed behavior for bare prose `UPG-####` / `CHG-YYYYMMDD-NNN` — retained and
  now correctly verified.
- All Fixes A–C from CHG-20260630-002 — untouched.

**Class:** `script-tooling` + `documentation` (corrective)
**Scope axis:** `self-dev only`
**Review profile:** `PROFILE-1` (corrective; acceptance criteria already defined by
the failed B8b and regression test set)
**Corrects:** `CHG-20260630-002`

---

## Acceptance Criteria

| # | Criterion | How it will be verified |
|---|---|---|
| C1 | Precheck on `changes/UPG-0031__CHG-20260630-002__review-delta-working-tree.md` exits 0 | Run `review --mode full --print-packet` on the change record without `--skip-prechecks`; confirm exit 0 |
| C2 | Inline code span containing `<!-- ... -->` does not cause following lines to be silently deleted by the precheck filter | Create a temp file with `` `<!-- test -->` `` on one line and `UPG-####` as bare prose on the NEXT line (not in code span); precheck must exit 2 (proving the range no longer swallows the next line) |
| C3 | Precheck on `status/self-development.md` still exits 0 | Run without `--skip-prechecks`; confirm exit 0 |
| C4 | Real bare `UPG-####` prose (not in a code span) still exits 2 | Run on a file with `feature_id: UPG-####`; confirm exit 2 |
| C5 | `bash -n scripts/codeos-review.sh` exits 0 | Syntax check |

---

## Implementation Notes

**Script fix (1 file, 2 sites):**

`run_prechecks` in `scripts/codeos-review.sh`:
- Changed: `sed '/<!--/,/-->/d' "${a}" | grep … | sed 's/\`[^`]*\`//g; …'`
- To: `sed 's/\`[^`]*\`//g' "${a}" | sed '/<!--/,/-->/d' | grep … | sed 's/…'`
- The `s/\`[^`]*\`//g` substitution is now the first step in the pipeline for both
  the UPG-#### and CHG-YYYYMMDD-NNN checks. The remaining `s/\`[^`]*\`//g` in the
  third sed step is now redundant but harmless.

**Artifact fixes (1 file, 3 hunks):**
- `changes/UPG-0031__CHG-20260630-002__review-delta-working-tree.md` line 100:
  `UPG-####` → `` `UPG-####` ``
- Same file lines 239–243: B3c fenced code block → blockquote lines starting with `>`
- Same file lines 270–274: false B8b transcript replaced; post-completion note added

---

## Reconciliation

All acceptance criteria verified.

| AC | Result | Evidence |
|---|---|---|
| C1 | PASS | See transcript below |
| C2 | PASS | See transcript below |
| C3 | PASS | See transcript below |
| C4 | PASS | See transcript below |
| C5 | PASS | See transcript below |

C1 — precheck on change record exits 0:
> (packet content begins — no error line before it)
> exit: 0

C2 — inline code span with `<!--` does not swallow next line:
> (file: line 1: `` `<!-- test -->` ``, line 2: `UPG-####`)
> error: precheck failed — literal placeholder `'UPG-####'` found (fill in the real UPG id)
> exit: 2
> (confirms the range no longer silently swallows line 2)

C3 — status/self-development.md exits 0:
> (packet content begins — no error line before it)
> exit: 0

C4 — real bare placeholder exits 2:
> error: precheck failed — literal placeholder `'UPG-####'` found in /tmp/real-ph.md (fill in the real UPG id)
> exit: 2

C5 — syntax check:
> exit: 0
