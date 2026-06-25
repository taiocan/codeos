# OAP Feature-Adoption Candidates for Codeos

*A critically-assessed list of the most valuable ideas from Orchestrated Agentic
Programming (OAP) that would be reasonable to fold into Codeos / DBA **core** at some point
in the future — independent of whether the full layered integration ever happens.*

---

## Status and scope

```yaml
document_type: analysis / backlog of ideas
status: NOT adopted — candidates only
generated_at: 2026-06-25
binding: none — this document changes no .codeos/ file
```

This is a **different lens** from `docs/oap-codeos-integration.md`. That document asks
"how do we run OAP *beside* Codeos?" This document asks "which discrete OAP ideas are worth
absorbing *into* Codeos itself, on their own merit?" The two overlap (the autonomy split
appears in both) but the framing differs: here each item is judged as a standalone Codeos
improvement, not as part of an integration.

Every candidate is assessed on a fixed shape so they can be compared honestly:

- **What it is** (in OAP)
- **Value to Codeos / Why**
- **Critical caveat / risk** — the honest reason it isn't a free win
- **Overlap with existing Codeos mechanism** — so we don't reinvent what's already there

Candidates are grouped into three tiers by cost-and-risk, not by raw value.

---

## Tier 1 — Adopt-soon (high value, low cost, low rule-risk)

These are doctrine/template additions that sharpen Codeos without touching a non-negotiable
rule. They could be added with ordinary edits and a review entry, not a pilot.

### 1.1 Standardized evidence / report vocabulary

- **What it is.** OAP forbids the word "done" as a feeling and mandates a fixed reporting
  vocabulary: **passed / failed / skipped / not-run / blocked / out-of-scope**, with the
  hard rule *"a skipped test is not a passing test; a test that was not run is not
  evidence."*
- **Value / why.** Codeos's reconciliation and replay stages live or die on honest
  evidence. A shared vocabulary makes Stage 5 (tests), Stage 6 (observation), and Stage 7
  (reconcile) reports machine-comparable and kills the most common silent failure — "all
  tests passed" masking skips and not-runs.
- **Critical caveat.** Almost a pure win; the only risk is vocabulary sprawl if it isn't
  pinned to one authoritative table. Keep it to the six terms.
- **Overlap.** Strong and complementary. Stage 7 already defines an **Evidence Quality
  Scale** (Specification → Static → Simulated → Real boundary → Production) and
  ALIGNED / GAP / MISMATCH / MISSING status codes in `prompts/07-reconcile.md`. The OAP
  vocabulary slots *underneath* the EQ scale at the per-check level — EQ measures
  *environment fidelity*, this measures *whether the check ran at all*. They are orthogonal
  and both useful.

### 1.2 Validation-debt framing

- **What it is.** OAP names the central economic fact: AI makes code cheap but not
  correctness; the bottleneck moves to proof, review, and judgment ("validation debt").
- **Value / why.** Codeos already *embodies* this (proof-before-code) but never names it.
  Naming it gives the manual a crisp motivating concept and a vocabulary for *why* the
  9 stages exist — useful for onboarding and for justifying the overhead to skeptics.
- **Critical caveat.** Purely conceptual; zero mechanical effect. Value is educational, so
  it belongs in the manual/preface, not in a stage prompt.
- **Overlap.** Conceptual sibling of the existing "anti-improvisation" framing in
  `docs/codeos-manual.md`. Additive, not duplicative.

### 1.3 Anti-pilot / control-inversion as a named failure mode

- **What it is.** OAP's most compact rule: if the execution agent starts directing the
  human through low-level chores (install this, paste that log), the control loop has
  inverted and must be redesigned.
- **Value / why.** Codeos sessions can drift into the human acting as a terminal for the
  AI. Naming "control inversion" as an explicit failure mode gives both human and Claude a
  shared trigger to stop and re-scope. Cheap diagnostic, real payoff.
- **Critical caveat.** Codeos deliberately keeps the human in *decision* gates; the
  subtlety is distinguishing legitimate decision-approval (good) from chore-piloting (bad).
  The doctrine must draw that line or it will be misread as "approve less."
