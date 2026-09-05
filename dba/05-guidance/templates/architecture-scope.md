---
artifact_type: architecture
features: [F-0001, F-0002]
approval: null
reader_model:  # stable-topic | known-to-new | whole-before-parts | preview-then-traverse
---

# Architecture Scope: [scope-id]

<!--
This artifact type's governance is configurable in .codeos/00-project/codeos.yaml (default:
governed). Whether or not it is currently governed, doctrine's decision-governance guarantee for
consequential architecture — explicit human approval — still applies and is recorded above via
`approval`, independent of that configuration.
-->

## Summary
- [key message]
- [key message]

Oversimplification risk: this summary can omit nuance the full artifact carries. Read the
relevant section directly before relying on the summary alone for a consequential decision.

<!--
One authoritative architecture artifact for a project-level structural scope.
Canonical path: .codeos/02-architecture/scopes/<scope-id>.md

The filename owns scope identity; front matter owns membership and approval. Git owns history.
Replace this guidance with only the architecture-significant decisions needed by the scope. The
policy's synthesis questions are reasoning prompts, not required headings; do not add empty sections
or N/A entries.
-->

## Platform Baseline Resolution

[State how this scope resolves the Platform Baseline's persistence/backend/webapp/runtime tiers for
the features in scope, and identify where integrated verification crosses the Postgres/Rust/Svelte
boundaries — e.g. which component owns the Playwright acceptance surface for a given journey.]

[Record the minimum project-level architecture, rationale or authority, component responsibilities,
and any feature-specific obligations that materially constrain implementation. Keep unresolved
questions visible and retain `approval: null` until explicit human approval.]
