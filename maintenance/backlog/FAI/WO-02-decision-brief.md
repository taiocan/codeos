# WO-02 — Decision Brief: The Narrowest Mechanism

PROTECTED Part A · after WO-01 · Charter: Goals, Invariants

## Goal

Choose the narrowest mechanism that removes WO-01's incidental invalidations and keeps every
necessary one.

## Brief

The brief follows the CLAUDE.md PROTECTED statement:

- **Goals needed.** Which of the Charter's four goals the table actually requires, and which are
  dropped.
- **Semantics.** For each binding that changes: what it judges, what it binds today, and what it
  should bind. Also how feature scope and dependencies are derived or declared, and where each one
  has a single authority.
- **Invariants.** The Charter invariants, plus compatibility for existing receipts and bundles.
- **Verification.** Tests per WO-01 row, and the WO-04 replay.
- **Versioning.** The owning policy and configuration.

## Acceptance criteria

The human approves or rejects the brief before any code.

## Results

**Brief (2026-10-16). Awaiting approval; no code before it.**

**Decision.** FAI reduces incidental evidence invalidation and duplicate reality evidence. It does
not change what a human UX receipt judges.
- Evidence validity is scoped to what the evidence observed and to the verifier that produced it.
- An inherited reality claim may bind authoritative upstream evidence.
- Genuine runtime dependencies keep propagating invalidation.
- Narrowing the broad Candidate Bundle inputs is deferred until observed harm justifies it.

**Semantics that change.** All changes come in Feature Workflow v11, selected by a new DBA-27.

| Binding | Judges | Binds today | Binds under v11 | WO-01 rows |
|---|---|---|---|---|
| `production_source_evidence` record | The reality-dependent claims, against the production source | Intent, Contract, Event Schema, whole `implementation_state` (tests included) | Intent, Contract, Event Schema, plus `evidence_scope_state` and `reality_verifier`. `evidence_scope_state` covers the non-test files of the Evidence Scopes it observed and their dependency manifests. `reality_verifier` is the sha256 of `reality_verification.command`. | 1c, 3a, 3b, 5 |
| F6 reconciliation receipt | The human's selection of reconciled production evidence | … plus `implementation_state` | … plus the non-test `product_state` in place of `implementation_state` | 3c |
| Inherited claim | That the claim restates an upstream claim's production result | Its own production run | The upstream feature's current selected production evidence; no own run | 2b, 4 |

1. **Evidence Scopes (goal 2).**
   - `codeos.yaml` `reality_verification.scopes` names path sets, for example
     `provider: [backend]` and `web: [web]`.
   - The evidence file states, for each claim, which named scopes it observed.
   - A claim that names no scope, or a project that declares none, binds every implementation
     path. Unknown therefore means broad, never narrow.
   - Test files never count. This uses the existing `is_test_path` and `product_state` rules.
2. **Dependency propagation (goal 3, narrow).** A scope path also binds:
   - the manifests and lockfile of the package that contains it (Cargo through `cargo metadata`;
     npm through the nearest `package.json` and lockfile);
   - `docker-compose.yml`.

   So rustls (`backend/Cargo.lock`) still stales F-0001 and F-0002.
3. **Verifier binding.** A change to the verifier stales the in-flight `production_source_evidence`
   records, which `check` then regenerates. Accepted currency still compares only the bundle and the
   governed inputs, as today. So a verifier edit does not stale accepted features, which avoids a
   fan-out across every feature that shares one verifier. Their next legitimate revalidation uses
   the new verifier.
4. **Inheritance (goal 4).**
   - The Reality Basis gains a fourth classification, `inherited`. Its source cell names the
     upstream `F-NNNN/claim-id`.
   - F5 then needs no own run for that claim. The record binds the upstream feature's
     F6-selected evidence identity, and it is current only while that feature's acceptance is
     VALID. So upstream staleness propagates.
   - Rendering the inherited value stays covered by the feature's fixture journeys.
   - The Candidate Bundle records the inherited identities (bundle schema 3), and the reviewer's
     readiness check reads them.

