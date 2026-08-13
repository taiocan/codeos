# Codebase Digest: [PROJECT_NAME]

<!--
PURPOSE:
A structural orientation document for Claude and the human team.
This file describes where risk lives in the current implementation.
It is NOT a design document. It is NOT an intent artifact. It makes no claims
about what the system should do — only about where modification is dangerous.

WHEN TO PRODUCE:
Complete this after your first working implementation is in place.
Update it after any behavioral refinement, architecture change, or ordinary maintenance that touches hubs, god functions,
or module dependency relationships (see Maintenance Trigger below).

HOW TO POPULATE:
Use openlore, a static analysis tool, or manual inspection. The DBA pipeline
is tool-agnostic. Any method that produces accurate structural facts is acceptable.

KEY PRINCIPLE:
Critical Hubs and God Functions are structural signals, not authoritative risk
rankings. A function with low fan-in and low fan-out may still be extremely
dangerous (authorization policy, cache invalidation, migration paths, transaction
boundaries). The Known High-Risk Zones section exists precisely for this reason.
Never let metrics override human architectural knowledge.

MAINTENANCE TRIGGER:
Any governed refinement, architecture change, or ordinary maintenance task that modifies a
listed hub, god function, or module dependency relationship must either:
  (a) update this file and refresh last_verified_commit, or
  (b) explicitly state in the refinement log why no digest update is required.

RETIREMENT RULE:
Entries for functions or modules that no longer exist in the verified commit must
be REMOVED — not commented out, not marked historical. Accumulated archaeology
is indistinguishable from current fact.
-->

---

## Overview

| Metric | Value |
|---|---|
| Modules | [N] |
| Entry points (no internal callers) | [N] |
| Critical hubs (high fan-in — widest blast radius) | [N] |
| God functions (high fan-out — coordinate many callers) | [N] |

---

## Entry Points

Functions with no internal callers. Natural starting points when tracing a feature.

| Function | Module / File | Notes |
|---|---|---|
| [function_name] | [path] | [e.g., "CLI entry for cmd_export"] |

---

## Critical Hubs

High fan-in functions. Modifying these has the widest blast radius.

**Before modifying any listed function:** behavioral changes require explicit contract
coverage. Non-behavioral changes (renaming, extracting helpers, testability) are exempt.

| Function | Module / File | Fan-in (approx) | Risk note |
|---|---|---|---|
| [function_name] | [path] | [N] | [why this is fragile] |

---

## God Functions

High fan-out orchestrators. These coordinate many downstream calls.

**Rule:** Extend a God Function only when the new behavior has an approved contract
clause. Adding behavior to a God Function without a contract clause is a DBA violation.

| Function | Module / File | Fan-out (approx) | What it orchestrates |
|---|---|---|---|
| [function_name] | [path] | [N] | [brief description] |

---

## Module Cluster Map

| Module | Role | What depends on it |
|---|---|---|
| [module_name] | [API layer / entry layer / internal / infrastructure] | [consumer list or "—"] |

---

## Known High-Risk Zones

Free-text. Human-authored knowledge that metrics cannot capture.

**Important:** This section supersedes the structural metrics above when behavioral
impact is higher. Low fan-in does not mean low risk. Authorization policy, cache
invalidation paths, migration code, and transaction boundaries often have low
structural metrics yet are the most dangerous areas to modify.

- [e.g., "auth/policy.rs: central authorization check. Any change requires contract
  coverage even though fan-in appears low — callers are indirect via middleware."]
- [e.g., "modules/project_schema: vocabulary resolution boundary. The Representation Ban
  invariant means changes here cascade silently if not verified at all consumers."]

---

<!-- METADATA -->
```yaml
last_verified_commit: [git sha — run 'git rev-parse HEAD' to get current value]
verification_method: [openlore | manual | static-analysis-tool]
reviewed_by: [human name]
updated_after_refinement: [refine_id or N/A]
```
