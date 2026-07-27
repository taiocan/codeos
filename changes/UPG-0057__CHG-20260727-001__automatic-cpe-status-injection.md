# Self-Development Change: UPG-0057__CHG-20260727-001 — automatic-cpe-status-injection

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0057
primary_feature_id: UPG-0057
change_id: CHG-20260727-001
slug: automatic-cpe-status-injection
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0057
related_features: [UPG-0056]
review_series: RVS__UPG-0057__CHG-20260727-001__S4
review_profile: PROFILE-5
review_state: REVIEWED
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

`CHG-A` (`CHG-20260726-003`) established the Controlled Plain English pattern and wired every
consuming prompt to it, including a fixed status line
(`Controlled Plain English status for this review: enabled` / `...: disabled`) that
`prompts/codeos-reviewer-task.md` recognizes among reviewed artifacts. But `CHG-A` discovered that
`tools/reviewer/src/packet.rs` embeds that shared template as static text — it has no code path to
read any config file on the invoker's behalf — so within `CHG-A` alone, that status line had to be
included by hand by whoever runs a review. This was explicitly accepted as `CHG-A`'s scope, not as
the discipline's final operating model, and `UPG-0057` was marked incomplete pending this change.

A human reviewer of that design rejected manual inclusion as the final state and proposed a
Rust-based automatic-injection-with-fail-closed-blocking alternative. That was weighed and replaced
with a narrower counter-proposal: `scripts/codeos-review.sh` (the bash wrapper `tools/reviewer`
already ships) can resolve the status itself and append a synthetic status artifact to the packet's
file list — using the Rust engine's existing "embed whatever paths you're given" behavior — with no
Rust changes and no fail-closed blocking of ordinary review generation over a cosmetic style toggle
(which would break the advisory/non-gatekeeping principle this toolkit protects everywhere else).
The human accepted this counter-proposal and set five refinement conditions, reproduced in full
below since they are this change's actual acceptance bar.

A second, related gap: several doctrine and prompt files instruct running the compiled
`codeos-reviewer` binary directly (bypassing the wrapper entirely), and `docs/reviewer-pipeline.md`
explicitly states direct binary invocation "remains a valid alternative." If that remains true,
automatic status injection in the wrapper is silently bypassable by anyone who invokes the binary
directly — the wrapper must become the one documented, supported entry point for this to actually
hold.

**What changes:**

