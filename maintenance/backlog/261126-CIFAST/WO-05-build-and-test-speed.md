# WO-CIF-05 — Build Reuse and Slow Tests

- **Class:** NORMAL. Each change must keep what every test verifies.
- **Goal:** compile each engine once per profile, and remove avoidable wall-clock time from tests.

## Evidence (runs 45767891878 and 45776215740)

- The reviewer engine compiles the workflow engine again in its own target directory.
- `candidate-tests.sh` takes about 66 s and `semantic-closure-tests.sh` about 30 s; the latter
  copies the repository into a scratch git repository first.
- Several integration tests sleep over a second so that recorded timestamps differ (for example
  `workflow/engine/tests/fai_fixture/mod.rs`, "committed after the acceptance second").

## Scope

- One shared `CARGO_TARGET_DIR` for both engines in the gate and CI, provided the release build
  and `verify-qualified-binary.sh` still find and verify the binaries they expect.
- Replace second-granularity sleeps with an injectable clock or explicit timestamps where the test
  only needs distinct times.
- Measure `semantic-closure-tests.sh` and `candidate-tests.sh`; remove copying or rebuilding that
  their assertions do not need.

## Acceptance

- The workflow engine compiles once per profile per run.
- `gate.sh full` time is recorded before and after, and every test still asserts what it did.
