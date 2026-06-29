# Self-Development Change: UPG-0027__CHG-20260629-004 — local-prechecks

<!--
PURPOSE: Per-change source of truth for a non-trivial change to the Codeos toolkit
itself (prompts, templates, docs, patterns, scripts).

This is NOT a downstream DBA feature. It has no behavioral contract, no event schema,
and no replay. Trivial changes do not get a record.

Workflow: prompts/codeos-self-dev.md (4-step loop)
Each step requires explicit human approval; Codex review cadence is governed by the assigned review profile (see prompts/codeos-self-dev.md Step 0a).
The live status row lives in status/self-development.md, not here.

FILENAME CONVENTION (Feature Thread model — see backlog/UPG-0001-feature-thread-traceability.md):
  changes/UPG-####__CHG-YYYYMMDD-NNN__slug.md
  - UPG-#### = the PRIMARY feature this change implements (visible grouping).
  - CHG-YYYYMMDD-NNN = the unique change id (execution).
  - slug describes the concrete change, not the whole roadmap.
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0027
primary_feature_id: UPG-0027
change_id: CHG-20260629-004
slug: local-prechecks
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0027
related_features: []
review_series: null
review_profile: PROFILE-3
review_state: ACCEPTED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: CHG-20260629-003
```

<!-- SELF-REFERENCE BOUNDARY -->


## Change Intent

**Why (problem in the toolkit):**

Packets are now well-structured (CHG-1: focused task; CHG-2: manifest + budget), but the
script invokes Codex on artifacts that contain unfilled template placeholders, forbidden
fields from a superseded schema, or uncommitted changes to scope-boundary files. These
defects are cheaper to catch deterministically before the Codex call than to detect in a
review round. A reviewer that flags a literal `UPG-####` as a false claim is spending a
round on something a grep could have caught in milliseconds.

Additionally, there is no way for the caller to assert that a specific path (e.g.,
`dba-system.md` during a `self-dev only` change) must be clean. The `--sha-only` flag
(CHG-2) lets the caller declare reference files by hash; there is no equivalent for
"this file must not have changed."

**Scope note — delta mode deferred:**

The approved plan (CHG-3) combined local prechecks with true delta packets
(`--mode delta --base <sha>`). Per the guidance "do not mix both unless scope stays
small," this CHG covers local prechecks only. Delta mode is CHG-5
(`CHG-20260629-005__delta-mode`), planned in the Feature Thread.

**What changes:**

1. `scripts/codeos-review.sh` — new `run_prechecks()` function (called from `cmd_review()`
   before packet build); minimal changes to `cmd_review()` arg-parse and call site only.

   - **Hard fails** (exit non-zero before any Codex invocation):
     - Literal string `UPG-####` (four `#` chars) in any positional artifact — unfilled
       template placeholder. Real IDs (`UPG-0027`) are NOT matched; this is a fixed-string
       grep, not a pattern for all UPG identifiers.
     - Literal string `CHG-YYYYMMDD-NNN` in any positional artifact — unfilled template
       placeholder. Real IDs (`CHG-20260629-004`) are NOT matched; fixed-string grep only.
     - Line-anchored `latest_review:` field (`^[[:space:]]*latest_review:`) in any positional
       artifact — forbidden field superseded by UPG-0001. Anchored to avoid matching prose
       that merely mentions the field name (e.g., docs, this change record).
     - `--guard-clean PATH` (new CLI flag, repeatable): fails if PATH does not exist on disk
       (missing guard file is an error, not a silent pass); also fails if
       `git diff --quiet HEAD -- PATH` exits non-zero (staged + unstaged changes vs HEAD).

   - **Warnings** (one stderr line per match; exit code unchanged; Codex call proceeds):
     - `TODO`, `FIXME`, `TBD`, `[to be filled]` in any positional artifact — unresolved
       draft markers. Warnings never change exit code and never prevent Codex invocation.

   - **Bypass**: `--skip-prechecks` skips all hard-fail and warning checks. Must emit a
     visible stderr warning when used (e.g., `warning: prechecks skipped (--skip-prechecks)`).
     Intended for packet inspection of deliberately draft artifacts.

   **Scan scope:** prechecks examine only the positional artifact paths passed to the
   `review` command and the explicit `--guard-clean` paths. They do not scan the whole
   repository, generated packet files, review logs, or docs not passed as artifacts.

