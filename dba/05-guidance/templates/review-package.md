# Inline Review Package

A Review Package is an inline convenience view, never a file, approval, or historical record.
Include only evidence needed for the current review boundary.

```text
## Review Package: [feature or architecture scope] — [boundary]

Decision being prepared: [Specification Package approval | Architecture Scope approval |
final acceptance]
Authoritative inputs: [paths]
Relevant implementation/test/runtime evidence: [paths, commands, or concise results]
Known tensions or non-aligned findings: [items or none]
Suggested areas to examine: [specific questions]
```

For a Specification Package, include the full Intent, Contract, and Event Schema. For final
acceptance, include the reconciliation and replay result. Regenerate the view from current
artifacts; durable human decisions are recorded only as required by the selected review policy.
