pub mod health;
pub mod middleware;
pub mod search;
pub mod translations;
pub mod verses;
pub mod visualize;

use crate::config::env::AppConfig;
use crate::ingestion::VisualizeCache;
use axum::Router;
use axum::extract::Extension;
use axum::routing::get;
use sqlx::PgPool;
use std::sync::Arc;

pub fn create_routes(
    pool: PgPool,
    config: AppConfig,
    visualize_cache: Arc<VisualizeCache>,
) -> Router {
    let app_config = Arc::new(config);

    Router::new()
        .route("/health", get(health::health_check))
        .route("/health/live", get(health::liveness))
        .route("/health/ready", get(health::readiness))
        .route("/translations", get(translations::list_translations))
        .route(
            "/translations/{translation}",
            get(translations::get_translation),
        )
        .route(
            "/translations/{translation}/books",
            get(translations::list_books),
        )
        .route(
            "/translations/{translation}/books/{book}",
            get(translations::get_book_chapters),
        )
        .route(
            "/translations/{translation}/books/{book}/chapters/{chapter}/verses/{verse}",
            get(verses::get_verse),
        )
        .route(
            "/translations/{translation}/books/{book}/chapters/{chapter}",
            get(verses::get_chapter),
        )
        .route("/search", get(search::search))
        .route(
            "/visualize/word-frequency/{translation}/{book}",
            get(visualize::word_frequency),
        )
        .route(
            "/visualize/cross-references/{translation}/{book}/{chapter}/{verse}",
            get(visualize::cross_references),
        )
        .route(
            "/visualize/timeline/{translation}",
            get(visualize::timeline),
        )
        .route(
            "/visualize/relationships/{translation}/{book}",
            get(visualize::relationships),
        )
        .layer(Extension(app_config))
        .layer(Extension(visualize_cache))
        .with_state(pool)
}
