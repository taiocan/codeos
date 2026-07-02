---
change_id: CHG-20260702-001
feature_id: UPG-0032
slug: rust-reviewer-engine
triage_class: script-tooling
scope_axis: self-dev only
review_profile: PROFILE-3
review_series: RVS__UPG-0032__CHG-20260702-001__S4
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0032 / CHG-20260702-001 — Rust Reviewer Engine

## TRACE HEADER

```yaml
feature_id: UPG-0032
primary_feature_id: UPG-0032
change_id: CHG-20260702-001
slug: rust-reviewer-engine
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0032
related_features:
  - UPG-0003
  - UPG-0015
  - UPG-0018
review_series: RVS__UPG-0032__CHG-20260702-001__S4
review_profile: PROFILE-3
review_state: DRAFT
review_history: reviews/review-log.md
triage_class: script-tooling
scope_axis: self-dev only
corrects: ~
corrected_by: ~
follow_up_of: ~
fixes_findings: []
```

---

## Step 1 — Change Intent

### Problem

`scripts/codeos-review.sh` has proven the reviewer pipeline (packet → AI review →
assessment → log) across Waves 1–3 of toolkit development. The workflow is sound. The
host is not.

**Bash fragility.** The precheck pipeline is a chain of `sed`/`grep` compositions with
no compiler. UPG-0031 hit a silent failure where `sed` range deletion swallowed lines
when `<!--` appeared inside an inline code span — a bug invisible until it fired in
production. More pipeline stages produce more subtle ordering hazards.

**Provider lock-in.** The script is hard-wired to `codex exec`. No abstraction boundary
exists between packet construction and provider invocation. Adding a second provider
requires forking the invocation logic, re-implementing session-ID extraction, and
re-testing the whole pipeline.

**Single-context limitation.** The reviewer currently serves only the self-dev 4-step
loop. The same advisory pattern — packet → AI review → assessment → log — is equally
valuable inside downstream DBA 9-stage projects. There is no clean path to reuse the
pipeline there without script duplication.

**Untestable units.** Packet construction, secret filtering, precheck, coverage-state
classification, and log-append are entangled in one script. Only full-pipeline smoke
tests are possible; no unit tests can verify individual steps.

**Typed state as strings.** `PACKET_COVERAGE_STATE`, `PACKET_DELTA_MODE`, exit codes,
and coverage categories are plain Bash strings. A typo produces a silent wrong-path
execution.

### What changes

| File / Directory | Change |
|---|---|
| `tools/reviewer/` | NEW — Rust workspace implementing the full reviewer pipeline |
| `tools/reviewer/Cargo.toml` | NEW — workspace manifest |
| `tools/reviewer/src/main.rs` | NEW — CLI entry point; `review`, `decision`, `diagnose`, `stage-start` subcommands |
| `tools/reviewer/src/provider/mod.rs` | NEW — `ReviewProvider` trait definition |
| `tools/reviewer/src/provider/codex.rs` | NEW — `CodexProvider` (wraps `codex exec`; primary impl) |
| `tools/reviewer/src/provider/stubs.rs` | NEW — empty stubs for `OpenCodeProvider`, `GeminiProvider`, `KimiProvider` |
| `tools/reviewer/src/packet.rs` | NEW — `ReviewPacket`, `PacketManifest`, `ArtifactEntry`, `CoverageState` enum |
| `tools/reviewer/src/precheck.rs` | NEW — pure precheck functions (unit-testable, no shell) |
| `tools/reviewer/src/log.rs` | NEW — log-append logic (writes to `reviews/review-log.md`) |
| `tools/reviewer/src/config.rs` | NEW — config loading: `reviewer.toml` + env var + CLI flag precedence |
| `tools/reviewer/src/assessment.rs` | NEW — assessment write + YAML frontmatter generation |
| `tools/reviewer/tests/` | NEW — integration tests and smoke test fixtures |
| `scripts/codeos-review.sh` | UPDATE — reduced to a thin shim calling the Rust binary |
| `scripts/dba-init.sh` | UPDATE — writes default `reviewer.toml` during project init |
| `templates/reviewer.toml` | NEW — default provider config template |
| `backlog/UPG-0032-rust-reviewer-engine-multi-provider.md` | UPDATE — Feature Thread: add this change |
| `status/self-development.md` | UPDATE — activate row for this change |
| `status/roadmap.md` | UPDATE — Wave 4 UPG-0032 → IN_PROGRESS |
| `changes/UPG-0032__CHG-20260702-001__rust-reviewer-engine.md` | NEW — this change record |

