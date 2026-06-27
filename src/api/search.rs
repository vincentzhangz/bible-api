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
use crate::models::{SearchResult, VerseResponse};

#[derive(Debug, Deserialize, IntoParams)]
pub struct SearchQuery {
    pub q: String,
    pub translation: Option<String>,
}

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
    let limit = config.search_limit as i64;

    let mut sql = String::from(
        r#"
        SELECT t.id, b.name, c.chapter_number, v.verse_number, v.text
        FROM verses v
        JOIN chapters c ON v.chapter_id = c.id
        JOIN translations t ON c.translation_id = t.id
        JOIN books b ON c.book_id = b.id
        WHERE to_tsvector('english', v.text) @@ plainto_tsquery('english', $1)
        "#,
    );

    if params.translation.is_some() {
        sql.push_str(" AND t.id = $2");
    }

    sql.push_str(
        r#"
        ORDER BY ts_rank(to_tsvector('english', v.text), plainto_tsquery('english', $1))
        LIMIT $3
        "#,
    );

    let mut q = sqlx::query_as::<_, (String, String, i32, i32, String)>(&sql).bind(query);

    if let Some(ref translation_id) = params.translation {
        q = q.bind(translation_id);
    }

    q = q.bind(limit);

    let results = q.fetch_all(&pool).await.map_err(AppError::Database)?;

    let search_results: Vec<SearchResult> = results.into_iter().map(VerseResponse::from).collect();

    Ok(Json(search_results))
}