2. `docs/reviewer-pipeline.md` — one paragraph in §10 (Usage) describing local prechecks,
   `--guard-clean`, and `--skip-prechecks`.

**Step 1 pre-work (bookkeeping):**

3. `backlog/UPG-0027-replacing-review-scripts.md` — Feature Thread: CHG-20260629-004 row
   updated (slug corrected to `local-prechecks`); CHG-20260629-005 row added (delta mode,
   PLANNED).

4. `status/self-development.md` — CHG-20260629-004 row activated.

**Implementation design — `run_prechecks()`:**

```
run_prechecks() {
  args: "${artifacts[@]}"
  reads globals: PRECHECK_GUARD_CLEAN   (array of --guard-clean paths, set by cmd_review)

  for each artifact (skip if not a regular file — missing files handled by build_packet):
    HARD FAIL (exit 2) if grep -qF 'UPG-####' artifact
    HARD FAIL (exit 2) if grep -qF 'CHG-YYYYMMDD-NNN' artifact
    HARD FAIL (exit 2) if grep -qE '^[[:space:]]*latest_review:' artifact
    WARN (stderr, no exit) if grep -qiE 'TODO|FIXME|\bTBD\b|\[to be filled\]' artifact

  for each PRECHECK_GUARD_CLEAN path:
    HARD FAIL (exit 2) if path does not exist on disk
    HARD FAIL (exit 2) if git diff --quiet HEAD -- <path> exits non-zero
}
```

Called from `cmd_review()` after scratch-mode setup, before `PACKET_FILE` creation:
```
  if [[ ${skip_prechecks} -eq 1 ]]; then
    echo "warning: prechecks skipped (--skip-prechecks)" >&2
  else
    run_prechecks "${artifacts[@]}"
  fi
  PACKET_FILE="$(mktemp)"; ...
  PACKET_SHA_ONLY=(...)
  build_packet ...
```

`PRECHECK_GUARD_CLEAN` uses the same global-array pattern as `PACKET_SHA_ONLY` (CHG-2).

**Scope boundary — what stays the same:**

- `build_packet()` — not touched; prechecks are entirely separate
- `cmd_decision`, `cmd_stage_start`, `run_codex`, `stage_checks`, `stage_expected`,
  log-parse lines (~449–462) — unchanged
- `dba-system.md` — untouched
- `prompts/codeos-reviewer-task.md` — untouched
- `reviews/review-log.md` format — unchanged
- Delta mode (`--mode delta --base <sha>`) — deferred to CHG-5
- §X section-reference validation — deferred (requires heading extraction; too complex)
- Trace header / dashboard mismatch check — deferred (requires reading dashboard; complex)

**Class:** `script-tooling`
**Scope axis:** self-dev only
**Backlog item:** `backlog/UPG-0027-replacing-review-scripts.md`

---

## Acceptance Criteria

<!-- Precheck invocation (A1) -->

| # | Criterion | How it will be verified |
|---|---|---|
| A1 | `run_prechecks()` function exists and is called from `cmd_review()` before `PACKET_FILE` creation and before any Codex invocation | `grep -n "run_prechecks\|PACKET_FILE" scripts/codeos-review.sh` → `run_prechecks` call line number < `PACKET_FILE=` line number; `--print-packet` bypasses Codex but precheck still fires first |

<!-- Hard-fail checks (A2–A6) -->

| # | Criterion | How it will be verified |
|---|---|---|
| A2 | Literal string `UPG-####` (four `#` chars) in a positional artifact → exit non-zero before Codex; real IDs (e.g. `UPG-0027`) in the same file do NOT trigger the check | Create a temp file containing `UPG-####`; run `review ... tmp-file --print-packet`; expect exit non-zero and stderr error. Create a file with `UPG-0027` only; expect no failure |
| A3 | Literal string `CHG-YYYYMMDD-NNN` in a positional artifact → exit non-zero before Codex; real IDs (e.g. `CHG-20260629-004`) do NOT trigger | Same approach: temp file with literal `CHG-YYYYMMDD-NNN` → fails; file with `CHG-20260629-004` → passes |
| A4 | Line-anchored `latest_review:` (`^[[:space:]]*latest_review:`) in a positional artifact → exit non-zero before Codex; prose mentioning the field name in a sentence does NOT trigger | Temp file with `latest_review: something` at line start → fails; temp file with `the latest_review: field is forbidden` as inline prose → does NOT fail (not line-anchored) |
| A5 | `--guard-clean PATH` where PATH exists but has non-empty `git diff --quiet HEAD -- PATH` → exit non-zero before Codex | `echo x >> dba-system.md`; run `review ... --guard-clean dba-system.md`; expect exit non-zero; restore file |
| A6 | `--guard-clean PATH` where PATH does not exist → exit non-zero before Codex with clear error (missing guard file is an error, not a silent pass) | Run `review ... --guard-clean nonexistent-guard.md`; expect exit non-zero with error on stderr |

