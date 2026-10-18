# WO-05 — Validation Evidence and Attribution Decision

PROTECTED Part A · after WO-04 · Charter criteria 4–7

## Goal

Approve the smallest semantics that let one validation action support several governed claims
without weakening feature-level traceability, Current Validity, or human control.

## Directions already set

The brief starts from these and reopens one only if evidence falsifies it: one unified validation
and evidence model with staged adapters; validation owned by product state plus claim, not by the
triggering feature; Contract-owned prospective claim IDs with a successor ID on semantic change;
legacy coarse claims until legitimately revised; several feature claims may consume one evidence
item without merging ownership; accepted journey topology stays stable when evidence goes stale;
one human session with explicit per-feature/claim outcomes and no implicit propagation; atomic
feature acceptance.

## Decision brief

The brief presents each item below as an independently approvable invariant within one model, not
as one all-or-nothing design. It must define, and receive human approval for:

- an immutable content-addressed Validation Evidence Set bound to the exact implementation/product
  state, environment, runner, input scopes, observations, and execution provenance;
- Contract-owned, owner-qualified claim IDs for new or legitimately revised Contracts, with a
  digest of governed meaning and a new ID for every material meaning change;
- claim granularity as independently reusable or invalidatable obligations, not individual test
  assertions, with no arbitrary count limit;
- coarse `feature/checkpoint/mechanic` claims for unchanged legacy Contracts, with no retrofit made
  solely for optimization;
- explicit claim-to-assertion or claim-to-observation attribution, while generated indexes remain
  derived caches and never claim authority;
- product/dependency scopes as validity dependencies and test, runner, or verifier identities as
  evidence-regeneration dependencies;
- accepted journey topology derived from Accepted Snapshots even while a feature is STALE, with
  dependent and inherited claims still blocked until current;
- one sealed Human Validation Session over exact implementation/product states, surfaces,
  viewports, and environments, containing only the per-feature/checkpoint/claim outcomes explicitly
  authorized by the human;
- Candidate Bundles consuming claim attributions while Stage 8 assessment, F8, and F9 remain
  feature-atomic;
- compatibility, schema ownership, CLI behavior, policy versions, and the unnumbered candidate
  configuration used before issuance.

## Protected invariant

Shared evidence may reduce execution or viewing, but it never manufactures claim coverage, human
judgment, review, acceptance, or currency. Unknown attribution fails closed.

## Acceptance criteria

1. The human approves the consequential semantics before WO-06 changes code or policy.
2. Same-surface/different-meaning and mixed-result cases remain representable.
3. A later feature, changed question, missing viewport, retained ID with changed meaning, false
   run-level attribution, duplicate ownership, and agent auto-population all fail closed.
4. Two distinct feature-owned claims may consume one evidence item without merging claim identity.
5. Legacy and prospective claim models coexist without rewriting history.
6. The brief names one normative owner and one executable derivation for each new fact.

## Brief (proposed 2026-10-18)

Grounded in the accepted WO-04 matrix. One normative owner for everything below: Feature Workflow
v13 (the unissued DBA-29 draft, extended; see decision (a)). Each new fact has one executable
derivation in `codeos_workflow`, which the reviewer reads (WO-02).

### 1. Validation Evidence Set (execution once)

- A verification run produces an immutable Validation Evidence Set at
  `.codeos/06-workflow/evidence-sets/<sha256>.json`: verification, environment (WO-03), normalized
  command, execution inputs, runner identity, observations (results, Observed Surface, production
  observations with their claim IDs, assertion results where an adapter reports them) and
  provenance (times, provider-call totals).
- **Execution identity** = sha256 of verification, environment, command, execution inputs and
  runner identity. Execution inputs are the implementation state plus the build contexts of every
  Compose service, derived from the Compose file. This closes the WO-04 finding (`30a6b63` changed
  the fixture emulator without changing the identity).
- `check` executes a verification only when no passed Evidence Set exists for its current execution
  identity. Otherwise it reuses that set and executes nothing.
- Validity dependencies (what makes a claim stale) are the product/dependency scopes. Regeneration
  dependencies (what forces a new run) are the test, runner and verifier identities. A
  Specification Package edit is a validity dependency of the claims it changes, never a reason to
  rerun a command that does not read it (WO-04: 4 such reruns).

### 2. Claims and attribution (ownership never merges)

- **Prospective claims.** A new or legitimately revised Contract has a `## Validation Claims`
  table: owner-qualified ID (`F-NNNN/<kebab-case>`, the Reality Basis pattern), claim text,
  checkpoint, evidence kind.
  - The claim's meaning digest covers its normalized text and checkpoint.
  - An ID whose digest differs from the digest recorded for it in that feature's latest bundle or
    acceptance fails closed: a changed meaning needs a successor ID.
  - Granularity: one independently reusable or invalidatable obligation, never an individual
    assertion. No count limit.
