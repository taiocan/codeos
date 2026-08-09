# Codeos Implementation Roadmap

> **This is mutable planning state, not Codeos doctrine.** [`../backlog/features.md`](../backlog/features.md)
> is the authoritative `UPG-#### → file` map (identity); **this file is authoritative for
> *order*** — it sequences that catalog into dependency-aware waves and tracks current reality.
> **Each feature still requires its own self-dev change and explicit human approval before
> implementation** — appearing in a wave here is *not* authorization. Live per-change status lives
> in [`self-development.md`](self-development.md); per-change detail in
> `../changes/UPG-####__CHG-YYYYMMDD-NNN__slug.md`.

Roadmap rows are keyed by **`UPG-####`** (the stable feature id). `Planned/active change` names the
`CHG-*` once one exists; change ids are execution detail, not feature identity. Some work landed
outside this sequence (see Current State) and is recorded truthfully without false retroactive
sequencing.

---

## Current State (completed / piloted / in flight — not in a wave)

Entries under **Change ID** are self-development *change* records (execution), not feature ids.

| Change ID | Feature ID | State | Notes |
|---|---|---|---|
| `0001-claude-split` | — (no backlog feature) | DONE | Split downstream doctrine (`dba-system.md`) from the self-dev guide (`CLAUDE.md`). |
| `0002-doc-consistency-rename` | UPG-0002 | DONE | Renamed doctrine-attribution refs in docs after the split. |
| `0003-implementation-roadmap` | — (planning) | DONE | Created this roadmap. |
| `0004-review-fixes` | — (reviewer findings) | DONE | Advisory-review follow-up fixes. |
| — | UPG-0003 | PILOTED | `reviewer-decision-brief`: advisory Bash reviewer (`scripts/codeos-review.sh` + `docs/reviewer-pipeline.md`). Manual, read-only, non-gatekeeping. |
| `CHG-20260627-001` | UPG-0001 | COMPLETE | Feature Thread traceability + stable IDs (review-series self-reference boundary; accepted by decision 2026-06-28). |
| `CHG-20260802-001` | UPG-0060 | COMPLETE | CHG-A of `deepseek-delegated-implementation`: built + piloted an opt-in, off-by-default DeepSeek Stage 4/5 implementer tool (`scripts/codeos-implement.sh`). Accepted 2026-08-03. |
| — | UPG-0060 | PILOTED (negative) | CHG-B gate: a realistic-feature measurement (EvidenceAtlas EA-0003 Stage 4) returned NOT NET-POSITIVE — the delegated arm cost the Claude-only arm plus ~5.4K Claude input tokens plus 28,437 DeepSeek tokens and saved zero Claude output tokens. **CHG-B will not be done.** Feature held at CHG-A; tool stays off by default with no downstream footprint. Re-test conditions named in the brief. Finding journaled as AJ-022. |
| `CHG-20260803-001` | UPG-0060 | COMPLETE | Harness correction — re-test **condition 0**, a prerequisite to any further delegation comparison: the gate measurement was confounded by CHG-A's own packet (forbade build manifests, no layout exemplar, instructed against required abstractions, JSON-escaped single-shot output, no compiler feedback). `prompt + script-tooling`, self-dev only. Does not re-open CHG-B and does not enable the mechanism anywhere. |
| `CHG-20260803-003` | UPG-0060 | COMPLETE | Narrow CHG-A bugfix: `jq --arg` passed the whole packet as one argv element, hitting Linux's 128 KiB single-argument cap once packets reached realistic size. Fixed with `--rawfile`; 34 tests pass, regression test mutation-verified. Also discharges the UPG-0060 freeze and records the completed re-test (negative — general Stage-4 delegation not adopted). |
| `CHG-20260803-002` | UPG-0062 | COMPLETE (negative) | Architecture-constrained delegation **premise test — FAILED**. Producing EA-0004's implementation design from approved artifacts cost 0.619 of implementing directly by output (≤0.40 required) and 0.802 weighted (≤0.50), against a threshold committed before any measurement existed. No Rust engine, no shim rewrite, no pilot — the sequencing guardrail worked. **Q2 answered independently: the governance gap is real** (10/10 mechanism allocations NEW DESIGN; EA-0001 ships 4 such decisions in code alone) → filed as UPG-0063. |
| `CHG-20260804-001` | UPG-0063 | COMPLETE | Deferral → Resolution trace for Stage 4. Approved artifacts sometimes explicitly defer a question; Stage 4 must resolve it to exist; nothing records how, or that the resolution is interim. Q0 confirmed the pattern across EA-0001 + PlotSpot F-0001/2/3 (method precommitted at `1b0dbd1`). Leanest hypothesis: a short subsection in `prompts/04-implement.md`, existing Stage 4 gate, no new stage or artifact. |
| `CHG-20260804-002` | UPG-0064 | COMPLETE | CHG-A: delegated Stage-4 **envelope alignment**. UPG-0051/0052/0063 built a governed envelope; the delegated path never received it — `codeos-implementer-task.md` mentions Architecture Baseline / Implementation Profile / Cohort / deferral **zero** times, and every input is flattened to `APPROVED ARTIFACT`. An integration defect in the harness, not an architecture-design problem. Harness only; the three-case pilot is CHG-B. Stage 5 explicitly out. |

