# Pattern: Rust DBA Project Structure

## When This Pattern Applies

This pattern documents Rust-specific conventions for DBA projects. It belongs to the
**language pattern layer** — it defines artifact structure conventions for Rust contexts.
It is not part of the core methodology and does not change how any stage prompt works.

Apply this pattern when:
- A DBA project is implemented in Rust
- The project has more than one binary or library crate
- Multiple DBA features share event emission or test infrastructure
- CI is needed and test isolation matters

**Consulted by:** Stage 4 (`.codeos/prompts/04-implement.md`), when the project's approved
Implementation Profile (`.codeos/dba-system.md` → "Implementation Profile") resolves
`primary_language: rust` for the feature being implemented; and optionally by Architecture
Synthesis (`.codeos/dba-system.md` → "Multi-Feature Architecture Synthesis Gate") when a core
cohort also exists. This pattern's recommendations are always advisory — consulted, never
overriding an approved Architecture Baseline or another project-specific decision.

---

## Cargo Workspace Layout

When a DBA project contains multiple modules, use a Cargo workspace from the start.
Adding a workspace later is an architectural refinement that touches every crate's
`Cargo.toml` — starting with it avoids that cost.

Recommended layout:

```
project/
├── Cargo.toml                     ← workspace root
├── Cargo.lock
├── .codeos/                       ← DBA toolkit symlink
├── CLAUDE.md
├── features/
│   └── registry.yaml
├── intents/
├── contracts/
├── events/
│   └── runtime_events.jsonl
├── modules/
│   ├── [feature_a]/               ← one crate per major module
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── [feature_b]/
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── dba_events/                ← shared event infrastructure (see below)
│   │   ├── Cargo.toml
│   │   └── src/
│   └── dba_test_support/          ← shared test support (see below)
│       ├── Cargo.toml
│       └── src/
└── tests/
    ├── behavioral/
    └── replay/
```

Workspace `Cargo.toml`:

```toml
[workspace]
members = [
    "modules/feature_a",
    "modules/feature_b",
    "modules/dba_events",
    "modules/dba_test_support",
]
resolver = "2"
```

**Per-module test command (workspace-idiomatic):**

```bash
cargo test -p <module>                          # single module
cargo test --workspace                          # all modules
```

The `--manifest-path modules/<module>/Cargo.toml` form still works but is not preferred
once a workspace exists. Historical refinement intent files may still use this form.

---

## Shared Event Infrastructure (`project_schema` in LucidPM)

In LucidPM, shared event infrastructure lives in `project_schema` rather than a separate
crate. The `EventEnvelope` struct and `emit_event` function are exported from
`project_schema/src/lib.rs` and used by all modules that already depend on `project_schema`.

### Actual API (`project_schema`)

```rust
use std::path::Path;
use serde_json::Value as JsonValue;

pub struct EventEnvelope<'a> {
    pub source_module:  &'a str,
    pub event_type:     &'a str,
    pub correlation_id: &'a str,
    pub payload:        JsonValue,
}

pub fn emit_event(events_file: &Path, envelope: EventEnvelope<'_>);
```

**Why struct over positional args:** Adding future fields (`schema_version`, `causation_id`)
does not require touching all call sites — only the `EventEnvelope` definition and callers
that need the new field.

### Module wrapper pattern

Each module defines a thin local wrapper that fills in the fixed fields:

```rust
fn emit_event(event_type: &str, correlation_id: &str, payload: Value) {
    project_schema::emit_event(Path::new(EVENTS_FILE), EventEnvelope {
        source_module: SOURCE_MODULE,
        event_type,
        correlation_id,
        payload,
    });
}
```

**Impersonation variant** (e.g., `logseq_sync::emit_task_event`): when a module must emit
events that appear indistinguishable from another module's events, the wrapper supplies a
literal `source_module` instead of `SOURCE_MODULE`. This is a contractual invariant and
must be preserved exactly — replay tests validate it.

```rust
fn emit_task_event(event_type: &str, correlation_id: &str, payload: Value) {
    project_schema::emit_event(Path::new(EVENTS_FILE), EventEnvelope {
        source_module: "task_model",  // contractual indistinguishability invariant
        event_type,
        correlation_id,
        payload,
    });
}
```

### Required base fields (enforced by `emit_event`)

```json
{
  "event_id": "uuid-v4",
  "event_type": "EventName",
  "timestamp": 1710000000000,
  "correlation_id": "uuid-v4",
  "source_module": "module_name",
  "payload": {}
}
```

`emit_event` must serialize and append as JSONL (one event per line, no trailing comma).
It must not truncate or rewrite the log — append only.

### Correlation ID propagation

Modules receive a `correlation_id: Uuid` at their entry point (from the CLI invocation
or test harness) and pass it through all calls that emit events. The correlation ID is
never generated inside domain logic — only at the invocation boundary.

---

## Shared Test Support Crate (`dba_test_support`)

### When to create

Create `dba_test_support` when two or more test files share setup logic: temp directory
creation, schema writing, binary invocation, or fixture loading.

### Core surface

```rust
use tempfile::TempDir;
use std::path::Path;

pub fn create_temp_project() -> TempDir;
pub fn write_project_schema(dir: &TempDir, content: &str);
pub fn assert_base_fields(event: &serde_json::Value);
pub fn load_fixture(path: &Path) -> Vec<serde_json::Value>;
pub fn run_binary_schema_isolated(
    bin: &str,
    args: &[&str],
    project_dir: &Path,
) -> std::process::Output;
```

