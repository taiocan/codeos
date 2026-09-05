# backend

The backend tier of the Codeos Platform Baseline: axum + sqlx (Postgres), the `backend` service in
`../docker-compose.yml`.

```sh
cargo run                        # needs DATABASE_URL (see .env.example / compose)
cargo fmt --check && cargo clippy --all-targets
cargo test
```

Migrations live in `migrations/` and run automatically at startup (`sqlx migrate`). Add a
migration file per feature that needs persistence — see
`dba/05-guidance/patterns/postgresql-reliability.md` and `rust-project-structure.md` in the
toolkit.
