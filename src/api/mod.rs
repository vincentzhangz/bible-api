pub mod health;
pub mod search;
pub mod translations;
pub mod verses;
pub mod visualize;

use axum::routing::get;
use axum::Router;
use axum::extract::Extension;
use sqlx::PgPool;
use std::sync::Arc;
use crate::config::env::AppConfig;

pub fn create_routes(pool: PgPool, config: AppConfig) -> Router {
    let app_config = Arc::new(config);

    Router::new()
        .route("/health", get(health::health_check))
        .route("/translations", get(translations::list_translations))
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
        .with_state(pool)
}
