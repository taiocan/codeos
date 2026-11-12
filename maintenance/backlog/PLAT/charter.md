# PLAT — Platform Assurance and Live Health Charter

- **Status:** APPROVED 2026-10-20
- **Scope:** CodeOS-wide; replay on JamFree and EvidenceAtlas

Planning context only. Git and governing policies remain authoritative.

## Problem and value

CodeOS reports that verification passed, but not what proves PostgreSQL, Rust, Svelte, and the
whole product path individually. Making that evidence visible in the existing status interface
should speed diagnosis, expose missing coverage, and prevent “the backend responded” from being
mistaken for “the product works.”

## Proposal

**Platform Assurance** is a read-only projection of current Development Evidence for each
Platform Baseline tier and the integrated product. It has no workflow authority and makes no claim
about current deployment availability.

`status --json` includes additive `platform_assurance` rows containing `area`, `status`, `subjects`,
and `basis`. Standard `status` shows a compact four-area summary; `status --platform` shows its
evidence and issues. There is no aggregate score.

| Area | Evidence |
|---|---|
| Persistence | `data_integrity` for persistence changes |
| Backend | smoke and behavior evidence for backend changes |
| GUI | Playwright evidence for GUI changes or visible outcomes |
| Integrated product | smoke, product journey, and applicable tier checks |

Statuses are `PASS`, `BLOCKED`, `STALE`, `UNOBSERVED`, and `NOT_APPLICABLE`. Every positive result
names its applicable features and exact evidence; absence never becomes `PASS`.

## Boundaries and value gate

- Extend the existing status engine; add no state, score, background service, monitoring database,
  default network access, or changed workflow behavior.
- Keep Platform Assurance separate from **Live Health** (time- and environment-bound availability)
  and **audit linkage** (GUI action to backend outcome and database commit).
- Defer Live Health until a real deployment has an authoritative source. Defer audit linkage until
  an approved project need and privacy boundary exist.
- Stop after replay if the projection merely repeats Verification status or does not improve a
  real diagnosis.

Work orders: [README](README.md).
