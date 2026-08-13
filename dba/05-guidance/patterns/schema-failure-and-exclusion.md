---
component_question: When should schema failures use dual events and vocabulary exclusions remain explicit?
out_of_scope: Universal feature requirements, schema ownership outside these cases, and implementation-specific error handling.
---

# Conventions: Schema Failure and Vocabulary Exclusion

Architectural conventions established during LucidPM R7–R10.
These are documented patterns, not behavioral requirements for all future features.
Apply when the described conditions are present; do not force the pattern otherwise.

---

## Convention: Dual-Event Schema Failure

When a vocabulary consumer command fails because schema loading fails, and the
command represents a distinct observable business operation, emit two events:

1. **project_schema** emits the root-cause event:
   `SchemaNotFound` | `SchemaParseError` | `SchemaValidationFailed`
   Records *why* schema loading failed.

2. **The consuming module** emits its own business-outcome event:
   `[CommandName]FailedSchemaInvalid`
   Records *that this specific command failed* because of it.

Both facts serve different consumers: one needs vocabulary diagnostics; the other
needs to know whether their command completed. These are different facts and belong
in separate events.

**When this applies:** Cross-module failure AND the consuming module has a distinct
observable business operation (a command users invoke and expect results from).

**When this does NOT apply:** If the failure has no meaningful consumer-module
business outcome to distinguish from the cross-module signal, the cross-module event
alone is sufficient. Not every command needs a wrapper failure event.

The observational event (e.g., `SyncRequested`, `RecordQueried`) IS emitted before
the schema check — it records that the command was received and processing began.

*Established by: R9 (logseq_sync), R10 (project_state)*

---

## Convention: SchemaTypeUnknown Ownership

When a vocabulary consumer processes items whose entity types are not recognized,
the unrecognized-type signal is emitted by **project_schema**
(`source_module: "project_schema"`), not by the consumer module.

**Rationale:** The signal represents a vocabulary-resolution fact ("this stored type
resolves to no concept") rather than a consumer-specific business decision. The
consumer responds to that fact (by excluding the item, logging it, etc.) but the
vocabulary fact itself is the vocabulary module's to record.

This is currently implemented via `project_schema::emit_type_unknown()`, which sets
`source_module: "project_schema"` regardless of which module calls it.

**When this applies:** The consumer calls the project_schema resolution API and
discovers an unrecognized type; the signal is about the vocabulary gap, not about
the consumer's response to it.

**Current status:** This is the project-level convention as of R10. It depends on
project_schema being a shared library. If that architectural relationship changes,
ownership of this signal should be reconsidered rather than mechanically preserved.

*Established by: R7 (priority_view), R10 (project_state)*
