# WO-05 — Opt-in Live Health Pilot

NORMAL · deferred until one real deployment has an authoritative observation source

## Goal

Test whether time- and environment-bound Live Health adds value without contaminating Platform
Assurance.

## Work

- Name the deployment, environment, source owner, freshness limit, and timeout behavior.
- Add an opt-in, one-shot `status --live` adapter; normal status remains offline.
- Report observation time, environment, component result, and explicit unknown or error states.
- Persist nothing and derive no workflow decision from the observation.

## Acceptance criteria

1. The entry deployment and authoritative source are recorded before implementation.
2. Timeout, partial availability, malformed output, and stale observation cases are tested.
3. Live results cannot change Platform Assurance, acceptance, or exit codes.
4. Any need for new authority or durable state escalates to a separate PROTECTED decision.
