---
feature_id: UPG-0032
slug: rust-reviewer-engine-multi-provider
title: Rust Reviewer Engine with Multi-Provider Support
status: IN_PROGRESS
priority: P2
depends_on: [UPG-0027]
related_features: [UPG-0015, UPG-0003, UPG-0018]
supersedes: [UPG-0018]
superseded_by: []
---

# Upgrade: rust-reviewer-engine-multi-provider — Rust Reviewer Engine with Multi-Provider Support

**Priority**: P2 (rises to P1 if a second provider is needed near-term)
**Status**: IN_PROGRESS
**Type**: script-tooling
**Supersedes**: UPG-0018 (Typed reviewer engine to replace the Bash pilot)
**Related**: UPG-0003 (reviewer pilot), UPG-0015 (decision-integrity), UPG-0027 (packet architecture)

---

## Problem

`scripts/codeos-review.sh` solves the right problem — build a packet, invoke an AI reviewer,
save the assessment, append the log — but Bash is the wrong long-term host for this pipeline,
and the tool currently serves only one context (self-dev) and one provider (OpenAI `codex`).

**Bash fragility.** The precheck pipeline is a chain of `sed`/`grep` compositions. UPG-0031
already hit a silent failure: `sed` range deletion swallowed lines when `<!--` appeared inside
an inline code span. The fix required knowing GNU sed's line-by-line range semantics. More
pipeline steps produce more subtle ordering hazards with no compiler to catch them.

**Provider lock-in.** The script is hard-wired to `codex exec`. Switching to OpenCode, Kimi
Code, or Gemini requires forking the invocation logic, re-implementing session-ID extraction,
and re-testing the whole pipeline. There is no abstraction boundary.

**Single-context limitation.** The reviewer currently operates only in the self-dev loop. The
same advisory reviewer pattern — packet → AI review → assessment → log — is equally valuable
inside the downstream DBA 9-stage workflow (before Stage 1 Intent approval, Stage 2 Contract,
Stage 3 Schema, etc.). A self-dev-only tool cannot serve that need without significant
script duplication or awkward path hacks.

**Untestable units.** Packet construction, secret filtering, precheck, coverage-state
classification, and assessment-write are entangled in one script. Only smoke tests are
possible; no unit tests.

**Typed state as strings.** `PACKET_COVERAGE_STATE`, `PACKET_DELTA_MODE`, exit codes, and
coverage categories are plain strings. A typo is a silent bug.

---

## Upgrade

A small Rust CLI in `tools/reviewer/` that owns the full reviewer pipeline, is configurable
by provider, and is designed to work in both self-dev and downstream DBA project contexts.

### Provider abstraction

```rust
trait ReviewProvider {
    fn name(&self) -> &str;
    fn invoke(&self, packet: &ReviewPacket, cfg: &ProviderConfig) -> Result<RawAssessment>;
    fn extract_session_id(&self, raw: &str) -> Option<String>;
}
```

Provider implementations: `CodexProvider` (wraps `codex exec`). Future add-ons:
`OpenCodeProvider`, `GeminiProvider`, `KimiProvider`. Each lives in its own module; adding a
new provider does not touch packet construction, precheck, or log-append logic.

Provider selection: `.codeos/reviewer.toml` or env var `CODEOS_REVIEWER_PROVIDER=opencode`.
CLI flag `--provider` overrides for one-off use.

### Typed packet and precheck

`ReviewPacket`, `PacketManifest`, `ArtifactEntry`, `CoverageState` enum. Precheck is a pure
function over an `&str` slice — no shell pipeline, fully unit-testable. Ordering hazards
(e.g. the UPG-0031 inline-code-span vs HTML-comment bug) become compile-time or test-time
failures, not silent runtime surprises.

### DBA 9-stage compatibility

The reviewer must work in both contexts without code duplication:

