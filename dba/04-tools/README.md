# Tool Ownership

This directory separates durable capabilities from mechanisms with an explicit removal decision.
The sibling layout keeps supported paths stable; lifecycle is not inferred from directory depth.

## Permanent core

- `initializer/` creates the minimum downstream layout. Under a configuration carrying
  `codeos_mechanics_policy` (DBA-5+), it also scaffolds the Platform Baseline skeleton from
  `initializer/skeleton/` (PostgreSQL + Rust/axum/sqlx + Svelte/SvelteKit + Docker) and
  `.codeos/00-project/codeos.yaml` — never for an earlier configuration.
- `configuration/` validates the selected configuration and active layout.
- `compatibility/` owns the explicit `supported | deprecated | historical-only` DBA lifecycle and
  the synthetic plus pinned-project replay used to qualify a release. A pinned project is
  supplementary compatibility evidence; it never defines universal policy.
- `release/` builds manifest-qualified executables and verifies their embedded release/build
  identity against the active installation manifest.
  It also issues configurations: `qualify-candidate.sh <name>` qualifies a committed Candidate
  configuration in a disposable copy and prints its Qualified semantic identity. After human
  approval of that identity, `issue-configuration.sh <name> DBA-N <identity>` publishes exactly it,
  with its frozen Semantic closure (DQA WO-01).
- `reviewer/` owns the advisory review interface and records. It invokes Codex; an assessment from
  any other model enters through `review --assessment` as advisory evidence that never satisfies a
  required review round. `codeos-review-deepseek.sh` is transport for that path and nothing more —
  model-specific knowledge stops there and does not enter the engine.

## Bounded capabilities

- `monitor/` builds an immutable, hash-verified snapshot of one project's CodeOS records for an
  external read-only report (DBAMON). It only reads the project and is never workflow, validity,
  or acceptance evidence.

- `architecture-migration/` is the one-way bridge from the supported legacy architecture format.
  When that starting format leaves the support window, delete its implementation, fixtures, tests,
  and corresponding upgrade guidance together.
- `implementer/` is the off-by-default DeepSeek experiment. UPG-0064 owns the deferred Stage 4
  decision; UPG-0066 owns the evidence-first Stage 5 pilot. Do not build a framework around it.

`tests/run.sh` only invokes capability-owned suites. It owns no test semantics of its own.

## DBA-25 project interfaces

Projects own `scripts/codeos-qualify` and every underlying tool, threshold, and System Constraint.
Codeos supplies current implementation, Charter, and configuration identities plus an output path;
it validates the JSON result bindings, the exact Charter constraint IDs, and the four minimal
engineering-baseline result IDs. It does not wrap a project test framework.

Operation v2 stores append-only identity, routing, and closure records at
`.codeos/06-workflow/{observations,operation-routes,operation-closures}.jsonl`. A closure is valid
only when its route-specific evidence is later than and bound to the recorded route.
