# Self-Development Change: UPG-0055__CHG-20260720-001 — reviewer-architecture-synthesis-stage-support

<!--
PURPOSE: Per-change source of truth for a non-trivial change to the Codeos toolkit
itself (prompts, templates, docs, patterns, scripts).

This is NOT a downstream DBA feature. It has no behavioral contract, no event schema,
and no replay. Trivial changes do not get a record.

Workflow: prompts/codeos-self-dev.md (4-step loop)
Each step requires explicit human approval; Codex review cadence is governed by the assigned review profile (see prompts/codeos-self-dev.md Step 0a).
The live status row lives in status/self-development.md, not here.
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0055
primary_feature_id: UPG-0055
change_id: CHG-20260720-001
slug: reviewer-architecture-synthesis-stage-support
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0055
related_features: [UPG-0051, UPG-0049]
review_series: RVS__UPG-0055__CHG-20260720-001__S4
review_profile: PROFILE-3
review_state: ACCEPTED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

<!-- SELF-REFERENCE BOUNDARY: this artifact is itself reviewed, so it must NOT embed the current
review round (which does not exist until after the packet is built). Reference the stable review
SERIES (review_series) + review_state; exact rounds live only in reviews/review-log.md and
reviews/codex/*. See prompts/codeos-self-dev.md → "Feature Thread & IDs" / "Self-Reference Boundary". -->


## Change Intent

**Why (problem in the toolkit):**

`codeos-reviewer` (`tools/reviewer/src/packet.rs`) accepts an arbitrary `stage` string, but its
`stage_expected`/`stage_checks` functions have no match arm for `"architecture-synthesis"` — the
stage id `UPG-0051` introduced. An invocation with that stage id falls through to the generic
fallback (`"(no expected-output template for stage)"` / `"(no stage-specific checklist for stage
{})"`) instead of a real checklist, the same way `discovery`, `brief`, `onboarding`, and `1`-`10`
already get one. `dba-system.md` currently documents this gap and directs use of the Review Waiver
mechanism as an interim path.

**What changes:**

1. `tools/reviewer/src/packet.rs`:
   - Add an `"architecture-synthesis"` match arm to `stage_expected` (line ~675, alongside `"10"`),
     summarizing `dba-system.md`'s "Multi-Feature Architecture Synthesis Gate" section and
     `prompts/03b-architecture-synthesis.md`'s 3-step pipeline in the same terse, single-sentence
     register as every existing entry.
   - Add a matching `"architecture-synthesis"` arm to `stage_checks` (line ~694), in the same
     terse `"  - clause; clause; clause."` register as every existing entry — observable review
     questions only (cohort declared, baseline version declared, every declared cohort member
     evaluated for architectural relevance, authoritative decisions distinguished from derived
     views, no synthesized behavior), never a restatement of implementation/tooling status like
     "Review Waiver no longer needed."
   - Extend the two existing tests that enumerate "new downstream identifiers" —
     `stage_expected_new_downstream_identifiers_are_real_not_placeholder` and
     `stage_checks_new_downstream_identifiers_are_real_not_placeholder` (both currently iterate
     `["discovery", "brief", "onboarding", "10"]`) — to also include `"architecture-synthesis"`.
     Unlike `UPG-0053`'s `dba-init.sh` change, this toolkit already has a real `cargo test` suite
     covering exactly this surface; this change uses it rather than a live scratch run.

**Scope boundary — what stays the same:**

- No change to any existing stage id's `stage_expected`/`stage_checks` text.
- No change to `dba-system.md` or `prompts/03b-architecture-synthesis.md` — this change only makes
  the reviewer aware of doctrine/prompt content that already exists (per `UPG-0055`'s own
  guardrail: `packet.rs` remains a consumer of doctrine, never a second authority — the new match
  arm text must not introduce any review criterion absent from those two files).
- No general external stage-policy mechanism (that's `UPG-0049`'s territory, explicitly out of
  scope here).
- No support added for any stage id other than `architecture-synthesis`.
- `stage_checks_unrecognized_identifier_still_falls_back_to_placeholder` (the negative-case test
  for a truly unknown stage id) is unaffected — a different, still-nonexistent stage string must
  continue falling back exactly as before.

**Class:** script-tooling
**Scope axis:** self-dev only
**Backlog item:** backlog/UPG-0055-reviewer-architecture-synthesis-stage-support.md

---

## Acceptance Criteria

| # | Criterion | How it will be verified |
|---|---|---|
| 1 | `stage_expected("architecture-synthesis")` returns real text, not the fallback placeholder. | Read the new match arm; `cargo test stage_expected_new_downstream_identifiers_are_real_not_placeholder` passes with `"architecture-synthesis"` added to its stage list. |
| 2 | `stage_checks("architecture-synthesis")` returns real text, not the fallback placeholder. | Read the new match arm; `cargo test stage_checks_new_downstream_identifiers_are_real_not_placeholder` passes with `"architecture-synthesis"` added to its stage list. |
| 3 | New match arm content summarizes only `dba-system.md`'s "Multi-Feature Architecture Synthesis Gate" section and `prompts/03b-architecture-synthesis.md` — introduces no review criterion absent from those two files (per `UPG-0055`'s own guardrail: `packet.rs` stays a consumer of doctrine, never a second authority). | Read-through cross-checking each clause in the new arms against the two source files. |
| 4 | No existing stage id's `stage_expected`/`stage_checks` text is changed. | `git diff -- tools/reviewer/src/packet.rs` shows only additions (new match arms + new test-array entries), zero modified lines in any existing arm. |
| 5 | Both "new downstream identifiers" tests extended to include `"architecture-synthesis"` in their stage arrays, without removing any existing entry (`discovery`, `brief`, `onboarding`, `10`). | Read the two test bodies; confirm all 5 stage strings present in each. |
| 6 | The negative-case fallback test (`stage_checks_unrecognized_identifier_still_falls_back_to_placeholder`) is unaffected — a genuinely unknown stage id still falls back exactly as before. | `git diff` shows this test body untouched; `cargo test stage_checks_unrecognized_identifier_still_falls_back_to_placeholder` passes. |
| 7 | Full existing test suite still passes — no regression introduced anywhere else in `packet.rs`. | `cargo test --manifest-path tools/reviewer/Cargo.toml` (or equivalent), full run, 0 failures. |
| 8 | New arm content matches the existing terse register: `stage_expected`'s entry is one sentence (semicolon-separated clauses, no line breaks); `stage_checks`'s entry follows the `"  - clause; clause; clause."` single-line format every other entry uses. | Visual comparison against the `"10"` entries in both functions. |
| 9 | Scope containment: only `tools/reviewer/src/packet.rs` is modified — no other reviewer source file, no `dba-system.md`, no prompt file. | `git status`/`git diff --stat` shows exactly one source file changed (plus expected self-dev bookkeeping in `status/self-development.md`). |

---

## Implementation Notes

<!-- Summary only — the git diff is the source of truth. -->

Single file touched: `tools/reviewer/src/packet.rs`. `git diff --stat` confirms 4 insertions/2
deletions — the two new match arms (additive) and the two test arrays' `"10"` → `"10",
"architecture-synthesis"` extensions (the only "deletions" are the old array literals being
replaced by the extended ones, exactly as planned).

New arm content traces directly to `dba-system.md`'s "Multi-Feature Architecture Synthesis Gate"
section (authoritative-decisions-vs-derived-views, cohort membership + version, behavioral gaps
return to earlier stage) and `prompts/03b-architecture-synthesis.md`'s exact 3-step names (Cohort
Evidence Review → Draft Baseline → Approval and Activation) — no invented criterion.

**Full `cargo test --manifest-path tools/reviewer/Cargo.toml` output** (corrects an earlier,
mistaken "35 tests" summary from truncated output — the real, verified total is 182):

```
$ cargo test --manifest-path tools/reviewer/Cargo.toml 2>&1 | grep "^test result:"
test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.77s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```
Sum: 182 tests, 0 failures across all 10 test binaries.

**The 4 directly relevant unit tests, isolated:**

```
$ cargo test --manifest-path tools/reviewer/Cargo.toml packet::tests::stage
running 4 tests
test packet::tests::stage_expected_numeric_1_to_9_unchanged_by_extension ... ok
test packet::tests::stage_expected_new_downstream_identifiers_are_real_not_placeholder ... ok
test packet::tests::stage_checks_unrecognized_identifier_still_falls_back_to_placeholder ... ok
test packet::tests::stage_checks_new_downstream_identifiers_are_real_not_placeholder ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 45 filtered out; finished in 0.00s
```

**Nothing was deferred or discovered out-of-scope during implementation.**

---

## Reconciliation

**Acceptance verification:**

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | `stage_expected("architecture-synthesis")` returns real text | PASS | New match arm at `packet.rs:676`; confirmed by `cargo test packet::tests::stage_expected_new_downstream_identifiers_are_real_not_placeholder ... ok`. |
| 2 | `stage_checks("architecture-synthesis")` returns real text | PASS | New match arm at `packet.rs:696`; confirmed by `cargo test packet::tests::stage_checks_new_downstream_identifiers_are_real_not_placeholder ... ok`. |
| 3 | Content traces only to `dba-system.md`/`prompts/03b-architecture-synthesis.md`, no invented criterion | PASS | Confirmed with full doctrine content in the Step 3 R3 review packet: Architecture Synthesis Gate, `core-baseline.md`, authoritative-vs-derived, cohort membership/versioning, and behavioral-gaps-return-to-earlier-stage all trace to `dba-system.md`; the 3-step pipeline names trace to `prompts/03b-architecture-synthesis.md`. |
| 4 | No existing stage id's text modified | PASS | `git diff --stat -- tools/reviewer/src/packet.rs`: 4 insertions, 2 deletions — the 2 "deletions" are the two test-array literals being extended (accounted for by AC5), zero changes to any existing match arm's return text. |
| 5 | Both test arrays extended, no entry removed | PASS | Both arrays now read `["discovery", "brief", "onboarding", "10", "architecture-synthesis"]`. |
| 6 | Negative-case fallback test unaffected | PASS | `stage_checks_unrecognized_identifier_still_falls_back_to_placeholder` body untouched in the diff; `cargo test` confirms it still passes. |
| 7 | Full test suite passes, no regression | PASS | `cargo test --manifest-path tools/reviewer/Cargo.toml`: **182 tests, 0 failures** across all 10 test binaries (full output embedded in Implementation Notes above). |
| 8 | New arm register matches existing style | PASS | Both new entries are single-line, semicolon-clause format, same length register as the `"10"` entries immediately preceding them. |
| 9 | Scope containment: `packet.rs` only | PASS | `git status --short`: only `tools/reviewer/src/packet.rs` plus expected self-dev bookkeeping (`status/self-development.md`, `reviews/review-log.md`, new `changes/`/`reviews/codex/` files) — no doctrine or prompt file touched. |

All 9 criteria PASS.

**Consistency sweep:** no orphaned links; no new file added that needed a cross-reference beyond
the two doctrine sources already confirmed for AC3. One **known, deliberately out-of-scope stale
reference** exists and is *not* fixed by this change (see findings table below): `dba-system.md`'s
"Multi-Feature Architecture Synthesis Gate" section still says `codeos-reviewer` has no dedicated
checklist for this stage id — true before this change, false after it ships, since this change is
exactly what makes it false. Correcting that sentence is outside this change's declared scope
(`dba-system.md` is explicitly not touched — see Scope boundary), so it is recorded here as an
expected follow-up, not silently left inconsistent without acknowledgment.

**Findings scope-triage:**

| Finding | Triage | Action |
|---|---|---|
| Step 3 R1: AC3 (doctrine alignment) and AC7 (test pass) claimed but unverifiable from the packet — neither doctrine files nor test output were included/embedded | IN-SCOPE BLOCKER | Fixed — embedded actual `cargo test` output (also corrected a wrong "35 tests" claim to the real 182); doctrine files added to the next review round |
| Step 3 R2: `--sha-only` for `dba-system.md`/the 03b prompt proves identity, not content — AC3 still unverifiable | IN-SCOPE BLOCKER | Fixed — re-ran with full file content for both doctrine files; R3 confirmed AC3 |

| Discovered during Step 4: `dba-system.md`'s "Multi-Feature Architecture Synthesis Gate" section documents the reviewer gap and directs use of the Review Waiver in the interim — that note becomes stale the moment this change ships (this change is exactly what retires the gap it describes) | OUT-OF-SCOPE BACKLOG | Not fixed here — outside this change's declared scope (no `dba-system.md` edits). Trivial one-line correction, analogous to the one made during `UPG-0054`, to be applied as a direct edit once this change is accepted — not filed as its own UPG. |

---
