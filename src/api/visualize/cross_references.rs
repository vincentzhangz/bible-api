use axum::{extract::{Path, State}, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Serialize)]
pub struct CrossReferenceTarget {
    pub book: String,
    pub chapter: i32,
    pub verse: i32,
    pub relationship: String,
}

#[derive(Debug, Serialize)]
pub struct CrossReferenceResponse {
    pub source: CrossReferenceSource,
    pub references: Vec<CrossReferenceTarget>,
}

#[derive(Debug, Serialize)]
pub struct CrossReferenceSource {
    pub book: String,
    pub chapter: i32,
    pub verse: i32,
}

#[derive(Debug, Deserialize)]
pub struct CrossRefPath {
    pub translation: String,
    pub book: String,
    pub chapter: i32,
    pub verse: i32,
}

pub async fn cross_references(
    State(pool): State<PgPool>,
    Path(params): Path<CrossRefPath>,
) -> Result<Json<CrossReferenceResponse>, (axum::http::StatusCode, String)> {
    let source_result = sqlx::query_as::<_, (String, i32, i32)>(
        r#"
        SELECT b.name, c.chapter_number, v.verse_number
        FROM verses v
        JOIN chapters c ON v.chapter_id = c.id
        JOIN translations t ON c.translation_id = t.id
        JOIN books b ON c.book_id = b.id
        WHERE t.id = $1 AND LOWER(b.name) = LOWER($2) AND c.chapter_number = $3 AND v.verse_number = $4
        "#,
    )
    .bind(&params.translation)
    .bind(&params.book)
    .bind(params.chapter)
    .bind(params.verse)
    .fetch_optional(&pool)
    .await
    .map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Database error".to_string(),
        )
    })?;

    let (source_book, source_chapter, source_verse) = source_result.unwrap_or_else(|| {
        (params.book.clone(), params.chapter, params.verse)
    });

    let references_result = sqlx::query_as::<_, (String, i32, i32, String)>(
        r#"
        SELECT b.name, c.chapter_number, v.verse_number, COALESCE(cr.relationship_type, 'related')
        FROM cross_references cr
        JOIN verses v ON cr.target_verse_id = v.id
        JOIN chapters c ON v.chapter_id = c.id
        JOIN books b ON c.book_id = b.id
        WHERE cr.source_verse_id = (
            SELECT v.id FROM verses v
            JOIN chapters c ON v.chapter_id = c.id
            JOIN translations t ON c.translation_id = t.id
            JOIN books b ON c.book_id = b.id
            WHERE t.id = $1 AND LOWER(b.name) = LOWER($2) AND c.chapter_number = $3 AND v.verse_number = $4
        )
        "#,
    )
    .bind(&params.translation)
    .bind(&params.book)
    .bind(params.chapter)
    .bind(params.verse)
    .fetch_all(&pool)
    .await
    .map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Database error".to_string(),
        )
    })?;

    let references: Vec<CrossReferenceTarget> = references_result
        .into_iter()
        .map(|(book, chapter, verse, relationship)| CrossReferenceTarget {
            book,
            chapter,
            verse,
            relationship,
        })
        .collect();

    Ok(Json(CrossReferenceResponse {
        source: CrossReferenceSource {
            book: source_book,
            chapter: source_chapter,
            verse: source_verse,
        },
        references,
    }))
}
