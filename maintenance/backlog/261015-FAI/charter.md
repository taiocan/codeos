# FAI — Feature-Atomic Invalidation Charter

- **Status:** APPROVED — Primoz Gorjup, 2026-10-16 (proposed 2026-10-15)
- **Scope:** CodeOS-wide; pilot project JamFree
- **Entry:** STAB WO-04 moved it ahead of the next P3 feature on 2026-10-16. P3 is paused and
  resumes from a fresh baseline after FAI.

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

**Scope after WO-01 (2026-10-16).**
- Goals 2 and 4 are the core.
- Production evidence also binds the verifier that produced it.
- Goal 3 covers only what keeps real runtime dependencies propagating, with rustls as the
  falsifier.
- Goal 1 is recorded as a latent coupling and deferred.
- When a governed Observed Surface changes, the F4 invalidation that follows is intentional and out
  of scope.
- A generic mechanism for refreshing evidence on request is out of scope.

1. **Explicit feature input scope.** A candidate binds only the implementation and dependency
   surfaces the feature depends on.
2. **Feature-scoped evidence bindings.** Production, browser and verification evidence bind their
   relevant surfaces rather than `implementation_state`, where a narrower identity exists.
3. **Explicit dependency propagation.** When one feature consumes another's output, that
   dependency is represented directly. A change propagates along declared dependencies, not because
   two features touch the same file.
4. **Shared evidence reuse.** A feature that inherits a reality claim binds the authoritative
   upstream evidence instead of reproducing the provider run.

Target (UX receipts as today; WO-01 row 1b showed that their invalidation was necessary):

```text
F-0005 UI changes → F-0005 stale
  → accepted features re-verify in fixture mode; F4 stale where the Observed Surface changed
  → production evidence stale only where it observed the changed surface or a declared dependency
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

At minimum, four cases become acceptance tests (WO-03 criterion 2):
1. **Shared page change (F-0005).** An unaffected feature stays current unless its Observed Surface
   or a declared dependency changed.
2. **Test-only change (F-0002 `b713734`).** Production evidence does not go stale because
   unrelated test code changed.
3. **Inherited reality evidence (F-0003, F-0004).** A downstream feature binds the authoritative
   upstream evidence where the claim is genuinely inherited.
4. **Shared runtime dependency (rustls `874728d`).** A genuine dependency still stales every
   feature that relies on it. FAI must not under-invalidate.

FAI is not assumed to remove every case. WO-01 classifies each one, and WO-02 drops any goal the
table does not need.

Work orders: [README](README.md).
