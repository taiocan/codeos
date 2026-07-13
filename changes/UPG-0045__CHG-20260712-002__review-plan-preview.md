# Self-Development Change: UPG-0045__CHG-20260712-002 — review-plan-preview

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0045
primary_feature_id: UPG-0045
change_id: CHG-20260712-002
slug: review-plan-preview
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0045
related_features: [UPG-0042, UPG-0031, UPG-0027]
review_series: RVS__UPG-0045__CHG-20260712-002__S4
review_profile: PROFILE-3
review_state: REVIEWED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

## Change Intent

**Why (problem in the toolkit):**

Before every `codeos-reviewer review <feature> <stage> <artifacts…>` call, the human/Claude has
to manually reconstruct what artifacts matter, which evidence mode is right for this round, and
roughly how big the packet will be — with no feedback until *after* Codex is already invoked
(see `docs/reviewer-pipeline.md` §14, §4b; `backlog/UPG-0042-reviewer-packet-efficiency.md`;
`backlog/UPG-0031-review-delta-mode-fix.md`'s `EMPTY_PACKET` fail-closed guard, which only fires
post-hoc). `backlog/UPG-0045-review-plan-preview.md` proposes a `plan` preview to close this gap,
leaving three implementation questions open for whoever picks it up. Having read the code, I can
now resolve all three concretely:

- **Q1 (what plan computes):** the brief's own cheaper option — "echo back what's passed,
  annotated with size/mode guidance" — turns out to already be *almost entirely computed* by
  `tools/reviewer/src/packet.rs::build()`. `review --print-packet` already builds the exact same
  packet Codex would see and prints it raw; the per-file bytes/mode, `review_content_bytes`,
  `estimated_review_tokens`, and `budget_status` values are all computed inside `build()` today
  but only ever baked into the packet's text body — never returned as struct fields. So the real
  gap isn't "no data exists," it's "the data exists but only as an opaque string." No new
  artifact-resolution machinery is needed; `UPG-0049`'s policy registry remains unnecessary here.
- **Q2 (command shape):** mirror `review`'s argument shape exactly, by reusing
  `cmd::review::parse_rest()` unmodified — `plan` accepts the identical
  `<feature> <stage> <artifacts…> [--mode delta --base <sha>] [--sha-only <path>]…` syntax `review`
  does, so there is zero syntax drift between planning a review and running one.
  `--fresh`/`--scratch`/`--print-packet`/`--skip-prechecks` parse the same way but are inert for
  `plan` (never invokes a provider or writes state, so "fresh session" and "scratch" have nothing
  to apply to).
- **Q3 (relationship to `--print-packet`):** `plan` calls the exact same `packet::build()`
  function `--print-packet` already calls (verified: `packet::build()` has no side effects — no
  `fs::write`, no `fs::create_dir`, confirmed by grep) — so `plan` cannot drift from what a real
  review would see, by construction. The only new code is (a) exposing the manifest data
  `build()` already computes as public struct fields instead of only string-formatting it, and
  (b) a summary printer that reads those fields instead of dumping the full packet text.

**What changes:**

- `tools/reviewer/src/packet.rs` — **additive only**, no change to existing behavior or to the
  `content()` string output:
  - Add public fields to `ReviewPacket`: `review_content_bytes: u64`, `estimated_review_tokens: u64`,
    `budget_bytes: u64`, `over_budget: bool` (mirrors the existing `budget_status` text already
    computed at line ~332, exposed as data instead of only as a formatted string).
  - Add a `bytes: u64` field to `ArtifactEntry` (currently `path`/`sha256`/`visibility` only),
    populated from the same `std::fs::metadata(...).len()` calls the function already makes for
    every artifact/sha-only path — no new I/O.
  - No change to `PacketBuildOptions`, no change to any existing field, no change to `content()`'s
    returned string.
