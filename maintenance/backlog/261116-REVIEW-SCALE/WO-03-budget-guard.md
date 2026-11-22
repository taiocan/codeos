# WO-03 — Budget as a Fixed Guard

NORMAL · JamFree `reviewer.toml` · after WO-02

## Goal

Set `packet_budget_bytes` to the largest legitimate first-review packet in the post-projection
qualified baseline plus about 15 % headroom. It stays fixed until an explicitly reviewed change and
is never auto-adjusted.

## Acceptance criteria

1. The value and its basis are recorded in the `reviewer.toml` comment.
2. An unexpected additional 50 KB required input pushes the packet past the budget, which fails
   closed and names the contributor.
