---
feature_id: UPG-0080
slug: doctrine-v5-full-stack-verified-legible
title: "DBA Doctrine v5: Full-Stack, Verified, Legible"
status: COMPLETE
priority: P1
depends_on: []
related_features: [UPG-0076, UPG-0077, UPG-0079]
supersedes: []
superseded_by: []
---

# Upgrade: DBA Doctrine v5 — Full-Stack, Verified, Legible

## Problem

A downstream project accumulated many backend features with no GUI in parallel, then drafted the
whole GUI as one late pass; the user first saw the result as a finished surface containing
elements they didn't recognize from backend reports alone. Doctrine v4 was deliberately
stack-agnostic and GUI verification (`svelte-gui-verification.md`) was advisory-when-selected, so
nothing forced architecture to resolve a GUI tier or forced a human to see a feature before
expensive verification was sunk into it. Separately, long artifacts (Charter, Intent, Contract,
Architecture Scope) were reviewed under time pressure and mistakes were missed; the reader-oriented
writing models from UPG-0079 were mechanically wired into delivery paths but produced no artifact
structure the user could notice. Governance was implicit and fixed in prose, with no single place
to see or configure which artifact types carried full process weight.

## Upgrade

Doctrine v5 (additive to v4): a Platform Baseline (PostgreSQL + Rust + Svelte + Docker) every
solution resolves by default unless the Charter records an exception; Feature Impact Accounting
(a feature records changed/unchanged per tier, with reason, rather than being forced to touch
every tier); an Early Development Preview after basic integration smoke and before the full
verification loop, with a feedback-routing rule so preview reactions can't silently redefine
approved behavior; Codeos Mechanics — a fixed, non-project-configurable set of delivery,
validation, and communication obligations (vertical-slice delivery, integration smoke, behavioral
and repeatability verification, mandatory Playwright and human-UX validation for any GUI-visible
outcome, summary-first artifacts, reader-oriented writing, terminology consistency) owned by a new
Codeos Mechanics policy; and artifact communication/governance transparency — every substantial
artifact carries a Summary, an Oversimplification Risk note, and a declared reader-model
progression, with one project configuration (`codeos.yaml`) naming the Platform Baseline, the
(display-only) fixed mechanics, and each artifact type's governed state, with Charter/Intent/
Contract/Event Schema permanently locked governed.

Architecture Synthesis policy v4 adds Platform Baseline resolution and verification-boundary
identification. Twelve templates were restructured to a single frontmatter shape (`artifact_type`
plus that type's approval-bearing metadata; `governed` is not a supported field — `codeos.yaml` is
the sole governance authority). Stage 1/2/4/5 prompts apply the four validation questions
(Behavior/Repeatability/Browser/Preview), Feature Impact Accounting, the Early Development Preview
and its routing rule, and the fixed Codeos Mechanics. The reference Platform Baseline skeleton
(axum + sqlx backend, SvelteKit web, one three-service `docker-compose.yml`) ships under
`dba/04-tools/initializer/skeleton/`; `dba-init.sh` scaffolds it automatically whenever the active
configuration carries `codeos_mechanics_policy` — detected from `dba-system.md`'s active pointer,
never a hardcoded version number.

## Scope

**In scope:** doctrine v5; the Codeos Mechanics policy; Architecture Synthesis v4; `codeos.yaml`
and its validator; single-shape artifact frontmatter with mandatory Summary/reader-model on
Intent, Contract, Charter, Architecture Scope, and Module Design Note; the reference Platform
Baseline skeleton and its initializer wiring; nine new terminology entries including the artifact-
governance-vs-decision-governance distinction; the DBA-4/DBA-5 adoption rule (no automatic
migration).

**Out of scope:** retrofitting any already-adopted DBA-4 project; new mechanics beyond the fixed
set (accessibility, security, performance — the Validation group is structured to accept them
later as genuinely project-selectable entries); changing Implementation Profile (kept exactly as
scoped to implementation-language choice, per the boundary UPG-0076 established); the toolkit's
own Component Boundary Contract (`component_question`/`out_of_scope`), which is unrelated to the
new downstream artifact frontmatter contract.

## Acceptance

- `bash dba/04-tools/tests/run.sh` passes with `DBA-5.yaml` active, including the new
  `project-config-contract.sh` suite and the initializer's Platform-Baseline-gating tests (both
  the DBA-5-shaped and DBA-4-shaped fixtures, each verified to actually fail when sabotaged).
- `project-config-contract.sh` fails closed on a downgraded core-four artifact type, an unlisted
  artifact type, or any attempt to change a fixed Codeos Mechanic.
- The reference skeleton runs for real: `docker compose up --build` brings up real Postgres, a
  real Rust backend, and a real SvelteKit app; the shipped Playwright test passes against it with
  no tier mocked.
- One real vertical pilot feature (a guestbook: leave and view notes) ran the full loop against
  the reference skeleton in a project initialized under `DBA-5.yaml` — Charter through Contract
  with the Validation Questions and Feature Impact Accounting filled in, Stage 4 implementation
  across all three tiers, a real Early Development Preview with one UX-only feedback item applied
  directly and one behavior-changing item correctly withheld from implementation and routed to a
  future Specification Package revision, Rust integration tests proving behavior and repeatability
  against real Postgres, and Playwright journeys proving the browser path — with the resulting
  reconciliation honestly recording three uncovered items (byte-identical text preservation under
  non-edge whitespace, the 1s latency requirement, and cross-restart durability) rather than
  overclaiming coverage.
- `dba-config-boundaries.sh` passed on `DBA-5.yaml` before the active pointer moved;
  `layout-contract.sh`'s hardcoded active-config references and independent `canonical_paths`
  guard were updated to match.

## Outcome

Completed and activated on 2026-09-05. `DBA-5.yaml` is `dba-system.md`'s active configuration.
Two real bugs were found and fixed during verification rather than assumed away, at two different
points: while restructuring templates, the reviewer engine's Architecture Scope parser initially
rejected the new `artifact_type`/`reader_model` fields, fixed as optional (`#[serde(default)]`)
fields so an already-adopted DBA-4 Architecture Scope still parses unchanged; and during the pilot
itself, the first Playwright journey failed because `adapter-node` does not trust the `Host` header
for its CSRF `Origin` check — the reference skeleton's `docker-compose.yml` now sets `ORIGIN`
explicitly for the `web` service (without it, any project's first SvelteKit form action returns
403).

DBA-4 remains valid and supported for any project that adopted it; DBA-5 is the default for newly
initialized projects and for a project that explicitly elects to upgrade. No automatic artifact
migration occurs.
