# WO-02 — Dependency-Accurate Validity

PROTECTED · after WO-01 · Part A approved by the human 2026-11-18

## Invariant

A change invalidates only the decisions whose declared dependencies changed. Acceptance, current
validity, evidence validity, review and release assurance are separate decisions, each with its own
dependency set. History is never rewritten.

## Part A — Rules (approved 2026-11-18)

**Today:** human receipts bind whole-project `implementation_state` (F4, F9) or whole-repository
`delivered_state` (F6). A Candidate Bundle cites verification records by line hash, timestamp
included. Validation Evidence Sets are content-addressed, but their content includes provenance
times. No evidence type has a freshness rule. WO-01 measured that 35–55 % of re-runs followed no
relevant change.

| # | Rule | Binds | Does not bind |
|---|---|---|---|
| R1 | **Acceptance identity.** F9 binds the governed semantic meaning of the accepted claims, including their normative semantic dependencies (for example Architecture or Charter constraints). | the semantic meaning of the accepted claims and their normative dependencies | implementation bytes, runtime, evidence records |
| R2 | **Current validity.** Accepted meaning plus the current required implementation, runtime and evidence state. Post-Acceptance Dependency Revalidation restores it without a new F9. | F9 + current implementation, runtime and evidence dependencies | — |
| R3 | **Evidence identity.** Content (what was proven) + provenance (the declared provenance and dependency values needed to establish applicability and trust, including the relevant execution inputs and source identity, compared as meaningful values) + an optional, evidence-type-specific freshness rule. | content, provenance, a freshness rule where declared | timestamps, record-line IDs, run IDs (`execution_identity` is a reuse key) |
| R4 | **Specification equivalence.** Structured fields compare by their exact canonical parsed form. Normative prose compares byte for byte unless a Semantic Equivalence Receipt covers the exact old → new delta. The receipt removes only that one specification dependency. | parsed structured fields, prose bytes, receipt digests | normalized prose |
| R5 | **Review identity.** Stage 8 depends on its review-input dependency set and on the semantic identity of the governing reviewer contract (derived by WO-04), not on the contract file's bytes. Release-level latent-defect assurance is a separate decision and never affects feature validity. | the Stage 8 dependency set, the reviewer contract's semantic identity | other checkpoints going stale |

**Cross-cutting.** Each owning policy (Feature Workflow, Workflow Governance, Review) declares the
dependency sets it owns. The only project-level declarations are the existing `--include`
additions. No path allowlists, no agent classification, no universal TTL.

**Consequences.** These follow from the rules and are not exceptions:

- **A `Dockerfile` or lockfile change.** Current Validity goes stale, and the verifications and
  any required reality evidence re-run (doctrine v9). Revalidation then restores Current Validity.
  F9 stays unchanged, and no input-role reclassification is involved.
- **A YAML quoting fix.** The parsed form is identical, so nothing goes stale.
- **A Reality Basis row correction.** The prose changed. Either one Semantic Equivalence Receipt
  covers it, or the change propagates normally.
- **A CodeOS run-ID or timestamp change.** Content and provenance are identical, so nothing goes
  stale.
- **A semantic reviewer-contract change (DBA-33, DBA-34).** Stage 8 re-runs. F9 and Current
  Validity are unaffected unless their own dependencies changed.
- **Freshness.** Advisory-database results record their database date, and deployment
  qualification always audits fresh. External reality evidence has no time rule.

**Deferred.**

- The cadence and scope of the release advisory review. Only the separation principle is fixed;
  measure before choosing a scope that could grow as features × changes.
- Input-role corrections, such as `Dockerfile`, and only if a role is wrong on its own merits.

**Terms**, added to `dba/05-guidance/terminology.md` in Part B:

- New: *Dependency set*, *Content identity*, *Freshness rule*, *Semantic Equivalence Receipt*.
- Updated: *Current Validity* (R2's split) and *Candidate Bundle* (cites evidence identity per R3).
- Reused unchanged: *Post-Acceptance Dependency Revalidation*.

## Part B — Implementation

One new DBA: Feature Workflow, Review and Workflow Governance as needed. Verified by:

- re-running the WO-01 replay under the new rules: cases classified as not relevant stop
  invalidating, and cases classified as relevant still invalidate;
- one fixture per rule.