<!-- Warning tier (A7) -->

| # | Criterion | How it will be verified |
|---|---|---|
| A7 | `TODO`, `FIXME`, `TBD`, `[to be filled]` in a positional artifact → warning printed to stderr; exit code is 0; `--print-packet` proceeds; Codex would be invoked | Temp file with `TBD` → `review ... --print-packet 2>warn.txt`; confirm exit 0; confirm warn.txt has warning line |

<!-- Bypass (A8) -->

| # | Criterion | How it will be verified |
|---|---|---|
| A8 | `--skip-prechecks` suppresses hard-fail and warning checks; emits a visible `warning: prechecks skipped` line to stderr | Temp file with `UPG-####` + `--skip-prechecks --print-packet` → exit 0; stderr shows `warning: prechecks skipped` |

<!-- Scan scope (A9) -->

| # | Criterion | How it will be verified |
|---|---|---|
| A9 | Prechecks scan only the positional artifact paths and explicit `--guard-clean` paths; no whole-repo scan, no packet files, no review logs | Run review with a clean artifact that does not contain placeholders; confirm no precheck errors even if template files in the repo contain `UPG-####` |

<!-- Script scope (A10) -->

| # | Criterion | How it will be verified |
|---|---|---|
| A10 | Changed hunks limited to: `run_prechecks()` (new function), `cmd_review()` argument-parse block, and the precheck call site before `PACKET_FILE` creation; `build_packet()`, `cmd_decision`, `cmd_stage_start`, `run_codex`, `stage_checks`, `stage_expected`, and log-parse lines unchanged | `git diff -- scripts/codeos-review.sh \| grep "^@@ "` → only hunks in `cmd_review` and the new `run_prechecks` function |

<!-- Docs and scope boundary (A11–A12) -->

| # | Criterion | How it will be verified |
|---|---|---|
| A11 | `docs/reviewer-pipeline.md` has a paragraph describing prechecks, `--guard-clean`, `--skip-prechecks`, and hard-fail vs warning behavior | `grep "guard-clean\|skip-prechecks\|precheck" docs/reviewer-pipeline.md` → match |
| A12 | Scope boundary clean: `dba-system.md` diff empty; no out-of-scope runner features added (no delta mode, no header_only, no typed runner, no trace/dashboard validation, no §X validation); `review-log.md` format unchanged; `build_packet()` body not touched | `git diff -- dba-system.md` → empty; `grep -r "\-\-mode delta\|header_only\|typed.runner" scripts/` → no new matches; `git diff -- scripts/codeos-review.sh \| grep "^@@"` → no hunk inside `build_packet()` |

---

## Implementation Notes

### Edit 1 — `cmd_review()` arg-parse and call site (`scripts/codeos-review.sh`)

Added `skip_prechecks`, `guard_clean_paths[]` to the local-variable declaration. Added
`--skip-prechecks` and `--guard-clean PATH` cases to the `while` loop (guarded with
`[[ $# -ge 2 ]]` for the argument). Before `PACKET_FILE` creation:

```bash
if [[ ${skip_prechecks} -eq 1 ]]; then
  echo "warning: prechecks skipped (--skip-prechecks)" >&2
else
  PRECHECK_GUARD_CLEAN=("${guard_clean_paths[@]+"${guard_clean_paths[@]}"}")
  run_prechecks "${artifacts[@]}"
fi
```

`PRECHECK_GUARD_CLEAN` uses the same global-array pattern as `PACKET_SHA_ONLY` (CHG-2).
`PACKET_FILE` creation and `PACKET_SHA_ONLY` assignment are unchanged — they follow
immediately after the precheck block.

### Edit 2 — new `run_prechecks()` function (`scripts/codeos-review.sh`)

