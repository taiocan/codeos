# SPERF — Status Evaluation Performance Charter

- **Status:** PROPOSED — Primoz Gorjup, 2026-10-15
- **Class:** NORMAL. Implementation only: the full report stays semantically identical.
- **Entry:** after STAB P3. STAB WO-04 decides it, and it ranks below [FAI](../FAI/charter.md). The
  STAB WO-03 latency trigger can bring it forward if latency blocks delivery.

Planning context only. Git and the governing policies stay authoritative.

## Problem

`status` on JamFree takes about 65 s with the debug build. The cause is repeated recomputation and
subprocess fan-out in the evaluator. Network, TomTom, Docker and rendering are not the cause.

| Scope | Wall | User CPU | System CPU |
|---|---:|---:|---:|
| Bootstrap only | 0.51 s | 0.42 s | 0.14 s |
| F-0001 only | 64.88 s | 55.65 s | 11.58 s |
| Full project JSON | 65.12 s | 55.76 s | 11.79 s |

A partial trace, so lower bounds only, recorded 1,559 git, 156 cargo metadata, 2,912 awk and 265 jq
executions.

Causes:
1. **Configuration resolution is not cached.** Each policy-version lookup restarts the shell
   resolver (`project.rs`, `resolved_project_configuration`).
2. **cargo metadata runs again on every call.** Each `implementation_paths()` call runs it, and
   JamFree tries two directories (`project.rs:375`, `:421`).
3. **Git runs three times per path.** `working_tree_state` runs `ls-tree`, `diff` and `ls-files`
   for each implementation path (`hashing.rs:24`).
4. **Checkpoints rebuild the same bindings independently.** There is no shared snapshot
   (`checker.rs:170`, `monitor.rs:197`).
5. **Bootstrap can be evaluated again inside feature evaluation** (`checker.rs:181`). This is
   minor today.

A release build or a shorter report would not remove the process launches.

## Goal

One command-scoped evaluation snapshot. It holds:
- the configuration and policy versions, resolved once;
- cargo metadata and implementation paths, computed once;
- one repository-wide Git inventory;
- each distinct binding hash, computed once;
- the Bootstrap result and per-feature assessments, reused when health and attention are built.

## Invariant

Status still verifies evidence against the current project and never trusts receipts blindly.
Nothing is cached across commands.

## Acceptance criteria

1. At most one configuration resolution per status command.
2. At most one successful cargo metadata resolution per project.
3. A small, fixed number of Git subprocesses, independent of checkpoint count.
4. Each distinct working-tree or product-state binding computed once.
5. The full report is semantically identical before and after.
6. A wall-time budget is tested separately from the deterministic count assertions. About 65 s
   down to low single-digit seconds is an estimate, not a target.

Work orders: [README](README.md).
