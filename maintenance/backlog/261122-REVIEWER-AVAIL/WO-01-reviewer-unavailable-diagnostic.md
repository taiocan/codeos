# WO-RA-01 — Typed REVIEWER_UNAVAILABLE Diagnostic

- **Class:** NORMAL, diagnostic only. Approved by the human 2026-11-22.
- **Baseline:** current reviewer engine (Reviewer Tool Contract v23).
- **Goal:** report Codex quota exhaustion as a stable, typed diagnostic. No review semantics
  change.

Planning context only. Git and the governing policies stay authoritative.

## A. CodeOS change

- `dba/04-tools/reviewer/engine/src/codex.rs`: add `ProviderFailure::UsageLimit`, recognized from
  the Codex JSONL error events and reusing the parsing in `failure_messages`. `invoke` returns it as
  a downcastable error.
- `dba/04-tools/reviewer/engine/src/cmd/review.rs`: in the Codex invocation `Err` arm, print the
  message below for `UsageLimit` and keep the existing message for every other failure:

  ```text
  REVIEWER_UNAVAILABLE: Codex usage limit reached. No review recorded; no capacity consumed.
  ```

- Keep exit code 3 (`EXIT_PROVIDER`). Write no review record. Invoke no other reviewer.

## B. Existing manual advisory workflow (no code change)

A human can explicitly ask for an advisory assessment from another model:

1. `plan --emit-packet <pkt>` on the frozen Candidate Bundle.
2. A fresh, isolated subagent sees the packet only.
3. `review --packet <pkt> --assessment <reply> --reviewer-label <exact model id>`.
4. The result is recorded as `source: external`, with no outcome and no round. The gate stays open.

If advisory findings lead to code or spec changes, the candidate is superseded. Its advisory result
remains historical, and a new packet and candidate are needed. The purpose is extra defect
discovery while waiting for Codex, not preparation for a Codex PASS.

## Deferred

Edit a governed reviewer prompt only if real use shows agents mishandling `REVIEWER_UNAVAILABLE`.

## Verification

- Unit test in `codex.rs`: usage-limit JSONL gives `UsageLimit`; other failures and malformed
  lines do not.
- `cargo test --manifest-path dba/04-tools/reviewer/engine/Cargo.toml`
- End-to-end check with a stub `codex` on PATH that prints the usage-limit JSONL and exits non-zero:
  exit 3, `REVIEWER_UNAVAILABLE` on stderr, no review record, no other reviewer invoked. Commit it
  as a test if the `tests/` harness supports a Codex stub; otherwise summarize it in the commit
  message.

## Acceptance

The verification passes and the human accepts.
