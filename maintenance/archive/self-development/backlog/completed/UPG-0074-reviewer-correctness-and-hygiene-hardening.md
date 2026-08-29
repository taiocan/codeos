---
feature_id: UPG-0074
slug: reviewer-correctness-and-hygiene-hardening
title: Reviewer Correctness and Hygiene Hardening
status: DONE — four items shipped 2026-08-29; a fifth (post-response validation) confirmed already implemented, no change needed
priority: P2
depends_on: []
related_features: [UPG-0069, UPG-0070, UPG-0071, UPG-0072, UPG-0073]
supersedes: []
superseded_by: []
---

# Upgrade: reviewer correctness and hygiene hardening

## Why this exists

A proposal to simplify the review process was received, backed by a corpus analysis of PlotSpot's
real review history (98 reviews under `.codeos/05-review/reviews/`). Before implementing it, three
parallel research passes verified every load-bearing claim against doctrine, policy, the reviewer
engine, and the PlotSpot corpus itself.

**The corpus numbers held up almost exactly**: 98 automated reviews, 197 review files, 13 MB of
evidence, 52 of 98 reviews at non-required stages, 77 findings, 2.04 hours total runtime — all
confirmed by direct recount. The cited F-0004 packet was real: 247,941 content bytes against a
50,000-byte budget, with `runtime_events.jsonl` passed in whole accounting for 116,861 bytes, 47% of
it. The cited round-identity bug (`REV__F-0004__8__R1` recorded where `R3` was substantively
correct) was real and had already required a manual log annotation to explain. The cited human-caught
defect (a decision recording "ranking evidence" as simultaneously permitted and prohibited) was a
genuine logic contradiction, not a stylistic nit. One number did not hold up: "37 human decisions" —
the log's own decision field is unfilled in all 98 records; the nearest analog is 26 approve-type
git commits, a different artifact.

**Two of the proposal's six recommendations were dropped** after verification showed they rested on
wrong premises and would have removed working, tested behavior:

- Restricting the CLI to four boundary names. Policy v2.md sets a default-timing floor, not a
  ceiling — nothing forbids an intermediate-stage review, and doctrine v3.md:98 explicitly permits a
  human to request one at any time. The proposal's four names also didn't match doctrine's own five
  (`purpose-approval`, `specification-approval`, `delivery-entry`, `final-acceptance`,
  `architecture-entry`).
- Simplifying the finding/concern/evidence schema. The classification field has five canonical
  values, each enforced with a dedicated regression test; the concern field has four, not three —
  the proposal's list omitted `UNCLASSIFIED`, the fail-closed default for an unparseable reply; the
  evidence grade has six. None of it is vestigial.

**Two more were drafted, then cut before shipping**, to keep this pass to the smallest set that
fixes demonstrated defects: a derived-status command and boundary-name CLI aliases. Both are
additive and low-risk, but neither makes a review faster or more reliable — they're navigation
ergonomics, not fixes. Candidates for a later, separately-scoped pass.

**Automatic compact-evidence construction — the real fix for a 5x-over-budget packet — is not part
of this pass and is not claimed as solved.** This is review *correctness and hygiene hardening*,
not the full review-process simplification the original proposal aimed at.

## What shipped

Reviewer engine: `dba/04-tools/reviewer/engine/src/{main,packet,precheck,log,cmd/review}.rs`.

**1. Pre-review preflight, warn-only.** Two checks added to the existing per-artifact precheck loop
in `cmd/review.rs::prepare()` (the same loop `check_no_unfilled_placeholders` etc. already run in):
a positional artifact matching `*runtime_events*.jsonl` is flagged with a suggestion to use
`--sha-only`/`--base` instead — the measured cause of the F-0004 bloat; and a file that changed in
the reviewed diff but isn't in the declared artifact set is flagged as unrelated-file drift. Both
warn; neither blocks — a full event log can occasionally be legitimately under review, so only the
opt-in budget-fail mode (below) actually refuses a run. A third originally-planned check — required
frontmatter fields "per `review-file.md`" — was dropped before being written: that template turned
out to be an optional, rarely-created process-measurement document, not an input-artifact schema,
so there was nothing real to check.

