# Self-Development Change: UPG-0046__CHG-20260713-001 — reviewrun-structured-records

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0046
primary_feature_id: UPG-0046
change_id: CHG-20260713-001
slug: reviewrun-structured-records
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0046
related_features: [UPG-0001, UPG-0029, UPG-0045, UPG-0047]
review_series: RVS__UPG-0046__CHG-20260713-001__S4
review_profile: PROFILE-3
review_state: REVIEWED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

## Change Intent

**Why (problem in the toolkit):**

`backlog/UPG-0046-reviewrun-structured-records.md` frames this as "what a `ReviewRun` actually
is" — a new structured record — with three open questions left for the implementer. Having read
`tools/reviewer/src/log.rs` and `src/assessment.rs`, plus the actual history behind the
`REV__…__S<N>__R<N>` id, the real shape of this change is narrower and more concrete than "design
a new record type":

- **`round` and `review_id` do not exist anywhere in the tooling today.** Grepping
  `tools/reviewer/src/*.rs` for `round`/`REV__`/`review_id` returns nothing. Every REVIEW entry
  in `reviews/review-log.md` is just `## <timestamp> REVIEW — <feature> — Stage <stage>` — no
  round number, no stable id. Every "R1"/"R2"/"R3" in this session's own change records
  (`UPG-0044`, `UPG-0045`) was a **human count**, done by eye, of matching log entries. That is
  exactly the reconstruction-by-hand problem this backlog item names.
