---
change_id: CHG-20260706-002
feature_id: UPG-0024
slug: release-evidence-package
triage_class: script-tooling
scope_axis: self-dev only
review_profile: PROFILE-3
review_series: RVS__UPG-0024__CHG-20260706-002__S1
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0024 / CHG-20260706-002 — Pre-Release Evidence Package

## TRACE HEADER

```yaml
feature_id: UPG-0024
primary_feature_id: UPG-0024
change_id: CHG-20260706-002
slug: release-evidence-package
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0024
related_features:
  - UPG-0011
  - UPG-0009
  - UPG-0023
review_series: RVS__UPG-0024__CHG-20260706-002__S1
review_profile: PROFILE-3
review_state: ACCEPTED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

---

## Step 1 — Change Intent

### Problem

Before release, a feature's evidence — approved artifacts, stage reports, reviewer history,
reconciliation/replay results, readiness checklist — is scattered across stage files,
reports, runtime logs, and reviewer summaries. There is no single generated overview a human
reviews before making a release decision.

### What changes

| File | Change |
|---|---|
| `tools/reviewer/src/cmd/generate_release_evidence.rs` | New: `generate-release-evidence` subcommand |
| `tools/reviewer/src/cmd/mod.rs` | Register `generate_release_evidence` module |
| `tools/reviewer/src/main.rs` | Add `GenerateReleaseEvidence` variant; dispatch before config resolution (same pattern as the three prior generators) |
| `tools/reviewer/tests/smoke.rs` | Smoke tests |
| `backlog/UPG-0024-release-evidence-package.md` | Feature Thread: CHG-20260706-002 activated (done) |
| `backlog/features.md` | Row → IN_PROGRESS (done) |
| `status/self-development.md` | Row activated (done) |
| `status/roadmap.md` | UPG-0024 → IN_PROGRESS (done) |

### Scope boundary — what stays the same

- `templates/feature-registry.yaml`, `templates/readiness-checklist.md`,
  `templates/review-file.md` — none modified. Where read at all, read as optional input only
  (see Design intent).
- `dba-system.md` — not touched.
- No existing subcommand's behavior changed (`review`, `decision`, `diagnose`, `stage-start`,
  `check-drift`, `generate-report`, `generate-adr-candidates`,
  `generate-approval-dashboard` all untouched).
- `scripts/codeos-review.sh` — not touched (shim passes through automatically, modulo
  UPG-0038's separate, pre-existing symlink-resolution bug).
- **No parsing of `reviews/review-log.md` (self-dev's format) or `reviews/[feature_id].md`
  (downstream's format) for "Reviewer briefs."** These are two genuinely different,
  free-form-ish structures depending on context (self-dev vs. downstream), and reliably
  auto-detecting/parsing either is exactly the "fragile free-form-log parser" UPG-0023's Step
  1 already ruled out for the same reason. "Reviewer briefs" stays `[FILL]` unconditionally —
  a deliberate scope-narrowing decision, not an oversight.
- **No parsing of stage reports, reconciliation results, replay results, verification-only
  reports, or readiness checklist instances.** None of these have a fixed, discoverable
  per-feature file path convention anywhere in the toolkit today (confirmed:
  `templates/readiness-checklist.md` is a template with no recommended instance path,
  unlike `templates/feature-registry.yaml` which explicitly recommends `features/
  registry.yaml`). All six of these fields stay `[FILL]` unconditionally.

### Design intent

`codeos-reviewer generate-release-evidence --feature <feature_id> [--registry <path>]`

**Mechanical inference is deliberately narrow** (flagged for approval): of the 11 backlog
fields (`Feature`, `Branch/PR`, `Approved artifacts`, `Stage reports`, `Reviewer briefs`,
`Reconciliation result`, `Replay result`, `Verification-only report`, `Readiness checklist`,
`Known limitations`, `Release decision`), only three get any mechanical inference at all:
- `Feature:` → the `--feature` value, always `[INFERRED]`.
- `Branch:` (split out of the backlog's combined "Branch/PR" field — see below) → `git
  rev-parse --abbrev-ref HEAD` from the repo root, always `[INFERRED]` (this tool must run
  inside a git repo, matching `generate-report`'s existing pattern).
- `PR:` and `Approved artifacts:` → **only if `--registry <path>` is given** and the
  feature's entry exists in that registry (UPG-0009's canonical schema, the same
  `Registry`/`FeatureEntry` parsing pattern `generate-approval-dashboard` already
  established): `pr` and `intent`/`contract`/`event_schema` fields, if non-null, become
  `[INFERRED]`. Otherwise `[FILL]`.

The remaining 7 fields (`Stage reports`, `Reviewer briefs`, `Reconciliation result`, `Replay
result`, `Verification-only report`, `Readiness checklist`, `Known limitations`, `Release
decision` — 8 by the backlog's own list, since `Known limitations` and `Release decision`
are always judgment) are always `[FILL]`.

**`--registry` is optional here — a deliberate divergence from `generate-approval-
dashboard`'s required `--registry`, flagged for approval.** There, the registry *was* the
entire input. Here, it's an optional enrichment: a feature can get a release evidence
package even with no registry at all (everything beyond `Feature`/`Branch` is `[FILL]`), and
an unreadable/unparseable `--registry` degrades gracefully — logged to stderr, registry-
derived fields fall back to `[FILL]` — rather than hard-failing the whole command the way
`generate-approval-dashboard`'s required `--registry` does. Missing/bad `--registry` is not
"nothing to reason about" here the way a missing required input is elsewhere; there is
always something to emit (`Feature`, `Branch`, and 8 `[FILL]` fields at minimum).

**Output** (`Branch` and `PR` split from the backlog's combined "Branch/PR" line, for
per-field `[INFERRED]`/`[FILL]` tagging clarity — everything else matches the backlog's
field list and order exactly):

```markdown
# Release Evidence Package