**2. Post-response validation — confirmed already implemented, nothing changed.** The plan assumed
gaps existed in how malformed Codex replies are handled. Direct inspection of
`assessment.rs::parse_findings` found otherwise: an unrecognized finding-block shape, an
off-canonical classification, and missing Evidence/Why/Required-action are all already counted
`unparsed` and already force `assessment_status: INCOMPLETE`, which already forces
`effective_concern` to at least `DO NOT ADVANCE` regardless of what the model claimed
(`assessment.rs:300-330`). An unparseable verdict line already falls back to `UNCLASSIFIED` with the
same effect. No code change was needed; inventing one to fill out a plan item would have been
duplicating existing, tested enforcement.

**3. Predecessor-review identity.** `--continues <review-id>` (optional; the no-flag path is
untouched). `log::resolve_continued_round` validates the reference against the log rather than
trusting the flag: the predecessor must exist under the same feature and stage (one lookup, scoped
by the same heading suffix `compute_review_round` already matches on), and its own predecessor
chain — if any — must terminate without cycling back on itself within the three-round budget. The
round used is the predecessor's own position-derived round, never wherever a longer chain
eventually terminates; a bug in an early draft conflated the two; both were positive-controlled and
seven unit tests cover the happy path, wrong feature, wrong stage, a nonexistent id, a genuine cycle,
and exceeding the budget. This enforcement is new and stricter than the ordinary path's — no code
anywhere currently caps the ordinary heading-count path at three rounds, only policy text does — and
that asymmetry is deliberate: `--continues` is a deliberate act with a moment to check; the ordinary
default derivation is not being changed by this upgrade.

**4. Opt-in packet-budget fail mode.** `CODEOS_PACKET_BUDGET_MODE=fail` (default unchanged: `warn`).
`packet.rs`'s existing over-budget branch — literally commented `// Budget check (warning only)` —
now refuses to build the packet when set, using the same diagnostic it already printed. No
downstream project is affected until it opts in.

**Bonus fix, discovered while building item 1, not separately planned:** `packet::glob_match` only
ever stripped a single leading or trailing `*` and compared the remainder literally. `PATH_EXCLUDES`'
own `"*runtime_events*.jsonl"` entry — two wildcards — has therefore never matched anything since it
was written; a bare `runtime_events.jsonl` reaching a packet via untracked-file auto-discovery was
never actually excluded. Confirmed independently in a standalone Python reproduction before touching
Rust. Generalized `glob_match` to handle any number of `*` segments; this restores behavior every
caller already assumed rather than introducing a new exclusion rule, so it is a conformance repair,
not a new decision. My own new `looks_like_events_log` helper reuses the same function and would not
have worked without this fix, which is how it was found.

## Verification

- 82 reviewer-engine unit tests (up from 71: 5 for the new precheck functions, 2 for `glob_match`/
  `looks_like_events_log`, 7 for `resolve_continued_round`), plus 3 new integration tests in
  `tests/plan_command.rs` (budget-fail-mode, artifact-hygiene-warns, artifact-hygiene-via-sha-only-
  does-not-warn). All pass; the whole engine test suite (8 targets) passes.
- The two most consequential new behaviors were positive-controlled: the budget-fail-mode test and
  the artifact-hygiene test were each confirmed to fail when their respective fix was temporarily
  removed, then confirmed to pass again once restored.
- A read-only dry run against PlotSpot's real `.codeos/05-review/reviews/` corpus, reconstructing
  the cited F-0004 packet's artifact list: the hygiene check fired correctly on
  `events/runtime_events.jsonl`, and the drift check fired correctly on nine files from PlotSpot's
  own concurrent, unrelated F-0005/F-0006/F-0007 work-in-progress — real organic drift, not a
  synthetic fixture. PlotSpot's working tree was unmodified by the run (same 19 pre-existing dirty
  entries before and after; packet and sidecar written to a scratch path outside the repo).
- The whole of `dba/04-tools/tests/run.sh` passes.

No approved boundary, required-reviewer policy, or doctrine adapter membership changed. Governance
class: NORMAL.
