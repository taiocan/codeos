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
- *Current Validity*, *Candidate Bundle* and *Post-Acceptance Dependency Revalidation* stay
  unchanged. Issued configurations freeze the definitions they use, so the v14 meanings are stated
  in the new terms and in Feature Workflow v14 instead (found by the configuration integrity check
  in Part B).

## Part B — Implementation

DBA-35 candidate (`maintenance/candidates/dba-35.yaml`): DBA-34 with Feature Workflow v14,
Workflow Governance v6 and Review Policy v18. The checkpoint dependency sets were approved by the
human on 2026-11-18 (F4/F8 judged surface and claims, F9 meaning, F6 by revalidation; Current
Validity never satisfies F7, F8 or F9).

**Engine** (`dba/04-tools/workflow/engine`, all gated on v14; DBA-34 and earlier unchanged):

- `validity.rs`: spec meaning (R4), resolution through Semantic Equivalence Receipts, content
  identity (R3), the reviewer configuration identity (R5), judged claims.
- `checker.rs`: v14 receipt keys; a v14 receipt is compared on its dependency set only; an accepted
  feature's F6 is satisfied by a PASSED revalidation; verification records bind spec meaning.
- `candidate.rs`: a schema 3 bundle cites content identities, spec meanings, the F6 receipt's
  content and the reviewer configuration.
- `revalidation.rs`: Current Validity compares spec meaning.
- `decide --checkpoint semantic_equivalence --artifact <a> --from <identity>`.

**Verification:**

- `tests/dependency_validity.rs`, 7 tests, each contrasted with DBA-34:
  - a runtime change keeps F9 and stales only Current Validity;
  - a PASSED revalidation satisfies F6 with no new receipt;
  - a YAML quoting fix stales no receipt, verification or bundle;
  - a prose change needs an exact equivalence receipt, and the receipt covers only that delta;
  - a re-run with identical content keeps the bundle;
  - a `codeos.yaml` comment stales neither F9 nor the bundle;
  - F4 ignores Contract prose but not Intent prose.
- The full engine suite and clippy are clean.

**WO-01 replay under v14** (`wo01-replay.py <JamFree> 80fb5ec --v14 [--rows]`; frontmatter
unquoting and Architecture Scope content approximate the engine):

| | F4 | F6 | Stage 8 | F8 | F9 |
|---|---|---|---|---|---|
| Re-asked today | 17 | 38 | 36 | 21 | 18 |
| Re-asked under v14 | 15 | 27 (+10 satisfied by revalidation) | 33 | 10 | 7 |

1. **Human decisions:** F4/F8/F9 drop from 56 to 32. Every case WO-01 judged not relevant stops
   re-asking, except 2 F8 cases. Those were spec prose edits without a revision bump, which R4
   deliberately treats as changes; one Semantic Equivalence Receipt would carry them (likewise 2 F6
   and 4 Stage 8 cases).
2. **Upper-bound "relevant" cases v14 no longer re-asks** (8): other features' product or
   declared-dependency changes now go through Current Validity and revalidation, and architecture
   changes through F9, by design.
3. **Stage 8 barely changes (36 → 33).** Verification records still bind the whole-project
   `implementation_state`, so any change re-opens every feature's candidate. Narrowing
   verification-record implementation scope is outside WO-02's rules and is the next measured
   lever. F8 and F9 receipts stay current through the re-review; only the review re-runs.
4. **Adoption cost:** records and receipts written before v14 carry no meaning keys and are
   compared as recorded. Adoption therefore re-runs no verification and spends no provider call.
   DBA-35 adoption requires no production-provider re-run and no F6/F8/F9 re-decision solely due
   to adoption; JamFree observed this on 2026-11-18. *(Corrected 2026-11-18 after the JamFree
   adoption: this note said that no mechanical verification re-runs. That was wrong. The execution
   identity of every shared verification includes the `codeos.yaml` bytes
   (`evidence_set::common_inputs`), so deterministic records go stale on adoption and Platform
   Assurance follows them. That is the WO-04 over-binding, not a v14 rule.)*

**Qualification** (2026-11-18): `qualify-candidate.sh dba-35` passed the suites, the compatibility
matrix and every replay. All accepted JamFree replay outcomes are unchanged.
`qualified_candidate_sha256: 2a27df26b90fc0ae28be542b523e5f9fd0932d7261ce184b6c4db65a00503080`.
Qualification found two adoption defects, both fixed before this identity:
- v14 records binding meaning under the old key names would have re-run every pre-v14
  verification (3cc7fdc);
