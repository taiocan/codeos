---
module: reviewer-engine
verified_against_commit: 175cca5
---

# Module Design Note: reviewer engine

<!--
Descriptive documentation of how `dba/04-tools/reviewer/engine` currently works. Explanatory only:
the reviewer tool contract selected by the active DBA configuration is the authoritative external
interface, and the code is the truth about actual implementation. If this note disagrees with
either, the note is stale and gets corrected.
-->

## Purpose

Turn a named set of repository files into one advisory, commit-pinned review record. The engine owns
evidence selection, secret redaction, packet construction, Codex invocation, response parsing, and
append-only record keeping.

It does not own review timing or waiver behavior (the selected review policy), approval authority
(every output is advisory; the human decides), or DBA lifecycle semantics. `inspect-architecture-scopes`
validates scope metadata and reports it — it never judges architectural sufficiency or decides
whether the architecture gate applies.

## How it works

The `review` path, which is the longest and contains every other command's work as a prefix:

1. `codeos-review.sh` requires the caller to be inside a Git repository, then resolves the compiled
   binary from the script's own **physical** location — following the `.codeos/toolkit` symlink to
   where the binary actually lives, rather than the calling project's root — and `exec`s it.
2. `main.rs` parses the CLI, discovers the repository root via `git rev-parse --show-toplevel`, and
   changes into it so every later relative path means the same thing.
   `inspect-architecture-scopes` returns here, before configuration is resolved, because it needs
   none.
3. `config::resolve` detects self-development (a root `dba-system.md` plus `dba/00-entry`) and
   selects `maintenance/reviews` over `.codeos/05-review/reviews` accordingly. Reasoning effort
   resolves environment → `reviewer.toml` → the compiled `high` default. Packet-budget mode resolves
   to fail by default or the explicit `warn` operator override; unknown values and TOML keys fail here.
4. `cmd::review::prepare` — shared verbatim with `plan` — validates the feature and stage
   identifiers, canonicalizes every supplied path and rejects any that resolves outside the
   repository, rejects a path passed as both an artifact and `--sha-only`, and resolves `--base` to
   a real commit.
5. `precheck` hard-fails on unfilled template placeholders and the retired `latest_review:` field,
   warns on draft markers, and enforces `--guard-clean` paths against `HEAD`.
6. `packet::build` selects the evidence: it computes the diff (against `--base` or `HEAD`), drops
   path- and size-excluded files, redacts secrets from both diff and artifact text, renders each
   artifact as full content, hash-only, delta, or omitted-with-reason, and derives a single
   `coverage_state` in which the most severe condition wins.
7. The packet text is assembled in order: the reviewer task prompt; the canonical reader-oriented
   guidance and Codeos terminology; the canonical project terminology when it exists; a manifest
   with byte counts and hashes; review context; stage-specific checks and expected output; the
   artifacts block; and the filtered diff. Communication context is not counted as review evidence.
8. **`plan` stops here** and prints the selection, including whether spending policy would refuse an
   oversized Codex review. Packet construction itself never enforces that policy. `review` continues.
9. Before round derivation or provider access, a Codex-backed review refuses an over-budget packet
   unless the `warn` operator override is active.
10. `log::compute_review_round` derives the round by counting matching entries already in the
    append-only log, and `format_review_id` produces `REV__<feature>__<stage>__R<N>`.
11. `codex::invoke` enforces the budget invariant again at the sole model-spawning boundary, creates
    an empty working directory, and applies a generated least-privilege Codex permission profile. A
    local preflight must read an allowed runtime canary and must fail to read repository, Codex-state,
    and unrelated canaries. Only then does Codeos start a fresh, ephemeral `codex exec`, suppressing
    project instructions, user configuration, and exec-policy rules and writing the packet to stdin.
    There is no provider-session resume or unisolated fallback. The working tree is compared before
    and after; any difference prints a read-only warning. With
    `--assessment` no process is started at all: the reply text is read from the named file and
    tagged `RunSource::External`, which is sequenced as `EXT__…__A<N>` and never advances the round.
12. `assessment` parses the response — `LOG SUMMARY`, `EVIDENCE`, and the finding blocks — and
    escalates the concern to the coverage floor when evidence was incomplete.