Feature: <feature_id> [INFERRED]
Branch: <branch> [INFERRED]
PR: <pr, or [FILL]>
Approved artifacts:
- Intent: <path, or [FILL]>
- Contract: <path, or [FILL]>
- Event schema: <path, or [FILL]>
Stage reports: [FILL]
Reviewer briefs: [FILL]
Reconciliation result: [FILL]
Replay result: [FILL]
Verification-only report: [FILL]
Readiness checklist: [FILL]
Known limitations: [FILL]
Release decision: [FILL]
```

Every generated report opens with:
```
> [INFERRED] fields were populated automatically from git and the feature registry (if
> --registry was given) — verify before submitting. [FILL] fields require human or model
> authorship. This package aggregates existing evidence; it is not itself a decision record
> — Release decision requires explicit human judgment and is never inferred.
```

Output is written to stdout (redirect to file; recommended downstream path
`reviews/release-evidence-[feature_id].md`, per the backlog's proposed artifact — this tool
does not write the file itself, matching the three prior generators' stdout-only
precedent). Dispatched before `config::resolve()` (no provider config required).

### Triage

- Class: `script-tooling`
- Scope axis: `self-dev only`
- Review profile: `PROFILE-3`
- Originating backlog id: `UPG-0024`

---

## Step 2 — Acceptance Criteria

### Functional I/O

**AC-1 — `Feature:` always `[INFERRED]`**
`Feature:` is the `--feature` value, tagged `[INFERRED]`, in every invocation.
_Verify in Step 4:_ smoke test asserting `Feature: <id> [INFERRED]` appears regardless of
whether `--registry` is given.

**AC-2 — `Branch:` always `[INFERRED]` from git**
`Branch:` is `git rev-parse --abbrev-ref HEAD` from the repo root, tagged `[INFERRED]`, in
every invocation (the tool relies on the existing pre-dispatch `discover_repo_root()` check
in `main.rs` to fail before this subcommand ever runs outside a git repo — no separate
not-a-repo handling needed here).
_Verify in Step 4:_ fixture git repo on a named branch; confirm `Branch: <name> [INFERRED]`.

**AC-3 — Report structure matches the backlog field list, `Branch`/`PR` split**
After the preamble banner (AC-6) and a blank line, the report body is exactly:
`# Release Evidence Package` heading, then `Feature:`, `Branch:`, `PR:`,
`Approved artifacts:` (with `- Intent:`, `- Contract:`, `- Event schema:` sub-items),
`Stage reports:`, `Reviewer briefs:`, `Reconciliation result:`, `Replay result:`,
`Verification-only report:`, `Readiness checklist:`, `Known limitations:`,
`Release decision:` — in that order, every time. (AC-6 governs what stdout literally starts
with; this AC governs the report content that follows it — the two are not in conflict.)
_Verify in Step 4:_ extract all `:` suffix lines from generated output (any invocation),
excluding the banner lines (which start with `>`);
confirm the set and order matches this list exactly, including the three `Approved
artifacts:` sub-items.

