# WO-DM-03 — Immutable Project-Monitor Snapshot

- **Class:** NORMAL. It is a read-only projection and changes no DBA semantics.
- **Goal:** one deterministic command produces a hash-verified snapshot of a project's CodeOS
  records for Power BI, in the AII layout.

## Scope

`dba/04-tools/monitor/codeos-monitor-snapshot.sh <project> <external-root>`:

1. Runs `codeos-workflow status --json --history --since all` and
   `codeos-reviewer review-status --json` through the qualified toolkit release.
2. Copies `verifications.jsonl` and `decisions.jsonl`. Writes a flat candidate index (feature,
   kind, trigger, timestamp), test-evidence case summaries, provider-ledger totals and
   runtime-event outcome counts.
3. Writes `manifest.json`: `schema_version`, `generated_at`, project, `dba_configuration`, toolkit
   release and build, and sha256 and row counts per file.
4. Builds the snapshot in a staging folder, verifies it there, and renames it into
   `snapshots/<id>/`. Then writes `current.json` (`snapshot_id`, `manifest_sha256`,
   `activated_at`) through a temporary file and an atomic rename.
5. `--verify-active` re-hashes the active snapshot and re-counts its rows.

It reads only, writes nothing under the project, and exports no packet text. It runs no git,
clippy or security tools; that is WO-04.

## Verification

- `--verify-active` passes on a JamFree snapshot.
- Corrupting one file makes it fail.
- JamFree's `git status` is unchanged afterwards.
- Row counts match the sources.

## Acceptance

The verification passes and the human accepts.
