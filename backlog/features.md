# Codeos Future Upgrade Backlog — Index & Feature-ID Map

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

> Each upgrade lives in its own file as a lightweight *upgrade-brief*. **This file is the
> authoritative `UPG-#### → file` map** (one row per backlog feature). Dependency-aware
> sequencing into waves lives in [`../status/roadmap.md`](../status/roadmap.md); live per-change
> status lives in [`../status/self-development.md`](../status/self-development.md). Stable feature
> IDs follow the [`UPG-0001`](UPG-0001-feature-thread-traceability.md) Feature Thread model.

---

## Feature-ID Map (authoritative — one `UPG-####` per file)

| Feature ID | File | Title | Pri | Status |
|---|---|---|---|---|
| UPG-0001 | [UPG-0001-feature-thread-traceability.md](UPG-0001-feature-thread-traceability.md) | Feature Thread Traceability & Stable IDs | P0 | COMPLETE |
| UPG-0002 | [UPG-0002-doc-consistency-doctrine-rename.md](UPG-0002-doc-consistency-doctrine-rename.md) | doc-consistency pass after CLAUDE.md→dba-system.md split | — | COMPLETE |
| UPG-0003 | [UPG-0003-reviewer-decision-brief.md](UPG-0003-reviewer-decision-brief.md) | Reviewer Agent for Stage-Gate Decision Briefs | P0 | PILOTED |
| UPG-0004 | [UPG-0004-stage-4-6-reports.md](UPG-0004-stage-4-6-reports.md) | Structured Stage 4–6 Reports | P0 | PROPOSED |
| UPG-0005 | [UPG-0005-current-verified-state.md](UPG-0005-current-verified-state.md) | Current Verified State Block | P0 | COMPLETE |
| UPG-0006 | [UPG-0006-reviewer-quality-scale.md](UPG-0006-reviewer-quality-scale.md) | Reviewer Summary Quality Scale | P2 | PROPOSED |
| UPG-0007 | [UPG-0007-solution-discovery-00b.md](UPG-0007-solution-discovery-00b.md) | Expanded 00b Solution Discovery / Feature Topology | P0 | PROPOSED |
| UPG-0008 | [UPG-0008-config-discovery.md](UPG-0008-config-discovery.md) | Configuration Discovery & Schema Track | P2 | PROPOSED |
| UPG-0009 | [UPG-0009-feature-registry.md](UPG-0009-feature-registry.md) | Feature Registry / Branch Binding | P2 | PROPOSED |
| UPG-0010 | [UPG-0010-verify-only-mode.md](UPG-0010-verify-only-mode.md) | Verification-Only Mode | P1 | PROPOSED |
| UPG-0011 | [UPG-0011-readiness-checklist.md](UPG-0011-readiness-checklist.md) | Lightweight PR / Pre-Release Readiness Checklist | P1 | PROPOSED |
| UPG-0012 | [UPG-0012-repair-before-next-feature.md](UPG-0012-repair-before-next-feature.md) | Repair-Before-Next-Feature Workflow Gate | P1 | PROPOSED |
| UPG-0013 | [UPG-0013-stage-4-activation-card.md](UPG-0013-stage-4-activation-card.md) | Stage 4 Activation Card | P1 | PROPOSED |
| UPG-0014 | [UPG-0014-reviewer-full-diff.md](UPG-0014-reviewer-full-diff.md) | Reviewer Agent with Full Diff Access | P2 | PROPOSED |
| UPG-0015 | [UPG-0015-reviewer-decision-integrity.md](UPG-0015-reviewer-decision-integrity.md) | Bind stage approval to reviewed provenance | P1 | PROPOSED |
| UPG-0016 | [UPG-0016-workflow-profiles.md](UPG-0016-workflow-profiles.md) | Branch / PR / CI Workflow Profiles | P1 | PROPOSED |
| UPG-0017 | [UPG-0017-stack-manifest.md](UPG-0017-stack-manifest.md) | Stack Manifest with Automatic Reconciliation | P2 | PROPOSED |
| UPG-0018 | [UPG-0018-reviewer-engine-v1.md](UPG-0018-reviewer-engine-v1.md) | Typed reviewer engine to replace the Bash pilot | P2 | SUPERSEDED by UPG-0032 |
| UPG-0032 | [UPG-0032-rust-reviewer-engine-multi-provider.md](UPG-0032-rust-reviewer-engine-multi-provider.md) | Rust Reviewer Engine with Multi-Provider Support | P2 | PROPOSED |
| UPG-0019 | [UPG-0019-ci-profile.md](UPG-0019-ci-profile.md) | CI Integration Profile | P3 | PROPOSED |
| UPG-0020 | [UPG-0020-stack-drift-detector.md](UPG-0020-stack-drift-detector.md) | Stack / Config Drift Detector | P3 | PROPOSED |
| UPG-0021 | [UPG-0021-stage-report-generator.md](UPG-0021-stage-report-generator.md) | Stage Report Generator | P3 | PROPOSED |
| UPG-0022 | [UPG-0022-00b-adr-generator.md](UPG-0022-00b-adr-generator.md) | 00b → ADR Candidate Generator | P3 | PROPOSED |
| UPG-0023 | [UPG-0023-approval-dashboard.md](UPG-0023-approval-dashboard.md) | Human Approval Dashboard | P3 | PROPOSED |
| UPG-0024 | [UPG-0024-release-evidence-package.md](UPG-0024-release-evidence-package.md) | Pre-Release Evidence Package | P3 | PROPOSED |
| UPG-0025 | [UPG-0025-reviewer-verification-packet.md](UPG-0025-reviewer-verification-packet.md) | Verification Packet for Reviewer Agent | P3 | PROPOSED |
| UPG-0026 | [UPG-0026-branch-helper.md](UPG-0026-branch-helper.md) | Optional Branch Creation Helper | P2 | PROPOSED |
| UPG-0027 | [UPG-0027-lean-review-runner-packet-architecture](UPG-0027-replacing-review-scripts.md) | Lean Review Runner and Packet Architecture | P0 | COMPLETE |
| UPG-0028 | [UPG-0028-reviewer-self-reference-recursion.md](UPG-0028-reviewer-self-reference-recursion.md) | Reviewer self-reference recursion (scoping) | P2 | PROPOSED |
| UPG-0029 | [UPG-0029-review-naming-and-thread-tooling.md](UPG-0029-review-naming-and-thread-tooling.md) | Review artifact durability + packet naming policy | P2 | COMPLETE |
| UPG-0030 | [UPG-0030-lean-self-development-review-profiles.md](UPG-0030-lean-self-development-review-profiles.md) | Lean Self-Development Review Profiles | P1 | COMPLETE |
| UPG-0031 | [UPG-0031-review-delta-mode-fix.md](UPG-0031-review-delta-mode-fix.md) | Review script delta-mode working-tree fix + fail-closed EMPTY_PACKET guard | P1 | COMPLETE |
| UPG-0033 | [UPG-0033-review-script-instrumentation.md](UPG-0033-review-script-instrumentation.md) | Review Script Instrumentation — Timing, Reconnect Count, Reasoning Effort | P1 | IN_PROGRESS |

> Priority `—` = no formal P-rank (a completed cleanup, or a discovery note). Feature IDs are
> assigned once and never reused. `UPG-0002` and `UPG-0003` are historical/piloted work given IDs
> during this migration **without** false retroactive sequencing — see their `status`.
> `UPG-0000` is **reserved for documentation examples** and is never assigned to a real feature.

---

## Recommended implementation order

Sequencing by `UPG-####` is maintained in [`../status/roadmap.md`](../status/roadmap.md)
(dependency-aware waves + current state). This file does not duplicate the wave plan; the roadmap
is authoritative for *order*, this map is authoritative for *identity*.

---

## Future candidates (no UPG-#### assigned yet)

| Candidate | Description |
|---|---|
| review-runner diagnostics | A `diagnose` subcommand for `codeos-review.sh` that independently checks: Codex CLI availability, trivial session creation, session ID parsing, minimal packet review, assessment write, and review-log append. Open only if API/session errors recur during a real review; must not change review semantics, packet construction, scope rules, or reviewer policy. |

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