(`UPG-0006` reviewer-quality-scale is **partially piloted** — the reviewer already emits an
evidence grade; the full scale is scheduled in Wave 1.)

---

## Waves (dependency-aware)

Sequencing principles: value first (P0 → P3); respect hard dependencies; build on the existing
reviewer pilot (UPG-0003); preserve "advisory-not-autonomous" and "evidence-integrity"; take
cheap pilot-adjacent wins early.

### Wave 1 — Transparency & state  *(P0, no deps, highest value / lowest risk)*
| Wave | Feature ID | Title | Priority | Depends on | Planned/active change | State |
|---|---|---|---|---|---|---|
| 1 | UPG-0004 | Structured Stage 4–6 Reports | P0 | — | CHG-20260630-001 | COMPLETE |
| 1 | UPG-0005 | Current Verified State Block | P0 | — | CHG-20260630-004 | COMPLETE |
| 1 | UPG-0006 | Reviewer Summary Quality Scale | P2 | UPG-0003 ✓ | CHG-20260701-008 | COMPLETE |

### Wave 2 — Discovery & registry
| Wave | Feature ID | Title | Priority | Depends on | Planned/active change | State |
|---|---|---|---|---|---|---|
| 2 | UPG-0007 | Expanded 00b Solution Discovery / Feature Topology | P0 | — | CHG-20260630-005 | COMPLETE |
| 2 | UPG-0008 | Configuration Discovery & Schema Track | P2 | UPG-0007 ✓ | CHG-20260701-006 | COMPLETE |
| 2 | UPG-0009 | Feature Registry / Branch Binding | P2 | — | CHG-20260701-007 | COMPLETE |

### Wave 3 — Evidence discipline & gates  *(P1)*
| Wave | Feature ID | Title | Priority | Depends on | Planned/active change | State |
|---|---|---|---|---|---|---|
| 3 | UPG-0010 | Verification-Only Mode | P1 | — | CHG-20260701-002 | COMPLETE |
| 3 | UPG-0011 | Lightweight PR / Pre-Release Readiness Checklist | P1 | — | CHG-20260701-003 | COMPLETE |
| 3 | UPG-0012 | Repair-Before-Next-Feature Workflow Gate | P1 | — | CHG-20260701-004 | COMPLETE |
| 3 | UPG-0013 | Stage 4 Activation Card | P1 | — | CHG-20260701-005 | COMPLETE |

### Wave 4 — Reviewer hardening & delivery
| Wave | Feature ID | Title | Priority | Depends on | Planned/active change | State |
|---|---|---|---|---|---|---|
| 4 | UPG-0032 | Rust Reviewer Engine with Multi-Provider Support | P2 | UPG-0027 ✓ | CHG-20260702-001 ✓ | COMPLETE |
| 4 | UPG-0014 | Reviewer Agent with Full Diff Access | P2 | UPG-0032 | CHG-20260702-004 | COMPLETE |
| 4 | UPG-0015 | Bind stage approval to reviewed provenance | P1 | UPG-0032 ✓ | CHG-20260702-002 | COMPLETE |
| 4 | UPG-0016 | Branch / PR / CI Workflow Profiles | P1 | — | CHG-20260702-003 | COMPLETE |
| 4 | UPG-0017 | Stack Manifest with Automatic Reconciliation | P2 | — | CHG-20260703-001 | COMPLETE |

