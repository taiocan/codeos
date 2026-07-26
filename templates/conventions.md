# Naming Conventions

## Events

Format: `<Entity><Action><Outcome>`

Examples:
- `CartItemAdded`
- `PaymentCaptureFailed`
- `UserLoginSucceeded`
- `OrderCreated`

Failure events end with `Failed`, `Rejected`, or `Timeout`:
- `CartItemAddFailed`
- `LoginRejected`
- `PaymentGatewayTimeout`

## Metrics

Format: `<feature>_<measurement>`

Examples:
- `add_item_latency_ms`
- `payment_failure_rate`
- `login_duration_ms`
- `session_creation_success_rate`

## Errors

Format: `snake_case_failure_reason`

Examples:
- `item_not_found`
- `payment_gateway_timeout`
- `session_expired`
- `invalid_input`

## Feature IDs

Format: `F-####` — 4-digit, zero-padded, sequential, permanent, never reused.

The feature id is the stable identity; pair it with a separate human-readable slug for
the label (`features/registry.yaml`'s `feature_id` / `slug` fields already model this
split). Assigned at the Feature Brief Synthesis step
(`.codeos/prompts/00b-feature-brief.md`): scan `features/registry.yaml` (if present) and
`backlog/F-####-*.md` filenames for the current max, then assign next. A refinement
(R-type) brief reuses its parent feature's id — it does not mint a new one.

Examples:
- `F-0001` — first feature assigned in a project
- `F-0002`, `F-0003`, … — subsequent features, in assignment order

Filenames combine id + slug:
- `backlog/F-0001-add-item-to-cart.md`
- `intents/F-0001.md`
- `contracts/F-0001_contract.md`
- `events/F-0001_schema.md`

## Correlation IDs

- Format: UUID v4
- Required on ALL events without exception
- Must propagate through the entire feature execution chain
- Every log line during feature execution must include `correlation_id`

## Optional Mechanism Status Convention

A minimal, reusable shape for a human-controlled on/off switch governing some AI-doctrine or
generation behavior — not a runtime product feature flag. Use this instead of inventing a new
status-file shape each time a feature needs one.

**Exact grammar.** After ignoring blank lines, the file must contain **exactly one line**: either

```yaml
status: enabled
```

or

```yaml
status: disabled
```

verbatim. No comments, no duplicate keys, no nested structures, no additional fields.

**Four outcomes only:**

| File state | Result |
|---|---|
| Absent | Disabled |
| Exact `status: disabled` | Disabled |
| Exact `status: enabled` | Enabled |
| Anything else (unreadable, extra content, any other value) | Stop and report a configuration error |

**Whitespace and line endings.** A trailing newline is allowed; leading or trailing blank lines are
allowed; both LF and CRLF line endings are accepted after normalizing line endings before
comparison. Internal whitespace is **not** normalized — `status:  enabled` (extra space),
`status: enabled ` (trailing space), a tab anywhere in the line, an inline comment, or an
uppercase/mixed-case variant (`Status: Enabled`) are all invalid and fall under "anything else."

**Missing means disabled, everywhere — no absence-policy options.** This is a deliberate
simplification, not an oversight: deleting the file does disable the mechanism, and that is an
acceptable trade-off for this class of switch, because the existing safeguards already cover it —
only an explicit human instruction creates, edits, or deletes the file (an agent never does so on
its own initiative); git records every change; ordinary change review catches an unauthorized
deletion the same way it catches any other unreviewed diff. No dedicated audit fields (schema
version, activation id, provenance, versioning, result codes) are added to compensate — git history
is the whole audit trail.

**No shared tool.** Each consuming feature reads the file directly — a one-line check in its own
prompt text — and implements exactly the four-outcome table above. A consumer may not broaden the
grammar (e.g. adding its own extra fields to this file) or invent additional states; a feature that
needs more than "enabled/disabled" defines its own separate mechanism rather than stretching this
one.

**Placement.** A project-local status file lives under `architecture/`, alongside other
project-level architecture artifacts. This convention does not itself scaffold any concrete status
file — a consuming feature's own change does that when it adopts the convention. Self-development
placement (a location analogous to `architecture/` for Codeos's own toolkit repository) is
intentionally left undecided by this convention — it is decided by the first feature that actually
needs a self-development status file, together with that feature's own change, not speculated on
here.
