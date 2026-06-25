# Codeos Future Upgrade Backlog — Index

## Backlog thesis

Codeos already has the stronger behavioral correctness machinery: intent, contract, event
schema, implementation, tests, runtime execution, reconciliation, replay, and targeted
refinement.

The useful future upgrades are not "more OAP" and not more ceremony. The useful upgrades are
operational supports around Codeos:

1. reduce human review load;
2. make Stage 4–6 execution transparent;
3. prevent stale context;
4. support feature topology discovery before Stage 1;
5. improve branch/PR/reviewer/CI discipline where useful;
6. preserve stack and configuration knowledge without stale manual documentation;
7. add read-only verification so evidence is not blurred by helpful edits.

The goal is to speed up Codeos development without weakening DBA advantages.

> Each upgrade now lives in its own file in this directory, as a lightweight
> *upgrade-brief* (Problem / Upgrade / Scope / Proposed artifact / Design notes / Value /
> Risk / Guardrail / DBA-philosophy note). This file is the index.

---

## Upgrade index

### P0 — Highest-priority upgrades

| # | Upgrade | One-line value |
|---|---|---|
| 1 | [reviewer-decision-brief](reviewer-decision-brief.md) | Independent reviewer compresses each stage artifact into a decision brief — saves human time, preserves human approval. |
| 2 | [stage-4-6-reports](stage-4-6-reports.md) | Structured implementation/test/runtime reports make hidden Stage 4–6 work transparent. |
| 3 | [current-verified-state](current-verified-state.md) | Auto-generated session-start state snapshot prevents stale-branch / stale-artifact mistakes. |
| 4 | [solution-discovery-00b](solution-discovery-00b.md) | Non-authoritative pre-Stage-1 feature-topology discovery without approving architecture early. |

### P1 — High-value workflow upgrades

| # | Upgrade | One-line value |
|---|---|---|
| 5 | [stage-4-activation-card](stage-4-activation-card.md) | Small activation-metadata card (branch, scope, reporting) without duplicating approved artifacts. |
| 6 | [workflow-profiles](workflow-profiles.md) | Optional branch/PR/CI profiles (simple / one-branch / split-PR), not a mandatory policy. |
| 7 | [verify-only-mode](verify-only-mode.md) | Strict read-only verification so checks never silently edit the evidence. |
| 8 | [readiness-checklist](readiness-checklist.md) | Lightweight operational merge/release gate distinct from Stage 7. |
| 9 | [repair-before-next-feature](repair-before-next-feature.md) | Unresolved work blocks new behavioral features (human override allowed). |

### P2 — Support and maintenance upgrades

| # | Upgrade | One-line value |
|---|---|---|
| 10 | [stack-manifest](stack-manifest.md) | Two-layer stack record with diff-triggered reconciliation (no stale manual docs). |
| 11 | [config-discovery](config-discovery.md) | Surface configuration needs in 00b; formalize only when behavior depends on them. |
| 12 | [reviewer-full-diff](reviewer-full-diff.md) | Reviewer inspects the filtered full diff, not only stage artifacts. |
| 13 | [reviewer-quality-scale](reviewer-quality-scale.md) | Reviewer labels evidence quality (A–E) so it never sounds more certain than the evidence. |
| 14 | [feature-registry](feature-registry.md) | Bind feature ID ↔ branch ↔ stage ↔ PR ↔ status; warn on git/filesystem disagreement. |
| 15 | [branch-helper](branch-helper.md) | Optional branch-naming helper/convention (doc-first). |

### P3 — Optional later upgrades

| # | Upgrade | One-line value |
|---|---|---|
| 16 | [reviewer-verification-packet](reviewer-verification-packet.md) | Reviewer can request read-only verification when confidence is low. |
| 17 | [ci-profile](ci-profile.md) | Map Codeos evidence types onto CI checks. |
| 18 | [release-evidence-package](release-evidence-package.md) | Generated pre-release evidence bundle from existing artifacts. |
| 19 | [stage-report-generator](stage-report-generator.md) | Auto-fill Stage 4–6 report skeletons from git/test/runtime inputs. |
| 20 | [stack-drift-detector](stack-drift-detector.md) | Block release when dependency/config changed without stack reconciliation. |
| 21 | [00b-adr-generator](00b-adr-generator.md) | Turn 00b architecture risks into routable ADR candidates. |
| 22 | [approval-dashboard](approval-dashboard.md) | Generated cross-feature overview of stage/review/blocker status. |

---

## Recommended implementation order

**Sprint 1 — Review burden and transparency** (highest value, lowest risk)
1. reviewer-decision-brief
2. stage-4-6-reports
3. current-verified-state

**Sprint 2 — Discovery and state discipline**
4. solution-discovery-00b
5. config-discovery (inside 00b)
6. feature-registry

**Sprint 3 — Clean evidence and readiness**
7. verify-only-mode
8. readiness-checklist
9. repair-before-next-feature

**Sprint 4 — Optional delivery maturity**
10. workflow-profiles
11. reviewer-full-diff
12. stack-manifest

**Sprint 5 — Advanced automation**
13. ci-profile
14. release-evidence-package
15. stack-drift-detector
16. approval-dashboard

> Current status: the first upgrade (`reviewer-decision-brief`) is being prototyped as a
> manual advisory Codex reviewer — see `docs/reviewer-pipeline.md`.

---

## What not to do yet

- Do not add full OAP integration doctrine as a core Codeos change.
- Do not make PRs mandatory at every stage.
- Do not make a large execution packet that duplicates intent, contract, and event schema.
- Do not make the stack manifest manually maintained only.
- Do not allow verification mode to edit files.
- Do not treat 00b output as approved feature list or architecture.
- Do not change Codeos non-negotiable rules until these smaller upgrades are piloted.

---

## Final product direction

The future Codeos roadmap should not be "OAP inside Codeos." It should be:

```text
Codeos Core:
- Intent / Contract / Event Schema / Implementation / Tests / Runtime / Reconciliation / Replay / Refinement

Codeos Operational Layer:
- Current verified state
- Stage 4–6 structured reports
- Reviewer decision briefs
- Verification-only mode
- Readiness checklist
- Feature/branch registry
- Optional PR profiles

Codeos Strategic Layer:
- Expanded 00b solution discovery
- Candidate feature topology
- Candidate event families
- Configuration discovery
- ADR candidates
- Architecture risk detection
```

That preserves the DBA core while adding speed, transparency, and quality.
