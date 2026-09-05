mod db;

use axum::{Json, Router, routing::get};
use serde::Serialize;
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[derive(Serialize)]
struct Health {
    status: &'static str,
    database: &'static str,
}

async fn health(pool: axum::extract::State<sqlx::PgPool>) -> Json<Health> {
    let database = match sqlx::query("SELECT 1").execute(&*pool).await {
        Ok(_) => "connected",
        Err(_) => "unreachable",
    };
    Json(Health {
        status: "ok",
        database,
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let pool = db::connect().await?;
    db::run_migrations(&pool).await?;

    let app = Router::new()
        .route("/health", get(health))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(pool);

    let addr: SocketAddr = std::env::var("BACKEND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse()?;
    tracing::info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
