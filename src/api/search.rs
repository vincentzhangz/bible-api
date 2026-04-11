use axum::{extract::{Query, State}, Json};
use serde::Deserialize;
use sqlx::PgPool;

use crate::models::SearchResult;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub translation: Option<String>,
}

pub async fn search(
    State(pool): State<PgPool>,
    Query(params): Query<SearchQuery>,
) -> Json<Vec<SearchResult>> {
    let query = params.q.clone();

    let results = if let Some(translation_id) = params.translation {
        sqlx::query_as::<_, (String, String, i32, i32, String)>(
            r#"
            SELECT t.id, b.name, c.chapter_number, v.verse_number, v.text
            FROM verses v
            JOIN chapters c ON v.chapter_id = c.id
            JOIN translations t ON c.translation_id = t.id
            JOIN books b ON c.book_id = b.id
            WHERE t.id = $1 AND to_tsvector('english', v.text) @@ plainto_tsquery('english', $2)
            ORDER BY ts_rank(to_tsvector('english', v.text), plainto_tsquery('english', $2))
            LIMIT 50
            "#,
        )
        .bind(&translation_id)
        .bind(&query)
        .fetch_all(&pool)
        .await
        .unwrap_or_default()
    } else {
        sqlx::query_as::<_, (String, String, i32, i32, String)>(
            r#"
            SELECT t.id, b.name, c.chapter_number, v.verse_number, v.text
            FROM verses v
            JOIN chapters c ON v.chapter_id = c.id
            JOIN translations t ON c.translation_id = t.id
            JOIN books b ON c.book_id = b.id
            WHERE to_tsvector('english', v.text) @@ plainto_tsquery('english', $1)
            ORDER BY ts_rank(to_tsvector('english', v.text), plainto_tsquery('english', $1))
            LIMIT 50
            "#,
        )
        .bind(&query)
        .fetch_all(&pool)
        .await
        .unwrap_or_default()
    };

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

    Json(search_results)
}
