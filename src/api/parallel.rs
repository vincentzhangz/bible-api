use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::IntoParams;

use crate::db;
use crate::error::AppError;
use crate::models::{ParallelResponse, ParallelTranslation, VerseText};

#[derive(Debug, Deserialize, IntoParams)]
pub struct ParallelQuery {
    /// Comma-separated translation IDs, e.g. `en-kjv,id-indo_tb`.
    pub translations: String,
}

#[derive(Debug, Deserialize)]
pub struct ParallelPath {
    pub book: String,
    pub chapter: i32,
}

#[utoipa::path(
    get,
    path = "/api/v1/parallel/{book}/{chapter}",
    params(
        ("book" = String, Path, description = "Book name or slug"),
        ("chapter" = i32, Path, description = "Chapter number"),
        ParallelQuery
    ),
    responses(
        (status = 200, description = "The chapter rendered in several translations", body = ParallelResponse),
        (status = 404, description = "Book not found")
    )
)]
pub async fn parallel(
    State(pool): State<PgPool>,
    Path(path): Path<ParallelPath>,
    Query(query): Query<ParallelQuery>,
) -> Result<Json<ParallelResponse>, AppError> {
    let requested: Vec<String> = query
        .translations
        .split(',')
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect();

    if requested.is_empty() {
        return Err(AppError::BadRequest(
            "at least one translation must be requested".to_string(),
        ));
    }

    let resolved = db::resolve_book(&pool, &path.book)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("book {} not found", path.book)))?;

    let mut translations = Vec::new();
    let mut reference_verses: Vec<i32> = Vec::new();

    for translation_id in requested {
        let meta = sqlx::query_as::<_, (String, String)>(
            "SELECT t.name, t.language FROM translations t WHERE t.id = $1",
        )
        .bind(&translation_id)
        .fetch_optional(&pool)
        .await
        .map_err(AppError::Database)?;

        let Some((name, language)) = meta else {
            continue;
        };

        let rows = sqlx::query_as::<_, (i32, String)>(
            r#"
            SELECT v.verse_number, v.text
            FROM verses v
            JOIN chapters c ON v.chapter_id = c.id
            WHERE c.translation_id = $1
              AND c.book_id = $2
              AND c.chapter_number = $3
            ORDER BY v.verse_number
            "#,
        )
        .bind(&translation_id)
        .bind(resolved.id)
        .bind(path.chapter)
        .fetch_all(&pool)
        .await
        .map_err(AppError::Database)?;

        if rows.is_empty() {
            continue;
        }

        let verses: Vec<VerseText> = rows
            .into_iter()
            .map(|(verse, text)| VerseText { verse, text })
            .collect();

        for verse in &verses {
            if !reference_verses.contains(&verse.verse) {
                reference_verses.push(verse.verse);
            }
        }

        translations.push(ParallelTranslation {
            id: translation_id,
            name,
            language,
            book_name: Some(resolved.name.clone()),
            verses,
        });
    }

    reference_verses.sort_unstable();

    Ok(Json(ParallelResponse {
        book: resolved.name,
        book_slug: resolved.slug,
        chapter: path.chapter,
        reference_verses,
        translations,
    }))
}
