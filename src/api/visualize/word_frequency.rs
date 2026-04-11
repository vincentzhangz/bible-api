use axum::{
    Json,
    extract::{Extension, Path, State},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;

use crate::config::env::AppConfig;
use crate::error::AppError;

#[derive(Debug, Serialize)]
pub struct WordFrequency {
    pub word: String,
    pub count: i64,
}

#[derive(Debug, Deserialize)]
pub struct WordFrequencyPath {
    pub translation: String,
    pub book: String,
}

/// Gets word frequency analysis for a book.
pub async fn word_frequency(
    State(pool): State<PgPool>,
    Extension(config): Extension<Arc<AppConfig>>,
    Path(params): Path<WordFrequencyPath>,
) -> Result<Json<Vec<WordFrequency>>, AppError> {
    let limit = config.word_frequency_limit;
    let results = sqlx::query_as::<_, (String, i64)>(&format!(
        r#"
            SELECT word, count
            FROM (
                SELECT unnest(string_to_array(lower(v.text), ' ')) as word
                FROM verses v
                JOIN chapters c ON v.chapter_id = c.id
                JOIN translations t ON c.translation_id = t.id
                JOIN books b ON c.book_id = b.id
                WHERE t.id = $1 AND LOWER(b.name) = LOWER($2)
            ) words
            WHERE length(word) > 3
            GROUP BY word
            ORDER BY count DESC
            LIMIT {}
            "#,
        limit
    ))
    .bind(&params.translation)
    .bind(&params.book)
    .fetch_all(&pool)
    .await
    .map_err(AppError::Database)?;

    let frequency: Vec<WordFrequency> = results
        .into_iter()
        .map(|(word, count)| WordFrequency { word, count })
        .collect();

    Ok(Json(frequency))
}