- `tools/reviewer/src/cmd/plan.rs` (new file) — a `run()` function taking the same `ReviewArgs`
  shape `review::run()` takes (reused type, not duplicated): runs the identical missing-artifact
  and precheck validations `review::run()` runs today (same `precheck` module calls — so `plan`
  accurately reports "this artifact would fail prechecks" before a round is spent), calls
  `packet::build()`, then prints a compact summary to stdout: resolved artifacts with
  path/mode/bytes, `review_content_bytes` vs. budget with a percentage, `estimated_review_tokens`,
  `coverage_state`, and one static reminder line pointing at
  `--mode delta --base <sha>` for R2+ (exact text already documented in
  `docs/reviewer-pipeline.md` §4b/§14 — not a new recommendation engine, just a fixed pointer).
  **Never invokes a provider, never writes to `reviews/` or any tracked file** — `packet::build()`
  is read-only and `plan` performs no writes of its own.
- `tools/reviewer/src/main.rs` — new `Commands::Plan { feature, stage, rest }` variant (mirrors
  `Commands::Review`'s shape) and a dispatch arm that calls `cmd::review::parse_rest(&rest)`
  (reused, not duplicated) then `cmd::plan::run(...)`.
- `tools/reviewer/src/cmd/mod.rs` — add `pub mod plan;`.
- `tools/reviewer/tests/plan_command.rs` (new file, following the `UPG-0043` per-tool-area smoke
  test split convention) — covers: basic plan output for a simple full-mode artifact set,
  `EMPTY_PACKET` reporting, delta mode, sha-only mode, a missing-artifact/precheck-failure case,
  and an explicit assertion that running `plan` leaves `reviews/` and the git working tree
  unchanged (no new file under `reviews/codex/`, no `reviews/review-log.md` append).
- `docs/reviewer-pipeline.md` — one small new subsection (near §11 "Usage" or as part of §14,
  final placement decided at Step 3) documenting `codeos-reviewer plan` with a usage example and
  the same "never calls Codex, never mutates state" guarantee stated above.

**Scope boundary — what stays the same:**

- No change to `scripts/codeos-review.sh` — it is a static locator shim (`docs/reviewer-pipeline.md`
  §10) that passes all arguments through verbatim; `plan` works through it automatically once the
  Rust binary supports the subcommand. This will be confirmed, not assumed, in Step 4.
  `.codeos/scripts/codeos-review.sh` in downstream projects inherits this for free the same way.
- No change to `review`'s existing behavior, output format, or exit codes — `packet::build()`'s
  signature, `PacketBuildOptions`, and the `content()` string are unchanged; only new fields are
  added to `ReviewPacket`/`ArtifactEntry`.
- No auto-selection or auto-downgrade of evidence mode — `plan` only recommends via fixed
  reference text; the human/Claude still chooses `--mode` explicitly on the real `review` call,
  per the guardrail in `backlog/UPG-0045-review-plan-preview.md`.
- No new artifact-resolution/policy-registry machinery (`UPG-0049` remains untouched, unstarted).
- No change to `CLAUDE.md`, `dba-system.md`, or any downstream doctrine.
- No change to any existing test in `tools/reviewer/tests/` (new coverage is additive, in a new
  file).

**Class:** script-tooling
**Scope axis:** self-dev only
**Backlog item:** backlog/UPG-0045-review-plan-preview.md

---

## Acceptance Criteria

| # | Criterion | How it will be verified |
|---|---|---|
| AC-1 | `codeos-reviewer plan` uses the same argument-parsing surface as `review`, via `cmd::review::parse_rest()` (or an identical shared parser) — not a second parser. | Code inspection: `main.rs`'s `Commands::Plan` dispatch arm calls `cmd::review::parse_rest(&rest)` (mirroring the existing `Commands::Review` arm) and passes the parsed result into `cmd::plan::run()`; `plan.rs` itself contains no argument-parsing loop. Grep confirms `parse_rest` has exactly one definition, called from both dispatch arms. |
| AC-2 | `plan` calls the same `packet::build()` function `review`/`--print-packet` use — no duplicate packet-construction logic. | Code inspection: single call site to `packet::build()` in `plan.rs`; grep for `fn build` confirms only one packet-construction function exists in `packet.rs` after this change. |
| AC-3 | `ReviewPacket` and `ArtifactEntry` expose already-computed metadata as fields: per-file bytes, `review_content_bytes`, `estimated_review_tokens`, `budget_status`/`over_budget`, and `coverage_state` (already present). | Read the struct definitions post-change; confirm each new field is populated from the same computation `build()` already performs (no new derivation logic), by diffing against the pre-change `build()` body. |
| AC-4 | Existing `review` behavior — packet content, invocation behavior, fail-closed `EMPTY_PACKET` handling, exit codes — is unchanged. | Full existing test suite (`cargo test`) passes unchanged after this change, with zero modifications to any pre-existing test; `git diff -- tools/reviewer/src/packet.rs` shows only additive struct-literal fields, no changes to existing computation/branches. |
| AC-5 | `--print-packet` output and `plan` output are consistent for identical inputs — same `coverage_state`, same `review_content_bytes`, same per-artifact visibility/bytes — differing only in that `plan` presents a summary instead of the full packet text. | Smoke test: run `review --print-packet` and `plan` on the same artifact set/mode/base, parse both outputs, assert `coverage_state`, `review_content_bytes`, and per-artifact byte counts match. |
| AC-6 | `plan` does not invoke Codex and does not write to `reviews/` or any other tracked file — it only builds and reports the packet plan. | Code inspection: `plan.rs` never calls `provider::resolve_provider` or `Provider::invoke`, never calls `std::fs::write`/`std::fs::create_dir_all`. Smoke test: reuse the same pre/post `git status --porcelain` comparison pattern `review.rs` already applies around its own provider invocation (`review.rs:183-208`, the `UPG-0034` read-only invariant check) around the *entire* `plan` execution, not just `reviews/` — assert the working tree is byte-identical before and after (broader than only diffing `reviews/review-log.md`, which was the under-verified original wording). |
| AC-7 | `plan` does not auto-select or mutate evidence mode; it may state the existing static delta-mode reminder, but mode choice (`--mode`/`--sha-only`) stays exactly what the caller passed. | Code inspection: `PacketBuildOptions.delta_mode`/`sha_only_paths` passed to `packet::build()` in `plan.rs` are taken verbatim from parsed args, never computed/overridden. Smoke test: default (no `--mode`) input produces `mode: full` in the summary, never silently switched to delta. |
| AC-8 | Oversized-packet diagnostics in `plan`'s summary include total size, the budget multiplier, top contributors by size, and the actionable `--mode delta --base <sha>` suggestion — matching the existing warning `build()` already emits to stderr for `review` (§ packet.rs budget check). | Smoke test: run `plan` with `CODEOS_PACKET_BUDGET_BYTES` set low against artifacts that exceed it; assert stdout contains the size, an overage multiple, at least one top contributor, and the literal suggested command text. |
| AC-9 | No changes to `scripts/codeos-review.sh` — it is a static locator shim that passes all arguments through verbatim (`docs/reviewer-pipeline.md` §10) — unless this step or Step 3 discovers an actual incompatibility, in which case this criterion is revised before Step 3 implementation proceeds. | `git diff --stat -- scripts/codeos-review.sh` empty at Reconcile; if non-empty, the change record documents why this criterion was revised and when. |
| AC-10 | Test coverage in `tests/plan_command.rs` includes at minimum: normal full-mode plan output, `EMPTY_PACKET` reporting, delta mode, sha-only mode, a missing-artifact/precheck-failure case, oversized-packet warning content (AC-8), no-Codex-invocation / no-`reviews/`-mutation (AC-6), and output parity with `--print-packet` metadata (AC-5). | `cargo test --test plan_command` passes; test names enumerated in Reconcile map 1:1 to this list. |
| AC-11 | Exit codes: `plan` reuses the existing `EXIT_*` constants (`EXIT_USAGE`, `EXIT_PACKET`, …) for the same conditions `review` already uses them for (missing artifact, bad `--mode`/`--base`, `EMPTY_PACKET`) — no new exit-code values are introduced for conditions that already have one. | Code inspection: every `return Ok(crate::EXIT_*)` in `plan.rs` matches an existing constant for an equivalent condition in `review.rs`; smoke tests assert the same exit codes for the same fault-injected inputs (e.g., missing artifact → `EXIT_PACKET`, exactly as `review` does today). |
| AC-12 | Idempotency: running `plan` twice in immediate succession with identical inputs and an unchanged repository produces identical output **except** for the packet's own `generated:` timestamp field (the same pre-existing exception `review --print-packet` already has). | Smoke test: run `plan` twice back-to-back on the same inputs; assert output is byte-identical after stripping the `generated:` line from both. |

**Class note:** `script-tooling` — AC-4/AC-9/AC-11/AC-12 are the I/O-behavior, exit-code, and
idempotency contracts this class requires per `prompts/codeos-self-dev.md` Step 2. No
downstream-compatibility criteria apply (self-dev only scope).

**Bookkeeping note:** `status/self-development.md`'s row activation/update for this change (Step
1 and this step) is standard Feature Thread bookkeeping required by `prompts/codeos-self-dev.md`
Step 1 ("Activate the row"), not part of the `plan` implementation itself — named here explicitly
per the same scope-wording lesson from `UPG-0044`'s Step 1 review.

