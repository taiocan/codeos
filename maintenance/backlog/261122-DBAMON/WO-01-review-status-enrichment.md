# WO-DM-01 — Enrich `review-status --json` per Assessment

- **Class:** NORMAL. The change is additive to a read-only projection.
- **Goal:** the reviewer's structured status carries the facts that today exist only in assessment
  frontmatter, so no consumer has to parse markdown.

## Scope

Add these fields to each `assessments[]` entry of `codeos-reviewer review-status --json`
(`dba/04-tools/reviewer/engine/src/cmd/status.rs`):

- `candidate_outcome`;
- findings counts by `severity`, by `classification` and by `blocker_kind`;
- `codex_cli_version`, `reviewer_build_sha256`, `codeos_release_id`;
- `source` (codex / external).

Existing keys and their values stay unchanged. Additive-field test: CodeOS itself benefits,
because the terminal and agents can read review outcomes and findings density without opening
assessment files.

## Verification

- `cargo test` passes, and the golden output gains only the new keys.
- On JamFree, the number of assessments equals the number of assessment files, and the outcome
  counts match the frontmatter.

## Acceptance

The verification passes and the human accepts.