**AC-4 — Registry-derived fields `[INFERRED]` only when genuinely available**
`PR:` and each `Approved artifacts:` sub-item are `[INFERRED]` with the registry's value
only when **all** of: `--registry` was given, the file parses, the `--feature` value matches
a `feature_id` in it, and that specific field (`pr`, `intent`, `contract`, `event_schema`) is
non-null in that entry. Any other combination (no `--registry`, unparseable file, feature
not found, or a null field) leaves that specific field `[FILL]` — evaluated independently
per field, not all-or-nothing.
_Verify in Step 4:_ fixture registry with one feature having `pr` and `intent` populated but
`contract`/`event_schema` null; confirm `PR:` and `- Intent:` are `[INFERRED]` while
`- Contract:` and `- Event schema:` are `[FILL]` in the same run.

**AC-5 — The 8 always-`[FILL]` fields**
`Stage reports:`, `Reviewer briefs:`, `Reconciliation result:`, `Replay result:`,
`Verification-only report:`, `Readiness checklist:`, `Known limitations:`, and
`Release decision:` are `[FILL]` in every invocation, regardless of `--registry` content —
the tool never attempts to derive any of these.
_Verify in Step 4:_ any fixture with `--registry` fully populated; confirm all 8 fields are
still `[FILL]`.

**AC-6 — Preamble present, verbatim**
Output begins with the banner verbatim:
```
> [INFERRED] fields were populated automatically from git and the feature registry (if
> --registry was given) — verify before submitting. [FILL] fields require human or model
> authorship. This package aggregates existing evidence; it is not itself a decision record
> — Release decision requires explicit human judgment and is never inferred.
```
_Verify in Step 4:_ assert the first four non-blank lines of stdout match this banner
verbatim.

**AC-7 — Unreadable or unparseable `--registry` degrades gracefully**
If `--registry <path>` is given but the file doesn't exist, can't be read, or isn't valid
YAML/doesn't match the expected shape: `PR:` and all `Approved artifacts:` sub-items fall
back to `[FILL]`; a stderr note is printed (`warning: cannot read/parse registry file
'<path>': <error>; registry-derived fields left as [FILL]`); the rest of the report (all
other fields) is still emitted in full; exit code remains 0.
_Verify in Step 4:_ (a) `--registry does-not-exist.yaml`; (b) `--registry <malformed-file>`;
confirm full report on stdout in both cases, stderr carries the warning, exit 0.

**AC-8 — Feature not found in a valid registry degrades gracefully**
If `--registry <path>` parses successfully but contains no entry matching `--feature`:
same graceful degradation as AC-7 — `PR:`/`Approved artifacts:` become `[FILL]`, a distinct
stderr note is printed (`warning: feature '<id>' not found in registry '<path>';
registry-derived fields left as [FILL]`), full report still emitted, exit 0.
_Verify in Step 4:_ fixture registry with only unrelated feature IDs; confirm the report is
still complete, the stderr message names both the feature id and the registry path, exit 0.

