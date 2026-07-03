---
change_id: CHG-20260703-003
feature_id: UPG-0021
slug: stage-report-generator
triage_class: script-tooling
scope_axis: self-dev only
review_profile: PROFILE-3
review_series: RVS__UPG-0021__CHG-20260703-003__S1
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0021 / CHG-20260703-003 — Stage Report Generator

## TRACE HEADER

```yaml
feature_id: UPG-0021
primary_feature_id: UPG-0021
change_id: CHG-20260703-003
slug: stage-report-generator
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0021
related_features:
  - UPG-0004
review_series: RVS__UPG-0021__CHG-20260703-003__S1
review_profile: PROFILE-3
review_state: ACCEPTED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

---

## Step 1 — Change Intent

### Problem

Filling `templates/stage-4-6-report.md` by hand for every feature is repetitive for fields
that can be derived mechanically (files changed from `git diff`, test counts from test output).
The risk is omission or tedious copy-paste. A generator closes the gap while preserving human
authorship for fields requiring judgment.

### What changes

| File | Change |
|---|---|
| `tools/reviewer/src/cmd/generate_report.rs` | New: `generate-report` subcommand |
| `tools/reviewer/src/cmd/mod.rs` | Register `generate_report` module |
| `tools/reviewer/src/main.rs` | Add `GenerateReport` variant; dispatch before config resolution |
| `tools/reviewer/tests/smoke.rs` | Smoke tests |
| `backlog/UPG-0021-stage-report-generator.md` | Feature Thread: CHG-20260703-003 activated |
| `status/self-development.md` | Row activated |
| `status/roadmap.md` | UPG-0021 → IN_PROGRESS |

### Scope boundary — what stays the same

- `templates/stage-4-6-report.md` — not modified
- `dba-system.md` — not touched
- No existing subcommands changed
- `scripts/codeos-review.sh` — not touched (shim passes through automatically)

### Design intent

`codeos-reviewer generate-report --stage <4|5|6> [--feature <id>] [--base <ref>] [--test-output <file>] [--events <file>]`

Every generated report opens with:
```
> [INFERRED] fields were populated automatically from CLI arguments, git, test output, or
> events — verify before submitting. [FILL] fields require human or model authorship.
```

**Stage 4 skeleton:**
- `Feature:` → `<feature-id>` if `--feature` provided, else `[FILL]`
- `Files changed:` → `git diff --name-only <base>..HEAD` output, tagged `[INFERRED]`
- All judgment fields (`Contract clauses implemented`, `Assumptions`, etc.) → `[FILL]`

**Stage 5 skeleton:**
- `Feature:` → same as above
- `Tests run/passed/failed/skipped` → parsed from `--test-output` file if provided → `[INFERRED]`; else `[FILL]`
- All test-category fields → `[FILL]`

**Stage 6 skeleton:**
- `Feature:` → same as above
- `Events captured:` → line count of `--events` JSONL file if provided → `[INFERRED]`; else `[FILL]`
- All other fields → `[FILL]`

Output is written to stdout (redirect to file). Dispatched before `config::resolve()`.

### Triage

- Class: `script-tooling`
- Scope axis: `self-dev only`
- Review profile: `PROFILE-3`
- Originating backlog id: `UPG-0021`

---

## Step 2 — Acceptance Criteria

### Functional I/O

**AC-1 — Stage 4 skeleton structure**
`generate-report --stage 4` outputs a Markdown block containing every field from the Stage 4
section of `templates/stage-4-6-report.md` in the same order. No fields are omitted.

"Every field" means every line in the template that ends with `:` — including nested
sub-items (list items indented under a parent field). For Stage 4 this includes the
three sub-items under `Approved artifacts used:`:
```
- Intent:
- Contract:
- Event schema:
```
These sub-items must appear in the output beneath their parent field, each with a value or
`[FILL]`. The parent field line itself (`Approved artifacts used:`) is also emitted, and per
AC-5 carries `[FILL]` rather than being left blank — it has no value of its own to infer.

_Verify in Step 4:_ extract every `:` suffix line from the template section and the
generated output; confirm the sets are identical and in the same order.

**AC-2 — Stage 5 skeleton structure**
`generate-report --stage 5` outputs every field from the Stage 5 section, in template order.
No sub-items exist in the Stage 5 template section; each line ending with `:` is a top-level
field.
_Verify in Step 4:_ same field-name diff method as AC-1.

**AC-3 — Stage 6 skeleton structure**
`generate-report --stage 6` outputs every field from the Stage 6 section, in template order.
Includes the two sub-items under `Raw logs committed:`:
```
- yes/no:
- if yes, why safe:
```
These sub-items must appear in the output beneath `Raw logs committed:`, each with a value or
`[FILL]`. The parent field line itself carries `[FILL]` (same rule as AC-1's
`Approved artifacts used:`).
_Verify in Step 4:_ same field-name diff method as AC-1.

**AC-4 — Inferred fields tagged `[INFERRED]`**
Every field populated automatically — from CLI arguments, git, test output, or events — is
tagged `[INFERRED]` in the output. This covers all six inferred fields:
- `Feature:` (from `--feature`) — all stages
- `Files changed:` (from `git diff --name-only`) — Stage 4
- `Tests run:`, `Tests passed:`, `Tests failed:`, `Tests skipped:` (from `--test-output`) — Stage 5
- `Events captured:` (from `--events` line count) — Stage 6

_Verify in Step 4:_ run each stage with all relevant inputs supplied; confirm every
inferred field carries `[INFERRED]`. Specifically: Stage 4 with `--feature` and `--base`,
Stage 5 with `--feature` and `--test-output`, Stage 6 with `--feature` and `--events`.

**AC-5 — Judgment fields tagged `[FILL]`**
Every field the tool cannot populate is tagged `[FILL]` (no blank lines or missing fields).
This includes parent labels for a nested sub-item group (`Approved artifacts used:`,
`Raw logs committed:`) — they carry no value of their own to infer, so they are `[FILL]` too,
same as any other unpopulated field. No field is ever emitted blank.
_Verify in Step 4:_ run without optional inputs; confirm every unpopulated field — including
the two parent labels above — contains `[FILL]`, not an empty value.

**AC-6 — Preamble present**
Every generated report begins with the warning block:
```
> [INFERRED] fields were populated automatically from CLI arguments, git, test output, or
> events — verify before submitting. [FILL] fields require human or model authorship.
```
_Verify in Step 4:_ assert the first non-blank line of output matches this preamble.

**AC-7 — `--feature` populates the Feature field**
When `--feature UPG-XXXX` is supplied, `Feature:` is set to `UPG-XXXX [INFERRED]`.
When omitted, `Feature:` is `[FILL]`.
_Verify in Step 4:_ smoke test with and without the flag.

**AC-8 — `--base` populates Files changed (Stage 4)**
For Stage 4 with `--base <ref>`, `Files changed:` is populated with the output of
`git diff --name-only <ref>..HEAD`, tagged `[INFERRED]`.

- If `git diff --name-only <ref>..HEAD` exits non-zero (e.g., unknown ref): field is `[FILL]`;
  error printed to stderr. Exit code remains 0. Not being inside a git repository at all is a
  separate, earlier failure mode: `main.rs`'s `discover_repo_root()` runs before any subcommand
  dispatches (shared by `review`, `decision`, `diagnose`, `check-drift`, and `generate-report`
  alike) and exits 2 in that case — `generate-report` never reaches `git_diff_files` to produce
  a `[FILL]` fallback for it, and is not expected to; this AC governs `git diff` failures once
  inside a valid repo, not repo discovery itself.
- If the diff is empty (zero files changed): field is `(none) [INFERRED]` — a known derived
  result, not an unknown one. No error is emitted. This is not treated as a fallback case.
- If `--base` is omitted: field is `[FILL]`.

_Verify in Step 4:_ (a) smoke test `--base <known-commit>` with staged changes; confirm
list matches `git diff --name-only`; (b) test `--base HEAD` (zero-diff); confirm output is
`(none) [INFERRED]` and exit code is 0; (c) test `--base nonexistent-ref`; confirm field is
`[FILL]` and error appears on stderr.

**AC-9 — `--test-output` populates test counts (Stage 5)**
For Stage 5, when `--test-output <file>` is provided, the tool parses `Tests run / passed /
failed / skipped` from the file by matching the `cargo test` summary line:

```
test result: <STATUS>. <N> passed; <M> failed; <P> ignored; <Q> measured; ...
```

Specifically: a line matching the regex
`test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored`
(case-sensitive; additional fields after `ignored` are ignored by the parser).

The parsed values populate:
- `Tests run:` = passed + failed (tests that actually executed) `[INFERRED]`
- `Tests passed:` = passed count `[INFERRED]`
- `Tests failed:` = failed count `[INFERRED]`
- `Tests skipped:` = ignored count `[INFERRED]`

`ignored` tests are not executed — they are skipped. `Tests run:` excludes them; `Tests
skipped:` captures them. `Tests not run:` is not derivable from a single test output and
remains `[FILL]`.

If the file does not contain a matching line, or if the file cannot be read, all four
fields fall back to `[FILL]`. No error is emitted for a non-matching file (graceful
fallback). A read error is printed to stderr.

If `--test-output` is not provided, all four fields are `[FILL]`.

_Verify in Step 4:_ (a) supply a fixture file containing a valid summary line; confirm
counts match the fixture values; (b) supply a fixture with no matching line (including a line
with a status other than `ok`/`FAILED`, e.g. a typo'd or unrecognized status word); confirm
graceful `[FILL]` fallback; (c) supply a nonexistent file path; confirm `[FILL]` and error
on stderr.

**AC-10 — `--events` populates Events captured (Stage 6)**
For Stage 6:
- `--events <file>` provided and readable: `Events captured:` is set to the line count of
  the file, tagged `[INFERRED]`. Exit code 0.
- `--events <file>` provided but unreadable or not found: `Events captured:` falls back to
  `[FILL]`; error printed to stderr; report still written to stdout; exit code 0.
- `--events` not provided: `Events captured:` is `[FILL]`.

_Verify in Step 4:_ (a) supply a fixture JSONL file; confirm line count appears tagged
`[INFERRED]`; (b) supply a nonexistent file path; confirm `[FILL]` fallback and error on
stderr.

**AC-11 — Output to stdout**
All report content is written to stdout. Progress notes and errors go to stderr only.
A user can redirect stdout to a file without capturing noise.
_Verify in Step 4:_ run `generate-report ... > /dev/null` and confirm stderr is empty on
success; run with a bad `--events` path and confirm the report still goes to stdout with
fallback `[FILL]`, error to stderr.

### Exit codes

**AC-12 — Exit 0 on success**
All valid `generate-report` invocations exit 0.
_Verify in Step 4:_ smoke tests assert `$?` is 0 for each stage.

**AC-13 — Exit 1 on bad usage**
Invalid `--stage` value (e.g., `--stage 7`) or unknown flags exit 1 with a usage message
on stderr.
_Verify in Step 4:_ smoke test `--stage 7`; check exit code and stderr message.

**AC-14 — Dispatch before config resolution**
`generate-report` runs without a configured provider. It dispatches before
`config::resolve()` is called (same pattern as `check-drift`).
_Verify in Step 4:_ run in an environment with no provider config; confirm it succeeds.

### Idempotency

**AC-15 — Output is deterministic for the same inputs**
Given identical git state, `--feature`, `--base`, `--test-output`, and `--events`, two
invocations produce byte-for-byte identical stdout (modulo timestamps, which this command
does not emit).
_Verify in Step 4:_ run twice and diff the outputs.

### Cross-reference integrity

**AC-16 — Template field coverage is total**
No field from `templates/stage-4-6-report.md` is silently dropped; the generator must
produce a placeholder for every field even if it cannot infer a value. "Every field" is
defined as every line whose content ends with `:` in the relevant template section — this
includes nested sub-items (indented list items ending with `:`).
_Verify in Step 4:_ extract all `:` suffix lines from each template section (using grep or
awk) and the corresponding generator output; assert the extracted sets are identical and
in the same order.

---

## Step 3 — Implement

### What was done

| File | Change |
|---|---|
| `tools/reviewer/src/cmd/generate_report.rs` | `generate-report` core: `run()`, Stage 4/5/6 skeleton builders, `git diff --name-only` inference, cargo-test-summary parser, event-line counter. (Written prior to this session; unchanged here.) |
| `tools/reviewer/src/cmd/mod.rs` | `pub mod generate_report;` registered. (Prior to this session.) |
| `tools/reviewer/src/main.rs` | Added `Commands::GenerateReport { stage, feature, base, test_output, events }` variant; dispatched **before** `config::resolve()` (mirrors `CheckDrift`, satisfies AC-14); added the unreachable post-config match arm. |
| `tools/reviewer/tests/smoke.rs` | Added 21 smoke tests (`smoke_generate_report_*`) covering AC-1 through AC-16: field-coverage diffing against `templates/stage-4-6-report.md` (AC-1/2/3/16), preamble (AC-6), `--feature` (AC-7), `--base` three cases incl. zero-diff and invalid ref (AC-8), `--test-output` four cases incl. unrecognized status (AC-9), `--events` two cases (AC-10), blanket `[FILL]`-never-blank check incl. parent labels (AC-5), stdout-only / stderr-on-error (AC-11), exit codes (AC-12/AC-13), no-provider-config dispatch (AC-14), determinism (AC-15). |

### Verification

`cargo build` and `cargo build --tests`: clean, no errors.
`cargo test --test smoke`: **63 passed, 0 failed** (42 pre-existing + 21 new). No regressions.

### Scope check

No edits to `templates/stage-4-6-report.md`, `dba-system.md`, `scripts/codeos-review.sh`, or any other existing subcommand's behavior — matches the Step 1 scope boundary. `Commands::CheckDrift` dispatch/handling untouched.

### Review round 1 fixes (selfdev-step-3, R1: DO NOT ADVANCE)

Codex flagged 3 IN-SCOPE BLOCKERs, all fixed:

1. **AC-5 blank parent fields** — `Approved artifacts used:` and `Raw logs committed:` were
   emitted with no value instead of `[FILL]`. Fixed in `generate_report.rs`: both now emit
   `<Label>: [FILL]`. AC-1/AC-3/AC-5 text in this record updated to state explicitly that
   parent labels carry `[FILL]`, not blank. Smoke test tightened to assert this directly
   instead of excluding these labels.
2. **AC-9 status not validated** — `parse_cargo_summary_line` accepted any status token
   before `. `, not just `ok`/`FAILED` as AC-9's stated regex requires. Fixed: status is now
   checked against `"ok"`/`"FAILED"` (case-sensitive) before parsing counts; anything else is
   treated as non-matching (graceful `[FILL]`, no error). Added
   `smoke_generate_report_test_output_unrecognized_status`.
3. **AC-8 "not a git repo" example unreachable** — `main.rs`'s `discover_repo_root()` runs
   before any subcommand dispatches (shared with `review`/`decision`/`diagnose`/`check-drift`)
   and exits 2 if not inside a git repo, so `generate-report` never reaches `git_diff_files`
   for that case. This is existing, consistent CLI behavior, not a regression to fix in code —
   the AC-8 text was corrected to describe it as a separate, earlier failure mode outside this
   AC's scope, rather than claiming `generate-report` gracefully handles it.

`cargo build`, `cargo build --tests`, `cargo test --test smoke` re-run clean after fixes:
**63 passed, 0 failed**.

---

## Step 4 — Reconcile

### Acceptance criteria verification

| AC | Verified by | Result |
|---|---|---|
| AC-1 Stage 4 field coverage/order | `smoke_generate_report_stage4_field_coverage` | PASS |
| AC-2 Stage 5 field coverage/order | `smoke_generate_report_stage5_field_coverage` | PASS |
| AC-3 Stage 6 field coverage/order | `smoke_generate_report_stage6_field_coverage` | PASS |
| AC-4 `[INFERRED]` tagging | `smoke_generate_report_feature_flag_inferred`, `_base_populates_files_changed`, `_test_output_valid_summary`, `_events_fixture` | PASS |
| AC-5 `[FILL]` tagging, no blanks (incl. parent labels) | `smoke_generate_report_all_fill_without_optional_inputs` | PASS |
| AC-6 Preamble present | `smoke_generate_report_preamble_present` | PASS |
| AC-7 `--feature` behavior | `smoke_generate_report_feature_flag_inferred` | PASS |
| AC-8 `--base` (populated / zero-diff / invalid ref) | `smoke_generate_report_base_populates_files_changed`, `_base_zero_diff`, `_base_invalid_ref` | PASS |
| AC-9 `--test-output` (valid / no-match / unrecognized-status / missing) | `smoke_generate_report_test_output_valid_summary`, `_no_matching_line`, `_unrecognized_status`, `_missing_file` | PASS |
| AC-10 `--events` (fixture / missing) | `smoke_generate_report_events_fixture`, `_events_missing_file` | PASS |
| AC-11 stdout/stderr split | `smoke_generate_report_stdout_only_no_stderr_on_success`, `_stdout_still_written_on_partial_error` | PASS |
| AC-12 exit 0 on success | `smoke_generate_report_exit_zero_all_stages` | PASS |
| AC-13 exit 1 on bad usage | `smoke_generate_report_invalid_stage_exits_usage` | PASS |
| AC-14 dispatch before config resolution | `smoke_generate_report_no_provider_config_required` | PASS |
| AC-15 deterministic output | `smoke_generate_report_deterministic_output` | PASS |
| AC-16 total template field coverage | Same test as AC-1/2/3 (label-set diff, not just presence) | PASS |

`cargo test --test smoke`: **63 passed, 0 failed** (verified again at Step 4, post round-2 fixes).

### Cross-reference sweep

- `grep -rln "check-drift\|CheckDrift"` across `docs/`, `backlog/`, `prompts/`, `templates/` returns only `backlog/UPG-0020-stack-drift-detector.md` — confirming `docs/reviewer-pipeline.md` does not maintain a general subcommand list (it documents the `review`/`decision` pipeline specifically). By the same precedent, `generate-report` requires no doc cross-reference update; this is not an omission, it matches how `check-drift` (UPG-0020) was integrated.
- `templates/stage-4-6-report.md`: untouched, confirmed via `git diff` (empty) — matches the Step 1 scope boundary of "not modified."
- `dba-system.md`, `scripts/codeos-review.sh`: untouched — confirmed via `git status` (no changes to either file across this change).
- `backlog/UPG-0021-stage-report-generator.md`: Feature Thread already carries the `CHG-20260703-003` row (activated at Step 1); no other edit made to it in this reconcile section.

### Reviewer scope triage (Step 4 findings)

R1 (DO NOT ADVANCE): 3 IN-SCOPE BLOCKERs — this record's dashboard/status prose made
unsupported claims of human approval and COMPLETE-state ahead of the actual gate, and cited
a `backlog/features.md` update outside the Step 1 file list without diff evidence. Fixed:
forward-looking claims removed from Step 4 prose (see prior revision); reworded to record
verification results only, deferring state-file edits until after this review + human
approval.
R2 (NO OBJECTION): confirmed the fix — "Step 4 completion is correctly left pending human
gate." No IN-SCOPE BLOCKER findings remained.

### Outcome

All 16 ACs verified against the final code and tests (table above). No in-scope blockers
open. No scope drift. Step 4 R2 NO OBJECTION; human APPROVE_STAGE recorded
(2026-07-03T15:37:59Z). Change record, `status/self-development.md`, `status/roadmap.md`,
`backlog/features.md`, and `backlog/UPG-0021-stage-report-generator.md` updated to COMPLETE
in this same pass, following that approval.
