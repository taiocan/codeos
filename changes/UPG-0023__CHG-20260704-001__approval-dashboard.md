---
change_id: CHG-20260704-001
feature_id: UPG-0023
slug: approval-dashboard
triage_class: script-tooling
scope_axis: self-dev only
review_profile: PROFILE-3
review_series: RVS__UPG-0023__CHG-20260704-001__S1
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0023 / CHG-20260704-001 — Human Approval Dashboard

## TRACE HEADER

```yaml
feature_id: UPG-0023
primary_feature_id: UPG-0023
change_id: CHG-20260704-001
slug: approval-dashboard
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0023
related_features:
  - UPG-0009
  - UPG-0003
review_series: RVS__UPG-0023__CHG-20260704-001__S1
review_profile: PROFILE-3
review_state: ACCEPTED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

---

## Step 1 — Change Intent

### Problem

Multiple in-flight features and their reviewer state (registry entries from UPG-0009, review
history in `reviews/review-log.md`) are hard to navigate as a human. There is no single
generated overview of what is active, what stage it is at, and what is blocking it — a human
has to read the registry and cross-reference change records/review log by hand.

### What changes

| File | Change |
|---|---|
| `tools/reviewer/src/cmd/generate_approval_dashboard.rs` | New: `generate-approval-dashboard` subcommand |
| `tools/reviewer/src/cmd/mod.rs` | Register `generate_approval_dashboard` module |
| `tools/reviewer/src/main.rs` | Add `GenerateApprovalDashboard` variant; dispatch before config resolution (same pattern as `check-drift` / `generate-report` / `generate-adr-candidates`) |
| `tools/reviewer/Cargo.toml` | Add `serde_yaml = "0.9"` dependency |
| `tools/reviewer/Cargo.lock` | Mechanically updated by adding the dependency above (not a separate decision) |
| `tools/reviewer/tests/smoke.rs` | Smoke tests |
| `backlog/UPG-0023-approval-dashboard.md` | Feature Thread: CHG-20260704-001 activated (done) |
| `status/self-development.md` | Row activated (done) |
| `status/roadmap.md` | UPG-0023 → IN_PROGRESS (done) |

### Scope boundary — what stays the same

- `templates/feature-registry.yaml` — not modified. Its existing schema (from UPG-0009) is
  the input contract this tool reads.
- `dba-system.md` — not touched.
- No existing subcommand's behavior changed (`review`, `decision`, `diagnose`, `stage-start`,
  `check-drift`, `generate-report`, `generate-adr-candidates` all untouched).
- `scripts/codeos-review.sh` — not touched (shim passes through automatically).
- `architectural_refinements:` (the second top-level key in the registry template) is out of
  scope — this tool reads only the `features:` list. Refinement-tracking in a dashboard is a
  distinct, un-approved capability, not silently dropped: it is simply not claimed here.
- `reviews/review-log.md` is **not read or parsed** by this tool (see Design intent — this is
  a deliberate scope-narrowing decision, not an oversight).

### Design intent

`codeos-reviewer generate-approval-dashboard --registry <path>`

**Data source decision (flagged for approval):** the backlog's proposed fields are `Active
features / Current stage / Reviewer recommendation / Open blockers / Next human decision /
Risk`. Of these, only three are mechanically present in `templates/feature-registry.yaml`'s
schema: a feature's `status` (used to select "active" features), `current_stage`, and
`blockers`. Nothing in the registry schema carries a live reviewer verdict, a "next decision"
projection, or a risk assessment — those live only in free-form review-log entries and change
records, which do not have a fixed, parseable structure suitable for mechanical extraction
(unlike the single well-delimited Markdown sections the two prior generators read). Rather
than build a fragile free-form-log parser to guess at these, this tool follows the same
`[INFERRED]`/`[FILL]` split as `generate-report` and `generate-adr-candidates`:
- `Active features:`, `Current stage:`, `Open blockers:` → `[INFERRED]`, read directly from
  the registry entry.
- `Reviewer recommendation:`, `Next human decision:`, `Risk:` → always `[FILL]` — there is no
  registry field to derive them from; a human or model fills these in after reading the
  registry entry and, if needed, the feature's change record / review log directly.

**Dependency decision (flagged for approval):** parsing `features/registry.yaml` reliably
(quoted strings, block-style lists, comments, arbitrary key order) needs a real YAML parser,
not a hand-rolled line scanner — unlike the Markdown-bullet extraction in
`generate-adr-candidates`, YAML's structural flexibility makes mechanical text scanning
fragile. This adds `serde_yaml` (`0.9.34`, its final release) as a new dependency, deserialized
into a `Registry { features: Vec<FeatureEntry> }` struct via `#[derive(Deserialize)]` — the
same pattern `config.rs` already uses for `toml::from_str`. Note: `serde_yaml` is
maintainer-archived (no further releases), but it is stable, widely used, and this is a
manually-invoked, read-only, local dev tool — not a network-facing or security-sensitive
parse path. If preferred, a maintained fork (e.g. `serde_norway`) can be substituted with no
change to this design.