**AC-9 — No `--registry` given is the normal case, not a degraded one**
If `--registry` is omitted entirely, `PR:`/`Approved artifacts:` are `[FILL]` with **no**
stderr message — this is expected, ordinary usage, not an error or a degradation to warn
about. Only AC-7/AC-8's cases (registry *was* given but couldn't be used) produce a stderr
note.
_Verify in Step 4:_ run with no `--registry`; confirm stderr is completely empty and exit
code is 0.

**AC-10 — `--feature` is required**
`--feature <id>` is a required flag. Omitting it, or passing an unknown flag, is a clap
usage error: exit 1, usage message on stderr, nothing on stdout.
_Verify in Step 4:_ smoke test `generate-release-evidence` with no `--feature`; confirm
exit 1.

**AC-11 — Output to stdout only**
All report content (banner + package) is written to stdout only. Stderr carries only the
AC-7/AC-8 warning notes when applicable, and is otherwise empty. Stdout is always the full
report — there is no empty-stdout case for this tool (unlike the prior three generators,
which have a "nothing found" empty-output path); `Feature`/`Branch` plus 8+ `[FILL]` fields
means there is always something to emit.
_Verify in Step 4:_ (a) run with no `--registry`; confirm stderr empty, stdout non-empty; (b)
run the AC-7/AC-8 cases; confirm stdout is still the full report, not empty, despite the
stderr warning.

### Exit codes

**AC-12 — Exit 0 always, given a valid `--feature` inside a git repository**
Within a git repository, every invocation with a valid `--feature` (any `--registry` state —
absent, bad, feature-not-found, or fully valid) exits 0. This tool itself has no usage-
failure path beyond clap's own handling of a missing `--feature` (AC-10). Running outside a
git repository at all is excluded from this AC's scope: `main.rs`'s shared
`discover_repo_root()` check (AC-2) already exits non-zero before *any* subcommand
dispatches in that case — this is pre-existing behavior common to every subcommand, not
something `generate-release-evidence` implements or is responsible for.
_Verify in Step 4:_ smoke tests for the no-registry, bad-registry, feature-not-found, and
fully-valid-registry cases, all run inside a temp git repo; assert exit 0 in all four.

**AC-13 — Dispatch before config resolution**
`generate-release-evidence` runs without a configured provider, dispatching before
`config::resolve()` (same pattern as the three prior generators).
_Verify in Step 4:_ run in a temp repo with no provider config set up at all; confirm no
config-resolution error occurs.

### Idempotency

**AC-14 — Deterministic output**
Given the same `--feature`, the same (or absent) `--registry` content, and unchanged git
branch state, two invocations produce byte-for-byte identical stdout.
_Verify in Step 4:_ run twice with identical arguments against the same fixtures; diff the
outputs.

### Cross-reference integrity

**AC-15 — `architectural_refinements:` never mistaken for a feature entry (when `--registry` given)**
When looking up `--feature` in a provided registry, the tool only searches the top-level
`features:` list — an `architectural_refinements:` entry with a colliding-looking `refine_id`
is never matched as if it were a feature.
_Verify in Step 4:_ fixture registry with a real feature and an `architectural_refinements:`
entry whose `refine_id` equals the `--feature` value being searched for; confirm the real
feature's data is used (or `[FILL]` if no real feature matches), never the refinement's.

---

## Step 3 — Implement

### What was done

| File | Change |
|---|---|
| `tools/reviewer/src/cmd/generate_release_evidence.rs` | New. `run()` builds the report; `resolve_registry_fields()` handles the three optional-registry degradation cases (absent, unreadable/malformed, feature-not-found); `git_branch()` derives `Branch:` via `git rev-parse --abbrev-ref HEAD`. |
| `tools/reviewer/src/cmd/mod.rs` | Registered `pub mod generate_release_evidence;` |
| `tools/reviewer/src/main.rs` | Added `Commands::GenerateReleaseEvidence { feature, registry }` (required `--feature`, optional `--registry`); dispatched before `config::resolve()`; unreachable post-config match arm added |
| `tools/reviewer/tests/smoke.rs` | 16 new tests (`smoke_release_evidence_*`) covering AC-1 through AC-15 (AC-7 split into two tests: unreadable file, malformed YAML) |