- **Legacy claims.** An unchanged legacy Contract keeps its coarse claims
  `F-NNNN/<checkpoint>/<verification>`. No retrofit is made for optimization, and no history is
  rewritten.
- **Attribution.** The per-feature verification record stays, so Candidate Bundles keep their keys.
  It now references the Evidence Set and states which observations or assertions cover which of
  that feature's claims.
  - A coarse legacy claim is attributed to the whole run, which is its meaning.
  - A prospective claim may be attributed to a whole run only if its Contract row declares
    evidence kind `run`. Otherwise it needs assertions or observations tagged with its ID. Missing
    or unknown attribution fails closed.
  - Tags live in test or verifier code, which may declare coverage but never define a claim.
    Generated indexes are caches with no authority.
  - Two features' claims may cite one Evidence Set. Each record names only its own feature's claim
    IDs, and one claim ID never appears in two features' records (duplicate ownership fails
    closed).
- **Staged adapters (decision (b)).** WO-06 ships `run` attribution and observation attribution
  (production claim IDs, which exist today, and human sessions). Assertion adapters
  (Playwright/Vitest tags, Rust) come later. Until then a prospective claim can use only `run` or
  observation evidence.

### 3. Shared production run (decision (c))

- `check --environment production` may name several features: `--subject A --subject B`. Each one
  must wait only on production evidence.
- The verifier receives the union of their owner-qualified claims in one definition and produces
  one Evidence Set. Each feature's F5 record attributes only its own claims' observations.
- A feature not named in that run cannot use its Evidence Set later; its claims are absent from the
  observations, so it fails closed.
- WO-04: 6 of 51 legs (162 receipted calls) were this case.

### 4. Human Validation Session

- `codeos-workflow session` records one sealed session over:
  - the exact implementation/product state;
  - the Observed Surfaces with their viewports;
  - the environment;
  - the outcomes the human states, one per feature/checkpoint (and claim, when the claim is
    prospective).
- Each outcome becomes that feature's receipt, binding the session identity. The session and its
  receipts are written atomically.
- Fails closed:
  - a feature whose current Observed Surface or viewports differ from the session's;
  - an outcome not stated explicitly (no wildcard, no "all features on this surface");
  - any addition after sealing;
  - a later feature;
  - a changed question (a different checkpoint or claim is a different outcome).
- Mixed results are allowed. Same surface with different meaning stays representable, because each
  outcome names its own question.
- CodeOS enforces explicitness. Who may run the command stays governed by the existing decision
  rules.
- WO-04: 15 judgments in 7 sessions.

### 5. Stable accepted topology

- The accepted journey topology comes from each feature's latest Accepted Snapshot (acceptance
  receipt and bundle), whether or not that feature is currently VALID.
- Current Validity still decides consumption: a claim depending on, or inherited from, a STALE
  upstream stays blocked until it is current.
- This also removes the per-consumer acceptance assessments from the frontier derivation. That is
  the main cost behind WO-04 status criterion 5 (136 assessments for 6 features). WO-06 brings it
  down to one projection per feature per command, with review records scanned once.

### 6. Unchanged

- Stage 8 assessment, F8 and F9 stay feature-atomic.
- Acceptance stays atomic.
- Bundles keep their keys.
- Configurations up to v12 behave byte-identically.
- Existing records and receipts are read as legacy and stay current under their recorded rule.

### Owners and derivations

| Fact | Normative owner | Executable derivation |
|---|---|---|
| Evidence Set, execution identity, execution inputs | Feature Workflow v13 | `evidence_set` module |
| Claim IDs, meaning digest, successor rule | Contract table, under v13 | `Subject::resolve` |
| Attribution and its currency | v13 | `verification_bindings_for_mechanic` + `verification::evaluate` |
| Shared production run | v13 | production `check` |
| Human Validation Session | v13 | `session` command, `decision_state` |
| Accepted topology | v13 | `accepted_frontier` |

### Fail-closed falsifiers (WO-06 tests)

- a later feature;
- a changed question;
- a missing viewport;
- a retained ID with changed meaning;
- run-level attribution not declared by the Contract;
- duplicate claim ownership;
- an outcome not stated by the human;
- a fixture emulator change;
- a mixed-result session;
- two features' claims citing one Evidence Set;
- a legacy and a prospective feature coexisting.

### Decisions for the human

- (a) One Feature Workflow v13 in DBA-29 carries both WO-03 and this work (recommended), rather
  than an additional v14.
- (b) Staged adapters as above (recommended), rather than assertion adapters in WO-06.
- (c) A shared production run takes explicitly named subjects (recommended), rather than CodeOS
  selecting every waiting feature.