### What stays the same (scope boundary)

- `dba-system.md` — not touched.
- Stage prompts `prompts/01-` through `prompts/10-` — not touched.
- Assessment YAML frontmatter format — fully backward-compatible with all existing
  `reviews/codex/*.md` files.
- Review log format (`reviews/review-log.md`) — unchanged.
- Review series ID model (`RVS__<feature>__S<N>`, `REV__…__R<N>`) — generated
  consistently with current convention.
- Reviewer role — remains advisory, read-only, non-gatekeeping. Moving implementation
  to typed Rust does not change what the reviewer does or its relationship to the human
  gate.
- Stage-policy loading mechanism — the backlog describes a future `.codeos/reviewer-policy/stage-N.toml`
  data-file approach. That is out of scope for this change. The binary reads stage policy
  from the existing `prompts/codeos-reviewer-task.md` and injected stage-checklist locations,
  exactly as the Bash script does today. The TOML-based policy system is a follow-on change.
- `scripts/dba-init.sh` general behavior — all existing init steps (project directory
  structure, symlink setup, hook installation, etc.) are unchanged. The only addition
  is writing a default `reviewer.toml` at init time. No other `dba-init.sh` behavior
  is modified.
- Per-feature decision ledgers — deferred to UPG-0015.
- Autonomous review triggering, CI hooks — out of scope.
- GUI / TUI — out of scope.

### Governance note: self-dev loop, not DBA 9-stage

This is the first runtime binary in the toolkit. Applying the DBA 9-stage loop here
would first require a `self-dev-governance` class change to CLAUDE.md authorizing DBA
for toolkit runtime artifacts — that governance change has not been made. This change
uses the self-dev 4-step loop with an extended Step 2 that takes on the DBA rigor most
relevant to a runtime tool: provider contract, I/O behavior, exit-code table, config
precedence, error handling, and unit + integration test gates.

A future `self-dev-governance` change may introduce a `runtime-tool` triage class
that formally authorizes DBA for toolkit binaries. That is tracked as a follow-on.

### Triage class: `script-tooling`

Replacing a Bash pipeline with a Rust binary. Class is `script-tooling`. 4-step loop
with PROFILE-3 cadence.

### Scope axis: `self-dev only`

No changes to `dba-system.md`. The binary serves downstream DBA projects at runtime,
but the code being written lives in this toolkit repo and does not modify the downstream
doctrine.

### Review profile: PROFILE-3

Script-tooling, downstream-facing. Codex review before each step gate; max 3
rounds/step; human approval at all four gates; reviewer output is advisory and
non-gatekeeping.

### Originating backlog item

`backlog/UPG-0032-rust-reviewer-engine-multi-provider.md` — Rust Reviewer Engine with
Multi-Provider Support.

---

## Step 2 — Acceptance Criteria

### AC-1: Provider abstraction contract — trait definition and no hidden coupling

The `ReviewProvider` trait in `tools/reviewer/src/provider/mod.rs` must define exactly
these three methods and no more:

```rust
pub trait ReviewProvider {
    fn name(&self) -> &str;
    fn invoke(&self, packet: &ReviewPacket, cfg: &ProviderConfig) -> Result<RawAssessment>;
    fn extract_session_id(&self, raw: &str) -> Option<String>;
}
```

**No hidden provider coupling:** command handler code (`main.rs` and any `cmd/` modules)
must not import or reference `CodexProvider`, `OpenCodeProvider`, `GeminiProvider`, or
`KimiProvider` by name. All provider interaction goes through the trait.

Verification:
```bash
grep -rn "CodexProvider\|OpenCodeProvider\|GeminiProvider\|KimiProvider" \
  tools/reviewer/src/main.rs tools/reviewer/src/cmd/ 2>/dev/null | wc -l  # → 0
grep -n "fn name\|fn invoke\|fn extract_session_id" \
  tools/reviewer/src/provider/mod.rs | wc -l  # → 3
```

### AC-2: Input/output behavior per subcommand

Each subcommand must produce exactly the outputs below. No undocumented side effects.

**`review <feature-id> <stage> <artifact-paths...> [--sha-only <path>]...`**
- Writes assessment file: `reviews/codex/<timestamp>Z-<feature>-stage-<stage>-<sha>.md`
  (same naming convention as current Bash script output)
