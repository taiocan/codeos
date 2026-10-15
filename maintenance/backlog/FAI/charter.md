# FAI — Feature-Atomic Invalidation Charter

- **Status:** PROPOSED — Primoz Gorjup, 2026-10-15
- **Scope:** CodeOS-wide; pilot project JamFree
- **Entry:** after STAB P3 ends. STAB WO-04 decides it formally. It is implemented earlier only if
  the fan-out blocks correct progression.

Planning context only. Git and the governing policies stay authoritative.

## Problem

CodeOS treats features as separate governance units. Several evidence identities are still broad
project or file state, so one change stales several features:

- `candidate::derive_inputs` takes every file changed since the Intent was approved;
- production evidence binds the whole `implementation_state`, test files included;
- human receipts recorded before Feature Workflow v10 bind the project-wide `product_state`.

Each stale feature then needs a full revalidation: journeys, about 27 TomTom calls, a Codex
assessment, and sometimes a human F4 decision. The P3 cases are in STAB WO-04:

- the rustls `Cargo.lock` patch;
- the F-0002 test-only repair `b713734`;
- the F-0005 `+page.svelte` change, which staled F-0001 to F-0004;
- F-0003 and F-0004 rerunning provider journeys for claims they inherit.

## Goals

The approach is dependency-scoped invalidation, not one file per feature.

1. **Explicit feature input scope.** A candidate binds only the implementation and dependency
   surfaces the feature depends on.
2. **Feature-scoped evidence bindings.** Production, browser and verification evidence bind their
   relevant surfaces rather than `implementation_state`, where a narrower identity exists.
3. **Explicit dependency propagation.** When one feature consumes another's output, that
   dependency is represented directly. A change propagates along declared dependencies, not because
   two features touch the same file.
4. **Shared evidence reuse.** A feature that inherits a reality claim binds the authoritative
   upstream evidence instead of reproducing the provider run.

Target:

```text
F-0005 UI changes → F-0005 stale
  → F-0001 UX stale only if its Observed Surface changed
  → F-0002–F-0004 stale only if a declared dependency or their Observed Surface changed
```

Today:

```text
+page.svelte changed → project identity changed → four accepted features stale → four revalidations
```

## Invariants

- **No false currency.** Necessary invalidations remain. For example, rustls carries the TomTom
  HTTPS calls of F-0001 and F-0002, so the rustls patch must still stale both.
- **Fail closed.** An unknown scope or dependency means stale.
- **No rewriting.** History and receipts are never rewritten.
- **Versioning.** Semantics are versioned through the owning policy.
- **Reviewer independence.** The reviewer stays advisory and independent.
- **Generic rules.** No project-specific rules in CodeOS.

## Measure

On the WO-01 cases, incidental invalidations fall to zero and every necessary invalidation remains.

Work orders: [README](README.md).