**Unchanged.**
- F4 and F8 Observed Surface semantics;
- Candidate Bundle input derivation;
- acceptance-currency rules;
- the reviewer's independence and advisory role.

**Must remain true.**
- No false currency: rustls must still stale F-0001 and F-0002 evidence.
- Fail closed when an Evidence Scope, the verifier or upstream evidence is unknown.
- No rewriting of history.
- No project-specific rules in CodeOS.

**Compatibility.**
- A v10 production-evidence record lacks `reality_verifier`, so under v11 it is stale. It is
  rerun only where it is still needed (on JamFree today, F-0003). Accepted features are not
  touched.
- F6 receipts recorded under v10 keep the bindings they recorded.
- Inheritance applies only when a Contract is approved or revised with the new classification.
  F-0003 and F-0004 keep their current claims until their next semantic revision.

**Verification.** One test per acceptance case:
1. a test-only change leaves production evidence current;
2. a shared UI change leaves evidence current unless its Evidence Scope includes that UI;
3. a verifier change stales the evidence, and `check` regenerates it;
4. an inherited claim binds upstream evidence and needs no run;
5. rustls stales the affected evidence;
6. a changed Observed Surface stales F4 exactly as today.

Cross-boundary workflow–reviewer tests cover the bundle's inherited evidence. WO-04 replays WO-01
on JamFree with no TomTom calls.

**Decisions needed.**
- **(a)** Evidence Scopes are named in `codeos.yaml`, and the verifier states which ones it
  observed. The
  verifier is project code, so this is self-attestation. It is bound by `reality_verifier` and
  read at Stage 8, which is the same trust we already place in its observations. The alternative
  is a scope column in each Contract's Reality Basis. That is human-approved per claim, but it
  needs every Contract revised.
- **(b)** Inheritance applies to new or revised Contracts only. F-0003 and F-0004 are not migrated.
- **(c)** Verifier changes do not stale accepted features.

- **(d) Terminology** (`dba/05-guidance/terminology.md`, updated with v11):
  - **Evidence Scope** (new): a named, project-declared set of implementation paths that a
    production-source verifier states it observed for a claim. It is distinct from Observed
    Surface.
  - **Inherited claim** (new): a reality-dependent restatement whose production truth is the
    named upstream claim's selected production evidence. It creates no observation of its own.
  - **Production-source evidence** (amended): "bound to the current reality-dependent claims,
    governed inputs, the Evidence Scopes it observed, and the verifier that produced it".

I recommend all four as written.

**Approved (2026-10-16)** — Primoz Gorjup: "DBA-27 / Feature Workflow v11 may introduce scoped
production-evidence bindings, verifier-dependent evidence regeneration, and explicit inherited
reality claims, while preserving genuine runtime dependency invalidation and leaving F4/Observed
Surface semantics unchanged." Decisions (a)–(d) are approved with these constraints:
- **(a)** The verifier selects from the declared scopes and never defines their contents. An
  unknown or missing scope, a malformed declaration, or an unresolved path fails closed.
- **(b)** The downstream binding covers the upstream feature, the upstream claim and evidence
  identity, and upstream Current Validity, so it never points at an older proof.
- **(c)** v11 states in one normative sentence that the verifier hash is an
  evidence-regeneration dependency, not by itself a post-acceptance validity dependency.
- **F6** binds the non-test product state; WO-02 does not try to make F6 fully feature-atomic.

Added test: a verifier that claims an undeclared Evidence Scope fails closed.

**Implementation note.** An inherited claim's currency follows the upstream feature's Current
Validity live, and always uses the upstream's current selected evidence. The Candidate Bundle
records the identity at candidate time for traceability. A frozen identity would leave the
downstream feature with no governed path back to VALID after an upstream revalidation, which is the
F-0003 dead end.
