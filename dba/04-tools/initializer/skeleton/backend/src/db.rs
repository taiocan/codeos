use sqlx::postgres::{PgPool, PgPoolOptions};

/// Connects using `DATABASE_URL`. Local default matches docker-compose's `db` service so
/// `cargo run` against `docker compose up db` works with no extra configuration.
pub async fn connect() -> anyhow::Result<PgPool> {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://codeos:codeos@localhost:5432/codeos".to_string());
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;
    Ok(pool)
}

/// Applies migrations in `./migrations` at startup. Safe to call every boot: sqlx tracks applied
/// migrations in its own `_sqlx_migrations` table and is a no-op once caught up.
pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}
