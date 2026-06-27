# Codeos Implementation Roadmap

> **This is mutable planning state, not Codeos doctrine.** `backlog/features.md` remains the
> stable backlog catalog (the briefs and their P0–P3 priorities). This file sequences that
> catalog into dependency-aware waves and tracks current reality. **Each item still requires
> its own self-dev change and explicit human approval before implementation** — appearing in a
> wave here is *not* authorization. Live per-change status lives in
> `status/self-development.md`; per-change detail in `changes/[id].md`.

Each backlog item is implemented as a self-dev change (`0004`, `0005`, …) through the 4-step
loop in `prompts/codeos-self-dev.md`.

---

## Current State (completed / piloted — not in a wave)

| Item | State | Notes |
|---|---|---|
| `0001-claude-split` | DONE | Split downstream doctrine (`dba-system.md`) from the self-dev guide (`CLAUDE.md`). |
| `0002-doc-consistency-rename` | DONE | Renamed doctrine-attribution refs in docs after the split. |
| `reviewer-decision-brief` (#1) | PILOTED | Advisory Bash reviewer: `scripts/codeos-review.sh` + `docs/reviewer-pipeline.md` + `docs/reviewer-artifact-schemas.md`. Manual, read-only, non-gatekeeping; no Claude Code hooks wired. |

(`reviewer-quality-scale` is **partially piloted** — the reviewer emits an evidence grade but
the full scale is not realized; the remaining work is scheduled once in Wave 1, not here.)

---

## Waves (dependency-aware)

Sequencing principles: value first (P0 → P3); respect hard dependencies; build on the existing
reviewer pilot; preserve "advisory-not-autonomous" and "evidence-integrity"; take cheap
pilot-adjacent wins early.

### Wave 1 — Transparency & state  *(P0, no deps, highest value / lowest risk)*
| Item | Pri | Depends on | Notes |
|---|---|---|---|
| `stage-4-6-reports` | P0 | — | Structured Implementation/Test/Runtime reports. **Next pickup.** |
| `current-verified-state` | P0 | — | Auto-generated session-start state snapshot. |
| `reviewer-quality-scale` | P2 | reviewer pilot ✓ | **Partially piloted** — formalize the A–E evidence scale + reviewer consistency (pilot already emits a grade). |

### Wave 2 — Discovery & registry
| Item | Pri | Depends on | Notes |
|---|---|---|---|
| `solution-discovery-00b` | P0 | — | Non-authoritative pre-Stage-1 topology discovery. |
| `config-discovery` | P2 | solution-discovery-00b | Config surface inside expanded 00b. |
| `feature-registry` | P2 | — | Feature ID ↔ branch ↔ stage ↔ PR binding; unlocks Wave 4–5 automation. |

### Wave 3 — Evidence discipline & gates  *(P1)*
| Item | Pri | Depends on | Notes |
|---|---|---|---|
| `verify-only-mode` | P1 | — | Strict read-only verification (never edits evidence). |
| `readiness-checklist` | P1 | — | Lightweight operational merge/release gate. |
| `repair-before-next-feature` | P1 | — | Unresolved work blocks new behavioral features (human override allowed). |
| `stage-4-activation-card` | P1 | — | Small activation card; references, never restates, approved artifacts. |

### Wave 4 — Reviewer hardening & delivery
| Item | Pri | Depends on | Notes |
|---|---|---|---|
| `reviewer-full-diff` | P2 | reviewer-decision-brief ✓ | Reviewer inspects filtered full diff, not only artifacts. |
| `reviewer-decision-integrity` | P1 | reviewer pilot ✓ | Bind approval to reviewed provenance (commit + diff_hash + workspace). |
| `workflow-profiles` | P1 | — | Optional branch/PR/CI profiles (simple / one-branch / split-PR). |
| `stack-manifest` | P2 | — | Two-layer stack record with diff-triggered reconciliation. |

### Wave 5 — Advanced automation & generators  *(mostly P3 + the big P1 rewrite)*
| Item | Pri | Depends on | Notes |
|---|---|---|---|
| `reviewer-engine-v1` | P1 | reviewer pilot proven | Typed (Rust/Python) reviewer engine replacing the Bash pilot — only after the pilot is proven. |
| `ci-profile` | P3 | stack-manifest | Map Codeos evidence types onto CI checks. |
| `stack-drift-detector` | P3 | stack-manifest | Block release on unreconciled dependency/config drift. |
| `stage-report-generator` | P3 | stage-4-6-reports | Auto-fill Stage 4–6 report skeletons. |
| `00b-adr-generator` | P3 | solution-discovery-00b | Turn 00b architecture risks into routable ADR candidates. |
| `approval-dashboard` | P3 | feature-registry, reviewer pilot | Generated cross-feature stage/review/blocker overview. |
| `release-evidence-package` | P3 | — | Generated pre-release evidence bundle from existing artifacts. |
| `reviewer-verification-packet` | P3 | verify-only-mode | Reviewer can request read-only verification when confidence is low. |
| `branch-helper` | P2 | — | Optional branch-naming helper; doc-first. |

---

## Immediate next pickups

1. `stage-4-6-reports` (Wave 1, P0, no deps) — foundational for later generators/dashboards.
2. `current-verified-state` (Wave 1, P0, no deps).

Each is taken as its own self-dev change (`0004`, `0005`, …) via the 4-step loop, with human
approval at every gate.

---

## Guardrails — "do NOT do yet" (from `backlog/features.md`)

- No full OAP integration doctrine as a core Codeos change.
- No mandatory PRs at every stage.
- No large execution packet that duplicates intent / contract / event schema.
- Stack manifest must never be manually-maintained-only.
- Verification mode must never edit files.
- 00b output is never an approved feature list or architecture.
- **Do not change Codeos non-negotiable rules** until these smaller upgrades are piloted.
