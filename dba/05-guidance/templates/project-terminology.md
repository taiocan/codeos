---
artifact_type: project_terminology
---

# [PROJECT_NAME] Terminology

<!-- Exempt from the Summary block and reader_model declaration: a glossary table is compact. -->

This is the canonical glossary for recurring project-wide specialized meanings. Create this file
only when such terminology exists. Do not add ordinary technical language or terms used by only one
feature.

## Ownership Rules

- A term listed here has one shared project meaning. New and revised governed artifacts reuse it
  exactly.
- An Intent defines only feature-local terms or legitimate narrower refinements that do not
  contradict a shared meaning.
- If a feature needs genuinely different semantics, qualify the term rather than silently
  overriding the shared meaning.
- Do not copy an existing approved Intent definition here merely for completeness. Promote its
  common meaning during the affected package's next substantive revision and remove duplication
  then.
- A glossary edit does not retroactively reinterpret an approved artifact. If a changed meaning
  would materially change an approved artifact, revise and reapprove that artifact.
- CodeOS/DBA terminology remains owned by
  `.codeos/toolkit/dba/05-guidance/terminology.md`.

## Definitions

| Term | Project-wide meaning |
|---|---|
| **[Specialized term]** | [One stable meaning required across features.] |

Delete the example row before use. Git preserves history; this file has no status, approval,
version, registry, or separate lifecycle.