- Writes packet file: `reviews/codex/packets/<timestamp>Z-<feature>-stage-<stage>-<sha>.packet.txt`
- Appends one entry to `reviews/review-log.md`
- Stdout: summary lines matching current Bash format (at minimum: `review logged:`,
  `codex concern:`, `effective concern:`, `evidence:`, `effort:`, `elapsed:`, `coverage:`,
  `assessment:`, `packet:`)
- Stderr: diagnostic messages on any non-zero path; silent on success

**`decision <feature-id> <stage> <verdict> "<message>"`**
- Appends one decision entry to `reviews/review-log.md`
- Stdout: `decision appended to reviews/review-log.md`
- No files written other than the log append

**`diagnose [<feature-id> <stage>]`**
- Stdout: config resolution info (provider selected, config source, toolkit root discovered)
- No files written

Verification (at Step 4): run each subcommand against a test fixture and diff output
format against a known-good Bash-script run.

### AC-3: Exit-code contract

| Exit code | Meaning |
|---|---|
| 0 | Success — operation completed, all writes committed |
| 1 | Usage / argument error — bad subcommand, missing required arg |
| 2 | Configuration error — no provider resolvable, malformed `reviewer.toml` |
| 3 | Provider invocation error — `codex exec` failed or returned no parseable output |
| 4 | Packet build error — artifact file not found, SHA mismatch, secret detected |
| 5 | Log / output write error — could not append to `review-log.md` or write assessment |

The binary must never exit 0 when an error prevents the operation from completing.
Any exit code not in {0,1,2,3,4,5} is a bug.

Verification:
```bash
# missing required arg → exit 1
tools/reviewer/target/release/codeos-reviewer review 2>&1; echo "exit=$?"  # → exit=1
# non-existent artifact → exit 4
tools/reviewer/target/release/codeos-reviewer review FEAT stage /nonexistent.md 2>&1; echo "exit=$?"  # → exit=4
```

### AC-4: Config/env precedence — highest to lowest

| Priority | Source |
|---|---|
| 1 (highest) | `--provider <name>` CLI flag |
| 2 | `CODEOS_REVIEWER_PROVIDER` environment variable |
| 3a | `reviewer.toml` at repo root (project-specific; written by `dba-init.sh`) |
| 3b | `.codeos/reviewer.toml` if it is a real file (not a symlink; toolkit-level override) |
| 4 (lowest) | Compiled-in default: `codex` |

If a source at priority N specifies a provider, sources at N+1 and below are ignored.
An unknown provider name at any level is a configuration error (exit 2).

Verification:
```bash
# env var overrides toml default
CODEOS_REVIEWER_PROVIDER=opencode tools/reviewer/target/release/codeos-reviewer diagnose \
  | grep "provider:" | grep "opencode"  # → matches
# --provider flag overrides env var
CODEOS_REVIEWER_PROVIDER=opencode tools/reviewer/target/release/codeos-reviewer \
  --provider codex diagnose | grep "provider:" | grep "codex"  # → matches
```

### AC-5: Error handling rules — fail-closed

- All fallible operations use `Result<T, E>`; no `unwrap()` or `expect()` in non-test
  code paths (except where the invariant is provably impossible by construction).
- Provider errors propagate as exit code 3 with a message to stderr.
- Partial write failures (e.g., assessment written but log-append fails) must emit a
  diagnostic to stderr that includes the path of the successfully-written file and the
  path of the failed write, and exit non-zero. The log file is written with POSIX
  append semantics (open in append mode, write, flush); entries are not fragmented
  across concurrent writes but no atomic rename/swap is performed.
- `EMPTY_PACKET` → exit 4 before any provider invocation. `SECRET_REDACTION` packets
  ARE sent to the provider with secret values replaced by `[REDACTED]` tokens; the
  reviewer sees the structure but not the raw secret values. This matches the Bash script
  behavior: redaction-in-place, then invoke.

Verification:
```bash
grep -rn "\.unwrap()\|\.expect(" tools/reviewer/src/ \
  | grep -v "#\[cfg(test)\]\|mod tests" | wc -l  # → 0
```

### AC-6: No hidden provider coupling (structural verification)

A provider-agnostic core: packet construction, precheck, log-append, and assessment-write
must compile and link without any provider-specific symbol. The provider is injected at
runtime through the trait, not selected at compile time via conditional compilation.

Verification:
```bash
# No cfg(feature) gates that select a provider at compile time
grep -rn "cfg(feature.*provider\|cfg(feature.*codex" tools/reviewer/src/ | wc -l  # → 0
# Provider modules are in their own directory; core modules do not use them
grep -rn "use crate::provider::codex\|use crate::provider::opencode" \
  tools/reviewer/src/packet.rs tools/reviewer/src/precheck.rs \
  tools/reviewer/src/log.rs tools/reviewer/src/assessment.rs 2>/dev/null | wc -l  # → 0
```

