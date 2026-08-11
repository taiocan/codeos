# OAP × Codeos: An Experimental Layered Integration Profile

> Historical comparison material, not a doctrine consumer. It is not updated when doctrine
> semantics change; use `dba-system.md` and the selected doctrine for current guarantees.

*A neutral proposal for running Orchestrated Agentic Programming (OAP) and Codeos /
Declarative Behavioral Architecture (DBA) on the same project — and a critical assessment
of where they reinforce each other and where they fight.*

---

## Status and provenance

```yaml
document_type: proposal / analysis
status: EXPERIMENTAL PROFILE — NOT adopted Codeos doctrine
generated_from: session analysis on OAP × Codeos integration
generated_at: 2026-06-25
binding: none — this document changes no .codeos/ file and approves no rule change
```

This is an **analysis deliverable, not a DBA artifact.** It carries no `approved_by`,
no `derived_contracts`, and feeds no stage gate. It does not modify `CLAUDE.md`, the stage
prompts, the templates, or any operative Codeos rule. Everything it proposes for the
Codeos *core* is explicitly deferred behind a pilot (see §6). The guiding sentence:

> **OAP × Codeos should be integrated as an experimental layered profile, not immediately
> merged into Codeos core doctrine. OAP wraps Codeos operationally; Codeos constrains OAP
> behaviorally; neither weakens the other's core invariant.**

OAP is Janez Perš's human-governed delivery doctrine: a long-context **strategic AI**
turns domain intent into architecture and work orders, a high-autonomy **execution agent**
works inside a hardened rebuildable runtime, and the human owns intent, risk, and release.
Codeos / DBA is this repository's method: a 9-stage, human-gated, artifact-constrained,
event-verifiable loop in which the implementation may contain nothing that does not trace
to an approved artifact.

---

## 1. Core finding

They are **complementary, not redundant**, because they govern different axes — and they
fit together unusually well because Codeos's stage boundaries already cleave along the
exact lines OAP cares about.

| Axis | OAP governs | Codeos / DBA governs |
|---|---|---|
| Primary concern | Operational control: who does what, where, and how work flows | Semantic control: what is built, and how behavioral correctness is evidenced, reconciled, and replayed |
| Control mechanism | Role split (human / strategic AI / execution agent), disposable runtime, PR-sized delegation, release gates, cross-model audit | Intent → Contract → Event Schema → Reconcile → Replay artifact chain; everything traceable to an approved artifact |
| Unit of work | The pull request | The feature (9 stages) |
| Truth source | Remote repo + PR + CI (VM is disposable) | Approved artifacts + append-only `runtime_events.jsonl` |
| Evidence | Tests, CI, decision briefs | Event spine + reconciliation table + replay verification |
| Silent on… | How to specify/verify correctness internally (no formal spec chain, no event model) | Runtime isolation, secrets, strategic/execution role split, PR mechanics, release readiness |

The two reinforce each other across their respective gaps — **symmetrically**, not as
"OAP is incomplete, Codeos fixes it." OAP gives Codeos an operational envelope it does not
define (where code runs, who is strategic vs executor, how slices become PRs, when it
ships). Codeos gives OAP a more explicit behavioral evidence chain for features whose
correctness must be specified and replayed. OAP already has real correctness discipline —
validation-debt awareness, tests-as-evidence, decision briefs, PR review, a project
constitution — it simply does not formalize the intent→contract→event→replay chain the
way DBA does. The honest framing:

> Codeos gives OAP a more explicit behavioral evidence chain for features whose
> correctness must be specified and replayed; OAP already provides broader delivery
> governance and validation discipline.

---

## 2. Recommended integration type: Layered (OAP outer, DBA inner)

Three candidate shapes were considered:

- **(A) Layered / nested — RECOMMENDED.** OAP is the outer operational shell; DBA is the
  inner correctness method invoked inside each work unit. OAP decides roles, runtime,
  slicing, release; DBA decides what each slice must be and proves it.
- **(B) Side-by-side parallel — REJECT.** This is the genuinely conflicting option: two
  constitutions over one `CLAUDE.md`, two competing gate systems, two truth models,
  ambiguity over who approves what. This is the failure mode to avoid.
- **(C) Full merge into one doctrine — REJECT (for now).** High effort; blends away the
  clean separation that makes (A) work; only worth it if productizing a single method
  later.

