# Upgrade: reviewer-engine-v1 — Typed reviewer engine to replace the Bash pilot

**Priority**: P2
**Status**: BACKLOG
**Type**: toolkit-upgrade
**Related**: reviewer-decision-brief, reviewer-decision-integrity, reviewer-pipeline (docs/reviewer-pipeline.md)

> **This is the future replacement direction for the Bash pilot — NOT part of the current MVP/PR
> implementation.** It is captured here architecturally so the direction is recorded; it is not to
> be designed deeply or built as part of the advisory-logging pilot.

## Problem

`scripts/codeos-review.sh` is a **manual pilot wrapper**. It is acceptable for validating the
workflow (build packet → read-only Codex → save packet + assessment → append log → optional human
decision), but Bash is the wrong long-term home for any reviewer logic that grows past that:

- no static types, no compile-time checks, brittle string/`sed`/`grep` parsing of metadata;
- secret/size filtering, redaction, and packet assembly are easy to get subtly wrong and hard to
  test;
- as soon as the reviewer gains real policy/provenance/decision logic (see
  `reviewer-decision-integrity.md`), Bash becomes a liability — the original scope drift accreted
  exactly there.

**Architecture decision:** Bash is the pilot engine only. It must **not** become the permanent
policy / provenance / decision-integrity engine. Anything beyond the pilot's allowed surface is
out-of-scope for the Bash wrapper and belongs to this typed engine (or to
`reviewer-decision-integrity.md`).

## Upgrade

A small **typed reviewer engine** (Rust or Python) that owns the reviewer pipeline:

- builds the review packet (context, scope contract, triage rule, artifacts, filtered diff);
- invokes Codex read-only (fresh reviewer session by default; resume is opt-in);
- redaction / size policy as tested, typed code;
- writes **structured JSONL review records** (machine-readable) as the source of truth, and
  **generates** the human-readable Markdown review log from them;
- policy files for per-stage checks (instead of inline `case` statements);
- the eventual decision-integrity guarantees (`reviewer-decision-integrity.md`) live here.

## Scope

New `tools/reviewer/` (or similar) typed CLI; `docs/reviewer-pipeline.md` updated to point at it;
the Bash wrapper reduced to a **40–80 line** thin shim or removed entirely. No hooks.

## Design notes

- Small CLI: `stage-start`, `review`, `decision` subcommands, same UX as the pilot.
- Policy files (e.g. per-stage check lists) as data, not code.
- Structured JSONL records; Markdown log is a generated view, never hand-edited.
- **Fresh reviewer session by default** (resume is the explicit opt-in), to avoid stale context.
- Tests required for: packet generation; secret/redaction; untracked / missing / oversize files;
  Codex invocation failure; malformed metadata; dirty workspace.

## Value

Medium–high (enables the decision-integrity work safely). Not urgent: the Bash pilot is sufficient
to validate whether the advisory reviewer earns its keep before investing in a typed engine.

## Risk

Premature build before the pilot proves value. Mitigated by treating this strictly as a backlog
direction until the advisory pilot has a track record.

## Guardrail

Until this exists, the Bash wrapper stays within the pilot surface (build packet, invoke Codex
read-only, save packet + assessment, append log, append human decision, best-effort filtering,
warnings). Any reviewer finding that asks Bash to become a stronger policy / provenance / decision
engine is **OUT-OF-SCOPE BACKLOG** under this item or `reviewer-decision-integrity.md` — not a
v0 fix.

## DBA-philosophy note

Keeps the reviewer **advisory and testable**. Moving policy/provenance into typed, tested code
strengthens evidence quality without moving the human gate or adding autonomy.