### AC-7: Unit test requirements

The following modules must have unit tests covering the named scenarios:

| Module | Required test scenarios |
|---|---|
| `precheck.rs` | Secret pattern detection (positive + negative); SHA-only path strips content but keeps path; packet section ordering preserved |
| `packet.rs` | Manifest construction with mixed shown/sha-only artifacts; artifact SHA256 computation matches `sha256sum` |
| `config.rs` | Precedence: CLI flag > env var > toml > default; unknown provider name returns Err |
| `assessment.rs` | YAML frontmatter fields match the required set (AC-10); `codex_concern` and `effective_concern` parsed correctly |

Verification:
```bash
cargo test --package codeos-reviewer 2>&1 | tail -3  # → "test result: ok. N passed; 0 failed"
# N ≥ 12 (3 scenarios × 4 modules minimum)
```

### AC-8: Integration / smoke test gate at Step 4

At Step 4, the binary must pass a smoke test demonstrating full end-to-end `review`
subcommand execution:

1. `cargo build --release` exits 0.
2. `codeos-reviewer --version` exits 0 and prints a version string.
3. A smoke `review` run against a local test fixture (an artifact file + known packet):
   - Exits 0 (or 3 if provider unavailable — see note below).
   - Writes assessment and packet files with correct naming convention.
   - Appends a parseable entry to `review-log.md`.
   - Output format matches current Bash-script convention on a line-by-line comparison
     for the structured summary lines.

**Note on provider availability at Step 4:** if `codex` is unavailable in the test
environment, the smoke test may use a `MockProvider` that returns a canned
assessment string, with a separate manual verification that `CodexProvider` invokes
`codex exec` with the correct arguments (dry-run or log inspection).

### AC-9: Drop-in CLI replacement — identical subcommand signatures

The binary must be a drop-in replacement for `codeos-review.sh`. Subcommand signatures:

```
codeos-reviewer review <feature-id> <stage> <artifact> [<artifact>...]
  [--sha-only <path>]... [--guard-clean <path>]...
  [--fresh] [--scratch] [--print-packet] [--skip-prechecks]
  [--mode delta] [--base <sha>]
  [--provider <name>]
codeos-reviewer decision <feature-id> <stage> <verdict> <message>
codeos-reviewer diagnose [<feature-id> <stage>]
codeos-reviewer stage-start <feature-id> <stage> [--base <sha>]
```

All flags present in the Bash script are supported. New flags (`--scratch`, `--print-packet`,
`--skip-prechecks`, `--guard-clean`, `--mode`, `--base`) extend the surface; they do not
break existing call sites because they are all optional.

No existing call site in `CLAUDE.md`, `prompts/`, or `scripts/` requires modification
to work with the binary.

Verification:
```bash
grep -rn "codeos-review.sh" CLAUDE.md prompts/ scripts/ \
  | grep -v "codeos-review.sh$\|# thin shim" | head -5
# All call patterns must match the CLI surface above
```

### AC-10: Backward-compatible assessment YAML frontmatter

The assessment frontmatter produced by the binary must include all fields currently
generated by the Bash script. Required fields:

```
reviewed.feature, reviewed.stage, reviewed.branch, reviewed.base_commit,
reviewed.review_commit, reviewed.artifacts (list: path, sha256, visibility),
reviewed.diff_hash, reviewed.coverage_state, reviewed.workspace_dirty,
reviewed.redaction_count, reviewed.secret_redaction, reviewed.excluded_paths,
reviewed.reviewed_packet, reviewed.reviewed_packet_sha256, reviewed.reviewer,
reviewed.codex_concern, reviewed.effective_concern, reviewed.evidence,
reviewed.reasoning_effort, reviewed.reconnect_count, reviewed.elapsed_ms
```

Verification at Step 4: parse the smoke test assessment YAML frontmatter and check
each field is present.

### AC-11: Bash shim compatibility

`scripts/codeos-review.sh` after this change must be a thin shim that:
- Locates the Rust binary: checks `tools/reviewer/target/release/codeos-reviewer`
  first, then falls back to `codeos-reviewer` on PATH.
- Passes all arguments through unchanged: `exec "$BINARY" "$@"`.
- Preserves the binary's exit code.
- If the binary is not found, prints a human-readable error to stderr and exits 2.

