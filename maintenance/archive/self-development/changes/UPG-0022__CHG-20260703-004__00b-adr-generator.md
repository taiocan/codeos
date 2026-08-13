---
change_id: CHG-20260703-004
feature_id: UPG-0022
slug: 00b-adr-generator
triage_class: script-tooling
scope_axis: self-dev only
review_profile: PROFILE-3
review_series: RVS__UPG-0022__CHG-20260703-004__S1
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0022 / CHG-20260703-004 — 00b → ADR Candidate Generator

## TRACE HEADER

```yaml
feature_id: UPG-0022
primary_feature_id: UPG-0022
change_id: CHG-20260703-004
slug: 00b-adr-generator
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0022
related_features:
  - UPG-0007
review_series: RVS__UPG-0022__CHG-20260703-004__S1
review_profile: PROFILE-3
review_state: ACCEPTED
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

### Functional I/O

**AC-1 — Section extraction boundary**
The tool locates the *first* line in the source file that is exactly equal to
`## Architectural Risks` after trimming trailing whitespace (case-sensitive, exact match —
not a prefix or substring match, so `## Architectural Risks (draft)` does not match). The
section spans from that line (exclusive) to the next line that starts with `## ` (any H2
heading), or EOF, whichever comes first. Only content in this span is scanned for risk
bullets. If the heading appears more than once, only the first occurrence's span is used.
_Verify in Step 4:_ fixture with the heading followed by bullets, then a later `## ` heading
with unrelated content; confirm only the first span's bullets are extracted and the later
section's content is not.

**AC-2 — Risk bullet extraction**
Within the section (AC-1), every line starting at column 0 with `- ` or `* ` (a literal
hyphen or asterisk, one space, then content) is one risk bullet. The candidate's identifying
text is that line's content after stripping the marker and surrounding whitespace, taken
verbatim — no markdown normalization, no re-wrapping. Indented or continuation lines (any
line not itself starting with `- `/`* ` at column 0) are not appended to the preceding
bullet's text and never form their own risk entries. A bullet whose content is empty after
stripping the marker (e.g. a bare `- ` with nothing else) is skipped — it does not produce a
risk entry.
_Verify in Step 4:_ fixture with (a) plain top-level bullets, (b) an indented continuation
line under a bullet, (c) a bare `- ` with no text, (d) intro prose before the first bullet.
Confirm exactly the expected bullets are extracted, in source order, with continuation lines
and prose ignored and the bare bullet skipped.

**AC-3 — Multiple risks: output structure and order**
For N risks found (N ≥ 1), the tool emits exactly one `# ADR Candidates` heading, followed by
N candidate groups in source order, each preceded by a `## Candidate <n>` subheading
(1-indexed, e.g. `## Candidate 1`, `## Candidate 2`, ...). Each group contains exactly these
seven fields plus the three-item route list, in this order:
```
Decision needed: <risk text> [INFERRED]
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
(The `## Candidate <n>` subheading is a Step 2 design addition beyond the single-example
block shown in Step 1's backlog-derived proposed artifact — needed to unambiguously delimit
multiple candidate groups in one document. It does not change any field's name or order.)
_Verify in Step 4:_ fixture with 3 risks; confirm one `# ADR Candidates` heading, three
`## Candidate N` subheadings in order 1/2/3, and each group's seven fields + route list
present, complete, and correctly ordered.

**AC-4 — `[INFERRED]` / `[FILL]` tagging**
`Decision needed:` is the only mechanically-derived field and is tagged `[INFERRED]`. All
other fields (`Why now`, `Features affected`, `Options`, `Risk if deferred`,
`Does this affect behavior`, `Recommended route`) are always `[FILL]` — the tool never
attempts to infer them from source content. No field is ever blank.
_Verify in Step 4:_ same fixture as AC-3; confirm exactly one `[INFERRED]` tag per candidate
group (on `Decision needed:`) and `[FILL]` on all six other fields, for every group.

**AC-5 — Preamble present**
Whenever at least one risk is found, output begins with this banner verbatim:
```
> [INFERRED] fields were populated automatically by extracting risk bullets from the source
> document — verify before submitting. [FILL] fields require human or model authorship.
> ADR candidates are non-authoritative until routed through Stage 1–3 or Stage 10.
```
_Verify in Step 4:_ assert the first three non-blank lines of stdout match this banner
verbatim, byte for byte.

