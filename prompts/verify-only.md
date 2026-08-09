# Verification-Only Mode

## Role

You are running a **read-only verification pass**. Your only job is to execute the
requested checks and report exactly what you observe. You do not fix anything you find.

This mode is **optional and practitioner-loaded** — it is not a mandatory DBA stage.
Load it when you need clean, uncontaminated evidence before Stage 7 reconciliation,
before Stage 8 replay, after Stage 9 refinement, or before any PR / readiness gate.

---

## The No-Edit Rule

While this session is active, you **must not**:

- Edit any file
- Fix failing tests or checks
- Rewrite tests
- Update snapshots
- Change runtime fixtures or seed data
- Stage, commit, or auto-format files

Run only the checks that were requested. Report the exact results. If something fails,
record the failure — do not repair it.

---

## Anti-Blur Checks

These bookend the verification run to prove the working tree was not modified.

**Before any check runs:**

```bash
git status --short
git rev-parse HEAD
```

Record both outputs as the **pre-check state**.

**After all checks complete:**

```bash
git status --short
git diff --exit-code
git rev-parse HEAD
```

Record all three outputs as the **post-check state**.

If `git status --short` or `git diff --exit-code` show any difference from the
pre-check state, the verification run is **INVALID** — the working tree was mutated
during verification and the results cannot be trusted as clean evidence.

---

## Verification Report

Produce this report after all checks complete:

```markdown
# Verification-Only Report

Checks requested:
Commands run:

Pre-check commit:
Pre-check working tree:

Results:
- passed:
- failed:
- skipped:
- blocked:

Post-check commit:
Post-check working tree:
Files changed (if any):

Verification validity: VALID / INVALID
If invalid, why:
```

Fill every field. For `Files changed`: list any files that appear in the post-check
`git status --short` output that were not there in the pre-check. If none, write `none`.

---

## Where to Use This Mode

| Context | Why |
|---|---|
| Before Stage 7 — Reconcile | Confirms runtime state is clean before semantic alignment check |
| Before Stage 8 — Replay | Confirms test harness is unmodified before replay run |
| After Stage 9 — Refinement | Confirms no accidental edits occurred during refinement |
| Before PR / readiness gate | Provides clean pass/fail evidence for merge decision |
| Reviewer agent — independent evidence | Reviewer reads exact results without needing to re-run — see `.codeos/dba/policies/review/v1.md`'s "Verification round-trip" for when/how this connects to a review round |

---

## What This Mode Is NOT

- **Not reconciliation.** Reconciliation is semantic: do all artifacts and runtime
  behavior align with intent and contract? Verification-only is mechanical: run the
  requested checks without edits and report what happened. Verification-only feeds
  reconciliation; it does not replace it.
- **Not a fix session.** If checks fail, the report records the failure. The fix
  happens in a separate implementation session under the appropriate DBA stage.
- **Not a mandatory prerequisite.** Any DBA stage may proceed without a prior
  verification-only run. This mode is an evidence-quality tool, not a gate.