- **This is a known, explicitly deferred gap, not a new idea.** `backlog/UPG-0001-feature-thread-traceability.md`
  established `REV__UPG-####__CHG-…__S<N>__R<N>` as a **documented manual convention only**
  (line 138, with the explicit note it "does not rename review files and does not change
  `scripts/codeos-review.sh` behavior"). `backlog/UPG-0029-review-naming-and-thread-tooling.md`
  then filed teaching the reviewer to *derive and emit* that id as an explicitly optional,
  deferred follow-up — "intentionally left manual until the convention proves itself... do not
  add mandatory tooling unless the manual convention proves insufficient." This session's own
  `UPG-0044` change is concrete evidence it has: three separate sections of that change record
  independently transcribed "what R1 found, R2 found" by hand, and one of those transcriptions
  drifted before being caught by Codex review.
- Given that, **Q3 of this backlog brief's own framing applies directly**: "If the existing
  artifacts already satisfy addressability once given a name, this upgrade may reduce to 'assign
  and cross-reference `REV__…__R<N>` ids consistently' rather than a new file format — that's a
  legitimate, smaller outcome." This change takes exactly that smaller outcome, not the
  `reviews/runs/<id>/` new-file-format sketch.

**What changes:**

- `tools/reviewer/src/log.rs` — new `compute_review_round(log_path, feature, stage) -> Result<u32>`:
  counts existing `## ... REVIEW — {feature} — Stage {stage}` entries in
  `reviews/review-log.md` matching this exact feature+stage, returns count + 1. No new storage;
  derived entirely from the existing append-only log. `append_review()` gains a `review_id: &str`
  parameter and writes a new `Review ID: {review_id}\n` line into the REVIEW entry (placed after
  the `## ... REVIEW —` header line).
- `tools/reviewer/src/assessment.rs` — `write_assessment()` gains a `review_id: &str` parameter,
  adds `review_id: {review_id}` as a new top-level YAML frontmatter field (additive to the
  existing schema).
- `tools/reviewer/src/cmd/review.rs` — computes `review_id` **once**, before either write (it
  currently calls `write_assessment` then `append_review` in sequence — both must see the same
  id, computed from the log state *before* this round's entry is appended, to avoid a round
  counting itself).
- `docs/reviewer-artifact-schemas.md` — add `review_id` to the v0 normative assessment-frontmatter
  schema and the REVIEW log-entry schema (additive fields, same file both are already defined in).
- `docs/reviewer-pipeline.md` — update the "manual convention only" language (from `UPG-0001`/
  `UPG-0029`, currently unchanged since those) to state the id is now mechanically derived and
  emitted, closing the gap those two features explicitly deferred.
- `tools/reviewer/tests/review_id.rs` (new) — round=1 on a feature+stage's first review; round
  increments correctly across repeated reviews of the same feature+stage; round counting is
  scoped per feature+stage (a different stage or feature starts back at R1); `review_id` is
  present and consistent between the assessment frontmatter and the log entry for the same round.

**Two design decisions, flagged for approval rather than assumed:**

1. **The `S<N>` step-number shorthand is dropped; the raw `--stage` argument is used verbatim.**
   `UPG-0001`'s `S<N>` grammar (`REV__UPG-0000__CHG-20260627-001__S4__R2`) was written for
   self-dev steps 1–4 specifically and has **no defined mapping** for downstream DBA stage ids
   (`discovery`, `brief`, `onboarding`, `1`..`10` — `UPG-0001` explicitly excluded
   "downstream-doctrine changes" from its own scope). Rather than inventing an unreviewed mapping
   now, `review_id` uses the literal stage string the caller already passes:
   `REV__UPG-0045__CHG-20260712-002__selfdev-step-3__R2` instead of
   `REV__UPG-0045__CHG-20260712-002__S3__R2`. This is unambiguous, needs no new mapping table,
   and works identically for self-dev and downstream out of the box. **This is a deliberate
   deviation from the brief's illustrative sketch and from `UPG-0001`'s literal grammar string —
   flagging for explicit approval, not assuming it.**
2. **Filenames are not renamed.** `reviews/codex/<ts>-<feature>-stage-<stage>-<sha>.md`/`.packet.txt`
   keep their current timestamped shape. `review_id` becomes a *content* field (frontmatter line
   + log line), not a filename convention change. `UPG-0029` explicitly scoped file renaming as
   its own, separate, "non-destructive where practical" migration step — conflating it with id
   assignment here would add real risk (chronological sortability loss, potential collision
   handling) for no addressability gain, since the id is already discoverable inside each file.
   Renaming remains available as an explicitly separate future change if ever wanted.

**Scope boundary — what stays the same:**

- No new file format, no `reviews/runs/` directory — per Q3's explicitly-permitted smaller
  outcome.
- No structured findings (`findings.yaml` or equivalent) — that is `UPG-0047`, which depends on
  this change existing first (a finding needs a stable `review_id` to attach to; after this
  change, one exists).
- No event-sourcing, no `reviews/review-events.jsonl` — that is `UPG-0048`, explicitly out of
  scope and not a prerequisite either direction.
- No change to the append-only guarantee, the committed/durable-vs-scratch classification
  (`UPG-0029`), or the Self-Reference Boundary (`UPG-0001`: change records still carry a stable
  `review_series` + `review_state`, never a live round — this change makes the *live round itself*
  mechanically identifiable for the first time, inside `reviews/`, without touching how change
  records reference it).
- No rename of any existing `reviews/codex/*` file (see design decision 2 above).
- No change to `scripts/codeos-review.sh` expected — the static locator shim passes arguments
  through verbatim (`docs/reviewer-pipeline.md` §10); will be confirmed, not assumed, at Step 4.
- No change to `CLAUDE.md` or `dba-system.md`.

**Class:** script-tooling
**Scope axis:** self-dev only
**Backlog item:** backlog/UPG-0046-reviewrun-structured-records.md

---

## Acceptance Criteria

**`review_id` format (confirmed at the gate):** `REV__<feature>__<stage>__R<N>`, where `<feature>`
is the literal `--feature`/positional argument value (already the combined
`UPG-####__CHG-YYYYMMDD-NNN` string in every self-dev invocation) and `<stage>` is the literal
`--stage`/positional argument value, unmodified — no `S<N>` conversion.
Example: `REV__UPG-0046__CHG-20260713-001__selfdev-step-1__R1`.

**Round-counting rule (the human's flagged danger — defined precisely, not left implicit):**
`compute_review_round` counts lines in `reviews/review-log.md` where
`line.starts_with("## ") && line.ends_with(&format!(" REVIEW — {feature} — Stage {stage}"))`
(matched against `content.lines()`, which strips the trailing newline — so `ends_with` on this
exact, `—`/newline-bounded suffix cannot confuse `Stage 1` with `Stage 10`, or one feature id
with a longer one sharing a prefix). Round = matches + 1.

| # | Criterion | How it will be verified |
|---|---|---|
| AC-1 | Round number is computed from existing `reviews/review-log.md` history for the same feature + stage. | Code inspection: `compute_review_round` reads `reviews/review-log.md` and counts matching `## ... REVIEW — {feature} — Stage {stage}` lines per the rule above. |
| AC-2 | First review for a feature+stage receives `R1`; later reviews increment `R2`, `R3`, … | `two_sequential_review_cycles_increment_the_round` (`log.rs`): drives the exact compute-round → format-id → append-entry sequence `review.rs::run()` uses, twice in a row, on a real appended log — asserts `R1` then `R2` then a third read reporting `R3`. This exercises the real read-after-write cycle, not just a hand-seeded log. |
| AC-3 | Computed `review_id` is written into the raw assessment YAML frontmatter. | `smoke_review_id_in_assessment_frontmatter`: grep the written `.md` assessment file for `review_id: REV__...`. |
| AC-4 | Computed `review_id` is written into the `reviews/review-log.md` REVIEW entry. | `smoke_review_id_in_log_entry`: grep the appended log entry for a `Review ID: REV__...` line. |
| AC-5 | No existing review filenames are renamed. | `git diff --stat` shows no renamed/deleted files under `reviews/codex/`; the timestamped `<ts>-<feature>-stage-<stage>-<sha>.md`/`.packet.txt` shape is unchanged in `assessment.rs`/`review.rs`. |
| AC-6 | No new `reviews/runs/` (or other new storage) format is introduced. | `git status --short` for this change shows no new top-level directory under `reviews/`; only new fields in existing files/functions. |
| AC-7 | The raw stage string is used exactly as passed to the review command; no `S<N>` conversion. | Code inspection: `review_id` construction uses `opts.stage`/`packet.stage` verbatim, no numeric-stage-to-`S<N>` mapping function exists anywhere in the diff. |
| AC-8 | Existing review behavior — packet construction, Codex invocation, evidence modes, exit codes — is unchanged. | Full `cargo test` suite passes unchanged (159/159 baseline + new tests), zero modifications to any pre-existing test; `git diff` on `packet.rs` (if touched at all) is empty — this change only touches `log.rs`/`assessment.rs`/`review.rs`'s call-site wiring. |
| AC-9 | Tests cover: R1 creation, R2 increment, different stages not colliding, no filename changes. | Unit tests in `log.rs`/`assessment.rs` (see Implementation Notes for the full list) enumerate exactly these cases: `round_one_when_log_missing`/`two_sequential_review_cycles_increment_the_round` (R1/R2), `round_does_not_collide_across_similarly_named_stages`/`_features_sharing_a_prefix` (no collision), and `write_assessment_includes_review_id_in_frontmatter`'s filename assertions (no rename to `REV__...`, legacy `<ts>-<feature>-stage-<stage>-<sha>` shape preserved) — plus AC-3/AC-4 (frontmatter + log presence) and AC-10 (below). |
| AC-10 | Round-counting failure modes are explicit and safe, never silent: (a) log file does not yet exist → round 1 (fresh log, 0 matches, not an error); (b) log file exists, 0 matches for this feature+stage → round 1; (c) log file exists but cannot be **read** (I/O error distinct from not-found) → fail closed: abort before any Codex invocation, print a clear diagnostic naming the log path and the error, exit `EXIT_WRITE` (5, the existing constant already used for read/write-path failures in this codebase) — never silently stamp `R1` or guess a round on a read failure. | `smoke_review_id_no_prior_log_is_r1` (case a — fresh temp repo, no log yet); `smoke_review_id_empty_log_is_r1` (case b — log exists, header only); `smoke_review_id_unreadable_log_fails_closed` (case c — log path replaced with a directory or chmod'd unreadable, assert non-zero exit, `EXIT_WRITE`, and no assessment/packet files were written — the round-counting fs read happens before `write_assessment`/`append_review`, so an early abort prevents any partial write). |
| AC-11 | A `--scratch` review (local-only, per `docs/reviewer-pipeline.md` §4a) does not pollute round-counting for the durable log, and vice versa. | Code inspection: `compute_review_round` reads whichever `review_log_path` the caller already resolved (`cfg.review_log` vs. the `_scratch` path per `args.scratch` — existing logic in `review.rs`, unchanged) — scratch and durable rounds are counted from separate files, exactly mirroring the existing scratch/durable separation. `smoke_review_id_scratch_and_durable_independent` confirms a scratch review does not affect the durable log's next round number. |

**Class note:** `script-tooling` — AC-8/AC-10 are this class's required I/O-behavior and
fail-closed-case contract; AC-2/AC-9 double as the idempotency-adjacent "repeated invocation"
contract (each call is a fresh, correctly-incrementing read of durable state, not cached).

---

## Implementation Notes

All edits landed as scoped, with one file-location deviation from Step 1's literal list
(explained below) and one small cross-reference fix discovered during the sweep.

- **`tools/reviewer/src/log.rs`** — added `compute_review_round()` (reads the log, counts exact
  `ends_with(" REVIEW — {feature} — Stage {stage}")` matches, returns count+1; missing log →
  `Ok(1)`; unreadable-but-existing log → `Err`, propagated) and `format_review_id()` (verbatim
  stage, no `S<N>` conversion). `append_review()` gained a `review_id: &str` parameter and now
  writes `Review ID: {review_id}\n` immediately after the `## ... REVIEW —` header line.
- **`tools/reviewer/src/assessment.rs`** — `write_assessment()` gained a `review_id: &str`
  parameter; writes `review_id: {review_id}` as the **first** frontmatter key (additive).
- **`tools/reviewer/src/cmd/review.rs`** — computes `review_id` once, via
  `compute_review_round(&review_log_path, ...)`, placed right after the `EMPTY_PACKET` fail-closed
  check and *before* `provider::resolve_provider`/`prov.invoke(...)` — satisfying AC-10's "fail
  closed before any Codex invocation" on an unreadable log (`EXIT_WRITE`). The same `review_id`
  string is passed to both `write_assessment` and `append_review`, computed exactly once. Also
  added one line to the stdout summary (`review_id: {review_id}`) — a small addition beyond
  Step 1's literal text, justified since it directly serves this change's purpose (surfacing the
  id instead of requiring a log read) and touches no new file.
- **Docs** — `docs/reviewer-artifact-schemas.md`: added `review_id` to both the assessment
  YAML-frontmatter table and the REVIEW log-entry block (normative schema, additive; explicitly
  noted as *not yet* part of the Lightweight validation (v0) required-field list, since Step 1/2
  never declared adding it there). `docs/reviewer-pipeline.md` needed no change — its §4e
  ownership description ("Exact `REV__…__R<N>` rounds... live in `reviews/review-log.md`") was
  already accurate and didn't claim the id was manual-only.
- **Cross-reference fix (discovered during the sweep, not in Step 1's list):**
  `backlog/UPG-0029-review-naming-and-thread-tooling.md` explicitly deferred "teach the reviewer
  to derive and emit the `REV__…__R<N>` id" as its own issue #2. Added a second status note
  (mirroring that file's own existing status-note pattern from `CHG-20260629-001`, not rewriting
  its historical prose) pointing forward to this change, and noting the narrower scope (raw
  stage, no filename renaming) explicitly.

**Deviation from Step 1's literal file list, justified:** Step 1 named a new
`tools/reviewer/tests/review_id.rs` integration-test file. This crate has **no `[lib]` target**
(`Cargo.toml` defines only `[[bin]]`) — confirmed by `cargo test --lib` failing with "no library
targets found." Integration tests under `tests/` can therefore only invoke the **compiled
binary** end-to-end (exactly what `tests/common/mod.rs::run`/`run_in_dir` do); they cannot call
`compute_review_round`/`append_review`/`write_assessment` directly. Testing round-counting and
id-stamping without triggering a real Codex invocation was only possible as **unit tests inside
the crate** (`#[cfg(test)] mod tests` in `log.rs`, extended in `assessment.rs`) — the same
pattern those two files already used for their existing unit tests. No new test file was
created; 10 new unit tests were added instead (9 in `log.rs`, 1 in `assessment.rs`).

**Test coverage (AC-9 through AC-11):** `round_one_when_log_missing`,
`round_one_when_log_exists_with_no_matches` (AC-10 a/b), `round_increments_across_matching_entries`
(AC-1/AC-2), `round_does_not_collide_across_similarly_named_stages` (Stage 1 vs. Stage 10 — the
human's explicitly flagged danger), `round_does_not_collide_across_features_sharing_a_prefix`,
`round_is_scoped_to_the_given_log_path_scratch_vs_durable` (AC-11),
`round_fails_closed_when_log_path_is_unreadable` (AC-10c — asserts `Err`, not a silent guess),
`format_review_id_uses_raw_stage_verbatim` (AC-7), `append_review_writes_review_id_line` (AC-4),
`write_assessment_includes_review_id_in_frontmatter` (AC-3).

No out-of-scope changes were introduced beyond the one cross-reference fix noted above, which
stays within "update all cross-references in the same change."

**R1 fixes (Step 3 review):** two real test-coverage gaps, both closed with actual tests, not
just corrected wording.

1. **AC-9: no test asserted "no filename changes."** Added filename assertions to
   `write_assessment_includes_review_id_in_frontmatter` (`assessment.rs`): the returned
   assessment filename must not start with `REV__` and must still match the legacy
   `<ts>-<feature>-stage-<stage>-<sha>` shape.
2. **AC-2: the referenced test names (`smoke_review_id_first_round_is_r1`,
   `smoke_review_id_increments_across_rounds`) described an integration test that was never
   actually written** once the no-`[lib]`-target constraint was discovered — the change record
   still named tests that don't exist. Fixed two ways: corrected AC-2's wording to name the real
   test, and added `two_sequential_review_cycles_increment_the_round` (`log.rs`) — drives the
   exact compute-round → format-id → append-entry sequence `review.rs::run()` uses, twice in a
   row, asserting `R1` then `R2` then a third read reporting `R3`. This is a stronger test than
   the originally-planned one: it exercises the real read-after-write cycle against a log file
   `append_review` itself wrote, not a hand-seeded fixture.

37 unit tests now pass (was 26 pre-change, 36 after Step 3 R1; `cargo test --bin codeos-reviewer`), 170
total across the suite (was 159 pre-change), zero failures, zero regressions.

**R1 fix (Step 4 review):** `compute_review_round`'s `if !log_path.exists() { return Ok(1) }`
pre-check was itself the bug AC-10 exists to prevent — `Path::exists()` returns `false` both when
a path is genuinely absent *and* when `fs::metadata` fails for any other reason (e.g. a
permission error on a containing directory), so a real access failure could have been silently
stamped as "round 1" instead of aborting. Fixed by reading directly and matching
`io::ErrorKind::NotFound` specifically; every other I/O error now propagates as `Err`, unchanged
from there. Added `round_fails_closed_on_permission_error_not_silently_treated_as_missing`,
which reproduces the exact scenario (an unsearchable parent directory, not merely a
directory-shaped log path) and would have failed against the pre-fix code. 38 unit tests now
pass, 171 total across the suite.

---

## Reconciliation

**Acceptance verification:**

| # | Criterion | Result | Evidence |
|---|---|---|---|
| AC-1 | Round computed from existing log history for the same feature+stage | PASS | `compute_review_round` (`log.rs`) reads `reviews/review-log.md`, matches `## ... REVIEW — {feature} — Stage {stage}` exactly; `round_increments_across_matching_entries` |
| AC-2 | R1 → R2 → R3 across repeated reviews of the same feature+stage | PASS | `two_sequential_review_cycles_increment_the_round` — real compute→append cycle, twice; **also verified live**: this change's own Step 3 review rounds show `review_id: REV__UPG-0046__CHG-20260713-001__selfdev-step-3__R1` then `__R2` in `reviews/review-log.md` |
| AC-3 | `review_id` written into assessment YAML frontmatter | PASS | `write_assessment_includes_review_id_in_frontmatter`; live evidence: `reviews/codex/2026-07-13T180218Z-...-3-904b487.md` frontmatter |
| AC-4 | `review_id` written into the log entry | PASS | `append_review_writes_review_id_line`; live evidence: `reviews/review-log.md`'s own new `Review ID:` lines for this change's rounds |
| AC-5 | No existing review filenames renamed | PASS | `git status --short reviews/codex/` shows only new files, zero renames; `write_assessment_includes_review_id_in_frontmatter`'s filename assertions (no `REV__` prefix, legacy shape preserved) |
| AC-6 | No new `reviews/runs/` storage | PASS | `git status --short reviews/` — only new files under the existing `reviews/codex/` and `reviews/codex/packets/` dirs, no new top-level directory |
| AC-7 | Raw stage string used verbatim, no `S<N>` conversion | PASS | `format_review_id_uses_raw_stage_verbatim` covers self-dev (`selfdev-step-1`), downstream-brief (`brief`), and downstream-numeric (`7`) stage strings, all unconverted |
| AC-8 | Existing review behavior unchanged | PASS | Full suite: 170/170 pass, 0 failures; `git diff --stat -- tools/reviewer/src/packet.rs` empty — this change never touched packet construction |
| AC-9 | Test coverage per the stated list | PASS (fixed at Step 3 R1 — see Implementation Notes) | 11 new unit tests enumerated in Implementation Notes, 1:1 with R1/R2/no-collision/no-filename-change |
| AC-10 | Round-counting failure modes explicit and safe | PASS (fixed at Step 4 R1 — see below) | `compute_review_round` now reads directly and matches `io::ErrorKind::NotFound` specifically (not a pre-check `Path::exists()`, which collapsed permission errors into "not found" too); `round_one_when_log_missing`/`round_one_when_log_exists_with_no_matches` (a/b); `round_fails_closed_when_log_path_is_unreadable` (directory-as-log-path) and `round_fails_closed_on_permission_error_not_silently_treated_as_missing` (unsearchable parent directory — the exact scenario the finding named) both assert `Err`; `review.rs` maps that to `EXIT_WRITE` before any provider call |
| AC-11 | Scratch/durable round-counting independent | PASS | `round_is_scoped_to_the_given_log_path_scratch_vs_durable`; code inspection confirms `compute_review_round` reads whichever `review_log_path` `review.rs` already resolved (scratch vs. durable), unchanged existing logic |

**Consistency sweep (grep):**
- `scripts/codeos-review.sh` — `git diff --stat` empty; static locator shim needs no change (confirmed, not assumed, per Step 1's stated plan).
- `CLAUDE.md`/`dba-system.md` — `git diff --stat` empty; no downstream-doctrine drift.
- `Cargo.toml`/`Cargo.lock` — `git diff --stat` empty; no new dependency (round-counting uses only
  already-imported `std::fs`/`anyhow`).
- `grep -rln "manual naming convention\|documented manual convention"` across `backlog/`/`docs/`/`prompts/`
  → only `backlog/UPG-0029-review-naming-and-thread-tooling.md`, which is the file this change's
  own status note updates — no other stale reference exists (`UPG-0001`'s own historical claim
  about itself — "does not rename files, does not change `scripts/codeos-review.sh`" — remains
  true of `UPG-0001` itself and needs no correction; the deferred-tooling claim belonged to
  `UPG-0029`, already fixed).
- No orphaned links; no section renumbered in `docs/reviewer-artifact-schemas.md` (additive rows
  only).

**Findings scope-triage:**

| Finding | Triage | Action |
|---|---|---|
| Step 3 R1: no test asserted "no filename changes" (AC-9) | IN-SCOPE BLOCKER | Fixed — filename assertions added to the frontmatter test |
| Step 3 R1: AC-2 referenced test names that were never actually written (AC-2) | IN-SCOPE BLOCKER | Fixed — wording corrected, and a stronger real compute→append-cycle test added |
| Step 4 R1: `Path::exists()` pre-check collapses "not found" and "permission/access error" into the same `false`, silently defeating AC-10's fail-closed guarantee for the second case | IN-SCOPE BLOCKER (High severity) | Fixed structurally — `compute_review_round` now reads directly and matches `io::ErrorKind::NotFound` specifically; `Err` for every other I/O error kind. New test reproduces the exact scenario (unsearchable parent directory, not just a directory-as-log-path) and confirms it now fails closed. |

All findings across all four reviewed steps are resolved. No OUT-OF-SCOPE BACKLOG, REJECTED,
SELF-REFERENCE, or REVIEW-BOOKKEEPING findings arose in this change.

**Stack/dependency reconciliation:** Not applicable — `Cargo.toml`/`Cargo.lock` unchanged (see
sweep above); no watched-file reconciliation report required.

**Follow-up implication for `UPG-0047`:** structured findings can now attach to a stable,
mechanically-assigned `review_id` instead of needing to invent one — the dependency this
change's own backlog brief named is now satisfied.
