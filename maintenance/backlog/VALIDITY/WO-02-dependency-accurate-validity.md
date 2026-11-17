# WO-02 — Dependency-Accurate Validity

PROTECTED · after WO-01 · direction approved by the human 2026-11-17

## Direction

**One authoritative dependency set/identity per governed validity decision.** It covers whatever
actually needs validity semantics (verification records, human receipts, reconciliation, candidate
evidence), without forcing every record type into one abstraction.

- Candidate Bundles reference evidence content identity, not record lines.
- A human receipt stays current while its dependency set is unchanged. No path allowlists and no
  agent classification.
- **Freshness is explicit and evidence-type-specific. Absent a freshness rule, content identity
  alone governs deterministic evidence.** No universal TTL. External reality and advisory-database
  evidence get their own rules; deployment qualification always audits fresh.

## Invariants

- No decision survives a change to what it depended on.
- History is never rewritten.
- Stage 8 and human gates keep their meaning; only what invalidates them narrows.

## Part A — Decision brief (human approves before code)

**First-class question: validity reuse and review reuse are different decisions.** A human or
evidence receipt may stay valid because its dependencies did not change. CodeOS may still choose
periodic or trigger-specific advisory re-review to catch defects missed earlier. The brief presents
the measured cost (WO-01: 17–20 of 36 Stage 8 reviews followed no relevant change) and the catch
rate (5 of the 10 Stage 8 findings came from such reviews). It asks which review guarantees are
intended, and doesn't presuppose keeping all reviews or dropping them all.

The dependency set for each governed validity decision, derived from WO-01's findings; content
identity per evidence class; the freshness rule per evidence class; definitions for "dependency
set", "content identity" and "freshness" proposed for `dba/05-guidance/terminology.md` (none exist
today).

## Part B — Implementation

As a new DBA. Verified by re-running the WO-01 replay (the classes must match Part A's prediction)
plus a fixture per evidence class.
