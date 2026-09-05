# Codeos DBA System — Active Configuration

> This is the stable downstream entrypoint for Codeos DBA. Project instructions load it through
> `.codeos/toolkit/dba-system.md`. Codeos toolkit self-development is governed separately by the repository
> root `CLAUDE.md`.

Active configuration: `.codeos/toolkit/dba/00-entry/configurations/DBA-5.yaml`

The configuration selects the authoritative version of each governed DBA component. Component
files own their semantics; the configuration only selects which versions are active.

The active configuration is the default for a newly initialized project and the target of an
explicit upgrade. A project that has already adopted a specific configuration continues under it
until that project's user explicitly elects to upgrade — the active pointer changing here does not
migrate, retrofit, or reinterpret an existing project's artifacts. A configuration a project has
adopted remains supported for that project's continued use. A configuration no project has adopted
and that is not active is a historical record only.

## Downstream Project Layout Contract

`.codeos/` is the project-local home for durable DBA information. `.codeos/toolkit` is the single
machine-local symlink to the shared Codeos toolkit and is ignored by Git. Project source, tests,
and runtime-produced evidence remain outside `.codeos/`.

Canonical durable locations are:

| Capability | Location | Creation |
|---|---|---|
| Project instructions | `.codeos/00-project/CLAUDE.md` | Required |
| Project configuration | `.codeos/00-project/codeos.yaml` | Required under DBA-5 and later; not applicable under DBA-4 |
| Solution Charter | `.codeos/00-project/charter.md` | Required before the first Specification Package approval |
| Learning Register | `.codeos/00-project/learnings.md` | Only when a material unresolved observation exists |
| Project terminology | `.codeos/00-project/terminology.md` | Only when shared project-specific terminology exists |
| Intent | `.codeos/01-specification/intents/<feature-id>.md` | Required per feature |
| Contract | `.codeos/01-specification/contracts/<feature-id>_contract.md` | Required per feature |
| Event Schema | `.codeos/01-specification/event-schemas/<feature-id>_schema.md` | Required per feature |
| Architecture Scope | `.codeos/02-architecture/scopes/<scope-id>.md` | Only when architecture synthesis applies |
| Implementation Profile | `.codeos/02-architecture/implementation-profile.yaml` | Only when adopted |
| Module Design Note | `.codeos/03-design/<module-slug>.md` | Only when the module warrants one |
| Saved framing or decomposition | `.codeos/00-discovery/<topic-slug>.md` | Only when it has durable value |
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

## Downstream Artifact Frontmatter and Communication Contract

Applies under DBA-5 and later to every artifact produced under the Downstream Project Layout
Contract above. It is distinct from the Component Boundary Contract below, which governs this
toolkit's own doctrine, policy, prompt, and tool files, not downstream project artifacts.

Every downstream artifact begins with:

```yaml
---
artifact_type: <type>
---
```

plus that artifact type's approval-bearing metadata when `<type>` is governed, and any other
type-specific fields its owning template or policy defines (for example `features` on an
Architecture Scope). Approval-bearing metadata is either an `approval` field (Charter, Architecture
Scope, and other single-decision artifacts) or the Specification Package's own `status` /
`approved_by` / `approved_at` triple (Intent, Contract, Event Schema — kept distributed because the
three record one joint package decision, not three separate gates). `governed` is not a supported
frontmatter key — an artifact never restates its own governance state. Charter, Intent, Contract,
and Event Schema always require it. Every other type's requirement is read from
`.codeos/00-project/codeos.yaml`'s `artifacts:` block, which is the sole authoritative source for
that state.

Every substantial human-facing artifact — governed or not — opens with a Summary block immediately
after this frontmatter:

```markdown
## Summary
- [key message]
- [key message]

Oversimplification risk: this summary can omit nuance the full artifact carries. Read the
relevant section directly before relying on the summary alone for a consequential decision.
```

and declares `reader_model: <stable-topic | known-to-new | whole-before-parts | preview-then-traverse>`
in its frontmatter, per `dba/05-guidance/reader-oriented-output.md`. Both requirements are exempt on
event schemas, logs, machine-structured data, compact tables, and any artifact short enough that a
summary would repeat it.

`.codeos/00-project/codeos.yaml` also names the solution's Platform Baseline and displays the fixed
Codeos Mechanics from the selected Codeos Mechanics policy. It is mechanically checked by
`bash dba/04-tools/configuration/project-config-contract.sh`, which fails closed on a downgraded
core-four artifact type, an unlisted artifact type, or an attempt to change a fixed mechanic.
Toggling a configurable artifact type's governed state changes its process weight only; it does not
suspend a doctrine-level human-control guarantee, none of which are represented as configuration
entries.

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

For every human-readable output, read and apply
`.codeos/toolkit/dba/05-guidance/reader-oriented-output.md`,
`.codeos/toolkit/dba/05-guidance/terminology.md`, and the project terminology at
`.codeos/00-project/terminology.md` when it exists.

Numeric workflow prefixes `01` through `09` are reserved for governed Stage IDs and correspond
exactly to Stages 1 through 9. Every non-stage workflow prompt uses a descriptive
`support-<role>.md` filename. Support workflows are not stages and MUST NOT be described as Stage
0, Stage 3b, or additional lifecycle stages.

The selected doctrine is the sole source of DBA semantic guarantees. Operational consequences may
be encoded only at the adapter boundaries the selected doctrine names, and only in a prompt that
declares ownership of one with a `DOCTRINE ADAPTER: <name>` marker. Exactly one prompt owns each
boundary. Other consumers refer to the doctrine or an adapter instead of restating its semantics.

The doctrine owns which boundaries exist; this contract owns how a prompt declares one. To find the
owner of a boundary, search the marker:

```bash
rg 'DOCTRINE ADAPTER: <name>' dba/03-prompts
```
