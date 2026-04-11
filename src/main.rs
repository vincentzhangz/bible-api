mod api;
mod config;
mod db;
mod error;
mod ingestion;
mod models;

use axum::{
    Router,
    extract::Extension,
    http::header::{HeaderName, HeaderValue},
    middleware as axum_middleware,
};
use std::net::SocketAddr;
use tower_http::{
    cors::{Any, CorsLayer},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use api::middleware::{create_rate_limiter, rate_limit};

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
    let pool = db::create_pool(
        &config.database_url,
        config.db_max_connections,
        config.db_acquire_timeout_secs,
    )
    .await?;

    tracing::info!("Running migrations...");
    db::run_migrations(&pool).await?;

    tracing::info!("Ingesting JSON files...");
    ingestion::ingest_all_json_files(&pool, &config.data_dir).await?;

    tracing::info!(
        "Starting API server on {}:{} with rate limit {}/{}",
        config.api_host,
        config.api_port,
        config.rate_limit_per_second,
        config.rate_limit_burst
    );

    let cors_layer = if config.cors_allowed_origins.iter().any(|o| o == "*") {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        let origins: Vec<_> = config
            .cors_allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(Any)
            .allow_headers(Any)
    };

    let rate_limiter = create_rate_limiter(config.rate_limit_per_second, config.rate_limit_burst);

    let x_content_type_options = HeaderName::from_static("x-content-type-options");
    let x_frame_options = HeaderName::from_static("x-frame-options");
    let strict_transport_security = HeaderName::from_static("strict-transport-security");

    let app = Router::new()
        .nest("/api/v1", api::create_routes(pool.clone(), config.clone()))
        .layer(axum_middleware::from_fn(rate_limit))
        .layer(axum_middleware::from_fn(api::middleware::request_id))
        .layer(TraceLayer::new_for_http())
        .layer(cors_layer)
        .layer(Extension(rate_limiter))
        .layer(SetResponseHeaderLayer::if_not_present(
            x_content_type_options,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            x_frame_options,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            strict_transport_security,
            HeaderValue::from_static("max-age=31536000"),
        ));

    let addr: SocketAddr = format!("{}:{}", config.api_host, config.api_port)
        .parse()
        .expect("Invalid address");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();

    #[cfg(unix)]
    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("Failed to install SIGTERM handler");

    #[cfg(unix)]
    {
        tokio::select! {
            _ = ctrl_c => {},
            _ = term.recv() => {},
        }
    }

    #[cfg(not(unix))]
    {
        ctrl_c.await.ok();
    }

    tracing::info!("Shutdown signal received, gracefully stopping server...");
}