- **Overlap.** Reinforces the existing CLAUDE.md stance ("the human is the author of
  intent, not a terminal operator"). Makes an implicit value into a checkable failure mode.

### 1.4 Non-goals as an explicit intent/contract field

- **What it is.** OAP makes **non-goals** a first-class, concrete guardrail in every work
  order ("do not add a migration", "do not change public API shape", "do not report skipped
  tests as passed").
- **Value / why.** Codeos forbids scope expansion as a rule, but a rule is weaker than a
  *named, feature-specific list*. An explicit `Non-goals:` field in the intent (and echoed
  in the contract) converts a general prohibition into concrete, per-feature guardrails the
  executor can check itself against.
- **Critical caveat.** Partial overlap risk — could feel redundant with "never add behavior
  beyond intent." The value is specificity, so it's only worth it if non-goals are concrete
  and feature-specific, not boilerplate.
- **Overlap.** Strengthens non-negotiable rule #3 ("never add behaviors beyond the current
  intent + contract + event schema"). A template change to `templates/intent.md`, not a new
  rule.

---

## Tier 2 — Worth piloting (real value, moderate cost or design work)

These have genuine value but introduce a new artifact, a new environment dependency, or
recurring cost. They warrant a small trial before becoming doctrine.

### 2.1 Per-gate decision brief

- **What it is.** OAP compresses each human decision into a short brief: **recommendation +
  goal-match + evidence + risk + decision-needed** — so the human stays at decision altitude
  instead of reading raw diffs and logs.
- **Value / why.** Codeos currently tends to present the full artifact at each gate. A
  short decision brief at stage transitions would directly fight the rubber-stamping
  failure mode and lower human decision load on multi-feature projects.
- **Critical caveat.** Must stay **ephemeral**, generated on demand — *not* a new stored
  artifact with its own lifecycle, or it becomes drift surface and another thing to keep in
  sync. The brief summarizes; the artifacts remain truth.
- **Overlap.** The Stage 7 reconciliation table is already most of a decision brief's
  evidence core, and **Human Navigation** already establishes the "generate a plain-language
  explanation on demand, save nothing" pattern. A decision brief is that same pattern
  applied at every gate, plus a risk/recommendation line. Low conceptual distance.

### 2.2 Rebuildable / disposable runtime boundary

- **What it is.** OAP's Part II: run the execution agent in a hardened, disposable VM with
  no production secrets and no irreplaceable state; durable truth lives in the repo so the
  VM can be destroyed and rebuilt freely.
- **Value / why.** Codeos has **no runtime-environment doctrine at all** today, yet Stage 6
  (Runtime Execution) literally runs the implementation and Stage 8 replays it. A runtime
  boundary makes that execution safe and reproducible, and improves Evidence Quality (real-
  boundary EQ 4 becomes achievable without endangering host secrets).
- **Critical caveat.** Codeos is a *method*, not infrastructure. Baking VM ops into core
  risks scope creep and platform-coupling. Deliver it as a **referenced `patterns/`
  document** (e.g. `patterns/oap-bounded-execution.md`), not as a core rule — guidance the
  method points to, not machinery the method owns.
- **Overlap.** Fills a genuine gap. Touches Stage 6 (`prompts/06-observe.md`) and Stage 8
  (`prompts/08-replay.md`) and the EQ scale, but adds rather than conflicts.

### 2.3 Cross-model audit cadence

- **What it is.** Build with one model, audit with another at defined trigger points —
  architecture audit (after scaffold), boundary audit (after auth/secrets/billing),
  maturity audit (before "beta/RC" language), follow-up audit (after remediation lands).
- **Value / why.** A second model that didn't write the code catches overclaiming,
  circular reasoning, and missing negative tests that the building thread is blind to. It
  would strengthen the existing Reviewer Activation Package against self-review bias.
- **Critical caveat.** Real cost (a whole second review pass) and not free of its own
  bias — cross-model is *less circular*, not *objective*. Reserve for high-risk features
  and the defined trigger points; don't run it every feature.
- **Overlap.** Extends the **Reviewer Activation Package** (`prompts/pipeline-reviewer.md`)
  and the **Architecture Journal** (which already archives findings as AJ-NNN entries). The
  audit→remediation→verify loop maps cleanly onto Stage 9 Refine.

---

## Tier 3 — Deferred / contentious (touches Codeos identity rules)

High potential value, but these brush against non-negotiable rules or Codeos's deliberate
scope. They should only follow a successful pilot, each with a mini-ADR.

### 3.1 Strategic / execution role split + bounded execution autonomy

- **What it is.** OAP separates the strategic role (planning, architecture, reconciliation)
  from the execution role (mechanical implementation, tests, runtime capture) and grants the
  executor bounded autonomy *between* gates.
- **Value / why.** This is the single biggest speed lever: it lets Stages 4–6 run at machine
  speed inside an approved packet without a human babysitting every step, while keeping the
  artifact gates intact. It is also the conceptual basis for the autonomy clarification in
  `docs/oap-codeos-integration.md` §3/§5.
- **Critical caveat.** Directly touches the non-negotiable rule against "autonomous
  planning, self-direction, or multi-step autonomous execution." Even framed as a
  *clarification* (planning autonomy forbidden, execution autonomy bounded), it is the most
  identity-sensitive change here. **Pilot first; mini-ADR per edit; restate the
  trace-to-artifact invariant beside any wording.**
- **Overlap.** Re-reads the existing non-negotiable rules rather than adding a mechanism.
  See the integration doc §3 Tension 1 for the proposed wording.

### 3.2 Release-readiness as a distinct gate

- **What it is.** OAP separates *implemented* from *supported / tested / documented /
  production-ready* via completeness levels and a release decision brief — release language
  is treated as part of technical correctness ("beta means beta").
- **Value / why.** It prevents agentic velocity from hardening into false maturity — a
  feature can pass all 9 DBA stages and still not be safe to ship. A lightweight release-
  honesty checklist would close the gap between "reconciled" and "releasable."
- **Critical caveat.** Codeos deliberately governs **feature correctness, not shipping.**
  Adding a release gate may be out of scope by design and could bloat the method. Offer it
  as an **optional** checklist for multi-feature projects, never a 10th mandatory stage.
- **Overlap.** Partly served by the Evidence Quality Scale (EQ 5 = Production) and the
  Feature Registry status index, but neither makes an explicit release judgment. This would
  be additive and optional.

---

## Recommendation

Pursue **Tier 1 freely** — these are doctrine and template sharpenings with no rule risk,
and several (evidence vocabulary, non-goals field) slot directly into existing mechanisms.
**Pilot Tier 2** — each adds real value but also a new artifact, dependency, or recurring
cost, so validate the shape on one feature before committing. **Gate Tier 3 behind the
integration pilot** in `docs/oap-codeos-integration.md` — they touch Codeos's identity
rules or deliberate scope, and changing those without empirical evidence is exactly the
premature-core-edit risk that proposal warns against.

---

*Companion document: `docs/oap-codeos-integration.md` — the layered integration proposal
and its risk register / pilot plan.*