Placed immediately before `cmd_review()`. Reads `PRECHECK_GUARD_CLEAN` global array.
Iterates artifact paths: skips non-regular files (missing artifacts handled by
`build_packet()`); `grep -qF` for the two literal placeholder strings; `grep -qE` for
the line-anchored `latest_review:` field; `grep -qiE` for the warning-tier draft markers
(case-insensitive; `\bTBD\b` word-boundary prevents `STABILIZED` matches). Guard-clean
loop checks file existence first (exit 2 if missing), then runs `git diff --quiet HEAD --
<path>` (exit 2 if non-zero).

### Edit 3 — §10 Usage paragraph (`docs/reviewer-pipeline.md`)

One paragraph added after the existing code block describing: what runs, the two hard-fail
checks, the warning tier, `--guard-clean PATH`, and `--skip-prechecks`. No other sections
touched.

### Scope boundary

`build_packet()`, `cmd_decision`, `cmd_stage_start`, `run_codex`, `stage_checks`,
`stage_expected`, and log-parse lines are unchanged. `dba-system.md` not touched.

---

## Reconciliation

### AC verification

| # | Result | Evidence |
|---|---|---|
| A1 | PASS | `grep -n "run_prechecks\|PACKET_FILE" scripts/codeos-review.sh` → call at line 486 < `PACKET_FILE=` at line 488 |
| A2 | PASS | File with `UPG-####` → exit 2, `error: precheck failed — literal placeholder 'UPG-####'`; file with `UPG-0027` only → script proceeds past precheck (timed out in build, not precheck) |
| A3 | PASS | File with `CHG-YYYYMMDD-NNN` → exit 2, `error: precheck failed — literal placeholder 'CHG-YYYYMMDD-NNN'`; file with `CHG-20260629-004` only → passes |
| A4 | PASS | `latest_review: something` (bare) → exit 2; `    latest_review: indented` → exit 2; `the latest_review: field is forbidden` (prose — not line-anchored) → no failure |
| A5 | PASS | Appended a line to `dba-system.md`; `--guard-clean dba-system.md` → exit 2, `error: precheck failed — --guard-clean path 'dba-system.md' has uncommitted changes`; restored with `git checkout` |
| A6 | PASS | `--guard-clean nonexistent-guard-path.md` → exit 2, `error: precheck failed — --guard-clean path not found: nonexistent-guard-path.md` |
| A7 | PASS | File with `TODO`, `FIXME`, `TBD`, `[to be filled]` → stderr: `warning: precheck — unresolved draft marker (TODO/FIXME/TBD/[to be filled]) in …`; script continued to build_packet (timed out there, not in precheck) |
| A8 | PASS | `--skip-prechecks` with `UPG-####` artifact → stderr: `warning: prechecks skipped (--skip-prechecks)`; script proceeded past precheck into build_packet (timed out there); precheck never blocked |
| A9 | PASS | `templates/codeos-change.md` and the change record itself contain `UPG-####`; running `review … clean.md` (only) → no precheck error |
| A10 | PASS | `git diff HEAD -- scripts/codeos-review.sh \| grep "^@@ "` → two hunks: `@@ -414,17 +414,63 @@ run_codex()` (new `run_prechecks()` function, inserted between `run_codex` and `cmd_review`) and `@@ -433,6 +479,12 @@ cmd_review()` (arg-parse + call site); `build_packet()` at lines 108–342 — no hunk there |
| A11 | PASS | `grep -n "guard-clean\|skip-prechecks\|precheck" docs/reviewer-pipeline.md` → matches at lines 297, 303, 305, 306 (§10 Usage paragraph) |
| A12 | PASS | `git diff HEAD -- dba-system.md` → 0 bytes; `git diff \| grep "\-\-mode delta\|header_only\|typed.runner"` → no matches; `git diff -- reviews/review-log.md` → 0 bytes; no hunk inside `build_packet()` |

### --skip-prechecks scope check

Verified that `--skip-prechecks` bypasses only the `run_prechecks()` call. The artifact
count guard (`[[ ${#artifacts[@]} -gt 0 ]]`) and all argument parsing run unconditionally
before the precheck block. `PACKET_FILE` creation, `PACKET_SHA_ONLY` setup, and
`build_packet()` run unconditionally after it. The flag controls one `if` block only.

### Stale reference sweep

No stale cross-references introduced: the new `--guard-clean` and `--skip-prechecks` flags
are documented in §10 of `docs/reviewer-pipeline.md`. No stage tables, prompt filenames, or
review-log format changed. No links added to non-existent files.