### Verification (AC-1 through AC-15)

- **AC-1/AC-2**: `smoke_release_evidence_feature_always_inferred`,
  `smoke_release_evidence_branch_always_inferred` — confirm `Feature:`/`Branch:` are always
  `[INFERRED]`, branch checked against an explicitly named fixture branch (not the ambient
  default, which can vary by `init.defaultBranch`).
- **AC-3**: `smoke_release_evidence_output_structure` — confirms all 15 field/sub-item labels
  appear in order, in a single pass over the string (each search starts after the previous
  match).
- **AC-4**: `smoke_release_evidence_registry_enrichment_per_field_independent` — a fixture
  with `pr`/`intent` populated but `contract`/`event_schema` null confirms independent
  per-field tagging in the same run.
- **AC-5**: `smoke_release_evidence_always_fill_fields` — a fully populated registry still
  leaves all 8 non-registry-derived fields `[FILL]`.
- **AC-6**: `smoke_release_evidence_preamble_present` — banner's first and last line present,
  and precedes the report heading.
- **AC-7**: `smoke_release_evidence_unreadable_registry_degrades_gracefully` (nonexistent
  path) and `smoke_release_evidence_malformed_registry_degrades_gracefully` (invalid YAML) —
  both confirm full report still emitted, distinct stderr wording, exit 0.
- **AC-8**: `smoke_release_evidence_feature_not_found_degrades_gracefully` — valid registry,
  no matching `feature_id`; stderr names both the feature id and the registry path.
- **AC-9**: `smoke_release_evidence_no_registry_is_silent` — asserts `stderr` is the empty
  string exactly, not just "no warning substring."
- **AC-10**: `smoke_release_evidence_feature_required` — exit 1, empty stdout.
- **AC-11**: `smoke_release_evidence_stdout_only_no_registry` — non-empty stdout, empty
  stderr, in the ordinary no-registry case.
- **AC-12**: `smoke_release_evidence_exit_zero_across_registry_states` — one test exercising
  all four registry states (absent/bad/not-found/fully-valid) against the same repo, all exit
  0.
- **AC-13**: `smoke_release_evidence_no_provider_config_required` — runs in a temp repo with
  no provider config, succeeds.
- **AC-14**: `smoke_release_evidence_deterministic_output` — two runs, identical stdout.
- **AC-15**: `smoke_release_evidence_architectural_refinements_never_treated_as_feature` — a
  registry with a real feature plus an `architectural_refinements` entry whose `refine_id`
  matches the *searched-for* id; confirms the refinement is never matched (fields fall back
  to `[FILL]`, not the refinement's non-existent `pr`/`intent`/etc. fields, since
  `FeatureEntry` doesn't even parse `architectural_refinements:` entries — they aren't part
  of the `features:` list `serde_yaml` deserializes into `Vec<FeatureEntry>`).

### Test run

```
cargo test -- --test-threads=1
test result: ok. 26 passed; 0 failed  (unit tests, unchanged)
test result: ok. 110 passed; 0 failed  (smoke tests: 94 prior + 16 new)
```
Single-threaded to avoid the pre-existing, unrelated `config::tests` race (UPG-0040);
`cargo build` clean (only pre-existing dead-code warnings, unrelated to this change).

### Scope check

`git diff --stat` for this change touches only the 4 files in the table above plus the
declared backlog/status bookkeeping (`backlog/UPG-0024-release-evidence-package.md`,
`backlog/features.md`, `status/self-development.md`, `status/roadmap.md`,
`changes/UPG-0024__CHG-20260706-002__release-evidence-package.md`,
`reviews/review-log.md`) — no other subcommand, template, or `dba-system.md` touched.

---

## Step 4 — Reconcile

