use axum::{
    Json,
    extract::{Extension, Query, State},
};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use utoipa::IntoParams;

use crate::config::env::AppConfig;
use crate::error::AppError;
use crate::models::SearchResult;

#[derive(Debug, Deserialize, IntoParams)]
pub struct SearchQuery {
    pub q: String,
    pub translation: Option<String>,
}

/// Searches verses by query text.
#[utoipa::path(
    get,
    path = "/api/v1/search",
    params(
        SearchQuery
    ),
    responses(
        (status = 200, description = "Search results", body = Vec<SearchResult>)
    )
)]
pub async fn search(
    State(pool): State<PgPool>,
    Extension(config): Extension<Arc<AppConfig>>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<SearchResult>>, AppError> {
    let query = &params.q;
    let limit = config.search_limit;

    let results = if let Some(ref translation_id) = params.translation {
        sqlx::query_as::<_, (String, String, i32, i32, String)>(&format!(
            r#"
                SELECT t.id, b.name, c.chapter_number, v.verse_number, v.text
                FROM verses v
                JOIN chapters c ON v.chapter_id = c.id
                JOIN translations t ON c.translation_id = t.id
                JOIN books b ON c.book_id = b.id
                WHERE t.id = $1 AND to_tsvector('english', v.text) @@ plainto_tsquery('english', $2)
                ORDER BY ts_rank(to_tsvector('english', v.text), plainto_tsquery('english', $2))
                LIMIT {}
                "#,
            limit
        ))
        .bind(translation_id)
        .bind(query)
        .fetch_all(&pool)
        .await
    } else {
        sqlx::query_as::<_, (String, String, i32, i32, String)>(&format!(
            r#"
                SELECT t.id, b.name, c.chapter_number, v.verse_number, v.text
                FROM verses v
                JOIN chapters c ON v.chapter_id = c.id
                JOIN translations t ON c.translation_id = t.id
                JOIN books b ON c.book_id = b.id
                WHERE to_tsvector('english', v.text) @@ plainto_tsquery('english', $1)
                ORDER BY ts_rank(to_tsvector('english', v.text), plainto_tsquery('english', $1))
                LIMIT {}
                "#,
            limit
        ))
        .bind(query)
        .fetch_all(&pool)
        .await
    }
    .map_err(AppError::Database)?;

    let search_results: Vec<SearchResult> = results
        .into_iter()
        .map(|(translation, book, chapter, verse, text)| SearchResult {
            translation,
            book,
            chapter,
            verse,
            text,
        })
        .collect();

    Ok(Json(search_results))
}