---

## Implementation Notes

All edits landed exactly as scoped in Step 1; no out-of-scope changes.

- **`tools/reviewer/src/packet.rs`** — additive only. Added `bytes: u64` to `ArtifactEntry`
  (populated at all 5 existing `ArtifactEntry` construction sites from the same
  `fs::metadata(...).len()` values already computed there; `0` for `missing`). Added
  `review_content_bytes`, `estimated_review_tokens`, `budget_bytes`, `over_budget`, and
  `diff_bytes` to `ReviewPacket`, populated at the single `Ok(ReviewPacket { .. })` return site
  from the same local variables `build()` already computed (`review_content_bytes`,
  `estimated_tokens`, `budget`, `diff_bytes`). No existing field, branch, or the `content()`
  string output changed.
- **`tools/reviewer/src/cmd/plan.rs`** (new) — `run()` reuses `ReviewArgs` (no new args struct),
  runs the identical delta-mode/missing-artifact/precheck validation `review::run()` runs, calls
  `packet::build()` once, then prints a summary from the new struct fields. Never calls
  `provider::resolve_provider`/`Provider::invoke`, never calls `fs::write`/`fs::create_dir_all`.
- **`tools/reviewer/src/main.rs`** — new `Commands::Plan { feature, stage, rest }` (mirrors
  `Commands::Review`'s shape) and a dispatch arm that calls `cmd::review::parse_rest(&rest)`
  (the parsing happens in `main.rs`, not inside `plan.rs` — this is the exact wiring AC-1 was
  fixed to describe correctly during Step 2 review) then `cmd::plan::run(...)`.
- **`tools/reviewer/src/cmd/mod.rs`** — added `pub mod plan;`.
- **`tools/reviewer/tests/plan_command.rs`** (new) — 8 tests covering AC-10's full list: normal
  full-mode output, `EMPTY_PACKET` (delta mode, no diff), delta mode with a real change,
  sha-only mode, missing-artifact (exit `EXIT_PACKET`=4), oversized-packet warning content
  (size/multiplier/top-3/suggested command), no-Codex/no-tree-mutation (pre/post
  `git status --porcelain` equality, matching the `UPG-0034` read-only invariant pattern), and
  output parity with `--print-packet` (`review_content_bytes` and `coverage_state` match for
  identical inputs). Duplicated the existing `setup_codeos_symlink` test helper locally (it was
  already private to `review_command.rs`, not shared) rather than promoting it into
  `tests/common/mod.rs` — keeps this change's touched-file set to only the new test file.
- **`docs/reviewer-pipeline.md`** — one new subsection ("Preview a plan before reviewing") added
  inside §14, after "Combining Modes" and before the section break — no section renumbered, no
  existing prose changed.
- **`scripts/codeos-review.sh`** — confirmed unchanged, as predicted: manually ran
  `bash scripts/codeos-review.sh plan UPG-0045 selfdev-step-3 tools/reviewer/src/cmd/plan.rs`
  and it worked identically to the direct binary invocation (the static locator shim passes
  `plan` through verbatim, exactly like every other subcommand). AC-9 holds without revision.

**Manual verification beyond the automated tests** (evidence for AC-6/AC-2/AC-5, run against the
real Codeos repo before writing the test file): missing artifact → exit 4; over-budget run via
`CODEOS_PACKET_BUDGET_BYTES=1000` → correct WARNING/largest-inputs/suggestion text; delta mode
with a real uncommitted change → `delta_diff` visibility; sha-only mode → correct
`path_sha_only` visibility; `git status --porcelain` output and `reviews/codex/` file count
identical before and after every `plan` invocation in the real repo.

No out-of-scope changes were introduced. No findings from Steps 1–2 remained open at Step 3.

**R1 fixes (Step 3 review):** two real bugs found, both fixed structurally, not just reworded.

1. **AC-8 violation — oversized-packet contributor ranking could disagree with `review`'s own
   warning.** The first implementation ranked `print_budget_warning`'s "largest inputs" from
   *all* `p.artifacts` by raw on-disk size, including `sha-only` and delta-mode entries — but
   `packet::build()`'s own stderr warning ranks only the subset actually counted into
   `review_content_bytes` (full-mode content + the diff). A large `sha-only` context file could
   have shown up in `plan`'s "largest inputs" while contributing zero bytes to the real budget
   total, actively misleading the reader about what to trim. **Fix:** added a
   `budget_contributors: Vec<(String, u64)>` field to `ReviewPacket`, populated directly from
   the same `file_contributors` local variable `build()` already accumulates for its own
   warning (not a re-derivation) — `plan.rs` now ranks from that field, guaranteeing identical
   output to `review`'s warning by construction, not by parallel logic staying in sync.
2. **AC-5/AC-12 test-coverage gap.** The parity test only checked aggregate
   `review_content_bytes` and `coverage_state`, not per-artifact bytes; no idempotency test
   existed. **Fix:** changed the per-artifact summary line from lossy `{:.1} KB` to exact
   `{} bytes` (also fixes a real precision problem — small files rounded to "0.0 KB", making
   per-artifact parity untestable at all) and added `smoke_plan_output_parity_with_print_packet`
   assertions comparing `tracked.md`'s exact byte count between `plan` and `--print-packet`'s
   manifest, plus a new `smoke_plan_idempotent_output` test (two back-to-back runs on unchanged
   repo state must be byte-identical — true without exception here, since `plan`'s summary,
   unlike the full packet, embeds no generation timestamp).

9 tests now pass in `plan_command.rs` (was 8); full suite still green (159 tests total, was 158;
`cargo test 2>&1 | grep "test result:"` across all 10 test binaries, zero failures).

---

## Reconciliation

**Acceptance verification:**

| # | Criterion | Result | Evidence |
|---|---|---|---|
| AC-1 | Same arg-parsing surface as `review`, via `cmd::review::parse_rest()` | PASS | `main.rs:253` `Commands::Plan` arm calls `cmd::review::parse_rest(&rest)` (identical to the `Commands::Review` arm); `plan.rs::run()` has no parsing loop, only `#[test]`-verified argument consumption |
| AC-2 | `plan` calls the same `packet::build()` function, no duplicate construction | PASS | `grep -c "^pub fn build" packet.rs` → 1; `packet::build(` called once in `plan.rs:121`, once in `review.rs:140` — same function |
| AC-3 | `ReviewPacket`/`ArtifactEntry` expose already-computed metadata as fields | PASS | `bytes`, `review_content_bytes`, `estimated_review_tokens`, `budget_bytes`, `over_budget`, `diff_bytes`, `budget_contributors` all present, populated from the exact local variables `build()` already computed (no new derivation) |
| AC-4 | Existing `review` behavior unchanged | PASS | `cargo test` — 159/159 pass, 0 failures, across all 10 test binaries including the pre-existing `review_command.rs` (21/21 unchanged); `git diff -- packet.rs` shows only additive struct-literal fields, zero changes to existing branches |
| AC-5 | `--print-packet`/`plan` parity (coverage, review_content_bytes, per-artifact bytes) | PASS | `smoke_plan_output_parity_with_print_packet` passes — asserts coverage_state, aggregate `review_content_bytes`, and `tracked.md`'s exact per-artifact byte count all match between the two commands |
| AC-6 | No Codex invocation, no `reviews/`/tracked-file mutation | PASS | `smoke_plan_never_invokes_codex_or_mutates_tree` passes — pre/post `git status --porcelain` byte-identical, no `reviews/` dir created, stdout never contains `review logged:`. Code inspection: `plan.rs` calls neither `provider::resolve_provider` nor any `fs::write`/`fs::create_dir_all` |
| AC-7 | No auto-selection/mutation of evidence mode | PASS | `plan.rs:113-114` passes `args.sha_only`/`args.delta_mode` verbatim into `PacketBuildOptions`; `smoke_plan_full_mode_basic` confirms default (no `--mode`) reports `mode: full` |
| AC-8 | Oversized diagnostics match `review`'s own warning (size, multiplier, top contributors, delta suggestion) | PASS (fixed at Step 3 R1 — see Implementation Notes) | `smoke_plan_oversized_packet_warning_content` passes; `print_budget_warning` ranks from `p.budget_contributors`, the identical list `build()`'s own stderr warning uses — not a parallel computation |
| AC-9 | No `scripts/codeos-review.sh` change | PASS | `git diff --stat -- scripts/codeos-review.sh` empty; manually confirmed `bash scripts/codeos-review.sh plan ...` works identically to the direct binary |
| AC-10 | Test coverage per the stated list | PASS | 9 tests in `plan_command.rs`: full-mode basic, missing-artifact, `EMPTY_PACKET`, delta-mode-changed, sha-only, oversized-warning, no-mutation, print-packet parity, idempotency — 1:1 with the AC-10 list |
| AC-11 | Exit codes reuse existing `EXIT_*` constants | PASS | `plan.rs` uses only `EXIT_USAGE`/`EXIT_CONFIG`/`EXIT_PACKET`/`EXIT_SUCCESS` (grep above), each at conditions matching `review.rs`'s use of the same constant; `smoke_plan_missing_artifact_exits_packet` confirms exit 4, matching `review`'s own precedent for the same condition |
| AC-12 | Idempotency (byte-identical output modulo timestamp) | PASS | `smoke_plan_idempotent_output` passes — `plan`'s summary embeds no timestamp at all, so output is fully byte-identical across repeated runs, no exception needed |

**Consistency sweep (grep):**
- No hardcoded subcommand list elsewhere in `docs/`/`prompts/` needed a `plan` entry — the two
  files matching `check-drift`/`generate-*` (`docs/ci-integration-profile.md`,
  `docs/registry-v2-migration.md`) each mention a single unrelated subcommand for its own
  purpose, not an exhaustive list.
- `codeos-reviewer --help` auto-includes `plan`'s doc-comment summary via `clap` — verified live,
  no manual help text to keep in sync.
- No stale `§14` cross-reference: all hits outside `docs/reviewer-pipeline.md` are in
  already-COMPLETE `UPG-0044` change/backlog records describing that section's prior creation,
  unaffected since `plan`'s doc addition landed *inside* existing §14 without renumbering.
- `git diff --stat -- CLAUDE.md dba-system.md` empty — no downstream-doctrine drift.
- No orphaned links; no new file created outside the declared set (`git status --short` above
  matches exactly the files named in Step 1's "What changes").

**Findings scope-triage:**

| Finding | Triage | Action |
|---|---|---|
| Step 2 R1: AC-1 wording contradicted the intended parser wiring | IN-SCOPE BLOCKER | Fixed — AC-1 verification text corrected to match `main.rs`-dispatch design |
| Step 2 R1: AC-6 under-verified the "any tracked file" guarantee | IN-SCOPE BLOCKER | Fixed — broadened to whole-working-tree `git status` comparison |
| Step 3 R1: oversized-contributor ranking could disagree with `review`'s own warning | IN-SCOPE BLOCKER | Fixed structurally — new `budget_contributors` field, single source of truth |
| Step 3 R1: AC-5/AC-12 test coverage incomplete (per-artifact parity, idempotency) | IN-SCOPE BLOCKER | Fixed — new assertions/test added; also fixed the underlying lossy-KB display bug this gap was hiding |

All findings across all three reviewed steps are resolved. No OUT-OF-SCOPE BACKLOG, REJECTED,
SELF-REFERENCE, or REVIEW-BOOKKEEPING findings arose in this change.

**Stack/dependency reconciliation:** Not applicable — no watched file (`Cargo.toml`,
`Cargo.lock`) changed; `tools/reviewer/Cargo.toml` dependencies are untouched (only new fields
on existing structs and a new module using only already-imported crates).
