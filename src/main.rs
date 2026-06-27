use axum::{
    Json, Router,
    extract::Extension,
    http::header::{HeaderName, HeaderValue},
    middleware as axum_middleware,
    response::Html,
    routing::get,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;

use bible_api::api::middleware::{create_rate_limiter, rate_limit};
use bible_api::api::visualize::{
    self, CharacterInfo, CharacterRelationship, CrossReferenceSource, CrossReferenceTarget,
    cross_references::CrossReferenceResponse, relationships::RelationshipsResponse,
    timeline::TimelineEvent, word_frequency::WordFrequency,
};
use bible_api::api::{search, translations, verses};
use bible_api::config::AppConfig;
use bible_api::ingestion::{VisualizeCache, ingest_all_json_files};
use bible_api::models::{
    BookResponse, BooksResponse, ChapterResponse, TranslationResponse, VerseResponse,
};
use bible_api::{api, db};

#[derive(OpenApi)]
#[openapi(
    paths(
        translations::list_translations,
        translations::get_translation,
        translations::list_books,
        translations::get_book_chapters,
        verses::get_chapter,
        verses::get_verse,
        search::search,
        visualize::word_frequency::word_frequency,
        visualize::cross_references::cross_references,
        visualize::timeline::timeline,
        visualize::relationships::relationships,
    ),
    components(schemas(
        TranslationResponse,
        BookResponse,
        BooksResponse,
        ChapterResponse,
        VerseResponse,
        WordFrequency,
        CrossReferenceSource,
        CrossReferenceTarget,
        CrossReferenceResponse,
        TimelineEvent,
        CharacterInfo,
        CharacterRelationship,
        RelationshipsResponse,
    )),
    info(
        title = "Bible API",
        version = "0.2.0",
        description = "API for browsing and searching the Bible"
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging();

    let config = AppConfig::from_env();

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
    ingest_all_json_files(&pool, &config.data_dir).await?;

    tracing::info!("Loading visualize cache...");
    let visualize_cache = Arc::new(VisualizeCache::load(&config.data_dir).await);

    tracing::info!(
        "Starting API server on {}:{} with rate limit {}/{}",
        config.api_host,
        config.api_port,
        config.rate_limit_per_second,
        config.rate_limit_burst
    );

    let app = build_app(pool.clone(), config.clone(), visualize_cache);

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

fn setup_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn build_app(
    pool: sqlx::PgPool,
    config: AppConfig,
    visualize_cache: Arc<VisualizeCache>,
) -> Router {
    let cors_layer = build_cors_layer(&config);
    let rate_limiter = create_rate_limiter(config.rate_limit_per_second, config.rate_limit_burst);
    let openapi = ApiDoc::openapi();

    Router::new()
        .nest(
            "/api/v1",
            api::create_routes(pool, config, visualize_cache),
        )
        .route(
            "/",
            get(|| async move {
                Html(
                    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Bible API</title>
    <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-gradient-to-br from-slate-900 to-slate-800 min-h-screen flex items-center justify-center text-white">
    <div class="text-center px-6 py-12">
        <h1 class="text-5xl font-bold mb-3">Bible API</h1>
        <p class="text-gray-400 text-lg mb-8">A RESTful API for Bible translations, built with Rust and PostgreSQL</p>
        <a href="/swagger-ui"
           class="inline-block bg-violet-600 hover:bg-violet-700 text-white font-semibold px-8 py-3 rounded-lg transition-colors">
            View Documentation
        </a>
    </div>
</body>
</html>"#,
                )
            }),
        )
        .route(
            "/api-docs/openapi.json",
            get({
                let openapi = openapi.clone();
                move || async move { Json(openapi) }
            }),
        )
        .route(
            "/swagger-ui",
            get(|| async move {
                Html(
                    r##"<!DOCTYPE html>
<html>
<head>
    <title>Bible API - Swagger UI</title>
    <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css" />
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
</head>
<body>
    <div id="swagger-ui"></div>
    <script>
        window.onload = function() {
            SwaggerUIBundle({
                url: "/api-docs/openapi.json",
                dom_id: "#swagger-ui",
                presets: [SwaggerUIBundle.presets.apis, SwaggerUIBundle.SwaggerUIStandalonePreset],
                layout: "BaseLayout"
            });
        };
    </script>
</body>
</html>"##,
                )
            }),
        )
        .layer(axum_middleware::from_fn(rate_limit))
        .layer(axum_middleware::from_fn(api::middleware::request_id))
        .layer(TraceLayer::new_for_http())
        .layer(cors_layer)
        .layer(Extension(rate_limiter))
        .layer(Extension(openapi))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("strict-transport-security"),
            HeaderValue::from_static("max-age=31536000"),
        ))
}

fn build_cors_layer(config: &AppConfig) -> CorsLayer {
    if config.cors_allowed_origins.iter().any(|o| o == "*") {
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
    }
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