- `scripts/codeos-review.sh` — before invoking the binary for a `review` or `plan` subcommand
  (the two subcommands that build a reviewer packet), resolve the Controlled Plain English status
  and inject it as an extra artifact:
  - **Context resolution:** if the caller's git root (already computed by the existing precondition
    check) equals `CODEOS_ROOT` (this repo itself), read the self-development config,
    `config/writing-discipline.yaml`; otherwise read the downstream project's
    `architecture/controlled-plain-english.yaml` relative to the caller's git root.
  - **Four-outcome resolution** (per `UPG-0056`'s Optional Mechanism Status Convention, unchanged):
    absent or exact `status: disabled` → `disabled`; exact `status: enabled` → `enabled`; anything
    else → configuration error.
  - **Valid resolved status (`enabled` or `disabled`):** write a synthetic temp file (via `mktemp`,
    a recognizable name pattern, quoted paths throughout, `trap` cleanup on both success and
    failure) containing the exact recognized line first, plus context lines:
    ```
    Controlled Plain English status for this review: enabled
    Source: config/writing-discipline.yaml
    Applicable scope: selfdev-step-3
    ```
    (or the downstream config path and the actual `<stage>` argument, as applicable), then append
    that temp file's path to the artifact list passed to the binary.
  - **Malformed or contradictory configuration** (the "anything else" outcome): stop with a clear
    error **before invoking the reviewer** — this is an invocation precondition failure, the same
    class as today's "binary not found" check, not a reviewer finding.
  - **Ordinary style non-compliance** (the content of generated prose): entirely unaffected by this
    change — remains a reviewer finding under existing advisory authority, never a packet-generation
    failure.
- `config/writing-discipline.yaml` (new file, new `config/` directory) — the self-development status
  file, proposed default `status: enabled` for consistency with `CHG-A`'s downstream default
  (flagged here for explicit human confirmation at this gate, since no prior decision fixed this
  file's own default).
- `dba-system.md` — "Default Advisory Review" → "How to run it" changes from
  `codeos-reviewer review <feature_id> <stage>` to `.codeos/scripts/codeos-review.sh review
  <feature_id> <stage>`, with a new sentence stating the wrapper is the supported entry point for
  automatic Controlled Plain English status injection, and that direct binary invocation bypasses
  it. The architecture-synthesis reviewer-coverage note (~line 285) and the Stage-ID cross-reference
  (~line 474) get the same command-form correction. The Call-site map's
  `codeos-reviewer-task.md` row is rewritten to describe automatic injection by the wrapper instead
  of manual inclusion by the invoker.
- `docs/reviewer-pipeline.md` — the "Direct binary invocation ... still works identically and
  remains a valid alternative" sentence (§12) is corrected: direct invocation still runs, but is no
  longer described as a supported alternative for Controlled Plain English purposes, since it
  bypasses the wrapper's injection step. A new subsection documents the injection mechanism itself
  (context resolution rule, four-outcome handling, temp-file hygiene, the three-way advisory
  distinction above).
- `prompts/00a-solution-discovery.md`, `prompts/pipeline-reviewer.md`, `prompts/03b-architecture-synthesis.md`
  — each has one inline `codeos-reviewer review ...` mention corrected to the wrapper form
  (`.codeos/scripts/codeos-review.sh review ...`), matching the doctrine-level correction above.
- `patterns/controlled-plain-english.md` — the "Consulted by" paragraph's sentence describing
  `codeos-reviewer-task.md`'s status line as "manually included by whoever invokes the review" is
  replaced with a description of automatic injection by `scripts/codeos-review.sh`; the pattern's
  configuration-neutral claim about `codeos-reviewer-task.md` itself is unchanged (it still never
  reads a config file — it receives an already-resolved line, now from the wrapper instead of a
  human).
- `CLAUDE.md` (repo root) — new `## Writing Discipline (Controlled Plain English)` section:
  references (does not redefine) `config/writing-discipline.yaml` and the pattern's layers; states
  Layer A always applies to self-dev chat; a per-section rule table (Change Intent / Acceptance
  Criteria / Implementation Plan sections = Layer B; Implementation Notes = factual reporting;
  review findings / Reconciliation = D1 always + D2 when enabled); restates the Reviewer Model (no
  historical-compliance audit); the `config/` vs `status/` placement note (evidence/config, not
  live mutable status); one new File Layout line for `config/`; the Assumptions-subsection
  convention (guidance only — no new formal field on `templates/codeos-change.md`; rendered only
  when a material assumption exists and no existing change-record section already represents it).
  The existing reviewer-invocation snippet in `CLAUDE.md` (~line 118) already uses
  `bash scripts/codeos-review.sh review ...` — no correction needed there.
- `prompts/codeos-self-dev.md` — new "Step 0b — Writing Discipline Check": read
  `config/writing-discipline.yaml` per the four-outcome table; apply the per-section rule table
  above when enabled. No new change-record trace-header field — non-retroactivity stays the
  one-sentence rule already stated in the pattern.
- `backlog/UPG-0057-controlled-plain-english-writing-discipline.md` — Feature Thread updated: new
  `CHG-20260727-001` row, and the "Status note" rewritten once this change completes to state that
  manual inclusion is no longer part of the operating model.
- `status/self-development.md` — dashboard row bookkeeping for this change's own step transitions
  (Step 1 → Step 2 → Step 3 → Step 4), updated at each gate, same as every other change in this
  repo's history; not a content edit to the mechanism itself.

**Scope boundary — what stays the same:**

- `tools/reviewer/src/*` (the Rust engine) is **not touched** by this change — no new subcommand, no
  new flag, no new parsing logic. The wrapper only appends an ordinary artifact path to the existing
  trailing-args list `review`/`plan` already accept.
- The exact recognized status-line text already implemented in `prompts/codeos-reviewer-task.md` by
  `CHG-A` is unchanged — this change produces that same line automatically instead of requiring a
  human to type it.
- `UPG-0056`'s Optional Mechanism Status Convention (the four-outcome table, whitespace pinning,
  missing-means-disabled rule) is consumed, not modified.
- No new Stage ID, no new mandatory approval gate, no new Non-Negotiable Rule.
- Historical `changes/*.md` records that mention `codeos-reviewer review` directly (many, across
  the toolkit's history) are **not** rewritten — those are append-only history of what command was
  run at the time, not living doctrine telling someone what command to run today. Only
  currently-authoritative doctrine and prompt text is corrected.
- Ordinary style/CPE non-compliance in generated prose remains advisory-only — this change never
  makes review-packet generation fail over content quality, only over the wrapper's own
  configuration-read failing.

**Human refinement conditions this change must satisfy (verbatim, from the gate that authorized this
CHG):**

1. Make the wrapper the authoritative entry point — establish `scripts/codeos-review.sh` as the
   supported review entry point, or identify/migrate every existing direct reviewer invocation, to
   prevent a silent manual-bypass path via direct `codeos-reviewer` binary invocation.
2. Preserve the advisory boundary — exactly: valid resolved status → inject it automatically; valid
   disabled/not-applicable status → inject that state and continue; malformed or contradictory
   configuration → fail before invoking the reviewer; ordinary style non-compliance → reviewer
   finding only, never a packet-generation failure.
3. Make the generated artifact explicit — the synthetic temp file is clearly synthetic and
   deterministic (status line + `Source:` + `Applicable scope:`), with safe temp-file handling,
   `trap` cleanup, quoted paths, and a recognizable filename.
4. Keep `CHG-A` and `CHG-B` historically honest — `CHG-A` established the pattern and consumer
   wiring; `CHG-B` completes automatic status delivery at the supported reviewer invocation
   boundary; manual inclusion is not part of the final operating model.
5. `CHG-B` acceptance criteria seed from the ten-item list the human specified, quoted verbatim:
   downstream stage ID resolves from architecture config; self-dev stage ID resolves from
   writing-discipline config; generated status artifact automatically appended; no
   operator-supplied status path required; enabled/disabled/not-applicable outcomes represented
   correctly; malformed/ambiguous config stops invocation with clear error; valid
   disabled/not-applicable status does not block review; Rust reviewer remains unchanged; all
   supported review paths use the wrapper or equivalent; temp files cleaned after success and
   failure.

**Class:** script-tooling + downstream-doctrine + self-dev-governance (three-way — see rationale
below).
**Scope axis:** both.
**Backlog item:** `backlog/UPG-0057-controlled-plain-english-writing-discipline.md`

**Why three classes, one change:** the core implementation (`scripts/codeos-review.sh`) is
script-tooling; correcting `dba-system.md` and downstream-facing prompt text to name the wrapper as
authoritative is downstream-doctrine; adding `CLAUDE.md`'s Writing Discipline section and
`prompts/codeos-self-dev.md`'s Step 0b is self-dev-governance. Splitting these three into separate
CHGs would separate one coherent "wrapper becomes the authoritative, self-resolving entry point"
change into artificially disconnected pieces reviewed against different, narrower criteria. Per
`CLAUDE.md`'s triage table, the union of all three classes' requirements applies (downstream-
compatibility criteria + grep cross-reference + reviewer scope-triage, and scope-drift review for
the `CLAUDE.md`/self-dev-loop edits). **Review profile:** `PROFILE-5` (the most stringent of the
three, since this touches `CLAUDE.md` itself).

---

## Acceptance Criteria

| # | Criterion | How it will be verified |
|---|---|---|
| 1 | Downstream review invocations resolve `architecture/controlled-plain-english.yaml` relative to the caller's git root (not through `.codeos`) | Read `scripts/codeos-review.sh`'s context-resolution logic; fixture test from a downstream-shaped directory |
| 2 | Self-development review invocations resolve `config/writing-discipline.yaml` relative to `CODEOS_ROOT` | Read the same logic; fixture test invoked from within this repo |
| 3 | The generated status artifact is automatically appended to the packet's artifact list for `review` and `plan` subcommands — no operator-supplied status path required | Two `--print-packet` smoke runs, one via `review` and one via `plan`, each showing the synthetic artifact's content without it being named on the command line |
| 4 | `enabled`, `disabled`, and absent-config (→ disabled) outcomes are all represented correctly in the injected line | Three fixture runs, one per case (`status: enabled`, `status: disabled`, no config file present), each asserting the exact injected line text |
| 5 | Malformed or ambiguous configuration stops invocation with a clear error before the reviewer runs | Fixture with an invalid config file; asserts non-zero exit and no packet/log/assessment file written |
| 6 | A valid disabled, not-applicable (absent config), or otherwise non-blocking status does not block review — it is injected and the review proceeds normally | Two fixture runs — one with `status: disabled`, one with no config file present — each asserting the reviewer still runs and produces an assessment |
| 7 | `tools/reviewer/src/*` is unchanged by this change | `git diff --stat -- tools/reviewer/src` is empty; `cargo test --release --manifest-path tools/reviewer/Cargo.toml` still passes at the same count as before this change |
| 8 | All currently-authoritative doctrine and prompt text that instructs running the reviewer names the wrapper (`.codeos/scripts/codeos-review.sh` downstream, `scripts/codeos-review.sh` self-dev), not the raw binary | grep for `` `codeos-reviewer review `` and `` `codeos-reviewer plan `` across `dba-system.md`, `CLAUDE.md`, `prompts/*.md`, `docs/reviewer-pipeline.md` (excluding `changes/*.md` history) returns no remaining bare-binary invocation instruction |
| 9 | `docs/reviewer-pipeline.md` no longer describes direct binary invocation as a valid supported alternative for review purposes | Read-through of §12; the corrected sentence states the bypass risk explicitly |
| 10 | Temp files are cleaned up after both a successful and a failing invocation | `trap` fixture test: run once to success, once forcing a mid-script failure, assert no leftover temp file matching the recognizable pattern in either case |
| 11 | The exact status-line text `codeos-reviewer-task.md` already recognizes (`Controlled Plain English status for this review: enabled`/`disabled`) is the temp file's first line, byte-identical to `CHG-A`'s implementation | grep the generated temp file in a fixture run against the exact string `codeos-reviewer-task.md` checks for |
| 12 | `CLAUDE.md`'s new Writing Discipline section does not introduce a new mandatory template field on `templates/codeos-change.md` | `git diff -- templates/codeos-change.md` is empty; the Assumptions-subsection convention is stated as guidance only |
| 13 | `config/writing-discipline.yaml`'s default value is explicitly confirmed at this gate (`status: enabled`, approved at the Step 1 gate) | Read the shipped file's content in Step 4; confirm it matches what was approved |

<!-- Downstream-compatibility: AC1, AC8, AC9 verify the downstream-facing correction.
Script-tooling: AC3, AC5, AC6, AC7, AC10, AC11 verify I/O behavior, fail-closed cases, cleanup.
Self-dev-governance: AC2, AC12, AC13 verify the CLAUDE.md/config addition doesn't drift scope. -->

---

## Implementation Notes

**`scripts/codeos-review.sh`.** Added automatic Controlled Plain English injection for the
`review` and `plan` subcommands only (the two that build a packet); every other subcommand passes
through the original unmodified `exec "${BINARY}" "$@"` path. Context resolution compares the
caller's own physical git root (`git rev-parse --show-toplevel`, resolved with `pwd -P`) against
`CODEOS_ROOT` — equal means a self-development review (`config/writing-discipline.yaml`), unequal
means a downstream review (`architecture/controlled-plain-english.yaml` relative to the caller's
own root, not through `.codeos`). Status resolution reads the file per `UPG-0056`'s four-outcome
table using a `mapfile`-less line-by-line filter (only genuinely empty lines count as "blank" —
whitespace-only lines are content, correctly making the file invalid if they're not the exact
status line); CRLF is normalized to LF before comparison; internal whitespace/case are not
normalized. A resolved `enabled`/`disabled` status is written to a `mktemp`-created, recognizably
named (`codeos-cpe-status.*`) temp file containing the exact line
`codeos-reviewer-task.md` already recognizes, plus `Source:` and `Applicable scope:` context lines,
then appended to the argument list passed to the binary. A malformed/contradictory file exits 7
*before* any temp file is created or the binary is invoked — an invocation precondition failure,
the same class as the existing "binary not found" exit 2.

**Implementation-time discovery (exec vs. cleanup):** the original script ended every path in
`exec "${BINARY}" "$@"`, which replaces the shell's own process image — a bash `EXIT` trap never
fires across `exec`, since the shell never "returns" to run it. Since a temp file's cleanup depends
on that trap, the injected-artifact branch cannot end in `exec`: it invokes the binary as an
ordinary subprocess and then exits with that subprocess's exact exit code, letting the normal
(non-`exec`) shell exit run the `trap` on both the success and failure paths. This was not
anticipated in the Step 1/2 design text (which only said "trap cleanup") and is called out here so
Reconcile's evidence for AC10 explains *why* the implementation isn't a straight `exec` for this
one branch. All other subcommands still end in `exec`, unchanged.

**`config/writing-discipline.yaml`** (new file, new `config/` directory) — contains exactly
`status: enabled`, per the default proposed in Change Intent and approved at the Step 1 gate.

**`dba-system.md`** — "Default Advisory Review"'s "How to run it" now shows
`.codeos/scripts/codeos-review.sh review <feature_id> <stage>`, with a new paragraph stating the
wrapper is the supported entry point for Controlled Plain English injection and that direct binary
invocation bypasses it. The architecture-synthesis reviewer-coverage note and the Stage-ID
cross-reference near "What You Do at Each Stage" got the same command-form correction. The
Call-site map's `codeos-reviewer-task.md` row now describes automatic injection by the wrapper
instead of manual inclusion by the invoker.

**`docs/reviewer-pipeline.md`** — the "Direct binary invocation ... remains a valid alternative"
sentence (§12) is corrected to state the wrapper is the supported entry point and that direct
invocation skips injection. A new §12a documents the injection mechanism itself (context
resolution, four-outcome handling, temp-file hygiene, the three-way advisory distinction). Five
further CLI example blocks in §§ on Full/Delta/SHA-Only/Combining Modes and "Preview a plan before
reviewing" were also corrected to the wrapper form — these are concrete runnable examples, not
abstract grammar, so they fall within AC8's "instructs running the reviewer" scope. **Not
touched:** `docs/superpowers/specs/2026-07-11-reviewer-packet-efficiency-design.md` — a dated
historical design spec, analogous to `changes/*.md` history, out of scope per Change Intent's
explicit exclusion of historical records; and `tools/reviewer/src/main.rs`'s own `--help` docstring
examples, which correctly refer to the binary by its own name since they describe running the
compiled binary directly for `--help` output, and are additionally protected by AC7's "no Rust
changes" boundary.

**`prompts/00a-solution-discovery.md`, `prompts/pipeline-reviewer.md`,
`prompts/03b-architecture-synthesis.md`** — each had exactly one inline `codeos-reviewer review
...` mention corrected to the wrapper form, matching the doctrine-level correction.

**`patterns/controlled-plain-english.md`** — the "Consulted by" paragraph's description of the
status line as "manually included by whoever invokes the review" is replaced with a description of
automatic injection by `scripts/codeos-review.sh` / `.codeos/scripts/codeos-review.sh`. The
configuration-neutral claim about `codeos-reviewer-task.md` itself is unchanged — it still never
reads a config file; it now receives an already-resolved line from the wrapper instead of a human.

**`CLAUDE.md`** — new "Writing Discipline (Controlled Plain English)" section: references (does
not redefine) the pattern; states Layer A is unconditional; the per-section rule table; the
Reviewer Model restated; the `config/` vs `status/` placement note; the Assumptions-subsection
convention as guidance only (confirmed no edit to `templates/codeos-change.md` — verified empty
diff below). One new `config/` line added to File Layout. The existing reviewer-invocation snippet
(`bash scripts/codeos-review.sh review ...`) already used the wrapper form and needed no
correction.

**`prompts/codeos-self-dev.md`** — new "Step 0b — Writing Discipline Check": reads
`config/writing-discipline.yaml` per the four-outcome table; applies the per-section rule table
when enabled; explicitly notes this step (writing discipline while authoring) is distinct from the
wrapper's own automatic injection (what the reviewer is told).

**Verification run during implementation** (ahead of formal Reconcile, to catch design errors
before they're locked in):
- `bash -n scripts/codeos-review.sh` — syntax OK.
- `plan`/`review --print-packet` smoke runs from within this repo (self-dev context) with
  `status: enabled`, `status: disabled`, and the config file absent — each produced the correct
  exact injected line, with no operator-supplied status path on the command line.
- A malformed config (`status: enabled` plus a second content line) exited 7 with a clear message,
  before any packet or temp file was produced.
- A scratch downstream-shaped git repo with `.codeos` symlinked to this repo and its own
  `architecture/controlled-plain-english.yaml` correctly resolved *that* file, not
  `config/writing-discipline.yaml`, confirming context resolution.
- No `codeos-cpe-status.*` temp file remained in `/tmp` after any of the above runs, including the
  malformed-config exit (which never creates one) and every successful run (cleaned up by the
  post-subprocess `rm`/trap).
- `git diff --stat -- tools/reviewer/src` and `git diff --stat -- templates/codeos-change.md` both
  empty, confirming AC7 and AC12.
- `cargo test --release --manifest-path tools/reviewer/Cargo.toml`: 182 passed, 0 failed — same
  count as before this change (no Rust files touched).

No out-of-scope items were discovered during implementation.

**Step 3 review evidence — recorded false positive (human decision, not a Codex finding):** Step 3
rounds R1-R3 (`REV__UPG-0057__CHG-20260727-001__selfdev-step-3__R1` through `__R3`) each reported
packet `coverage: SECRET_REDACTION (redactions: 1)`. All three redactions point to the same
location: the literal text `Secret / non-secret:` — an empty template field label inside a
structured-entry template in `prompts/00a-solution-discovery.md` (pre-existing content, not
touched by this change; it asks a human to classify a *future* config item as secret or not, it
does not contain any actual secret value). This is a confirmed reviewer tooling false positive, not
an unresolved implementation defect, not a real secret, and not something introduced or modified by
this change. Recorded here per explicit human decision at the Step 3 gate, so it is not mistaken
for outstanding work in a future reread of this record.

---

## Reconciliation

**Acceptance verification:**

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Downstream review invocations resolve `architecture/controlled-plain-english.yaml` relative to the caller's git root | PASS | Scratch downstream-shaped repo with `.codeos` symlinked to this repo, its own `architecture/controlled-plain-english.yaml`; `.codeos/scripts/codeos-review.sh review fake-downstream 2 --print-packet dummy.md` correctly showed `Source: <scratch-repo>/architecture/controlled-plain-english.yaml`, not `config/writing-discipline.yaml` |
| 2 | Self-development review invocations resolve `config/writing-discipline.yaml` relative to `CODEOS_ROOT` | PASS | `bash scripts/codeos-review.sh review smoke-test selfdev-step-1 --print-packet CLAUDE.md` from within this repo showed `Source: /home/rimo/projects/Codeos/config/writing-discipline.yaml` |
| 3 | Status artifact automatically appended for `review` and `plan`, no operator-supplied path | PASS | `plan smoke-test selfdev-step-1 --print-packet CLAUDE.md` listed the synthetic artifact among resolved artifacts; `review smoke-test selfdev-step-1 --print-packet CLAUDE.md` showed its full 3-line content — neither command named the file on its own command line |
| 4 | `enabled`/`disabled`/absent all represented correctly in the injected line | PASS | Three fixture runs: `status: enabled` → line ends `enabled`; `status: disabled` → line ends `disabled`; config file absent (moved aside) → line ends `disabled` |
| 5 | Malformed/ambiguous config stops invocation before the reviewer runs | PASS | `status: enabled` plus a second content line → exit 7, error message printed, no packet/log/assessment file written, no temp file created |
| 6 | Valid disabled/not-applicable status does not block review | PASS | Two real (non-`--print-packet`) invocations, both actually invoking Codex and producing a real assessment: (a) `status: disabled` — `review AC6-smoke-test selfdev-step-1 CLAUDE.md` → `REV__AC6-smoke-test__selfdev-step-1__R1`, NO OBJECTION; (b) config file absent (moved aside) — `review AC6-absent-smoke-test selfdev-step-1 CLAUDE.md` → `REV__AC6-absent-smoke-test__selfdev-step-1__R1`, DO NOT ADVANCE. The verdict differs because that run's only artifact (`CLAUDE.md`) happened to contain substantive content the reviewer engaged with — irrelevant to this AC, which only requires that the injected status not block invocation. Both runs prove exactly that: the reviewer ran and produced a real assessment in both the disabled and absent cases |
| 7 | `tools/reviewer/src/*` unchanged | PASS | `git diff --stat -- tools/reviewer/src` empty (checked before and after all edits); `cargo test --release --manifest-path tools/reviewer/Cargo.toml` — 182 passed, 0 failed, same count as before this change |
| 8 | All currently-authoritative doctrine/prompt text names the wrapper, not the raw binary | PASS | `grep -rn "codeos-reviewer review \|codeos-reviewer plan " dba-system.md CLAUDE.md prompts/ docs/reviewer-pipeline.md` (excluding `changes/*.md` history and `tools/reviewer/src/main.rs`'s own `--help` docstring, both explicitly out of scope) returns no remaining bare-binary invocation instruction |
| 9 | `docs/reviewer-pipeline.md` no longer describes direct binary invocation as a supported alternative | PASS | §12's corrected sentence and new §12a state the wrapper is the supported entry point and that direct invocation skips injection; §10 rewritten to match |
| 10 | Temp files cleaned up after success and failure | PASS | `ls /tmp/codeos-cpe-status.*` returned "No such file or directory" after every successful smoke run. Two distinct failure shapes verified: (a) a pre-creation failure (malformed config, exit 7) never creates a temp file in the first place; (b) a post-creation failure — `review AC10-fail-test selfdev-step-1 /nonexistent/path/does-not-exist.md`, a valid status resolved and a temp file was created, but the underlying binary then failed (exit 4, "artifact not found") — the temp file was still cleaned up (confirmed via `ls /tmp/codeos-cpe-status.*` immediately after, same "No such file or directory" result), proving the `trap`-driven cleanup covers a failure that occurs after the file already exists, not only the case where none was ever created |
| 11 | Exact recognized status-line text, byte-identical to `CHG-A`'s implementation | PASS | Injected line read exactly `Controlled Plain English status for this review: enabled` / `...: disabled` in every fixture — the same string `codeos-reviewer-task.md` (`CHG-A`) already greps for |
| 12 | No new mandatory template field on `templates/codeos-change.md` | PASS | `git diff -- templates/codeos-change.md` empty; the Assumptions-subsection convention in `CLAUDE.md` is stated as guidance only |
| 13 | `config/writing-discipline.yaml`'s default value confirmed at the Step 1 gate | PASS | File contains exactly `status: enabled\n`, matching what was approved |

**Consistency sweep (grep):** `git status --short` on every file named in Change Intent's "What
changes" shows exactly those files touched, nothing else — no stray edits. The stray `§13 below`
cross-reference found in Step 3 R3 (should have been `§12a`) is fixed. No remaining "manually
included" / "included by hand" / "whoever invokes" language in `patterns/controlled-plain-
english.md`, `dba-system.md`, or `CLAUDE.md` (grep clean, re-checked at Reconcile). No stray
`codeos-reviewer review `/`codeos-reviewer plan ` bare-binary instruction remains in any
currently-authoritative file (AC8's evidence, re-verified at Reconcile after the R2/R3 fixes).

**Findings scope-triage (all Step 1-4 review rounds, this CHG):**

| Finding | Round | Classification | Resolution |
|---|---|---|---|
| Missing `status/self-development.md` in Change Intent's "What changes" | Step 1 R1 | IN-SCOPE BLOCKER | Fixed |
| Acceptance Criteria prematurely drafted inside Step 1 submission | Step 1 R1 | IN-SCOPE BLOCKER | Fixed — moved to Step 2 placeholder |
| Step 2 placeholder falsely claimed the ten-item list was quoted in Change Intent | Step 1 R2 | IN-SCOPE BLOCKER | Fixed — ten-item list quoted verbatim |
| AC3/AC6/AC8 claimed broader coverage than their verification steps checked | Step 2 R1 | IN-SCOPE BLOCKER | Fixed — verification steps now name every case explicitly |
| `docs/reviewer-pipeline.md` §10 contradicted the implemented wrapper behavior | Step 3 R1 | IN-SCOPE BLOCKER | Fixed |
| Secret-redaction coverage flag on a pre-existing, unrelated template field | Step 3 R1-R3 | Confirmed false positive (not a finding requiring a fix) | Recorded in Implementation Notes per explicit human decision at the Step 3 gate |
| "Preview a plan before reviewing" prose still named the raw binary | Step 3 R2 | IN-SCOPE BLOCKER | Fixed |
| Stray `§13 below` cross-reference (should be `§12a`) | Step 3 R3 | IN-SCOPE NON-BLOCKER | Fixed |
| Trace header's `review_series` was `null` though deterministically computable in advance; `review_state` should be `IN_REVIEW`, not `DRAFT`, while awaiting a round | Step 4 R1 | IN-SCOPE BLOCKER | Fixed — stable `RVS__…__S4` series id filled in, `review_state: IN_REVIEW` |
| AC6 marked PASS with only one of its two required real-invocation cases run (disabled, not absent-config) | Step 4 R1 | IN-SCOPE BLOCKER | Fixed — second real invocation run with the config file absent, evidence added |
| AC10 marked PASS without proving cleanup after a failure occurring *after* temp-file creation | Step 4 R1 | IN-SCOPE BLOCKER | Fixed — post-creation-failure test run (missing artifact, exit 4), evidence added |
| This backlog Feature Thread's Reviews table recorded every live round and the CHG-B Changes-table row said `DRAFT`, both violating `codeos-self-dev.md`'s stated surface-ownership model | Step 4 R2 | IN-SCOPE BLOCKER | Fixed — Reviews table condensed to series-row + accepted-verdict-summary format for the whole backlog file; Changes-table row corrected to `IN_PROGRESS` |
| The same backlog file's "Findings Tracked" table still stored full finding prose ("Why" text), contradicting the Feature Thread's own "Compact links/IDs only" header and `codeos-self-dev.md`'s "ids and links, not full review text" rule — the Step 4 R2 fix corrected the Reviews table but missed this second table | Step 4 R3 | IN-SCOPE BLOCKER | Fixed — condensed to one row per review series (series id, classification counts, a pointer to "see change record Reconciliation"), removing all duplicated finding prose. **PROFILE-5's 3-round/step budget is now exhausted for Step 4** — fixed inline per `CLAUDE.md`'s budget-exceeded rule; no further automatic round; escalated to human decision |

No OUT-OF-SCOPE BACKLOG, REJECTED, or SELF-REFERENCE findings this CHG. No follow-up feature
spawned.

**Completion statement.** All 13 Acceptance Criteria pass. `tools/reviewer/src/*` and
`templates/codeos-change.md` are unchanged, confirming the "no Rust changes, no new template
field" scope boundary held. `scripts/codeos-review.sh` is now the sole path that automatically
resolves and injects Controlled Plain English status for both the downstream and self-development
branches of a review; every currently-authoritative doctrine and prompt reference to running the
reviewer names that wrapper, not the raw binary. Manual inclusion (`CHG-A`'s interim description)
is no longer part of the operating model — automatic injection replaces it. Per the human's
gate decision authorizing `CHG-B`: **`CHG-A` established the pattern and consumer wiring; `CHG-B`
completes automatic status delivery at the supported reviewer invocation boundary.** With this
change accepted, **`UPG-0057` as a whole is complete** — both `CHG-A` and `CHG-B` have reached
`COMPLETE`.
