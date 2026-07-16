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