**Output:** one entry per registry feature with `status: active`, in registry order:

```markdown
# Approval Dashboard

## <feature_id>: <slug>

Active features: <feature_id> [INFERRED]
Current stage: <current_stage, or "not started"> [INFERRED]
Reviewer recommendation: [FILL]
Open blockers: <blockers, one per line, or "(none)"> [INFERRED]
Next human decision: [FILL]
Risk: [FILL]
```

Every generated report opens with:
```
> [INFERRED] fields were populated automatically from the feature registry — verify before
> submitting. [FILL] fields require human or model authorship. This dashboard is a navigation
> aid, not a decision record — the registry and change records remain authoritative.
```

**No active features found:** if the registry parses successfully but contains zero entries
with `status: active`, the tool writes nothing to stdout, writes an explanatory note to
stderr, and exits 0 (valid-but-empty, not a usage failure) — same philosophy as
`generate-adr-candidates`'s AC-7.

**Unreadable / missing `--registry` file, or a YAML parse error:** exit 1 (`EXIT_USAGE`),
stderr names the path and the error — `--registry` is the sole required input, so there is
nothing to reason about without it (same reasoning as `generate-adr-candidates`'s AC-8).

Output is written to stdout (redirect to file; recommended downstream path
`reviews/approval-dashboard.md`, per the backlog's proposed artifact — this tool does not
write the file itself, matching `generate-report`/`generate-adr-candidates`'s stdout-only
precedent). Dispatched before `config::resolve()` (no provider config required).

### Triage

- Class: `script-tooling`
- Scope axis: `self-dev only`
- Review profile: `PROFILE-3`
- Originating backlog id: `UPG-0023`

---

## Step 2 — Acceptance Criteria

### Functional I/O

**AC-1 — Registry parsing via `serde_yaml`**
`--registry <path>` is deserialized into a `Registry { features: Vec<FeatureEntry> }` struct
via `#[derive(Deserialize)]` + `serde_yaml::from_str` (matching `config.rs`'s existing
`toml::from_str` pattern). `FeatureEntry` only needs the fields this tool actually reads:
`feature_id`, `slug`, `status`, `current_stage` (`Option<i64>` — the template allows `null`),
`blockers` (`Vec<String>`, defaults to empty if the key is absent). Unknown/unused keys
(`description`, `type`, `branch`, `intent`, `contract`, `event_schema`, `pr`, `last_commit`,
`reconciliation_status`, `replay_status`) are silently ignored by serde's default behavior —
no struct field is needed for them. The top-level `architectural_refinements:` key is also
ignored the same way (matching the Step 1 scope boundary).
_Verify in Step 4:_ fixture registry with a full-schema entry (all fields populated) and one
with only the fields this tool reads; confirm both parse successfully and produce identical
dashboard output for the fields that matter.

**AC-2 — Active-feature selection, registry order preserved**
Only entries with `status: active` appear in the output, in the same order they appear in
the registry file (not sorted, not reordered).
_Verify in Step 4:_ fixture with `active`, `suspended`, `complete`, `blocked` entries
interleaved; confirm only `active` entries appear, in original order.

**AC-3 — Output structure: one heading, one subsection per active feature**
Output is exactly one `# Approval Dashboard` heading, followed by one `## <feature_id>:
<slug>` subsection per active feature (in AC-2's order), each containing exactly the six
fields in this order: `Active features:`, `Current stage:`, `Reviewer recommendation:`,
`Open blockers:`, `Next human decision:`, `Risk:`.
_Verify in Step 4:_ fixture with 3 active features; confirm one `# Approval Dashboard`
heading, three `## <feature_id>: <slug>` subsections in order, each with all six fields
present and correctly ordered.

**AC-4 — `[INFERRED]` fields, including edge-case values**
`Active features:` is the feature's own `feature_id`, tagged `[INFERRED]`. `Current stage:`
is `current_stage`'s value if present, or the literal string `not started` if `current_stage`
is `null` — both tagged `[INFERRED]` (a `null` stage is a known, derived fact, not an
unknown one, matching `generate-report`'s `(none) [INFERRED]` precedent for zero-diff).
`Open blockers:` lists each entry in `blockers` one per line if non-empty, or the literal
string `(none)` if the list is empty — both tagged `[INFERRED]`.
_Verify in Step 4:_ (a) a feature with `current_stage: 4` and two blockers; confirm both
values and the `[INFERRED]` tag; (b) a feature with `current_stage: null` and `blockers: []`;
confirm `not started [INFERRED]` and `(none) [INFERRED]`.

**AC-5 — `[FILL]` fields, always**
`Reviewer recommendation:`, `Next human decision:`, and `Risk:` are always `[FILL]` for every
active feature — the tool never attempts to derive them from the registry, `review-log.md`,
or any other source (per the Step 1 scope boundary explicitly excluding `review-log.md`
parsing).
_Verify in Step 4:_ same fixtures as AC-3/AC-4; confirm all three fields are `[FILL]` in
every subsection, regardless of registry content.

**AC-6 — Preamble present, verbatim**
Whenever at least one active feature is found, output begins with this banner verbatim:
```
> [INFERRED] fields were populated automatically from the feature registry — verify before
> submitting. [FILL] fields require human or model authorship. This dashboard is a navigation
> aid, not a decision record — the registry and change records remain authoritative.
```
_Verify in Step 4:_ assert the first three non-blank lines of stdout match this banner
verbatim.

**AC-7 — No active features found**
If the registry parses successfully but contains zero entries with `status: active`
(including an empty `features:` list), the tool writes nothing to stdout, writes
`error: no active features found in <path>` to stderr, and exits 0 — valid-but-empty, not a
usage failure (same philosophy as `generate-adr-candidates`'s AC-7).
_Verify in Step 4:_ (a) fixture with only non-active entries; (b) fixture with an empty
`features: []` list; confirm empty stdout, the stderr message naming the path, exit 0 in
both cases.

**AC-8 — Missing or unreadable `--registry` file**
If the path given to `--registry` does not exist or cannot be read, the tool writes nothing
to stdout, writes `error: cannot read registry file '<path>': <os error>` to stderr, and
exits 1 (`EXIT_USAGE`) — `--registry` is the sole required input, so there is nothing valid
to reason about without it.
_Verify in Step 4:_ `--registry does-not-exist.yaml`; confirm exit 1, stderr names the path.

**AC-9 — YAML parse error**
If the file exists and is readable but is not valid YAML, or does not deserialize into the
expected `Registry` shape (e.g. `features:` is not a list), the tool writes nothing to
stdout, writes `error: cannot parse registry file '<path>': <parse error>` to stderr
(distinct message from AC-8's), and exits 1 — same reasoning as AC-8: no valid input to
reason about.
_Verify in Step 4:_ (a) fixture with malformed YAML (e.g. unbalanced quotes); (b) fixture
where `features:` is a string instead of a list; confirm exit 1 and a stderr message
distinct from AC-8's in both cases.

**AC-10 — `--registry` is required**
`--registry <path>` is a required flag (not optional, like `generate-adr-candidates`'s
`--source`). Omitting it, or passing an unknown flag, is a clap usage error: exit 1, a usage
message on stderr, nothing on stdout.
_Verify in Step 4:_ smoke test `generate-approval-dashboard` with no `--registry`; confirm
exit 1.

**AC-11 — Output to stdout only**
All report content (banner + dashboard) is written to stdout only, never stderr. On a
successful non-empty run, stderr is empty. In the AC-7/AC-8/AC-9 cases, stdout is exactly
empty — no partial banner, no partial dashboard content.
_Verify in Step 4:_ (a) run against a fixture with active features; confirm stderr is empty;
(b) run each of the AC-7/8/9 cases; confirm stdout is exactly empty in each.

### Exit codes

**AC-12 — Exit 0 on success**
Any invocation that finds ≥ 1 active feature exits 0.
_Verify in Step 4:_ smoke test with a valid fixture; assert exit 0.

**AC-13 — Dispatch before config resolution**
`generate-approval-dashboard` runs without a configured provider, dispatching before
`config::resolve()` (same pattern as `check-drift` / `generate-report` /
`generate-adr-candidates`).
_Verify in Step 4:_ run in a temp repo with no provider config set up at all; confirm no
config-resolution error occurs.

### Idempotency

**AC-14 — Deterministic output**
Given an unchanged registry file, two invocations produce byte-for-byte identical stdout.
_Verify in Step 4:_ run twice against the same fixture; diff the outputs.

### Cross-reference integrity

**AC-15 — `architectural_refinements:` is never treated as a feature**
A registry file containing both `features:` and `architectural_refinements:` top-level keys
produces output derived only from `features:` — no refinement entry ever appears as, or is
mistaken for, a dashboard feature subsection.
_Verify in Step 4:_ fixture with both keys populated (including a refinement entry whose
`status`/`refine_id` could superficially resemble a feature entry); confirm the dashboard
output is unaffected by the `architectural_refinements:` content.

---

## Step 3 — Implement

### What was done

| File | Change |
|---|---|
| `tools/reviewer/Cargo.toml` | Added `serde_yaml = "0.9.34"` dependency (`cargo add serde_yaml`), matching the Step 1 design decision. |
| `tools/reviewer/Cargo.lock` | Mechanically updated by `cargo add`/`cargo build` to lock `serde_yaml` and its transitive dependencies (`itoa`, `ryu`, `unsafe-libyaml`). Not a separate design decision — a direct, expected side effect of the `Cargo.toml` change above. |
| `tools/reviewer/src/cmd/generate_approval_dashboard.rs` | New. `run()` reads `--registry`, deserializes via `serde_yaml::from_str` into a minimal `Registry { features: Vec<FeatureEntry> }` struct (only the fields the tool reads: `feature_id`, `slug`, `status`, `current_stage: Option<i64>`, `blockers: Vec<String>` — unknown keys and `architectural_refinements:` are ignored by serde's default behavior, no code needed for that). Filters to `status == "active"`, preserving order, and emits one `# Approval Dashboard` heading + one `## <feature_id>: <slug>` subsection per active feature. |
| `tools/reviewer/src/cmd/mod.rs` | `pub mod generate_approval_dashboard;` registered. |
| `tools/reviewer/src/main.rs` | Added `Commands::GenerateApprovalDashboard { registry }` variant (`registry` required `String`, satisfying AC-10); dispatched before `config::resolve()` (mirrors the three prior generators); added the unreachable post-config match arm. |
| `tools/reviewer/tests/smoke.rs` | Added 17 smoke tests (`smoke_dashboard_*`) covering AC-1 through AC-15. |

### Verification (AC-1 through AC-15)

`cargo build`: clean. `cargo test --test smoke`: **94 passed, 0 failed** (77 pre-existing + 17
new). No regressions.

One test-authoring bug was caught and fixed during this pass: several fixture YAML strings
used Rust's trailing-backslash line-continuation across multiple source lines
(`"...\n\` followed by an indented next line), which strips **all** leading whitespace from
the continued line — silently flattening the YAML's deliberate 2-/4-space nesting and
producing invalid YAML (`missing field 'slug'`). Fixed by switching those 7 fixtures to raw
string literals (`r#"..."#`) with real source-level newlines and indentation, which sidesteps
the escaping pitfall entirely. Confirmed against `templates/feature-registry.yaml` directly
(the real template, not just fixtures) before writing the test suite:
```
$ codeos-reviewer generate-approval-dashboard --registry templates/feature-registry.yaml
> [INFERRED] fields were populated automatically from the feature registry — verify before
...
## UPG-0000: example-feature

Active features: UPG-0000 [INFERRED]
Current stage: 1 [INFERRED]
Reviewer recommendation: [FILL]
Open blockers: (none) [INFERRED]
Next human decision: [FILL]
Risk: [FILL]
```
This also confirms AC-15 against real content: the template's `architectural_refinements:`
key produced no extraneous output.

### Scope check

No edits to `templates/feature-registry.yaml`, `dba-system.md`, `scripts/codeos-review.sh`,
or any other existing subcommand's behavior — matches the Step 1 scope boundary.

---

## Step 4 — Reconcile

### Acceptance criteria verification (fresh evidence)

| AC | Verified by | Result |
|---|---|---|
| AC-1 Registry parsing, minimal struct fields | `smoke_dashboard_full_vs_minimal_schema_identical_output`; live check against real `templates/feature-registry.yaml` | PASS |
| AC-2 Active-only, registry order | `smoke_dashboard_only_active_features_in_registry_order` | PASS |
| AC-3 Output structure | `smoke_dashboard_output_structure` | PASS |
| AC-4 `[INFERRED]` edge cases (`null` stage, empty blockers) | `smoke_dashboard_inferred_edge_cases` | PASS |
| AC-5 `[FILL]` always | `smoke_dashboard_fill_fields_always_present` | PASS |
| AC-6 Preamble verbatim | `smoke_dashboard_preamble_present` | PASS |
| AC-7 No active features (two cases) | `smoke_dashboard_no_active_features_only_non_active`, `smoke_dashboard_no_active_features_empty_list` | PASS |
| AC-8 Missing/unreadable registry | `smoke_dashboard_missing_registry_file` | PASS |
| AC-9 YAML parse error (two cases) | `smoke_dashboard_malformed_yaml`, `smoke_dashboard_wrong_shape_yaml` | PASS |
| AC-10 `--registry` required | `smoke_dashboard_registry_required` | PASS |
| AC-11 Stdout-only | `smoke_dashboard_stdout_only` | PASS |
| AC-12 Exit 0 on success | `smoke_dashboard_exit_zero_on_success` | PASS |
| AC-13 Dispatch before config resolution | `smoke_dashboard_no_provider_config_required` | PASS |
| AC-14 Deterministic output | `smoke_dashboard_deterministic_output` | PASS |
| AC-15 `architectural_refinements:` never a feature | `smoke_dashboard_architectural_refinements_never_treated_as_feature`; confirmed live against the real template (which has both keys) | PASS |

**Raw evidence:**
```
$ cargo build --release
    Finished `release` profile [optimized] target(s)

$ cargo test -- --test-threads=1
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out  (unit tests)
test result: ok. 94 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out  (smoke tests)
```
Note: the default parallel `cargo test` intermittently fails 1 unrelated unit test
(`config::tests::toml_overrides_default`) due to a pre-existing env-var race in
`config.rs` — untouched by this change, confirmed via `git diff --stat -- src/config.rs`
(empty), and confirmed deterministic (0 failures) with `--test-threads=1`. Filed as
follow-up **UPG-0040**, not fixed here.

```
$ git diff --stat -- templates/feature-registry.yaml dba-system.md scripts/codeos-review.sh
(empty — none of these files touched)

$ ./tools/reviewer/target/release/codeos-reviewer generate-approval-dashboard --registry templates/feature-registry.yaml
> [INFERRED] fields were populated automatically from the feature registry — verify before
...
## UPG-0000: example-feature
Active features: UPG-0000 [INFERRED]
Current stage: 1 [INFERRED]
Reviewer recommendation: [FILL]
Open blockers: (none) [INFERRED]
Next human decision: [FILL]
Risk: [FILL]
```
(The template's `architectural_refinements:` entry produces no extraneous output — live
confirmation of AC-15, not just the fixture-based test.)

### Cross-reference sweep

- `git diff tools/reviewer/src/main.rs | grep '^-'` shows zero removed lines — every existing
  subcommand's dispatch code is untouched, only new code added.
- Swept the repo for any other doc referencing `generate-approval-dashboard` or
  `reviews/approval-dashboard.md` (the backlog's suggested output path) outside this change's
  own files: none found — no stale reference to update elsewhere.

### Reviewer scope triage (Step 4 findings)

R1 (NO OBJECTION): no findings raised. All 15 ACs verified; the pre-existing `config.rs`
test race (filed as `UPG-0040`) was correctly treated as OUT-OF-SCOPE BACKLOG, not a blocker.

### Post-approval discovery (before final close-out)

While demonstrating the tool practically (2026-07-06), `generate-approval-dashboard` was run
against `/home/rimo/projects/FundFlow/features/registry.yaml` — a real downstream project's
actual registry, distinct from the canonical `templates/feature-registry.yaml` used in all
Step 2/3/4 verification. It failed to parse: FundFlow's registry lacks `slug` and
`current_stage`/`blockers` fields entirely, and uses a `stageN`-embedded `status` vocabulary
instead of `active`/`suspended`/`complete`/`blocked`.

This is **not an in-scope defect**: Step 1's scope boundary explicitly named
`templates/feature-registry.yaml`'s schema as "the input contract this tool reads," and
every acceptance criterion was verified against that schema (including live confirmation
against the template file itself, not only fixtures). UPG-0023 never claimed compatibility
with FundFlow's specific, independently-drifted registry shape. Filed as follow-up
**UPG-0041** — deliberately scoped to *decide* the schema reconciliation (six open questions
recorded there) rather than patch the dashboard tool or FundFlow's registry hastily.

### Outcome

All 15 ACs verified against the final artifacts (table above), including live confirmation
against the real `templates/feature-registry.yaml`. No in-scope blockers open. No scope
drift. Step 4 R1 NO OBJECTION; human APPROVE_STAGE recorded. Two follow-ups filed during
this change's lifecycle — `UPG-0040` (pre-existing test race, unrelated) and `UPG-0041`
(real-world registry schema drift, discovered post-implementation) — both deliberately
scoped as separate, undecided work rather than folded in here. Change record,
`status/self-development.md`, `status/roadmap.md`, `backlog/features.md`, and
`backlog/UPG-0023-approval-dashboard.md` updated to COMPLETE in this same pass, following
that approval.
