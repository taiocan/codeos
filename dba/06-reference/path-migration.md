---
component_question: How should downstream references be updated after the Codeos internal path reorganization?
out_of_scope: Compatibility aliases, unrelated project migrations, component semantics, and automated migration tooling.
---

# Codeos Internal Path Migration

`.codeos/dba-system.md` remains the stable downstream entrypoint. Codeos-owned internal paths were
reorganized into the numbered `dba/` package and are not retained through aliases.

Update explicit downstream references as follows:

| Old path | New path |
|---|---|
| `.codeos/prompts/<workflow-prompt>` | `.codeos/dba/03-prompts/workflow/<workflow-prompt>` |
| `.codeos/prompts/<review-prompt>` | `.codeos/dba/03-prompts/review/<review-prompt>` |
| `.codeos/prompts/codeos-implementer-task.md` | `.codeos/dba/03-prompts/delegation/codeos-implementer-task.md` |
| `.codeos/scripts/codeos-review.sh` | `.codeos/dba/04-tools/reviewer/codeos-review.sh` |
| `.codeos/scripts/codeos-implement.sh` | `.codeos/dba/04-tools/implementer/codeos-implement.sh` |
| `.codeos/scripts/dba-init.sh` | `.codeos/dba/04-tools/initializer/dba-init.sh` |
| `.codeos/templates/<file>` | `.codeos/dba/05-guidance/templates/<file>` |
| `.codeos/patterns/<file>` | `.codeos/dba/05-guidance/patterns/<file>` |
| `.codeos/terminology.md` | `.codeos/dba/05-guidance/terminology.md` |

Existing project work products and state keep their project-local paths. No compatibility symlinks,
forwarding wrappers, or duplicate resources are provided.
