# WO-SD-04 — sqlx 0.9 in the Skeleton (Deferred)

- **Class:** deferred.
- **Goal:** remove the lockfile-only rsa and libsqlite3-sys advisories, which sqlx 0.8's optional
  drivers bring into `Cargo.lock`.
- **Trigger:** the host toolchain is at least Rust 1.94 (sqlx 0.9.1 declares 1.94), and there is a
  reason to move, such as a compiled advisory or a needed feature.
- **Before starting:** confirm which libsqlite3-sys and rsa versions sqlx 0.9 locks, and record the
  migration changes the skeleton source needs.
