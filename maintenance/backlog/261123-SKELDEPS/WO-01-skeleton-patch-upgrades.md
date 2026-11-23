# WO-SD-01 — Skeleton Patch Upgrades

- **Class:** NORMAL.
- **Path:** `dba/04-tools/initializer/skeleton/backend`.

## Scope

- Update `Cargo.lock`: url 2.5.8 → 2.5.9 (with form_urlencoded 1.2.3) and tokio 1.53.1 → 1.54.1.
- Raise the tokio floor in `Cargo.toml` to `1.54.1`.
- Remove the unused `chrono` feature from sqlx.
- Add one line to `backend/README.md`: chrono is flagged unmaintained (RUSTSEC-2026-0543); a
  feature that needs time types should use a maintained crate that sqlx supports, such as `time`.

## Verification

- `cargo build`, `cargo clippy --all-targets -- -D warnings` and `cargo test`, all with `--locked`.
- `cargo audit` reports only rsa and libsqlite3-sys, and `cargo tree -i` shows that neither is
  built.
- `bash dba/04-tools/initializer/tests/dba-init-tests.sh` passes.
- `docker build` of the skeleton backend succeeds. It binds no ports and does not touch JamFree's
  stack.
- A rebuilt static snapshot shows the skeleton with 2 vulnerabilities (both lockfile only) and
  0 warnings.

## Acceptance

The verification passes and the human accepts.