13. `validate_schema` fail-closes before anything durable is written. Then the packet file, the
    assessment file, and the log entry are written in that order, each error message naming what
    already landed.

`plan` and `review` share `prepare`, so `plan --emit-packet` exports the bytes `review` would send:
there is one packet construction path, and an external assessment is therefore evidence about the
same packet a Codex-backed review would have seen.

## Main parts

- **`main.rs`** — CLI surface, repository-root discovery, and the stable exit codes
  (`0` success, `1` usage, `2` config, `3` provider, `4` packet, `5` write).
- **`config.rs`** — resolves effort and the self-development versus downstream record locations.
- **`cmd/`** — one module per operation. `review.rs` holds `prepare`, the shared validation and
  evidence-selection path.
- **`precheck.rs`** — artifact hygiene gates and the secret-redaction regexes used by `packet`.
- **`packet.rs`** — the largest part: evidence selection, coverage state, budget accounting, and the
  per-stage check and expected-output tables.
- **`codex.rs`** — the only module that knows the Codex CLI: effective-isolation preflight, flags,
  JSONL event shapes, and process handling.
- **`run.rs`** — `ReviewerRun` and `RunSource`: the reply text plus where it came from. A provenance
  record, not a provider abstraction — there is no dispatch, trait, or configuration behind it, and
  the two sources carry different fields so nothing is filled with placeholder measurements.
- **`assessment.rs`** — response parsing, concern escalation, the assessment file, and the
  fail-closed schema check.
- **`log.rs`** — round computation, append-only log entries, and decision provenance.

## Data and state

**In:** artifact paths and flags; `CODEOS_REASONING_EFFORT`, `CODEOS_PACKET_BUDGET_BYTES`, and
`CODEOS_PACKET_BUDGET_MODE`;
`reviewer.toml`; the reviewer task prompt at `dba/03-prompts/review/codeos-reviewer-task.md`; the
reader-oriented guidance and applicable canonical terminology; Git (branch, `HEAD`, diffs, tracked
status, porcelain status).

**State written:** the append-only review log; one assessment `.md` and one packet `.txt` per review
under the resolved review root (or `.codeos-state/reviewer-scratch` with `--scratch`). Codex provider
sessions are ephemeral and no Codeos session state is written. Existing entries and assessments are
never rewritten.

**Out:** an exit code, a stdout summary naming the review id, both concerns, coverage, and the record
paths.

## Design choices

- **`plan` and `review` share `prepare`.** A dry run exercises the same validation, selection, and
  packet construction, so it cannot disagree with the review it previews.
- **The round is derived, never stored.** Counting log entries keeps one authoritative
  representation instead of a counter that could drift from the log it describes.
- **Coverage escalates, never lowers.** `concern_floor` raises the reviewer's own concern when
  evidence was redacted or omitted; nothing can lower it. An incomplete evidence set cannot yield a
  clean result.
- **Fail-closed at each boundary.** Schema validation runs before any durable write. An untracked
  artifact under `--base` is an error rather than a silently empty diff. A log that cannot be *read*
  is an error, while a log that does not *exist* is round 1 — `Path::exists()` is deliberately
  avoided because it collapses those two cases.
- **Findings are parsed only before the first line-anchored `LOG SUMMARY:`.** The CLI transcript
  echoes both the packaged prompt and the answer, so scanning the whole text would double-count
  every finding and could match the prompt's own placeholder line.
- **Codex details are quarantined in `codex.rs`.** Isolation mechanics, command flags, and event
  shapes are implementation details; the effective packet-only boundary is the guarantee.

## Dependencies and boundaries

The reviewer tool contract selected by the active DBA configuration owns the external interface:
operations, path rules, configuration precedence, and record guarantees. This engine implements that
contract and must not widen it silently.

The engine reads the reviewer task prompt from the toolkit and shells out to `git` and `codex`. It
must not write outside the resolved review root and `.codeos-state/`, must not invoke Codex unless the
effective preflight denies repository and user-state reads, and must configure the Codex run without
write or command-network access. It must not turn Codex output into an approval — the review policy
owns timing and waivers, and the human owns the decision.