The fit is structural, not forced. DBA's stages mostly align with OAP's reasoning-vs-
execution split, with Stages 6–8 requiring explicit role handling (runtime execution can
be mechanical; reconciliation is strategic; replay may need both strategic judgment and
execution commands):

| DBA stage | OAP role | OAP unit |
|---|---|---|
| 1 Intent, 2 Contract, 3 Event Schema | Strategic AI + human gates | Discovery + work order |
| 4 Implement, 5 Tests, 6 Runtime Execution | Execution agent — may implement, test, run, and capture runtime evidence inside an approved packet (disposable VM) | One PR-sized delegation |
| 7 Reconcile, 8 Replay | Strategic AI audits evidence; execution agent may only run requested replay commands under a bounded verification packet | Decision brief |
| 9 Refine | Strategic AI drafts repair order; execution agent runs the repair PR only after approval | Repair PR |

In this mapping: the DBA approved-artifact chain **=** the OAP work order; the DBA
reconciliation table **=** a core evidence section of the OAP decision brief (not the
whole brief — see §3); DBA replay verification **=** an OAP audit pass.

**Feature↔PR mapping is a default, not a rule.** A DBA feature should map to one OAP
reviewable unit by default, but large or risky features may be split into multiple
PR-sized delegations (artifacts → implementation+tests → runtime/replay hardening →
refinement) as long as the approved artifact chain stays coherent. OAP treats PR-sized
delegation as a context-management and review mechanism, not a rigid one-feature-one-PR
equation.

---

## 3. The two genuine tensions (critical assessment)

This is not a frictionless marriage. Two real conflicts must be resolved, or the
combination degrades.

### Tension 1 — Autonomy posture (head-on contradiction)

OAP's thesis is **bounded high autonomy**: the execution agent does PR-sized work at
machine speed in a disposable VM without step-by-step supervision. Codeos states the
opposite as a non-negotiable rule: *"NEVER add autonomous planning, self-direction, or
multi-step autonomous execution"* and *"Every stage transition requires explicit human
approval."* Naively merged, an OAP execution agent drives straight through DBA's gates —
an illegal state under current Codeos rules.

**Resolution — clarify, do not relax, the rule.** This is a clarification of what
"autonomous" means, not a weakening of Codeos's identity. The two systems gate at
different granularities: DBA gates at **artifact boundaries** (intent, contract, schema);
OAP autonomy lives at **implementation granularity, between gates.** Distinguish two kinds
of autonomy:

- **Planning autonomy — stays forbidden.** Codeos must still forbid autonomous product
  planning, scope expansion, stage transition, and artifact approval.
- **Execution autonomy — permitted, bounded.** Inside an explicitly approved execution
  packet (primarily Stages 4–6, plus repair/verification sub-packets), the executor may
  perform mechanical implementation, test execution, and runtime evidence capture inside
  an OAP runtime boundary — never crossing an artifact gate and never expanding scope.

Proposed wording (a clarification, **proposed here, not applied**):

> Codeos forbids autonomous product planning, scope expansion, stage transition, and
> artifact approval, but may permit bounded executor autonomy for mechanical
> implementation, test execution, and runtime evidence capture inside an approved
> Stage 4–6 work packet, under an OAP runtime boundary.

This preserves the **core DBA invariant** — *the implementation may contain nothing that
does not trace to an approved artifact* — because the executor still receives the approved
chain as its only mandate. **Risk to watch:** an executor that reads "bounded
high-autonomy zone" may over-read Stages 4–6. The clarification must be narrowly worded
and the invariant restated alongside it. This change is deferred behind a pilot (§6) — it
is not part of this document's effect.

### Tension 2 — Overhead stacking → rubber-stamping (the strongest objection)

Both systems are heavyweight. OAP itself warns it is "too heavy for one-off scripts"; DBA
runs 9 gated stages per feature. Stack them additively and you get ceremony-on-ceremony —
and OAP's own named failure mode (the human rubber-stamps weak evidence just to keep
moving) becomes near-certain.

