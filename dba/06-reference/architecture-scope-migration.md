---
component_question: How should a downstream project migrate legacy architecture cohorts into current scope artifacts?
out_of_scope: Architecture approval, new architecture design, ongoing scope management, and migration-tool implementation.
---

# One-Time Architecture Scope Migration

The lean Architecture Synthesis policy does not read the legacy registry/cohort schema. Convert a
downstream project once before using the new policy:

```bash
python3 .codeos/dba/04-tools/architecture-migration/migrate-architecture-synthesis-v2.py .
python3 .codeos/dba/04-tools/architecture-migration/migrate-architecture-synthesis-v2.py . --apply
```

The tool requires Python 3 and PyYAML.

The first command validates without writing. The second creates
`architecture/scopes/<scope-id>.md`, removes legacy architecture fields from the registry, expands
Implementation Profile cohort selectors and exceptions to explicit feature ids when present, and
removes the two singleton artifacts and their manual history copies. Commit legacy inputs before
running it; Git is the retained history.

Migration stops instead of guessing when membership, ids, approval state, versions, or approval
metadata disagree; when only one legacy artifact is approved; when more than one legacy cohort is
present behind the singleton artifacts; or when a target already exists. Resolve the reported
conflict explicitly and rerun. There is no dual-schema or compatibility mode after conversion.