### Wave 5 — Advanced automation & generators  *(mostly P3 + the big P2 rewrite)*
| Wave | Feature ID | Title | Priority | Depends on | Planned/active change | State |
|---|---|---|---|---|---|---|
| 5 | UPG-0018 | Typed reviewer engine (replace Bash pilot) | P2 | — | — | SUPERSEDED by UPG-0032 |
| 5 | UPG-0019 | CI Integration Profile | P3 | UPG-0017 | CHG-20260706-001 | COMPLETE |
| 5 | UPG-0020 | Stack / Config Drift Detector | P3 | UPG-0017 | CHG-20260703-002 | COMPLETE |
| 5 | UPG-0021 | Stage Report Generator | P3 | UPG-0004 | CHG-20260703-003 | COMPLETE |
| 5 | UPG-0022 | 00b → ADR Candidate Generator | P3 | UPG-0007 | CHG-20260703-004 | COMPLETE |
| 5 | UPG-0023 | Human Approval Dashboard | P3 | UPG-0009, UPG-0003 | CHG-20260704-001 | COMPLETE |
| 5 | UPG-0024 | Pre-Release Evidence Package | P3 | — | CHG-20260706-002 | COMPLETE |
| 5 | UPG-0025 | Verification Packet for Reviewer Agent | P3 | UPG-0010 | CHG-20260706-003 | COMPLETE |
| 5 | UPG-0026 | Optional Branch Creation Helper | P2 | — | CHG-20260707-001 | COMPLETE |

### Unsequenced (discovery notes / governance follow-ups — not yet waved)
| Feature ID | Title | Priority | Depends on | Planned/active change | State |
|---|---|---|---|---|---|
| UPG-0027 | Lean Review Runner and Packet Architecture | P0 | — | CHG-20260629-002…005 | COMPLETE |
| UPG-0042 | Reduce Reviewer Packet Bloat for Large Stable Files | P2 | UPG-0027 ✓, UPG-0032 ✓ | — | PROPOSED |
| UPG-0043 | Split Monolithic Smoke Test File by Tool Area | P3 | — | — | PROPOSED |
| UPG-0044 | Refresh Reviewer Pipeline Architecture Documentation | P3 | — | CHG-20260712-001 | COMPLETE |
| UPG-0045 | Review Plan Preview — `codeos-reviewer plan` | P2 | — | — | PROPOSED |
| UPG-0046 | ReviewRun Structured Records | P2 | — | — | PROPOSED |
| UPG-0047 | Structured Finding Lifecycle | P2 | UPG-0046 | — | PROPOSED |
| UPG-0048 | Review Ledger Event Sourcing (Speculative — Long-Term) | P3 | UPG-0046, UPG-0047 | — | PROPOSED |
| UPG-0049 | External Review Policy Registry | P3 | — | — | PROPOSED |
| UPG-0051 | Multi-Feature Architecture Synthesis Gate | P1 | — | CHG-20260719-001 | COMPLETE |
| UPG-0052 | Implementation Profile Framework and Rust-First Default Profile | P2 | — | CHG-20260719-002 | COMPLETE |
| UPG-0053 | Implementation-Profile Scaffolding in dba-init.sh | P3 | UPG-0052 | CHG-20260719-003 | COMPLETE |
| UPG-0054 | Contract-to-Implementation Failure Boundary | P2 | — | CHG-20260719-004 | COMPLETE |
| UPG-0055 | Reviewer Support for the `architecture-synthesis` Stage ID | P3 | — | CHG-20260720-001 | COMPLETE |
| UPG-0056 | Optional Mechanism Status Convention | P2 | — | CHG-20260726-001 | COMPLETE |
| UPG-0057 | Controlled Plain English Writing Discipline | P3 | UPG-0056 | CHG-20260726-003, CHG-20260727-001 | COMPLETE |
| UPG-0058 | Cohort Logical Design — a Second Architecture Synthesis Output | P1 | UPG-0051 | CHG-20260726-002 | COMPLETE |
| UPG-0059 | Wave-Gated Batch Review for Multi-Feature Stage 1-3 Cohorts | P2 | UPG-0051, UPG-0058 | CHG-20260728-001 | COMPLETE |
| UPG-0028 | Reviewer self-reference recursion (scoping) | P2 | — | — | COMPLETE (backlog-only, no CHG — see backlog note) |
| UPG-0029 | Review artifact durability + packet naming policy | P2 | UPG-0001 | CHG-20260629-001 | COMPLETE |
| UPG-0030 | Lean Self-Development Review Profiles | P1 | — | CHG-20260629-001 | COMPLETE |
| UPG-0031 | Review script delta-mode fix + fail-closed guard | P1 | UPG-0027 | CHG-20260630-002/003 | COMPLETE |
| UPG-0033 | Review Script Instrumentation — Timing, Reconnect Count, Reasoning Effort | P1 | UPG-0027 | CHG-20260701-001 | COMPLETE |
| UPG-0034 | Rust Reviewer: Read-Only Invariant Check | P3 | UPG-0032 | CHG-20260702-006 | COMPLETE |
| UPG-0035 | Rust Reviewer: --sha-only exit-code parity | P3 | UPG-0032 | CHG-20260702-005 | COMPLETE |
| UPG-0036 | Stack Manifest & Drift Reconciliation Dogfooding | P2 | UPG-0017 ✓, UPG-0020 ✓ | CHG-20260705-001 | COMPLETE |
| UPG-0037 | Default Advisory Review Across the Full Downstream DBA Workflow | P1 | UPG-0003 ✓, UPG-0032 ✓, UPG-0014 ✓, UPG-0015 ✓ | CHG-20260705-002 | COMPLETE |
| UPG-0038 | Fix codeos-review.sh Binary Resolution for Symlinked Downstream Projects | P1 | UPG-0032 ✓ | CHG-20260707-002 | COMPLETE |
| UPG-0039 | Resolve the 00b Prompt-Filename Collision (Discovery -> 00a) | P3 | UPG-0007 ✓ | CHG-20260707-003 | COMPLETE |
| UPG-0040 | Fix Flaky config::tests Race on CODEOS_REVIEWER_PROVIDER Env Var | P2 | — | CHG-20260707-004 | COMPLETE |
| UPG-0041 | Reconcile feature-registry.yaml Schema vs Real-World Drift (FundFlow) | P2 | UPG-0009 ✓, UPG-0023 ✓ | CHG-20260707-005 | COMPLETE |
| UPG-0065 | Modular DBA Configuration Architecture | P1 | — | CHG-20260807-001, CHG-20260808-001, CHG-20260808-002, CHG-20260809-001 | IN_PROGRESS |