**Resolution — deduplicate gates; never sum them.** The integration must make DBA gates
*be* the OAP human decision points, not a second layer on top. There is **one set of
gates.** The strategic AI compresses each gate into short decision material (the DBA
reconciliation table doubles as OAP's decision brief), keeping the human at decision
altitude rather than drowning them in approvals. *If the combined system ever produces two
approvals for one decision, the integration is wrong.*

### Lesser frictions (resolvable, lower risk)

- **Truth-source layering (with a hygiene caveat).** OAP: repo/PR is truth, VM disposable.
  DBA: `runtime_events.jsonl` is append-only durable truth. No conflict *provided*
  artifacts + durable evidence live in the repo layer, never VM-ephemeral state. But raw
  runtime logs must not be committed blindly — they can carry PII, test payloads,
  accidental secrets, large/nondeterministic data, and local paths. **Rule:** durable
  evidence lives in the repo, but raw runtime logs are committed only when sanitized,
  bounded, and intended as test/replay fixtures; otherwise keep raw logs as CI artifacts
  or local evidence and commit only derived replay fixtures.
- **Constitution collision.** Both claim `CLAUDE.md`. Codeos already owns it; OAP's
  operational rules (runtime boundary, PR workflow, report format, secrets, non-goals)
  belong in a referenced `docs/oap-runtime-boundary.md` or
  `patterns/oap-bounded-execution.md` — not in `.codeos/`, which is the toolkit symlink
  layer, not the project layer. A merge, not a conflict — and deferred behind the pilot.
- **Decision brief is more than the reconciliation table.** The DBA reconciliation table
  is a strong behavioral audit and becomes a *core evidence section* of the OAP decision
  brief — but the brief also needs branch/commit/PR, changed files, tests
  run/skipped/blocked, CI status, security considerations, docs changes, deployment risk,
  and open questions.
- **Anti-pilot vs human-in-every-gate.** DBA's frequent approvals could look like
  "piloting the human." But they are *decision* approvals (intent/contract/schema), not
  *chores* (installing deps) — exactly the human involvement OAP wants. Keep approvals at
  decision altitude and this is a non-issue.

---

## 4. Scope boundary — when to apply which

The integration is selective, and Codeos already hands us the seam:

- **Behavioral features** (alter a contract or event schema) → **DBA-inside-OAP**, full
  9-stage chain as the work-order content.
- **Non-behavioral structural work** (refactors, workspace restructuring, dependency
  consolidation) → Codeos already routes these to its **Architectural Refinement** alternate
  loop (Stage 10: Scope → Impact → Implement → Verify → Reconcile), which has no
  contract/schema. Here OAP's looser work-order model is the natural fit; pair it with a
  DBA arch-refinement record.
- **Throwaway scripts / prototypes / single-dev low-risk** → use neither stack, or
  OAP-lite. Both doctrines explicitly disclaim this zone.

---

## 5. Required changes to each system (all deferred behind the pilot)

These are the **eventual target, not this document's effect.** Nothing here is applied by
this file.

### Changes to Codeos / DBA (eventual, pilot-gated)

1. **Clarify (not relax) the anti-autonomy rule** — distinguish forbidden planning
   autonomy from permitted execution autonomy inside an approved Stage 4–6 packet
   (§3 Tension 1). Restate the trace-to-approved-artifact invariant alongside it. Worded
   as clarification, this does not change the spirit of Codeos.
2. **Add a runtime-boundary doctrine** (currently absent): disposable VM, no production
   secrets, where code runs. Adopt OAP Part II preflight — as a referenced `docs/` or
   `patterns/` document, not in `.codeos/`.
3. **Add role definitions**: strategic AI (Stages 1–3, 7–9) vs execution agent
   (Stages 4–6), and the anti-pilot / anti-rubber-stamp discipline.
4. **State the evidence-durability + sanitization rule** (§3 hygiene caveat): artifacts +
   sanitized replay fixtures in repo; raw logs as CI/local evidence.
5. **Add a (default, non-rigid) PR/branch binding**: a Stage-4–6 packet maps to one PR by
   default; the reconciliation table is a core section of the PR decision brief.

### Changes to OAP (eventual, pilot-gated)

1. **Adopt the DBA artifact chain as the required content of work orders** for behavioral
   features — replace loose "acceptance criteria in domain language" with intent +
   contract + event schema; keep loose work orders for architectural-refinement-type work.
2. **Adopt event-spine + replay as a first-class evidence type** alongside tests/CI.
3. **Accept finer-grained artifact gates** as legitimate strategic-layer decisions (extend
   "human works from short decision material" to include artifact approval at decision
   altitude — these are decisions, not chores, so not anti-pilot violations).
4. **Adopt DBA's truth-authority rules only inside DBA-governed behavioral feature work**
   (runtime-vs-intent precedence, safety/invariant primacy). They must not replace OAP's
   broader release authority, repository truth, CI evidence, security review, or human
   release decision, which span beyond the behavioral-feature scope.

---

## 6. Phased rollout — pilot before any core edit

The single most important discipline: **do not touch `CLAUDE.md` or the stage prompts as a
first step.** Those changes alter Codeos's operative behavior and identity rules; they
require their own review and an empirical pilot.

**Phase 1 — Analysis only (this document):**
- This `docs/oap-codeos-integration.md` exists as a neutral proposal / experimental
  layered profile, parallel to `docs/codeos-manual.md`.
- No modification to `CLAUDE.md`, prompts, or any `.codeos/` file.

**Phase 2 — Pilot one feature:**
- Pick one pilot feature: low-risk, non-security-critical, small enough to complete in one
  PR, but rich enough to exercise intent, contract, event schema, runtime events,
  reconciliation, and replay. (A trivial feature won't test the integration; a risky one
  is too dangerous for the first pilot.)
- Run DBA-inside-OAP **without changing any core Codeos rule** — work within current rules
  and observe where they bind.
- Record friction: gate count, human decision load, artifacts produced, evidence quality,
  PR size, whether the executor over-read its packet.
- **Rollback rule:** if the pilot produces duplicate approvals, executor scope expansion,
  unclear truth ownership, or weak decision material, **stop the integration** and keep
  OAP × Codeos as analysis only — do not proceed to any `CLAUDE.md`/prompt edit.

**Phase 3 — Narrow clarification (only if the pilot justifies it):**
- Add the narrowly worded "bounded execution packet" clarification (§3 Tension 1) —
  preferring a `docs/` / `patterns/` document first (`docs/oap-runtime-boundary.md` or
  `patterns/oap-bounded-execution.md`).
- Only then consider minimal `CLAUDE.md` / prompt references, each with a mini-ADR.

**Phase 4 — Canonical doctrine:**
- Promote to canonical Codeos doctrine only after repeated successful use.

---

## 7. Risk register

| Risk | Mitigation |
|---|---|
| Executor over-reads "bounded autonomy" and crosses an artifact gate | Narrow wording; restate trace-to-artifact invariant beside it; pilot observation |
| Gate-stacking → human rubber-stamps | Deduplicate gates; reconciliation table = the decision material; one approval per decision |
| Core Codeos identity rules edited prematurely | Phase-gated; clarification (not relaxation); ADR per edit |
| Raw runtime logs leak PII/secrets into repo | Commit only sanitized, bounded replay fixtures; raw logs → CI/local |
| Truth-authority model over-generalized into OAP | Scope DBA truth authority to behavioral-feature work only |
| Combined overhead kills velocity | Selective scope (§4): heavy chain for behavioral features only |

---

## 8. Pilot checklist (Phase 2 exit criteria)

- [ ] One behavioral feature completed under current Codeos rules + OAP role split
- [ ] Total human approvals counted; no duplicate approval for a single decision
- [ ] Executor stayed inside Stages 4–6; no scope expansion, no gate crossing
- [ ] Evidence durable + sanitized; no raw-log leakage
- [ ] Decision brief = reconciliation table + OAP operational evidence, both present
- [ ] Documented verdict: is a `CLAUDE.md`/prompt clarification actually needed?

---

## 9. Verdict

Complementary on orthogonal axes, with a structurally clean fit because DBA's stages
already split along OAP's reasoning-vs-execution and gate-vs-autonomy lines. The right
model is **layered (OAP outer, DBA inner)**; the wrong model is the parallel operation —
that one genuinely conflicts. Two real tensions must be actively managed: **clarify
(don't relax) autonomy** so execution is bounded to between-gate Stage 4–6 packets, and
**deduplicate gates** so the combined process doesn't collapse into rubber-stamping. The
pairing is stronger than either alone — OAP supplies operational governance, Codeos a more
explicit behavioral evidence chain — but it should enter as an **experimental layered
profile validated by a pilot**, never as an immediate edit to Codeos core doctrine.

---

*Companion document: `docs/oap-adoption-candidates.md` — a separate lens listing discrete
OAP features worth absorbing into Codeos core some day, each critically assessed.*
