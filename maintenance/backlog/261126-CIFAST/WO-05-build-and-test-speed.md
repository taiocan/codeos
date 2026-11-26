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

## Result (2026-11-26)

Measured first (CI run 45805764489, full gate 226 s; local warm 313 s). The largest avoidable cost
was not compilation but `verify-configuration-manifest.sh`: 12 s per run, run about five times per
suite (directly, twice in semantic-closure tests, twice in candidate tests), because it recomputes
36 frozen closures one after another.

| Change | Local before | Local after |
|---|---|---|
| Closures computed concurrently, compared in manifest order (same first failure, checked by tamper) | 12.4 s | 1.1 s |
| `semantic-closure-tests.sh` (effect of the above) | 15.7 s | 6.1 s |
| `candidate-tests.sh` (effect of the above) | 56.6 s | 32.9 s |
| Both engine `cargo test` suites run concurrently; either failure fails `run.sh` | 153 s | 106 s |
| `gate.sh full` | 313 s | 217 s |

Not done, with reasons:

- **Shared `CARGO_TARGET_DIR`.** The two lockfiles resolve different versions of serde, syn, libc and
  others, so a shared directory still compiles the workflow engine twice. Compiling it once needs
  one Cargo workspace and one lockfile, which moves the binary paths that release builds and
  downstream toolkits use. The saving (about 8 s in tests, about 20 s in the main-only release
  build) does not justify that. The "compiles once per profile" acceptance is therefore not met.
- **Second-granularity sleeps.** About 12–15 s in total. Removing them needs either future-dated
  commits or a clock override in the engine; both change how evidence timestamps are judged, an
  integrity risk out of proportion to the saving.

No assertion changed.

CI on 963a149 (run 45857333202, success): full gate 168 s (was 226 s), whole run 268 s (was 326 to
392 s). Candidate tests 30 s (was 45 s); the engine suites and remaining scripts 106 s (was 139 s).

Status: accepted by the human 2026-11-26 without the two dropped items.