---

## Immediate next pickups

Waves 1–5 complete. Wave 5: UPG-0018 SUPERSEDED by UPG-0032; UPG-0019 COMPLETE 2026-07-06 (CHG-20260706-001); UPG-0020 COMPLETE 2026-07-03 (CHG-20260703-002); UPG-0021 COMPLETE 2026-07-03 (CHG-20260703-003); UPG-0022 COMPLETE 2026-07-03 (CHG-20260703-004); UPG-0023 COMPLETE 2026-07-06 (CHG-20260704-001); UPG-0024 COMPLETE 2026-07-06 (CHG-20260706-002); UPG-0025 COMPLETE 2026-07-07 (CHG-20260706-003); UPG-0026 COMPLETE 2026-07-07 (CHG-20260707-001). UPG-0028 COMPLETE 2026-07-07 (backlog-only, closed as substantially resolved by UPG-0027/UPG-0001, no CHG). UPG-0038 COMPLETE 2026-07-07 (CHG-20260707-002). UPG-0039 COMPLETE 2026-07-07 (CHG-20260707-003). UPG-0040 COMPLETE 2026-07-07 (CHG-20260707-004). UPG-0041 COMPLETE 2026-07-10 (CHG-20260707-005).

Each is taken as its own self-dev change (a fresh `CHG-*`) via the 4-step loop, with human
approval at every gate.

---

## Guardrails — "do NOT do yet" (from `../backlog/features.md`)

- No full OAP integration doctrine as a core Codeos change.
- No mandatory PRs at every stage.
- No large execution packet that duplicates intent / contract / event schema.
- Stack manifest must never be manually-maintained-only.
- Verification mode must never edit files.
- 00b output is never an approved feature list or architecture.
- **Do not change Codeos non-negotiable rules** until these smaller upgrades are piloted.
