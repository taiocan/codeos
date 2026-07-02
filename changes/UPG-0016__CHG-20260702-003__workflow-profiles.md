---
change_id: CHG-20260702-003
feature_id: UPG-0016
slug: workflow-profiles
triage_class: documentation
scope_axis: self-dev only
review_profile: PROFILE-2
review_series: RVS__UPG-0016__CHG-20260702-003__S4
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0016 / CHG-20260702-003 — Workflow Profiles

## TRACE HEADER

```yaml
feature_id: UPG-0016
primary_feature_id: UPG-0016
change_id: CHG-20260702-003
slug: workflow-profiles
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0016
related_features: []
review_series: RVS__UPG-0016__CHG-20260702-003__S4
review_profile: PROFILE-2
review_state: DRAFT
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

---

## Step 1 — Change Intent

### Problem

DBA projects have no documented guidance for branch/PR/CI discipline. Stage 9 says "commit
and push" — adequate for small solo work, but as features grow there is no model for:

- When to create a feature branch vs commit directly to main
- When to open a draft PR, and what CI should run
- How to split large or risky features so the reviewer agent gets digestible diffs
- What evidence the reviewer agent should be given access to at review time

Without documented profiles, every project invents its own discipline ad hoc, and the
reviewer agent never gets the full diff needed to detect scope drift or hidden behavior.

### What changes

| File | Change |
|---|---|
| `docs/workflow-profiles.md` | New: three workflow profiles (A — simple local, B — branch-per-feature, C — split PRs) with selection guidance and reviewer-agent access model |
| `backlog/UPG-0016-workflow-profiles.md` | Feature Thread updated |
| `status/self-development.md` | Row activated |
| `status/roadmap.md` | UPG-0016 → IN_PROGRESS |

### Scope boundary — what stays the same

- `dba-system.md` — not touched; downstream doctrine unchanged
- Existing Stage 9 behavior — profiles are optional, not mandatory; Stage 9 commit/push
  is still valid for small work
- All other docs, prompts, templates, scripts — no changes; no cross-references needed
- Reviewer pipeline scripts (`codeos-review.sh`) — the "reviewer agent access" checklist
  in the doc is design guidance only, not a script change (that belongs to UPG-0014)
- `dba-system.md` cross-references — none required; this is an additive doc

### What is explicitly deferred (scope boundary)

- Automating branch creation or PR opening (tooling, UPG-0014 area)
- CI configuration templates or GitHub Actions setup
- Per-project `.codeos/workflow-profile.yml` config file (future)
- Mandatory profile selection in `dba-init.sh`

### Triage

- Class: `documentation` (normative)
- Scope axis: `self-dev only`
- Review profile: `PROFILE-2` (Codex review at Reconcile only, max 2 rounds; Step 1–3
  gates require human approval but no intermediate Codex review)
- Originating backlog id: `UPG-0016`

---

## Step 2 — Acceptance Criteria

Each criterion is independently verifiable against `docs/workflow-profiles.md`.

### AC-1 — All three profiles are present and complete

The doc defines exactly three profiles (A, B, C), each with:
- A name and use-when guidance
- A concrete workflow sequence (what steps to take, in what order)
- No mandatory language that overrides the existing Stage 9 commit/push default

### AC-2 — Profile selection guidance is clear and non-contradictory

The doc provides guidance on when to use each profile (not just what each profile does).
Guidance must be consistent — no two profiles claim the same "use when" condition.

### AC-3 — Reviewer-agent access model is documented

The doc specifies what the reviewer agent should be given access to when running in a
branch/PR context. The checklist is marked as guidance (informational), not a pipeline
contract or script requirement.

### AC-4 — No mandatory policy language

No profile may be described as required, mandatory, or the only correct approach for any
category of work. The current Stage 9 commit/push-to-main flow must remain explicitly valid
for small work (per the backlog brief guardrail).

### AC-5 — No downstream doctrine touched

`dba-system.md` is not modified. No stage descriptions, vocabulary, or behavioral contracts
in the downstream doctrine are altered. Verified by `git diff` showing no change to
`dba-system.md`.

### AC-6 — No broken internal references

Every file path, feature ID, or section link mentioned inside `docs/workflow-profiles.md`
either exists in the repo or is clearly marked as a future artifact.

### AC-7 — Deferred scope is absent from the doc

The following are NOT present in the doc:
- Per-project `.codeos/workflow-profile.yml` config schema
- `dba-init.sh` profile selection prompts
- GitHub Actions or CI configuration snippets
- Any claim that profile selection is automated

---

## Step 3 — Implementation

### File created

`docs/workflow-profiles.md` — new doc, ~140 lines. Sections:
- Selection table (A/B/C with use-when conditions)
- Profile A: simple local flow
- Profile B: branch per feature (recommended default); explains one-commit-per-stage rationale
- Profile C: split PRs table (PR 1–4)
- Reviewer-agent access model + checklist (marked as guidance, not pipeline contract)
- "What Profile B Does Not Require" (no mandatory PR template, CI, or automated branch creation)
- "Relationship to Existing Workflow" (explicit: profiles are optional; DBA gates unchanged)

### AC pre-check

| AC | Result |
|---|---|
| AC-1 | 3 profiles present, each with use-when + sequence ✅ |
| AC-2 | Distinct conditions: A=solo/small/low-risk, B=3+ stages/CI/review, C=large/risky/multi-subsystem ✅ |
| AC-3 | Reviewer-agent section present; checklist explicitly "guidance...not enforced by the pipeline" ✅ |
| AC-4 | "What Profile B Does Not Require" section; "profile selection is a human decision...not enforced by any tool"; Stage 9 explicitly preserved ✅ |
| AC-5 | `git diff dba-system.md` = 0 lines ✅ |
| AC-6 | No file paths or links referencing non-existent artifacts ✅ |
| AC-7 | No config schema, no `dba-init.sh`, no GitHub Actions, no automation claims ✅ |