### Acceptance criteria verification (fresh evidence)

| AC | Verified by | Result |
|---|---|---|
| AC-1 `Feature:` always `[INFERRED]` | `smoke_release_evidence_feature_always_inferred` | PASS |
| AC-2 `Branch:` always `[INFERRED]` | `smoke_release_evidence_branch_always_inferred` (named fixture branch, not ambient default) | PASS |
| AC-3 Report structure/order | `smoke_release_evidence_output_structure` | PASS |
| AC-4 Registry fields independent per-field | `smoke_release_evidence_registry_enrichment_per_field_independent` | PASS |
| AC-5 8 always-`[FILL]` fields | `smoke_release_evidence_always_fill_fields` (fully populated registry) | PASS |
| AC-6 Preamble verbatim, precedes heading | `smoke_release_evidence_preamble_present` | PASS |
| AC-7 Bad `--registry` degrades gracefully | `smoke_release_evidence_unreadable_registry_degrades_gracefully` + `smoke_release_evidence_malformed_registry_degrades_gracefully` | PASS |
| AC-8 Feature-not-found degrades gracefully | `smoke_release_evidence_feature_not_found_degrades_gracefully` | PASS |
| AC-9 No `--registry` is silent | `smoke_release_evidence_no_registry_is_silent` (stderr `== ""` exactly) | PASS |
| AC-10 `--feature` required | `smoke_release_evidence_feature_required` (exit 1, empty stdout) | PASS |
| AC-11 stdout-only, no empty-output case | `smoke_release_evidence_stdout_only_no_registry` | PASS |
| AC-12 Exit 0 across all registry states | `smoke_release_evidence_exit_zero_across_registry_states` (4 states, one repo) | PASS |
| AC-13 Dispatch before config resolution | `smoke_release_evidence_no_provider_config_required` | PASS |
| AC-14 Deterministic output | `smoke_release_evidence_deterministic_output` | PASS |
| AC-15 `architectural_refinements:` never matched | `smoke_release_evidence_architectural_refinements_never_treated_as_feature` | PASS |

### Fresh test run

```
cargo test -- --test-threads=1
test result: ok. 26 passed; 0 failed   (unit tests, unchanged)
test result: ok. 110 passed; 0 failed  (smoke tests: 94 prior + 16 new)
```

### Cross-reference sweep

- `git status --short` — only the 4 declared implementation files plus the declared
  backlog/status bookkeeping and review artifacts; no stray files.
- `git diff --stat -- tools/reviewer/src/cmd/` — only `mod.rs` (1-line module registration)
  changed among existing files; no other subcommand's `.rs` file touched.
- `git diff --stat -- dba-system.md` — empty; downstream doctrine untouched.
- `git diff --stat -- templates/` — empty; no template modified (registry lookup only reads
  a user-supplied path, never a fixed template file).

### Reviewer scope triage (Step 4 findings)

Step 1 R1 (NO OBJECTION) flagged one non-blocking gap (`backlog/features.md` omitted from
the "What changes" table), fixed inline. Step 2 R1 (DO NOT ADVANCE) found two genuine
internal contradictions in the AC wording (AC-3 vs. AC-6 disagreeing on what stdout starts
with; AC-12's universal exit-0 claim ignoring the shared not-a-git-repo precondition), both
fixed; R2 came back clean. Step 3 R1 (NO OBJECTION) found no blockers — implementation
matched the reconciled ACs on the first pass. This Step 4 review is the first review of the
fully reconciled state, not yet run as of this writing.

### Outcome

All 15 ACs verified against the final artifacts with fresh evidence (table above). No
in-scope blockers open. No scope drift — no other subcommand, `dba-system.md`, or template
touched. Step 4 R1 NO OBJECTION; human APPROVE_STAGE recorded (2026-07-06). Change record,
`status/self-development.md`, `status/roadmap.md`, `backlog/features.md`, and
`backlog/UPG-0024-release-evidence-package.md` updated to COMPLETE in this same pass,
following that approval.
