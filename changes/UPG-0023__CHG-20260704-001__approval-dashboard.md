---
change_id: CHG-20260704-001
feature_id: UPG-0023
slug: approval-dashboard
triage_class: script-tooling
scope_axis: self-dev only
review_profile: PROFILE-3
review_series: RVS__UPG-0023__CHG-20260704-001__S1
review_state: DRAFT
status: IN_PROGRESS
loop_step: 1-Intent
---

# Change: UPG-0023 / CHG-20260704-001 — Human Approval Dashboard

## TRACE HEADER

```yaml
feature_id: UPG-0023
primary_feature_id: UPG-0023
change_id: CHG-20260704-001
slug: approval-dashboard
state: IN_PROGRESS
current_step: 1-Intent
implements:
  - UPG-0023
related_features:
  - UPG-0009
  - UPG-0003
review_series: RVS__UPG-0023__CHG-20260704-001__S1
review_profile: PROFILE-3
review_state: DRAFT
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

*(to be written after Step 1 approval)*

---

## Step 3 — Implement

*(to be written after Step 2 approval)*

---

## Step 4 — Reconcile

*(to be written after Step 3 approval)*
