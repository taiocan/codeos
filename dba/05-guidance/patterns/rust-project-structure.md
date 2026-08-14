---
component_question: What minimal Rust structures help implement and verify DBA features?
out_of_scope: Core DBA semantics, mandatory layouts, project architecture, and non-Rust guidance.
---

# Pattern: Rust Project Structure

Consult this advisory pattern when an approved Implementation Profile resolves to Rust. Approved
behavior, applicable architecture, and the existing project structure remain authoritative.

## Use the Smallest Native Structure

- Keep a single package for a single cohesive binary or library.
- Use a Cargo workspace when the project has multiple independently built crates.
- Introduce shared event or test support only after at least two consumers need the same mechanical
  behavior.
- Prefer an existing shared crate with a test-support feature over a new crate for a small helper
  set.

Shared code must remain mechanical. Event serialization, correlation propagation, fixture loading,
and isolated temporary-project setup are suitable; feature decisions and domain aggregation are
not. Apply the shared-infrastructure boundary pattern when multiple feature crates use a hub.

## Verification

Use project-native commands. A typical workspace baseline is:

```text
cargo fmt --check
cargo clippy --workspace --all-targets
cargo test --workspace
```

Exact edition, MSRV, lint strictness, crate boundaries, CI, and runtime layout are project choices,
not requirements of this pattern.
