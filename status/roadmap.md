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
| 4 | UPG-0032 | Rust Reviewer Engine with Multi-Provider Support | P2 | UPG-0027 ✓ | — | PROPOSED |
| 4 | UPG-0014 | Reviewer Agent with Full Diff Access | P2 | UPG-0032 | — | PROPOSED |
| 4 | UPG-0015 | Bind stage approval to reviewed provenance | P1 | UPG-0032 | — | PROPOSED |
| 4 | UPG-0016 | Branch / PR / CI Workflow Profiles | P1 | — | — | PROPOSED |
| 4 | UPG-0017 | Stack Manifest with Automatic Reconciliation | P2 | — | — | PROPOSED |

### Wave 5 — Advanced automation & generators  *(mostly P3 + the big P2 rewrite)*
| Wave | Feature ID | Title | Priority | Depends on | Planned/active change | State |
|---|---|---|---|---|---|---|
| 5 | UPG-0018 | Typed reviewer engine (replace Bash pilot) | P2 | — | — | SUPERSEDED by UPG-0032 |
| 5 | UPG-0019 | CI Integration Profile | P3 | UPG-0017 | — | PROPOSED |
| 5 | UPG-0020 | Stack / Config Drift Detector | P3 | UPG-0017 | — | PROPOSED |
| 5 | UPG-0021 | Stage Report Generator | P3 | UPG-0004 | — | PROPOSED |
| 5 | UPG-0022 | 00b → ADR Candidate Generator | P3 | UPG-0007 | — | PROPOSED |
| 5 | UPG-0023 | Human Approval Dashboard | P3 | UPG-0009, UPG-0003 | — | PROPOSED |
| 5 | UPG-0024 | Pre-Release Evidence Package | P3 | — | — | PROPOSED |
| 5 | UPG-0025 | Verification Packet for Reviewer Agent | P3 | UPG-0010 | — | PROPOSED |
| 5 | UPG-0026 | Optional Branch Creation Helper | P2 | — | — | PROPOSED |

### Unsequenced (discovery notes / governance follow-ups — not yet waved)
| Feature ID | Title | Priority | Depends on | Planned/active change | State |
|---|---|---|---|---|---|
| UPG-0027 | Lean Review Runner and Packet Architecture | P0 | — | CHG-20260629-002…005 | COMPLETE |
| UPG-0028 | Reviewer self-reference recursion (scoping) | P2 | — | — | PROPOSED |
| UPG-0029 | Review artifact durability + packet naming policy | P2 | UPG-0001 | CHG-20260629-001 | COMPLETE |
| UPG-0030 | Lean Self-Development Review Profiles | P1 | — | CHG-20260629-001 | COMPLETE |
| UPG-0031 | Review script delta-mode fix + fail-closed guard | P1 | UPG-0027 | CHG-20260630-002/003 | COMPLETE |
| UPG-0033 | Review Script Instrumentation — Timing, Reconnect Count, Reasoning Effort | P1 | UPG-0027 | CHG-20260701-001 | COMPLETE |

---

## Immediate next pickups

Wave 1 complete. Wave 2 complete. Wave 3 complete. Next: Wave 4 — UPG-0032 Rust Reviewer Engine (PROPOSED, no active change yet).

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