- requiring v14-only keys of pre-v14 receipts would have staled every F4–F9 receipt (1042816).

## Annex — Checkpoint dependency decision (human, 2026-11-18)

**Decision.** Approve F4/F8/F9 carry-forward, but do not let Current Validity automatically satisfy
F7/F8/F9.

**Core invariant.** Post-acceptance revalidation restores runtime/evidence validity only. Every
checkpoint remains valid only while its own dependency set is unchanged.

| Checkpoint | Depends on | Reuse rule |
|---|---|---|
| F4 | judged claim/question + observed surface | reuse if unchanged |
| F6 | spec semantics + delivered/runtime state + required evidence | satisfied by PASSED revalidation |
| F7 | review inputs/evidence + reviewer-contract semantic identity | reuse if unchanged |
| F8 | judged claim/question + observed surface + relevant human outcome | reuse if unchanged |
| F9 | governed meaning + accepted claims + normative semantic dependencies | reuse if unchanged |

Constraints:

- F4/F8 bind the judged claim, not the whole specification.
- F8 binds the relevant session outcome, not necessarily the entire shared session file.

**Expected behavior:**

- A Dockerfile or runtime-only change → revalidate runtime and evidence; keep F4/F7/F8/F9 if their
  dependencies are unchanged.
- A reviewer-contract change → rerun F7 only.
- A surface or UX change → rerun the affected F4/F8.
- A claim or meaning change → rerun whichever checkpoints depend on that meaning.
- A timestamp or run-ID-only change → no invalidation.

**Implementation order:**

1. Define checkpoint dependency sets.
2. Implement evidence/content identity and spec equivalence.
3. Make F6 satisfiable through post-acceptance revalidation.
4. Reuse F4/F7/F8/F9 independently from their dependency sets.
5. Replay WO-01.

**Short version:** do not replace the old broad chain with a broad Current Validity shortcut. Make
each checkpoint independently dependency-accurate.

### Conformance of the DBA-35 candidate to this annex (2026-11-18)

Conforms: the F6 rule, F9, the session outcome (attestation checks only this feature's outcome and
surface), the reviewer-contract rule (at version granularity, pending WO-04), timestamp and run-ID
changes, and the implementation order. Two open points, unresolved:

1. **F4/F8 also bind the Intent's meaning**, not only the judged claims and surface. A change to the
   Intent's prose therefore re-asks F4/F8. Contract and Event Schema prose do not. Conforming means
   dropping `intent_meaning` from the F4/F8 dependency set. Note that a feature without
   `observation:human` claims at F4/F8 then binds only its surface.
2. **F7 re-runs on a runtime change** when the changed file is one of the feature's Candidate
   Bundle implementation inputs, such as `web/Dockerfile` (role `product`) or a lockfile (role
   `dependency`). The implementation is a review input, so its F7 dependencies did change. Keeping
   F7 would require removing runtime inputs from Stage 8's review inputs, which is a separate
   decision.

## Acceptance (human, 2026-11-18)

**WO-02 Part B: ACCEPTED.** The DBA-35 candidate identity
`2a27df26b90fc0ae28be542b523e5f9fd0932d7261ce184b6c4db65a00503080` is approved for issuance,
only if `issue-configuration.sh` recomputes exactly that identity. JamFree adoption remains
separate. The annex's open points are accepted as implemented, within Boundary 1.

**Boundary 1 — F4/F8 stay scoped to the feature's outcome.** A shared Human Validation Session is
inspectable as a whole, but F4/F8 validity binds only the relevant feature or claim outcome and the
judged surface, never unrelated outcomes in the same session. "Intent meaning" means the governed
meaning of the judged feature's own Intent, never unrelated project intent content. The candidate
conforms: session attestation checks only this feature's outcome and surface, sealed sessions are
immutable, and `intent_meaning` is the feature's own Intent.

**Boundary 2 — deferred Part A consequence: the advisory audit's as-of identity or date.** DBA-35
introduces no advisory freshness rule. That identity must be implemented before any future validity
rule depends on advisory age.

**Not folded into WO-02:** narrowing the whole-project `implementation_state` in verification
records, the measured Stage 8 lever. It deserves its own bounded follow-up.
