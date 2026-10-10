# Solution Charter — CodeOS Professional Engineering Control Panel

- **Status:** APPROVED — approved by Primoz Gorjup, 2026-10-10
- **Scope:** CodeOS-wide
- **Baseline:** Engineering Control Panel v1 already implemented
- **Purpose:** Extend existing `codeos-workflow status` into the primary professional engineering
  control panel for CodeOS projects and, later, the sensor interface for bounded development
  automation.

## 1. Problem

CodeOS already knows substantial facts about workflow progression, acceptance, current validity,
qualification, review, evidence, release integrity, compatibility, and operational observations,
but they are distributed across commands and records. The user still has to reconstruct:

1. Is the project progressing correctly?
2. Is previously accepted work still valid?
3. Which engineering foundations are healthy or weak?
4. Why is something blocked or stale?
5. What evidence supports the reported state?
6. What is the next legitimate action?
7. Who may perform that action?
8. Is CodeOS itself creating avoidable engineering friction?

The control panel shall provide these answers without creating another source of workflow
authority.

## 2. Product intent

Create a **professional engineering control panel** that derives a comprehensive view of the
current software-development state from existing CodeOS authority. It shall evolve the existing
`codeos-workflow status` rather than introduce a parallel monitoring system.

Target loop: authoritative CodeOS state → engineering status projection → attention / engineering
health → explainable legitimate next action → human / assisted / automatic execution → existing
CodeOS verification → updated authoritative state ↺.

The panel observes and explains. Existing CodeOS mechanisms continue to govern progression,
evidence, review, validation, acceptance, and operational learning.

## 3. Outcomes

**O-1 — Complete engineering visibility.** One coherent project view covers workflow progress,
historical acceptance, current validity, specification, architecture, implementation integrity,
verification, evidence, independent assessment, project qualification, security and supply chain,
release qualification, compatibility, deployment readiness, and operational observations. The view
MUST distinguish **accepted history** from **current validity**.

**O-2 — Explainable attention.** Every material problem identifies WHAT is wrong, WHY it is
reported, WHAT surface is affected, WHAT authority/evidence establishes it, WHAT the legitimate next
action is, and WHO may execute that action. A user or development agent should not need to
rediscover these facts from the repository before acting.

**O-3 — Engineering-process learning.** CodeOS derives factual indicators of development friction
from existing immutable records, e.g. `NOT_READY`, `EXECUTION_INVALID`, blocked assessments,
Candidate Bundles per accepted change, evidence invalidation causes, repeated human decisions,
qualification failures, open/closed observations, implementation escapes, observation closure
time, and time spent blocked by tooling or human decisions where derivable. The purpose is to
expose **where engineering effort is being lost**, not to produce developer productivity scores.

**O-4 — Machine-actionable next steps.** The status projection exposes structured next-action
information for humans and tools. Each action includes at least: action type, subject, reason,
actor class, applicable authority, and required command/mechanism where deterministic. Actor
classes remain `AUTO`, `ASSISTED`, `HUMAN`. Actor class describes **who may execute the action**.
It grants no approval, acceptance, or decision authority.

**O-5 — Bounded automated feedback loop.** After separate PROTECTED authorization, CodeOS MAY
execute explicitly allowlisted `AUTO` actions. Initial candidates are limited to deterministic
operations such as: run required verification; run project qualification; prepare a Candidate
Bundle; regenerate deterministic evidence; invoke an already-authorized independent assessment.
Every automated action MUST be followed by a fresh authoritative state projection. Automation MUST
stop at a HUMAN boundary.

## 4. Functional scope

**ECP-1 — Core status projection — IMPLEMENTED.** Workflow state; current issue; next action;
`AUTO | ASSISTED | HUMAN` actor; deterministic attention list; stale accepted-feature visibility;
open Operation v2 observations; historical acceptance separated from current validity. This is the
baseline for all later work.

**ECP-2 — Engineering Health Projection.** Extend `status` with authoritative engineering-health
facts already available to CodeOS. Initial domains:

| Domain | Typical source |
|---|---|
| Product & scope | Charter, decomposition |
| Specification | Intent, Contract, Event Schema |
| Architecture | Architecture Scope, change-impact state |
| Implementation | repository/build identities |
| Verification | F5 evidence |
| Evidence integrity | Candidate Bundle, F6 |
| Independent assessment | F7 |
| Project quality | `scripts/codeos-qualify` |
| Security | qualification baseline |
| Supply chain | qualification/audit results |
| Release | CodeOS qualification manifest |
| Compatibility | support replay |
| Deployment | pre-deployment constraints |
| Operational learning | Operation v2 |

Example statuses: Product `CURRENT`, Verification `PASS`, Independent review `PASSED`, Release
`QUALIFIED`, Deployment `BLOCKED`, Operational learning `1 OPEN`. Project-defined measurements MAY
also appear (e.g. `Provider calls 27 / leg MEASURED`, `Approved limit none`). `MEASURED` MUST NOT be
converted into `PASS` without an approved constraint.

**ECP-3 — Explainable Attention.** Extend attention records with enough information to diagnose
the condition: subject, state, issue, reason, affected_surface, unaffected_surface where useful,
authority, evidence identities, next_action, actor. Example: `F-0003 ACCEPTED / CURRENTLY STALE`;
reason: applicable implementation identity changed; affected: browser verification; unchanged:
Intent, Contract, Architecture; next action: `[AUTO] run bounded revalidation`. The explanation
MUST be derived from existing authoritative state.

**ECP-4 — Process Health and Trends.** Derive engineering-process statistics from existing records,
with current-window and historical summaries, e.g. for the last 30 days: accepted changes, Candidate
Bundles, BLOCKED assessments, NOT_READY, EXECUTION_INVALID, evidence invalidations, repeated human
decisions, observations opened, observations closed, implementation escapes. Where timestamps
permit, also derive feature/change lead time, observation closure time, waiting-for-human time, and
blocked-on-tooling time. These values are diagnostic evidence. They MUST NOT mechanically produce a
maturity score.

**ECP-5 — Structured Action Interface.** Make an existing next action machine-readable enough that
an external tool or later executor does not need to reinterpret free text, e.g.
`{"subject": "F-0003", "actor": "AUTO", "action": "run_verification", "reason": "accepted browser
evidence is stale", "authority": "Feature Workflow v7", "command": "codeos-workflow check F-0003"}`.
This remains part of the **read-only status projection**. No action is executed by ECP-5.

**ECP-6 — Bounded AUTO Execution.** Requires a separate **PROTECTED approval before implementation
or activation**. If approved, it SHALL:

1. accept only an explicit allowlist of deterministic `AUTO` actions;
2. execute one action at a time;
3. record tool/result provenance through existing mechanisms;
4. derive status again after execution;
5. stop if the result changes the legitimate next action;
6. stop at every `ASSISTED` or `HUMAN` boundary.

It MUST NOT automatically: approve a Charter; approve specification; approve architecture; make an
F8 decision; make an F9 acceptance decision; choose product meaning; invent a threshold; resolve an
Observation decision; reinterpret a reviewer judgment.

## 5. System Constraints

- **C-1 — Read-only projection.** Status and health monitoring MUST NOT create or modify workflow
  authority, evidence, decisions, approvals, acceptance, or Observation state.
- **C-2 — Existing truth remains authoritative.** The panel MUST derive from existing CodeOS
  artifacts, receipts, records, identities, qualification output, and repository state.
  It MUST NOT maintain a competing authoritative monitoring database. Disposable caches MAY be used only if
  the complete view can be reconstructed from canonical sources.
- **C-3 — Acceptance and validity remain separate.** Historical acceptance and current validity
  MUST never be collapsed into one status. `Historical acceptance COMPLETE` with `Current validity
  STALE` is valid and MUST remain representable.
- **C-4 — No fabricated engineering health.** A health result may be shown only when CodeOS has an
  authoritative basis for it. Absence of evidence is reported as `UNRESOLVED`, `UNOBSERVED`,
  `NOT_APPLICABLE`, or `NOT_CONFIGURED` as appropriate. It MUST NOT silently become `PASS`.
- **C-5 — Project thresholds remain project-owned.** CodeOS MUST NOT invent universal thresholds
  for performance, accessibility severity, viewport sizes, provider calls, latency, licences,
  product reliability, or domain-specific quality. The control panel reports approved constraints
  and measurements.
