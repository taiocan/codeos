# Tool Ownership

This directory separates durable capabilities from mechanisms with an explicit removal decision.
The sibling layout keeps supported paths stable; lifecycle is not inferred from directory depth.

## Permanent core

- `initializer/` creates the minimum downstream layout.
- `configuration/` validates the selected configuration and active layout.
- `reviewer/` owns the Codex-backed advisory review interface and records.

## Bounded capabilities

- `architecture-migration/` is the one-way bridge from the supported legacy architecture format.
  When that starting format leaves the support window, delete its implementation, fixtures, tests,
  and corresponding upgrade guidance together.
- `implementer/` is the off-by-default DeepSeek experiment. UPG-0064 owns the deferred Stage 4
  decision; UPG-0066 owns the evidence-first Stage 5 pilot. Do not build a framework around it.

`tests/run.sh` only invokes capability-owned suites. It owns no test semantics of its own.
