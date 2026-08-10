# Codeos DBA System — Active Configuration

> This is the stable downstream entrypoint for Codeos DBA. Project instructions load it through
> `.codeos/dba-system.md`. Codeos toolkit self-development is governed separately by the repository
> root `CLAUDE.md`.

Active configuration: `.codeos/dba/configurations/DBA-1.yaml`

The configuration selects the authoritative version of each governed DBA component. Component
files own their semantics; the configuration only selects which versions are active.

At the start of a DBA session:

1. Read the active configuration above.
2. Treat its component paths as relative to the `.codeos/` toolkit root.
3. Read the `doctrine` component fully.
4. Read each other selected component when its policy or tool applies to the current work.

All selected components are jointly authoritative. A reference to `doctrine`, `review_policy`,
`architecture_synthesis_policy`, `implementation_profile_policy`,
`controlled_plain_english_policy`, or `reviewer_tool_contract` means the component file selected
under that key by the active configuration.

Unversioned canonical resources are not selected through this configuration. Continue to use their
normal paths, including `.codeos/prompts/`, `.codeos/templates/`, `.codeos/patterns/`,
`.codeos/scripts/`, `.codeos/tools/`, and `.codeos/terminology.md`.