- **C-6 — Facts over scores.** No universal project-quality, engineering-maturity,
  developer-performance, or percentage-complete score SHALL be introduced. The monitor reports
  evidence and states; human assessment may use those facts.
- **C-7 — Deterministic derivation first.** Workflow state, health state, issue classification,
  actor classification, and legitimate-action derivation SHOULD be deterministic where existing
  CodeOS authority permits. LLMs MUST NOT determine authoritative project state.
- **C-8 — Human authority remains protected.** Actor `AUTO` or `ASSISTED` grants execution
  capability only. It MUST NOT transfer decision authority. Human-controlled boundaries defined by
  the active DBA configuration remain human-controlled. F8 and F9 remain HUMAN.
- **C-9 — No domain leakage.** A pilot project may prove the framework but MUST NOT introduce its
  domain rules into generic CodeOS monitoring. JamFree-specific viewports, provider behavior, route
  semantics, latency expectations, and provider budgets remain JamFree-owned.
- **C-10 — Explainability.** Every blocking, stale, or unresolved state SHOULD be traceable to its
  source authority or evidence identity. A user must be able to understand why CodeOS reports the
  condition.
- **C-11 — Compatibility.** New monitor fields MUST follow explicit status-schema compatibility
  rules. Additions MUST NOT silently change existing workflow semantics, exit codes, progression, or
  targeted `check` behavior.
- **C-12 — Lean runtime.** Monitoring SHOULD require no background AI, no dedicated server, no
  permanent dashboard database, and no duplicated workflow engine. CLI/JSON remain the canonical
  presentation interfaces.

## 6. Attention model

The panel SHOULD use only a small number of presentation-level attention classes:

- **BLOCKER** — legitimate progression or deployment cannot continue;
- **STALE** — previously valid evidence/state is no longer current;
- **UNRESOLVED** — a known question or obligation lacks valid disposition;
- **WARNING** — important information that does not currently block.

These are presentation classifications only. They MUST NOT replace underlying workflow states.

## 7. Presentation

Primary interfaces remain `codeos-workflow status` and `codeos-workflow status --json`. Optional
filtering or drill-down MAY later expose `status F-0003`, `status --engineering`, and
`status --history`. A future HTML view MAY consume the same JSON. It MUST NOT introduce separate
state or lifecycle semantics. Separate `monitor` and `plan` subsystems are not required by this
Charter.

## 8. Success criteria

The full control-panel scope is successful when a user can obtain authoritative answers to all of
these questions without manually reconstructing repository state:

1. **Where are we in the development process?**
2. **What has historically been accepted?**
3. **What remains currently valid?**
4. **Which professional engineering foundations have current evidence?**
5. **What is blocked, stale, unresolved, or noteworthy?**
6. **Why is each condition reported?**
7. **What is the next legitimate action?**
8. **Is that action AUTO, ASSISTED, or HUMAN?**
9. **Where is engineering effort being lost repeatedly?**
10. **What observations show that the product or development process needs improvement?**

For the later automation scope:

11. An approved `AUTO` action can be executed without bypassing any human authority.
12. State is re-derived after every automated action.
13. Automation cannot silently cross an `ASSISTED` or `HUMAN` boundary.

## 9. Out of scope

A project-management system; issue tracking replacement; employee productivity scoring; universal
software-quality thresholds; an AI project manager; a second workflow engine; autonomous
architecture or specification decisions; automatic approval or acceptance; automatic changes to
CodeOS governance; a permanent monitoring database; a large web dashboard; deployment/runtime
operations standards not already governed elsewhere.

## 10. Delivery sequence

ECP-1 Core Control Panel (COMPLETE) → ECP-2 Engineering Health Projection → ECP-3 Explainable
Attention → ECP-4 Process Health + History → ECP-5 Structured Action Interface → real use /
observation → ECP-6 Bounded AUTO Execution, only after separate PROTECTED approval.

ECP-2 through ECP-5 are the current full monitoring scope. ECP-6 is the logical feedback-loop
extension, but the Charter deliberately does **not pre-authorize autonomous execution**, preserving
the human-control principle central to CodeOS.
