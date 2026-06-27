use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;

use crate::db;
use crate::error::AppError;

#[derive(Debug, Serialize, ToSchema)]
pub struct CrossReferenceTarget {
    pub book: String,
    pub chapter: i32,
    pub verse: i32,
    pub relationship: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CrossReferenceResponse {
    pub source: CrossReferenceSource,
    pub references: Vec<CrossReferenceTarget>,
}

#[derive(Debug, Serialize, ToSchema)]
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

#[utoipa::path(
    get,
    path = "/api/v1/visualize/cross-references/{translation}/{book}/{chapter}/{verse}",
    params(
        ("translation" = String, Path, description = "Translation ID"),
        ("book" = String, Path, description = "Book name"),
        ("chapter" = i32, Path, description = "Chapter number"),
        ("verse" = i32, Path, description = "Verse number")
    ),
    responses(
        (status = 200, description = "Cross-references for verse", body = CrossReferenceResponse)
    )
)]
pub async fn cross_references(
    State(pool): State<PgPool>,
    Path(params): Path<CrossRefPath>,
) -> Result<Json<CrossReferenceResponse>, AppError> {
    let source_result = db::find_verse_info(
        &pool,
        &params.translation,
        &params.book,
        params.chapter,
        params.verse,
    )
    .await?;

    let (source_book, source_chapter, source_verse) = source_result.unwrap_or_else(|| {
        (
            params.book.clone(),
            params.chapter,
            params.verse,
        )
    });

    let verse_id = db::find_verse_id(
        &pool,
        &params.translation,
        &params.book,
        params.chapter,
        params.verse,
    )
    .await?;

    let references_result = match verse_id {
        Some(id) => {
            sqlx::query_as::<_, (String, i32, i32, String)>(
                r#"
                SELECT b.name, c.chapter_number, v.verse_number, COALESCE(cr.relationship_type, 'related')
                FROM cross_references cr
                JOIN verses v ON cr.target_verse_id = v.id
                JOIN chapters c ON v.chapter_id = c.id
                JOIN books b ON c.book_id = b.id
                WHERE cr.source_verse_id = $1
                "#,
            )
            .bind(id)
            .fetch_all(&pool)
            .await
            .map_err(AppError::Database)?
        }
        None => vec![],
    };

    let references: Vec<CrossReferenceTarget> = references_result
        .into_iter()
        .map(
            |(book, chapter, verse, relationship)| CrossReferenceTarget {
                book,
                chapter,
                verse,
                relationship,
            },
        )
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
