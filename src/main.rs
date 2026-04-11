mod api;
mod config;
mod db;
mod ingestion;
mod models;

use axum::Router;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::AppConfig::from_env();

    tracing::info!("Connecting to database...");
    let pool = db::create_pool(&config.database_url).await?;

    tracing::info!("Running migrations...");
    db::run_migrations(&pool).await?;

    tracing::info!("Ingesting JSON files...");
    ingestion::ingest_all_json_files(&pool, &config.data_dir).await?;

    tracing::info!("Starting API server on {}:{}", config.api_host, config.api_port);

    let app = Router::new()
        .nest("/api/v1", api::create_routes(pool.clone(), config.clone()))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    let addr: SocketAddr = format!("{}:{}", config.api_host, config.api_port)
        .parse()
        .expect("Invalid address");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