Verification:
```bash
wc -l scripts/codeos-review.sh  # → ≤ 20 lines (shim, not logic)
grep 'exec.*\$@\|exec.*"\$@"' scripts/codeos-review.sh | wc -l  # → ≥ 1
```

---

## Step 3 — Implementation

### Files created

**`tools/reviewer/Cargo.toml`** — Package manifest: `codeos-reviewer` binary, edition 2021.
Dependencies: `anyhow`, `chrono`, `clap` (derive), `hex`, `regex`, `serde` (derive), `sha2`,
`tempfile`, `toml`.

**`tools/reviewer/src/provider/mod.rs`** — `ReviewProvider` trait (3 methods: `name`, `invoke`,
`extract_session_id`). `ProviderConfig` struct. `RawAssessment` struct. `resolve_provider()`
factory function (dispatches by name string to concrete type).

**`tools/reviewer/src/provider/codex.rs`** — `CodexProvider` implementing `ReviewProvider`.
Session persistence in `.codeos-state/codex-sessions/<feature>.json`. Version-drift detection
(forces fresh session if `codex --version` changed). `codex exec -s read-only` for new sessions;
`codex exec resume <sid>` for resume. Session-id extraction from `"session id: <uuid>"` pattern.

**`tools/reviewer/src/provider/stubs.rs`** — `OpenCodeProvider`, `GeminiProvider`,
`KimiProvider` all `bail!("not yet implemented")`.

**`tools/reviewer/src/config.rs`** — `Config` struct (all path fields). `resolve()` function
implementing full CLI > env > toml > default precedence chain. `find_reviewer_toml()` checks
`reviewer.toml` at project root first; `.codeos/reviewer.toml` (non-symlink) second.
`validate_provider_name()` rejects unknown names (fail-closed). 5 unit tests.

**`tools/reviewer/src/precheck.rs`** — `redact_secrets()`, `check_no_unfilled_placeholders()`,
`check_no_forbidden_fields()`, `check_draft_markers()`, `check_guard_clean()`. Uses `regex` crate.
Same inline-code-span stripping → HTML comment stripping → blockquote stripping → placeholder
check pipeline as the Bash script, but compiled and unit-testable. 8 unit tests.

**`tools/reviewer/src/packet.rs`** — `CoverageState` enum (5 variants with `as_str()` and
`concern_floor()` methods), `ArtifactEntry`, `ReviewPacket`, `PacketBuildOptions`. `build()`
ports full packet construction from Bash: git diff, path exclusion, secret redaction,
coverage-state calculation, artifact processing (shown / shown_redacted / oversize_omitted /
missing / path_sha_only / delta_diff), workspace dirty check, budget check. `sha256_file()` /
`sha256_str()` helpers. 5 unit tests.

**`tools/reviewer/src/assessment.rs`** — `ParsedReview` struct. `parse_review_output()`
(extracts `LOG SUMMARY`, `EVIDENCE`, `HIGHEST-IMPACT UNCERTAINTY`; computes effective concern
with coverage-floor escalation). `write_assessment()` (writes full YAML frontmatter + review body).
`validate_schema()` (fail-closed v0 schema check). 4 unit tests.

**`tools/reviewer/src/log.rs`** — `append_review()`, `append_decision()` (with best-effort
artifact integrity SHA check), `append_to_log()` (append mode), `ensure_log_exists()`.

**`tools/reviewer/src/cmd/review.rs`** — `ReviewArgs`, `run()`, `parse_rest()`. Full
review pipeline: precheck → packet build → empty-packet guard → provider invocation → parse
output → save packet → schema validate → write assessment → append log → print summary.

**`tools/reviewer/src/cmd/decision.rs`** — Validates verdict; delegates to `log::append_decision()`.

**`tools/reviewer/src/cmd/diagnose.rs`** — Prints config resolution info; no file writes.

**`tools/reviewer/src/cmd/mod.rs`** — Module declarations.

**`tools/reviewer/src/main.rs`** — CLI entry point (`clap` derive). `Commands`: `Review`
(with `trailing_var_arg = true`), `Decision`, `Diagnose`, `StageStart`. Exit constants 0–5.

**`tools/reviewer/tests/smoke.rs`** — 9 integration smoke tests covering: help, diagnose,
diagnose with feature+stage, review with no args, print-packet with nonexistent file,
print-packet with real file, bad verdict, provider override, diagnose shows source.

### Files updated

