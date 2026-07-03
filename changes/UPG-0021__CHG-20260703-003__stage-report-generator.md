---
change_id: CHG-20260703-003
feature_id: UPG-0021
slug: stage-report-generator
triage_class: script-tooling
scope_axis: self-dev only
review_profile: PROFILE-3
review_series: RVS__UPG-0021__CHG-20260703-003__S1
review_state: DRAFT
status: IN_PROGRESS
loop_step: 3-Implement (in progress)
---

# Change: UPG-0021 / CHG-20260703-003 — Stage Report Generator

## TRACE HEADER

```yaml
feature_id: UPG-0021
primary_feature_id: UPG-0021
change_id: CHG-20260703-003
slug: stage-report-generator
state: IN_PROGRESS
current_step: 3-Implement
implements:
  - UPG-0021
related_features:
  - UPG-0004
review_series: RVS__UPG-0021__CHG-20260703-003__S1
review_profile: PROFILE-3
review_state: DRAFT
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
`[FILL]`. The parent field line itself (`Approved artifacts used:`) is also emitted.

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
`[FILL]`.
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
_Verify in Step 4:_ run without optional inputs; confirm every unpopulated field contains
`[FILL]`, not an empty value.

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

- If git exits non-zero (e.g., unknown ref, not a git repo): field is `[FILL]`; error
  printed to stderr. Exit code remains 0.
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
counts match the fixture values; (b) supply a fixture with no matching line; confirm
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

*(to be written after Step 2 approval)*

---

## Step 4 — Reconcile

*(to be written after Step 3 approval)*
