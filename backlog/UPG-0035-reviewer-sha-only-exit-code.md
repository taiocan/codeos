---
feature_id: UPG-0035
title: "Rust Reviewer: --sha-only missing-path exit code parity (Bash=2, Rust=4)"
priority: P3
status: COMPLETE
depends_on: UPG-0032
origin_change: CHG-20260702-001
---

# UPG-0035 — Rust Reviewer: --sha-only missing-path exit code parity

## Problem

When a `--sha-only` path does not exist:
- Bash `codeos-review.sh` exits **2** (usage/config error class)
- Rust `codeos-reviewer` exits **4** (packet error class) — the missing-path check in
  `packet::build()` propagates as `Err` and is caught by the `EXIT_PACKET` handler

The difference is minor in practice (both are non-zero / error), but consumers scripting
against the binary's exit codes may observe different codes for the same input error.

## What to fix

In `cmd/review.rs`, add an explicit `--sha-only` existence check before calling
`packet::build()`, mirroring the existing positional-artifact check:

```rust
for so in &args.sha_only {
    if !Path::new(so).exists() {
        eprintln!("error: --sha-only path not found: {}", so);
        return Ok(crate::EXIT_USAGE);  // exit 1, matching Bash's exit 2 intent
    }
}
```

Note: Bash exits 2 (config class); the Rust `EXIT_USAGE` is 1. For strict parity, exit
`EXIT_CONFIG` (2), but `EXIT_USAGE` (1) is more semantically correct for a bad CLI argument.
Decide at implementation time.

## Scope

Self-dev only. Touches `tools/reviewer/src/cmd/review.rs` only.

## Priority note

P3 — purely cosmetic exit-code difference; no functional impact on normal usage.

## Feature Thread

| Change ID | State | Notes |
|---|---|---|
| CHG-20260702-005 | COMPLETE | `changes/UPG-0035__CHG-20260702-005__reviewer-sha-only-exit-code.md` |