**AC-6 — No `## Architectural Risks` section found**
If no line in the source file exactly matches `## Architectural Risks` (AC-1's match rule),
the tool writes nothing to stdout, writes
`error: no "## Architectural Risks" section found in <path>` to stderr, and exits 0 — this is
a valid-but-empty result, not a usage failure.
_Verify in Step 4:_ fixture with no such heading; confirm empty stdout, stderr names the
source path, exit code 0.

**AC-7 — Section found but no bullets**
If the `## Architectural Risks` section (AC-1) exists but contains zero valid risk bullets
(AC-2 — including a section containing only prose, or only a bare empty bullet), the tool
writes nothing to stdout, writes
`error: "## Architectural Risks" section in <path> contains no risk bullets` to stderr, and
exits 0.
_Verify in Step 4:_ fixture with the heading present but only prose beneath it; confirm empty
stdout, a stderr message distinct from AC-6's, exit code 0.

**AC-8 — Missing or unreadable `--source` file**
If the path given to `--source` does not exist or cannot be read, the tool writes nothing to
stdout, writes `error: cannot read source file '<path>': <os error>` to stderr, and exits 1
(`EXIT_USAGE`) — distinct from AC-6/AC-7 because there is no valid input to reason about at
all. (This intentionally diverges from `generate-report`'s optional-input graceful-`[FILL]`
pattern: `--source` is this tool's sole required input, not an auxiliary one — there is
nothing meaningful to generate without it.)
_Verify in Step 4:_ `--source does-not-exist.md`; confirm exit 1, stderr names the path.

**AC-9 — `--source` is required**
`--source <path>` is a required flag (not optional). Omitting it, or passing an unknown flag,
is a clap usage error: exit 1, a usage message on stderr, nothing on stdout.
_Verify in Step 4:_ smoke test `generate-adr-candidates` with no `--source`; confirm exit 1.

**AC-10 — Output to stdout only**
All report content (banner + candidate groups) is written to stdout only, never stderr. On a
successful non-empty run, stderr is empty. In the AC-6/AC-7/AC-8 cases, stdout is exactly
empty — no partial banner, no partial candidate content.
_Verify in Step 4:_ (a) run against a fixture with risks; confirm stderr is empty; (b) run
each of the AC-6/7/8 cases; confirm stdout is exactly empty in each.

### Exit codes

**AC-11 — Exit 0 on success**
Any invocation that finds ≥ 1 risk bullet exits 0.
_Verify in Step 4:_ smoke test with a valid fixture; assert exit 0.

**AC-12 — Dispatch before config resolution**
`generate-adr-candidates` runs without a configured provider, dispatching before
`config::resolve()` (same pattern as `check-drift` / `generate-report`).
_Verify in Step 4:_ run in a temp repo with no provider config set up at all; confirm no
config-resolution error occurs (exit 0 either via AC-6's no-section path or a successful
candidate-producing run).

### Idempotency

**AC-13 — Deterministic output**
Given an unchanged source file, two invocations produce byte-for-byte identical stdout.
_Verify in Step 4:_ run twice against the same fixture; diff the outputs.

### Cross-reference integrity

**AC-14 — Non-authoritative guardrail is inseparable from output**
The third preamble line ("ADR candidates are non-authoritative...") is part of the same
constant banner as the first two lines (AC-5) and is never conditionally omitted by any flag
or code path while candidate content is still emitted.
_Verify in Step 4:_ confirm the full banner (AC-5) appears in every test that produces
non-empty stdout; there is no separate flag or branch that prints candidates without it.

---

## Step 3 — Implement

### What was done

| File | Change |
|---|---|
| `tools/reviewer/src/cmd/generate_adr_candidates.rs` | New: `generate-adr-candidates` core. `run()` reads `--source`, locates the first exact `## Architectural Risks` line, bounds the section at the next `## ` heading or EOF, extracts top-level `- `/`* ` bullets (skipping empty-after-strip bullets, ignoring indented continuation lines and prose), and emits one `# ADR Candidates` heading followed by one `## Candidate <n>` group per risk in source order. |
| `tools/reviewer/src/cmd/mod.rs` | `pub mod generate_adr_candidates;` registered. |
| `tools/reviewer/src/main.rs` | Added `Commands::GenerateAdrCandidates { source }` variant (`source` a required `String`, so clap itself enforces AC-9); dispatched before `config::resolve()` (mirrors `check-drift` / `generate-report`, satisfies AC-12); added the unreachable post-config match arm. |
| `tools/reviewer/tests/smoke.rs` | Added 14 smoke tests (`smoke_generate_adr_*`) covering AC-1 through AC-14: section boundary (AC-1), bullet extraction incl. continuation/prose/bare-bullet (AC-2), multi-candidate structure and order (AC-3), `[INFERRED]`/`[FILL]` counts (AC-4), verbatim banner (AC-5), no-section / empty-section / missing-source distinct exit paths (AC-6/7/8), missing `--source` clap usage error (AC-9), stdout-only across success and all three empty/error cases (AC-10), exit 0 on success (AC-11), no-provider-config dispatch (AC-12), determinism (AC-13), guardrail present across single- and multi-candidate runs (AC-14). |

### Verification

`cargo build` and `cargo build --tests`: clean, no errors.
`cargo test --test smoke`: **77 passed, 0 failed** (63 pre-existing + 14 new). No regressions.
One test bug was caught and fixed during this pass: `smoke_generate_adr_inferred_and_fill_tagging`
initially counted `[INFERRED]` across the whole stdout, which double-counts the banner's own
use of that literal substring; fixed to count only within the `# ADR Candidates` section.

### Scope check

No edits to `prompts/00b-solution-discovery.md`, `dba-system.md`, `scripts/codeos-review.sh`,
or any other existing subcommand's behavior — matches the Step 1 scope boundary. `check-drift`
and `generate-report` dispatch/handling untouched.

---

## Step 4 — Reconcile

### Acceptance criteria verification

| AC | Verified by | Result |
|---|---|---|
| AC-1 Section extraction boundary | `smoke_generate_adr_section_boundary` | PASS |
| AC-2 Bullet extraction (continuation/prose/bare bullet) | `smoke_generate_adr_bullet_extraction` | PASS |
| AC-3 Multi-candidate structure and order | `smoke_generate_adr_multiple_candidates_structure` | PASS |
| AC-4 `[INFERRED]`/`[FILL]` tagging | `smoke_generate_adr_inferred_and_fill_tagging` | PASS |
| AC-5 Preamble present, verbatim | `smoke_generate_adr_preamble_present` | PASS |
| AC-6 No section found | `smoke_generate_adr_no_section_found` | PASS |
| AC-7 Section found, no bullets | `smoke_generate_adr_section_empty` | PASS |
| AC-8 Missing/unreadable `--source` | `smoke_generate_adr_missing_source_file` | PASS |
| AC-9 `--source` required | `smoke_generate_adr_source_required` | PASS |
| AC-10 Stdout-only (success + all 3 empty/error cases) | `smoke_generate_adr_stdout_only` | PASS |
| AC-11 Exit 0 on success | `smoke_generate_adr_exit_zero_on_success` | PASS |
| AC-12 Dispatch before config resolution | `smoke_generate_adr_no_provider_config_required` | PASS |
| AC-13 Deterministic output | `smoke_generate_adr_deterministic_output` | PASS |
| AC-14 Guardrail inseparable from output | `smoke_generate_adr_guardrail_inseparable_from_output` | PASS |

`cargo test --test smoke`: **77 passed, 0 failed** (verified again at Step 4).

### Cross-reference sweep

- `grep -rln "generate-adr-candidates\|GenerateAdrCandidates"` across `docs/`, `backlog/`, `prompts/`, `templates/` returns only `backlog/UPG-0022-00b-adr-generator.md` — matching the same precedent as `check-drift` (UPG-0020) and `generate-report` (UPG-0021): neither of those subcommands required a `docs/reviewer-pipeline.md` cross-reference either, since that doc documents the `review`/`decision` pipeline specifically, not a general subcommand list.
- `prompts/00b-solution-discovery.md`: untouched, confirmed via `git diff --stat` (empty output) — matches the Step 1 scope boundary.
- `dba-system.md`, `scripts/codeos-review.sh`: untouched, confirmed via `git status --short` (no output for either path).

### Reviewer scope triage (Step 4 findings)

R1 (NO OBJECTION, first round): no findings raised. The reviewer explicitly noted the Step 4
outcome text was "careful not to mark the change COMPLETE before the Step 4 review and human
gate" — confirming the AJ-013 discipline (established in UPG-0021) held on this change.

### Outcome

All 14 ACs verified against the final code and tests (table above). No in-scope blockers
open. No scope drift. Step 4 R1 NO OBJECTION; human APPROVE_STAGE recorded
(2026-07-03T16:38:52Z). Change record, `status/self-development.md`, `status/roadmap.md`,
`backlog/features.md`, and `backlog/UPG-0022-00b-adr-generator.md` updated to COMPLETE in
this same pass, following that approval.