**Context A — self-dev** (this toolkit's 4-step loop):
- Working directory: the Codeos toolkit repo.
- Stage labels: `selfdev-step-1` through `selfdev-step-4`.
- Review policy: loaded from `prompts/codeos-reviewer-task.md`.
- Config: `scripts/codeos-review.sh` → replaced by the Rust CLI directly.

**Context B — downstream DBA projects** (the 9-stage loop):
- Working directory: the downstream project root (which has `.codeos/` symlinked to the
  toolkit).
- Stage labels: `stage-1` through `stage-9` (and `stage-10` for arch-refine).
- Review policy: loaded from `.codeos/prompts/codeos-reviewer-task.md` (same file via symlink).
- Config: `.codeos/reviewer.toml` in the downstream project.
- The Rust CLI must auto-discover `.codeos/` from the current working directory (or accept
  `--toolkit-root` for explicit override).

**Shared invariants** (both contexts):
- Reviewer is advisory, read-only, non-gatekeeping — unchanged.
- Human approval required at every gate — unchanged.
- Assessment YAML frontmatter format — unchanged (backward-compatible with existing reviews).
- Review log format (`reviews/review-log.md`) — unchanged.
- Review series ID model (`RVS__<feature>__S<N>`, `REV__…__R<N>`) — generated consistently.

**`dba-init.sh` integration:** project initialization writes a minimal `.codeos/reviewer.toml`
(provider = `codex` by default) so new projects get reviewer config out of the box.

### Stage-specific checklists

Per-stage policy (currently inline `case` statements in the Bash script) moves to data:
`.codeos/reviewer-policy/stage-N.toml` (or YAML). The Rust engine loads the policy for the
active stage and injects it into the packet preamble. New stages or checklist updates require
only a data file change, not a code change.

### CLI surface

Identical subcommands to the Bash script (`review`, `decision`, `diagnose`) with the same
flags — drop-in replacement. Downstream projects that call `codeos-review.sh` switch to the
Rust binary with no other changes.

### Structured records (optional, additive)

Optionally emit `.jsonl` alongside the markdown assessment as a machine-readable record.
Lays the groundwork for UPG-0015 decision-integrity without changing the Markdown format.

---

## Scope

| In scope | Out of scope |
|---|---|
| `tools/reviewer/` Rust workspace | `dba-system.md`, stage prompts, `dba-init.sh` behavior |
| Provider trait + Codex impl | Autonomous review triggering, CI hooks |
| `OpenCodeProvider`, `GeminiProvider`, `KimiProvider` stubs | Per-stage policy file content (separate change) |
| Precheck as pure Rust functions with unit tests | GUI / TUI |
| `.codeos/reviewer.toml` provider config | Changes to assessment YAML frontmatter |
| Auto-discovery of `.codeos/` for downstream context | Per-feature decision ledgers (UPG-0015) |
| `dba-init.sh` writes default `reviewer.toml` | |
| Drop-in CLI replacement for `codeos-review.sh` | |

`scripts/codeos-review.sh` is reduced to a thin shim calling the Rust binary, or removed
entirely once the Rust CLI passes all existing smoke tests.

---

## Value

High. Unblocks multi-provider use, makes the precheck pipeline safe by construction,
enables downstream DBA projects to use the reviewer at stage gates without script duplication,
and lays the foundation for UPG-0015 decision-integrity. The Bash pilot has now proven the
workflow; this is the natural next step.

## Risk

| Risk | Mitigation |
|---|---|
| Rust toolchain required to build | Ship compiled binary alongside the repo; keep Bash shim as fallback during migration |
| Behavior regression vs Bash | Parallel operation period; gate the switch on passing all current smoke tests against same inputs |
| Over-engineering provider trait | Keep the trait minimal — only what `codex` currently needs; provider stubs can be empty impls |
| dba-init.sh coupling | Minimal coupling — init only writes `reviewer.toml`; no binary compilation at init time |

## Timing

The Bash pilot has proven the workflow across multiple completed changes and an e2e test (the
pre-condition from UPG-0018 is now met). The multi-provider and downstream-DBA-compatibility
needs are the signals to invest. Suggested: Wave 3 or Wave 4, ahead of UPG-0015
decision-integrity, since provider flexibility affects the architecture of anything built on
top of the reviewer.

## DBA-philosophy note

Keeps the reviewer advisory, read-only, and human-gated. Moving implementation to typed,
tested Rust strengthens evidence quality and safety without adding autonomy. Provider
abstraction is purely a delivery mechanism — it does not change the reviewer's role or
the human gate.

---

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260702-001 | `Archive/self-development/changes/UPG-0032__CHG-20260702-001__rust-reviewer-engine.md` | Full Rust implementation: provider trait, CodexProvider, typed packet, precheck, CLI drop-in replacement for codeos-review.sh | IN_PROGRESS |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
| UPG-0018 | Superseded by this feature | — |
