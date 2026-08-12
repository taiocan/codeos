# Architecture Synthesis Gate

<!-- DOCTRINE ADAPTER: architecture-entry
Conditional: applies only when the selected architecture policy requires architecture approval. -->

## Your Role

Guide Architecture Synthesis for one project-level scope. Produce one concise architecture artifact;
do not implement features, invent behavior, or create additional workflow state. Read the selected
`architecture_synthesis_policy` component before proceeding.

## Applicability and Preconditions

Use this prompt when implementation would otherwise settle an unresolved project-level or
cross-feature structural decision. If no such decision exists, do not create an architecture
artifact.

Identify every affected feature. Each must have an approved, mutually consistent Intent, Contract,
and Event Schema before architecture can be approved. A draft scope may be started earlier, but
unapproved feature material is not architectural authority.

Run `.codeos/scripts/codeos-review.sh inspect-architecture-scopes` and stop if inspection fails.
Reuse the existing matching scope when one exists; otherwise create `architecture/scopes/` if
needed and use `.codeos/templates/architecture-scope.md` to draft
`architecture/scopes/<scope-id>.md`. The filename is the scope identity.

## Synthesis

Read the approved Specification Packages for all scope members and any relevant approved project
architecture. Review them together for conflicting responsibility or data ownership, dependency
direction, lifecycle or failure assumptions, integration contracts, and event semantics.

Use the selected architecture policy's synthesis reasoning frame. Record only
architecture-significant answers; do not reproduce the frame as mandatory sections, a matrix, or a
disposable check report. Every governed architectural component must have one clear owned
responsibility. Record interfaces, state, runtime placement, constraints, and feature applicability
only when material.

For each material decision, state enough rationale or authority to show that it comes from approved
requirements or an explicit human architectural decision. Do not mechanically map every paragraph
to a source.

If a question is behavioral or would create a new quality requirement, name the affected feature and
return it to Intent, Contract, or Event Schema. If responsibility overlaps, dependency direction
conflicts, data authority is unclear, or integration assumptions contradict, keep `approval: null`
and present the conflict for human resolution.

## Approval

Present the complete scope artifact for one explicit human approval. Before approval, state:

`AWAITING HUMAN APPROVAL OF THE ARCHITECTURE SCOPE`

After approval, record non-empty `approval.by` and `approval.at`. Any later material edit to the
architecture or membership first resets `approval` to `null`. Do not update a registry, numeric
version, pointer, hash, or history file. Git preserves prior approved revisions.

A material later change to membership or architecture first sets `approval: null`, then uses this
same synthesis and approval boundary. Reassess completed work only where targeted impact analysis
finds an actual conflict.

Architecture review is advisory and optional. Use it only when the identified architectural risk
justifies independent review; it does not create another gate or persisted review state.
