# Codeos DBA System — Active Configuration

> This is the stable downstream entrypoint for Codeos DBA. Project instructions load it through
> `.codeos/dba-system.md`. Codeos toolkit self-development is governed separately by the repository
> root `CLAUDE.md`.

Active configuration: `.codeos/dba/00-entry/configurations/DBA-2.yaml`

The configuration selects the authoritative version of each governed DBA component. Component
files own their semantics; the configuration only selects which versions are active.

Only the active DBA configuration is supported. Inactive configurations are historical records.

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
2. Treat its component paths as relative to the `.codeos/` toolkit root.
3. Read the `doctrine` component fully.
4. Read each other selected component when its policy or tool applies to the current work.

All selected components are jointly authoritative. A reference to `doctrine`, `review_policy`,
`architecture_synthesis_policy`, `implementation_profile_policy`, or `reviewer_tool_contract`
means the component file selected under that key by the active configuration.

Unversioned canonical resources are not selected through this configuration. Continue to use their
normal paths under `.codeos/dba/03-prompts/`, `.codeos/dba/04-tools/`, and
`.codeos/dba/05-guidance/`.

The selected doctrine is the sole source of DBA semantic guarantees. Operational consequences may
be encoded only at genuine execution boundaries marked `DOCTRINE ADAPTER`; other consumers refer
to the doctrine or an adapter instead of restating its semantics.

## Doctrine Adapter Index

This index is descriptive navigation, not a second authority source:

| Boundary | Owning prompt |
|---|---|
| `specification-approval` | `.codeos/dba/03-prompts/workflow/03-event-schema.md` |
| `delivery-entry` | `.codeos/dba/03-prompts/workflow/04-implement.md` |
| `final-acceptance` | `.codeos/dba/03-prompts/workflow/08-replay.md` |
| `architecture-entry` | `.codeos/dba/03-prompts/workflow/03b-architecture-synthesis.md` |