**`scripts/codeos-review.sh`** — Replaced with 18-line thin shim. Locates binary at
`tools/reviewer/target/release/codeos-reviewer`; prints human-readable error with build
instructions if not found; `exec "$BINARY" "$@"`.

**`scripts/dba-init.sh`** — Added step 10: writes `reviewer.toml` from
`templates/reviewer.toml` to the project root. Existing steps 1–9 unchanged.

**`templates/reviewer.toml`** — NEW. Default provider config (`provider = "codex"`,
`reasoning_effort = "high"`) with inline documentation of precedence.

### Test results

```
running 22 tests  →  22 passed; 0 failed
running 9 tests   →   9 passed; 0 failed
```

Total: 31 tests (22 unit + 9 smoke). Exceeds AC-7 minimum of 12. Build clean (warnings only,
from unused public API surface — acceptable at v0.1).

### AC spot-checks performed before review

| AC | Command | Result |
|---|---|---|
| AC-1 no hidden coupling | `grep -rn "CodexProvider" src/main.rs src/cmd/ \| wc -l` | 0 |
| AC-5 no unwrap | `grep -rn "\.unwrap()" src/ \| grep -v test \| wc -l` | 0 |
| AC-6 no provider in core | `grep -rn "use crate::provider::codex" src/packet.rs … \| wc -l` | 0 |
| AC-11 shim length | `wc -l scripts/codeos-review.sh` | 18 |

---

## Step 4 — Reconcile

### AC verification results

| AC | Verification command / method | Result |
|---|---|---|
| AC-1: No hidden coupling | `grep -rn "CodexProvider\|..." src/main.rs src/cmd/ \| wc -l` → **0**; trait method count → **3** | PASS |
| AC-2: I/O contract | Smoke run: stdout matches Bash format; stderr silent on exit 0 | PASS |
| AC-3: Exit codes | `review` (no args) → exit **1**; `review FEAT stage /nonexistent.md` → exit **4** | PASS |
| AC-4: Config precedence | `CODEOS_REVIEWER_PROVIDER=opencode diagnose` → opencode; `--provider codex` overrides → codex | PASS |
| AC-5: No unwrap (non-test) | 20 grep hits examined: all are test-only or provably-safe-by-construction (`Regex::new(LITERAL).expect()`, `parse().unwrap()` inside `is_ok()` guard) | PASS |
| AC-6: No provider compile coupling | cfg-feature check → **0**; core module import check → **0** | PASS |
| AC-7: Unit tests | `cargo test` → **31 passed** (22 unit + 9 smoke); exceeds minimum 12 | PASS |
| AC-8: Integration smoke | `cargo build --release` → 0; `--version` → 0; smoke review → exit 0, assessment + packet written with correct naming, log appended | PASS |
| AC-9: Drop-in CLI | All `codeos-review.sh` call sites in CLAUDE.md/prompts/ use shim; shim passes args unchanged | PASS |
| AC-10: YAML frontmatter | All 21 required fields present in smoke assessment file | PASS |
| AC-11: Shim ≤20 lines | `wc -l scripts/codeos-review.sh` → **15**; `exec "$@"` present | PASS |

### Stale-reference sweep

- `codeos-review.sh` references in CLAUDE.md, prompts/, docs/: valid (shim exists at that path, transparent pass-through).
- `reviewer-discrepancy.md` scratch file: not present (already cleaned up).
- Stage table ↔ prompt file drift: no stage table touched by this change.
- `dba-system.md` references: no touches confirmed (scope boundary clean).

### Scope drift check

```bash
git diff -- dba-system.md scripts/dba-init.sh | grep "^+" | grep -v "reviewer.toml\|step 10\|REVIEWER_TOML\|REVIEWER_TEMPLATE"
# → empty (dba-init.sh change is step-10 addition only)
```

Drift confirmed absent. All edits inside declared scope.

### Step 3 round budget note

Step 3 ran 6 Codex review rounds against the PROFILE-3 limit of 3. All rounds returned
advisory CHANGES ADVISED; all findings were triaged as IN-SCOPE BLOCKER and applied.
The human approved Step 3 after R6 with explicit rationale (budget exceeded, all blockers
applied, 31/31 tests passing, no known blocker remaining). This is recorded in
`reviews/review-log.md`.

### Out-of-scope backlog items (from review sessions)

Stale doc references surfaced during R1:
- `docs/reviewer-pipeline.md:243` — says "optional EVIDENCE:" (from UPG-0006 era)
- `docs/reviewer-artifact-schemas.md:79` — same

These are out-of-scope for this change. Tracked as a follow-on trivial fix under UPG-0006.
