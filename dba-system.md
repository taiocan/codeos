# Codeos DBA System — Active Configuration

> This is the stable downstream entrypoint for Codeos DBA. Project instructions load it through
> `.codeos/toolkit/dba-system.md`. Codeos toolkit self-development is governed separately by the repository
> root `CLAUDE.md`.

Active configuration: `.codeos/toolkit/dba/00-entry/configurations/DBA-3.yaml`

The configuration selects the authoritative version of each governed DBA component. Component
files own their semantics; the configuration only selects which versions are active.

Only the active DBA configuration is supported. Inactive configurations are historical records.

## Downstream Project Layout Contract

`.codeos/` is the project-local home for durable DBA information. `.codeos/toolkit` is the single
machine-local symlink to the shared Codeos toolkit and is ignored by Git. Project source, tests,
and runtime-produced evidence remain outside `.codeos/`.

Canonical durable locations are:

| Capability | Location | Creation |
|---|---|---|
| Project instructions | `.codeos/00-project/CLAUDE.md` | Required |
| Solution Charter | `.codeos/00-project/charter.md` | Required before the first Specification Package approval |
| Learning Register | `.codeos/00-project/learnings.md` | Only when a material unresolved observation exists |
| Project terminology | `.codeos/00-project/terminology.md` | Only when shared project-specific terminology exists |
| Intent | `.codeos/01-specification/intents/<feature-id>.md` | Required per feature |
| Contract | `.codeos/01-specification/contracts/<feature-id>_contract.md` | Required per feature |
| Event Schema | `.codeos/01-specification/event-schemas/<feature-id>_schema.md` | Required per feature |
| Architecture Scope | `.codeos/02-architecture/scopes/<scope-id>.md` | Only when architecture synthesis applies |
| Implementation Profile | `.codeos/02-architecture/implementation-profile.yaml` | Only when adopted |
| Saved discovery | `.codeos/00-discovery/<topic-slug>.md` | Only when it has durable value |
| Full-Solution Concept | `.codeos/00-discovery/solution-concept.md` | Only when whole-solution reasoning has durable value |
| Refinement record | `.codeos/04-refinement/<feature-id>-<slug>.md` | Only when it has durable value |
| Reviewer configuration | `.codeos/05-review/reviewer.toml` | Only when defaults are overridden |
| Automated review records | `.codeos/05-review/reviews/` | Created and owned by the reviewer tool |
| Review measurement | `.codeos/05-review/measurements/<name>.md` | Only when it has durable value |

Implementation Profile replacement may additionally use
`.codeos/02-architecture/proposals/implementation-profile-v<N>.yaml` and
`.codeos/02-architecture/history/implementation-profile-v<N>.yaml` as specified by the selected
Implementation Profile policy. These directories are not a general architecture lifecycle.

Root `CLAUDE.md` and `AGENTS.md` are small, real discovery adapters. Runtime event evidence remains
at `events/runtime_events.jsonl` when used. Implementation and tests use project-native paths.
`.codeos-state/` is non-authoritative, ignored operational state and is created only when a tool
needs cross-command state.

This section is the sole semantic owner of canonical downstream locations. Workflow prompts name
canonical output paths for operational clarity; consumers load artifact types from this contract
without independently redefining the layout. Templates and tools implement these locations, and
the layout contract check enforces them.

## Component Boundary Contract

A governed Markdown component created or semantically revised under this model begins with exactly
this two-field responsibility boundary:

```yaml
---
component_question: The single question this component answers.
out_of_scope: The nearest responsibilities this component must not absorb.
---
```

The question defines the component's responsibility. `out_of_scope` names likely ownership
mistakes, not every dependency. Neither field summarizes or overrides the component's rules. No
other boundary metadata is supported.

A DBA configuration MUST NOT become active unless it passes the focused boundary-contract test
immediately before the active-configuration pointer changes:

```bash
bash dba/04-tools/configuration/dba-config-boundaries.sh dba/00-entry/configurations/DBA-N.yaml
```

After changing the pointer, verify that it names the candidate configuration that passed the test.

At the start of a DBA session:

1. Read the active configuration above.
2. Treat its component paths as relative to the `.codeos/toolkit/` root.
3. Read the `doctrine` component fully.
4. Read each other selected component when its policy or tool applies to the current work.

All selected components are jointly authoritative. A reference to `doctrine`, `review_policy`,
`architecture_synthesis_policy`, `implementation_profile_policy`, or `reviewer_tool_contract`
means the component file selected under that key by the active configuration.

Unversioned canonical resources are not selected through this configuration. Continue to use their
normal paths under `.codeos/toolkit/dba/03-prompts/`, `.codeos/toolkit/dba/04-tools/`, and
`.codeos/toolkit/dba/05-guidance/`.

The selected doctrine is the sole source of DBA semantic guarantees. Operational consequences may
be encoded only at genuine execution boundaries marked `DOCTRINE ADAPTER`; other consumers refer
to the doctrine or an adapter instead of restating its semantics.

## Doctrine Adapter Index

This index is descriptive navigation, not a second authority source:

| Boundary | Owning prompt |
|---|---|
| `purpose-approval` | `.codeos/toolkit/dba/03-prompts/workflow/00-charter.md` |
| `specification-approval` | `.codeos/toolkit/dba/03-prompts/workflow/03-event-schema.md` |
| `delivery-entry` | `.codeos/toolkit/dba/03-prompts/workflow/04-implement.md` |
| `final-acceptance` | `.codeos/toolkit/dba/03-prompts/workflow/08-replay.md` |
| `architecture-entry` | `.codeos/toolkit/dba/03-prompts/workflow/03b-architecture-synthesis.md` |