### `create_temp_project`

Creates a `TempDir` with:
- `events/` subdirectory (for the runtime event log)
- Empty `events/runtime_events.jsonl`

Every behavioral test calls this at the start. Tests do not share state.

### `write_project_schema`

Writes a `project-schema.yaml` into the temp project dir. Used by all tests that
exercise vocabulary-driven features. The default schema must match the previously
hardcoded vocabulary exactly — this is the backward-compatibility regression baseline.

### `run_binary_schema_isolated`

Invokes a binary with the test's temp directory set as the config root (via env var
override). This prevents any globally-installed schema from contaminating test assertions.
Tests that deliberately exercise the no-schema path must NOT use this runner.

### `assert_base_fields`

Asserts that an event JSON object has all six required base fields with non-empty,
non-null values. Call on every event before asserting payload content.

### Alternative: feature-gated module in an existing shared crate

If the project already has a shared library crate depended on by all modules (e.g.,
a `project_schema` or `core` crate), add the test helpers there under a `test-support`
feature rather than creating a new crate. This avoids a new crate for a small helper set.

```toml
# shared_crate/Cargo.toml
[features]
test-support = []
```

```rust
// shared_crate/src/lib.rs
#[cfg(any(feature = "test-support", test))]
pub mod test_support;
```

```toml
# consuming_module/Cargo.toml
[dev-dependencies]
shared_crate = { path = "../shared_crate", features = ["test-support"] }
```

The `test` cfg-flag half of `any(feature = "test-support", test)` lets the shared
crate's own tests call the helpers without a circular dev-dependency.

**LucidPM uses this pattern.** `load_fixture` lives in
`project_schema::test_support::load_fixture(name: &str) -> Vec<Value>`, where `name` is
the filename within `tests/replay/fixtures/`. The path is resolved using
`env!("CARGO_MANIFEST_DIR")` baked into the compiled library — correct because all
modules sit at the same directory depth (`modules/<module>/`).

**What was NOT extracted** (no shared pattern existed in the codebase):
- `assert_base_fields`: each module checks its own `source_module` value inline
- `DEFAULT_SCHEMA`: no replay test uses a schema constant

---

## Replay Test Pattern

### Purpose

Replay tests verify that the system is deterministically replayable: given the same
event stream, it produces the same state transitions.

### Structure

Each feature with approved tests has a file `tests/replay/[feature_id]_replay.test.rs`.

```rust
#[test]
fn test_[feature_id]_replay_produces_identical_state() {
    // 1. Run the feature and capture the event stream
    let dir = create_temp_project();
    run_binary_schema_isolated("feature_bin", &["--arg", "val"], dir.path());
    let events = load_fixture(&dir.path().join("events/runtime_events.jsonl"));

    // 2. Assert schema conformance of each event
    for event in &events {
        assert_base_fields(event);
        assert!(KNOWN_EVENT_TYPES.contains(
            &event["event_type"].as_str().unwrap()
        ));
    }

    // 3. Verify correlation chain integrity
    let correlation_ids: std::collections::HashSet<_> = events.iter()
        .map(|e| e["correlation_id"].as_str().unwrap())
        .collect();
    assert_eq!(correlation_ids.len(), 1, "all events must share one correlation_id");

    // 4. Store event stream as fixture and re-run to confirm determinism
    // (Determinism check: running with identical inputs produces schema-conformant output;
    // exact replay of JSONL is verified by the replay test suite, not the binary itself)
    let event_types: Vec<_> = events.iter()
        .map(|e| e["event_type"].as_str().unwrap())
        .collect();
    assert_eq!(event_types, EXPECTED_HAPPY_PATH_EVENT_SEQUENCE);
}
```

### What replay tests do NOT do

Replay tests do not mock the binary or intercept internal calls. They invoke the real
binary and inspect the JSONL output. They are integration tests, not unit tests.

---

## Recommended Toolchain/Lint Baseline

This is a **recommendation**, not mandatory project configuration — no Implementation Profile or
Architecture Baseline requires every Rust project to adopt these identically. A reasonable
starting point:

```text
rust-toolchain.toml
Cargo.toml workspace lint policy
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Exact MSRV, edition, and lint strictness stay project-specific decisions — a project may record
its actual choices in its Architecture Baseline (see `dba-system.md` → "Multi-Feature
Architecture Synthesis Gate") when one exists, or simply in its own `rust-toolchain.toml`/
`Cargo.toml` when it doesn't. This pattern does not mandate a specific MSRV, a specific edition,
or `-D warnings` strictness for every project — only that *if* a project wants a baseline, this
is a reasonable Rust-idiomatic starting point.

---

## When NOT to Apply This Pattern

- Single-binary project: no workspace needed; a single `Cargo.toml` at the root is fine
- Single-module project: no shared event crate needed; emit events inline
- Projects in other languages: apply the same conceptual structure (shared event helpers,
  isolated test setup) using that language's idioms — this document is Rust-specific

---

## Relationship to DBA Methodology

This pattern lives in the language pattern layer. It does not change:
- The 9-step DBA loop
- The required base fields for events (those are in `CLAUDE.md`)
- The approval gates at each stage
- The append-only constraint on `events/runtime_events.jsonl`

It only specifies how to organize Rust code to implement what the methodology requires.
