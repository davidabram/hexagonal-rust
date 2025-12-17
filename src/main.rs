#![allow(dead_code)]

mod adapters;
mod domain;
mod observability;
mod ports;
mod services;

use anyhow::Context;
use axum::{
    routing::{get, post},
    Router,
};
use sqlx::sqlite::SqlitePoolOptions;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing::info;

use adapters::inbound::http::{create_subscription_handler, health_check_handler, AppState};
use adapters::outbound::sqlite::{
    SqliteBillingProfileRepository, SqlitePlanRepository, SqliteSubscriptionRepository,
};
use observability::{init_observability, shutdown_tracer, ObservabilityConfig};
use services::SubscriptionService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let otel_config = ObservabilityConfig::from_env();
    let _guard = init_observability(otel_config)?;

    let database_url =
        std::env::var("DATABASE_URL").context("DATABASE_URL environment variable not set")?;

    let _host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .context("PORT must be a valid u16")?;

    info!(database_url = %database_url, "connecting to database");

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .context("failed to connect to database")?;

    info!("running database migrations");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("failed to run database migrations")?;

    let plan_repo = SqlitePlanRepository::new(pool.clone());
    let billing_repo = SqliteBillingProfileRepository::new(pool.clone());
    let subscription_repo = SqliteSubscriptionRepository::new(pool.clone());

    let subscription_service = SubscriptionService::new(plan_repo, billing_repo, subscription_repo);

    let state = AppState::new(subscription_service);

    let app = Router::new()
        .route("/health", get(health_check_handler))
        .route("/api/subscriptions", post(create_subscription_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!(address = %addr, "starting HTTP server");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context("failed to bind to address")?;

    let result = axum::serve(listener, app).await.context("server error");

    shutdown_tracer();

    result
}
