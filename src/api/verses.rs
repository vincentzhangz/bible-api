use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::AppError;
use crate::models::{ChapterResponse, VerseResponse};

#[derive(Debug, Deserialize)]
pub struct VersePathParams {
    pub translation: String,
    pub book: String,
    pub chapter: i32,
    pub verse: i32,
}

/// Gets a specific verse by translation, book, chapter, and verse number.
pub async fn get_verse(
    State(pool): State<PgPool>,
    Path(params): Path<VersePathParams>,
) -> Result<Json<VerseResponse>, AppError> {
    let result = sqlx::query_as::<_, (String, String, i32, i32, String)>(
        r#"
        SELECT t.id, b.name, c.chapter_number, v.verse_number, v.text
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
    .map_err(AppError::Database)?;

    match result {
        Some((translation, book, chapter, verse_num, text)) => Ok(Json(VerseResponse {
            translation,
            book,
            chapter,
            verse: verse_num,
            text,
        })),
        None => Err(AppError::NotFound("Verse not found".to_string())),
    }
}

#[derive(Debug, Deserialize)]
pub struct ChapterPathParams {
    pub translation: String,
    pub book: String,
    pub chapter: i32,
}

/// Gets all verses in a chapter by translation, book, and chapter number.
pub async fn get_chapter(
    State(pool): State<PgPool>,
    Path(params): Path<ChapterPathParams>,
) -> Result<Json<ChapterResponse>, AppError> {
    let result = sqlx::query_as::<_, (String, String, i32)>(
        r#"
        SELECT t.id, b.name, c.chapter_number
        FROM chapters c
        JOIN translations t ON c.translation_id = t.id
        JOIN books b ON c.book_id = b.id
        WHERE t.id = $1 AND LOWER(b.name) = LOWER($2) AND c.chapter_number = $3
        "#,
    )
    .bind(&params.translation)
    .bind(&params.book)
    .bind(params.chapter)
    .fetch_optional(&pool)
    .await
    .map_err(AppError::Database)?;

    let (translation, book, chapter) = match result {
        Some(r) => r,
        None => return Err(AppError::NotFound("Chapter not found".to_string())),
    };

    let verses = sqlx::query_as::<_, (i32, String)>(
        "SELECT verse_number, text FROM verses WHERE chapter_id = (SELECT id FROM chapters WHERE translation_id = $1 AND book_id = (SELECT id FROM books WHERE LOWER(name) = LOWER($2)) AND chapter_number = $3) ORDER BY verse_number"
    )
    .bind(&params.translation)
    .bind(&params.book)
    .bind(params.chapter)
    .fetch_all(&pool)
    .await
    .map_err(AppError::Database)?;

    let verse_responses: Vec<VerseResponse> = verses
        .into_iter()
        .map(|(verse_num, text)| VerseResponse {
            translation: translation.clone(),
            book: book.clone(),
            chapter,
            verse: verse_num,
            text,
        })
        .collect();

    Ok(Json(ChapterResponse {
        translation,
        book,
        chapter,
        verses: verse_responses,
    }))
}
