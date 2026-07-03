---
change_id: CHG-20260703-004
feature_id: UPG-0022
slug: 00b-adr-generator
triage_class: script-tooling
scope_axis: self-dev only
review_profile: PROFILE-3
review_series: RVS__UPG-0022__CHG-20260703-004__S1
review_state: DRAFT
status: IN_PROGRESS
loop_step: 1-Intent
---

# Change: UPG-0022 / CHG-20260703-004 — 00b → ADR Candidate Generator

## TRACE HEADER

```yaml
feature_id: UPG-0022
primary_feature_id: UPG-0022
change_id: CHG-20260703-004
slug: 00b-adr-generator
state: IN_PROGRESS
current_step: 1-Intent
implements:
  - UPG-0022
related_features:
  - UPG-0007
review_series: RVS__UPG-0022__CHG-20260703-004__S1
review_profile: PROFILE-3
review_state: DRAFT
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

---

## Step 1 — Change Intent

### Problem

`prompts/00b-solution-discovery.md` (UPG-0007) produces a `## Architectural Risks` section
listing named design risks discovered during pre-Stage-1 domain exploration. Today those
risks just sit in a planning document — there is no mechanical step turning "we noticed
this risk" into a routable ADR candidate, so risks are easy to lose between a discovery
session and someone actually deciding what to do about them (the UPG-0022 backlog brief's
stated problem: "Expanded 00b may identify architecture risks but they can be lost").

### What changes

| File | Change |
|---|---|
| `tools/reviewer/src/cmd/generate_adr_candidates.rs` | New: `generate-adr-candidates` subcommand |
| `tools/reviewer/src/cmd/mod.rs` | Register `generate_adr_candidates` module |
| `tools/reviewer/src/main.rs` | Add `GenerateAdrCandidates` variant; dispatch before config resolution (same pattern as `check-drift` / `generate-report`) |
| `tools/reviewer/tests/smoke.rs` | Smoke tests |
| `backlog/UPG-0022-00b-adr-generator.md` | Feature Thread: CHG-20260703-004 activated (done) |
| `status/self-development.md` | Row activated (done) |
| `status/roadmap.md` | UPG-0022 → IN_PROGRESS (done) |

### Scope boundary — what stays the same

- `prompts/00b-solution-discovery.md` — not modified. The `## Architectural Risks` section
  format it already documents (a Markdown section containing risk bullets) is the input
  contract this tool reads; the prompt itself is not touched.
- `dba-system.md` — not touched.
- No existing subcommand's behavior changed (`review`, `decision`, `diagnose`, `stage-start`,
  `check-drift`, `generate-report` all untouched).
- `scripts/codeos-review.sh` — not touched (shim passes through automatically).
- No change to what counts as an "approved" artifact: ADR candidates remain explicitly
  non-authoritative output (per the backlog brief's Guardrail), same as 00b discovery output
  itself is already labeled CANDIDATE/HYPOTHESIZED and non-authoritative.

### Design intent

Follows the same mechanical-inference-plus-explicit-authorship-marker pattern established by
`generate-report` (UPG-0021): mechanically extractable content is tagged `[INFERRED]`; content
requiring judgment is tagged `[FILL]`. Nothing here becomes an approved decision — the tool's
own output states this explicitly (see banner below), matching the backlog brief's Guardrail
("Candidates only; non-authoritative until routed through Stage 1–3 or Stage 10").

`codeos-reviewer generate-adr-candidates --source <path>`

**Input contract:** `--source <path>` names a Markdown file (a 00b Solution Discovery
document). The tool locates the `## Architectural Risks` section (from that heading to the
next `## ` heading or EOF) and reads each top-level bullet line (`- ` or `* ` at column 0
within that section) as one named risk. Sub-bullets/continuation lines are not treated as
separate risks — the single top-level bullet's raw text is treated as one risk entry's
identifying text; nested detail is out of scope for this generator (a human/model still
elaborates the ADR candidate's non-mechanical fields).

**Output:** one ADR Candidate block per risk found, in source order, each shaped per the
UPG-0022 backlog brief's Proposed artifact:

```markdown
# ADR Candidates

Decision needed: <risk bullet text> [INFERRED]
Why now: [FILL]
Features affected: [FILL]
Options: [FILL]
Risk if deferred: [FILL]
Does this affect behavior: [FILL]
Recommended route: [FILL]
- Stage 1–3
- Stage 10
- no action yet
```

Every generated output opens with:
```
> [INFERRED] fields were populated automatically by extracting risk bullets from the source
> document — verify before submitting. [FILL] fields require human or model authorship.
> ADR candidates are non-authoritative until routed through Stage 1–3 or Stage 10.
```
(exact wording to be finalized in Step 2 — the third line is the non-negotiable guardrail
carry-through from the backlog brief; it must survive verbatim into the generated artifact
so a reader of the *generated output alone*, without the backlog brief in hand, cannot
mistake a candidate for an approved decision.)

**No risks found:** if the `## Architectural Risks` section is absent or contains no
top-level bullets, the tool does not fabricate a candidate. It prints a message to that
effect (exact stdout/stderr placement and exit code to be pinned down as an AC in Step 2)
rather than emitting an empty or placeholder ADR block.

Output is written to stdout (redirect to file). Dispatched before `config::resolve()` (no
provider config required — same as `check-drift` and `generate-report`).

### Triage

- Class: `script-tooling`
- Scope axis: `self-dev only`
- Review profile: `PROFILE-3`
- Originating backlog id: `UPG-0022`

---

## Step 2 — Acceptance Criteria

*(to be written after Step 1 approval)*

---

## Step 3 — Implement

*(to be written after Step 2 approval)*

---

## Step 4 — Reconcile

*(to be written after Step 3 approval)*
