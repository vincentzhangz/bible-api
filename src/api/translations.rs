use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::AppError;
use crate::models::{BookResponse, BooksResponse, Translation, TranslationResponse};

#[derive(Debug, Deserialize)]
pub struct BookPathParams {
    pub translation: String,
    pub book: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/translations",
    responses(
        (status = 200, description = "List of translations", body = [TranslationResponse])
    )
)]
pub async fn list_translations(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<TranslationResponse>>, AppError> {
    let translations = sqlx::query_as::<_, Translation>(
        "SELECT id, name, language, license_id, source, json_hash FROM translations ORDER BY name",
    )
    .fetch_all(&pool)
    .await
    .map_err(AppError::Database)?;

    let response: Vec<TranslationResponse> = translations.into_iter().map(Into::into).collect();

    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/translations/{translation}",
    params(
        ("translation" = String, Path, description = "Translation ID")
    ),
    responses(
        (status = 200, description = "Translation details", body = TranslationResponse),
        (status = 404, description = "Translation not found")
    )
)]
pub async fn get_translation(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> Result<Json<TranslationResponse>, AppError> {
    let translation = sqlx::query_as::<_, Translation>(
        "SELECT id, name, language, license_id, source, json_hash FROM translations WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await
    .map_err(AppError::Database)?;

    match translation {
        Some(t) => Ok(Json(t.into())),
        None => Err(AppError::NotFound(format!("translation {} not found", id))),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/translations/{translation}/books",
    params(
        ("translation" = String, Path, description = "Translation ID")
    ),
    responses(
        (status = 200, description = "List of books", body = BooksResponse)
    )
)]
pub async fn list_books(
    State(pool): State<PgPool>,
    Path(translation): Path<String>,
) -> Result<Json<BooksResponse>, AppError> {
    let books = sqlx::query_as::<_, BookResponse>(
        "SELECT b.id, b.name, b.testament FROM books b \
         JOIN chapters c ON b.id = c.book_id \
         WHERE c.translation_id = $1 \
         GROUP BY b.id, b.name, b.testament \
         ORDER BY b.ord",
    )
    .bind(&translation)
    .fetch_all(&pool)
    .await
    .map_err(AppError::Database)?;

    Ok(Json(BooksResponse { translation, books }))
}

#[utoipa::path(
    get,
    path = "/api/v1/translations/{translation}/books/{book}",
    params(
        ("translation" = String, Path, description = "Translation ID"),
        ("book" = String, Path, description = "Book name")
    ),
    responses(
        (status = 200, description = "Chapter numbers", body = Vec<i32>)
    )
)]
pub async fn get_book_chapters(
    State(pool): State<PgPool>,
    Path(params): Path<BookPathParams>,
) -> Result<Json<Vec<i32>>, AppError> {
    let chapters = sqlx::query_as::<_, (i32,)>(
        "SELECT c.chapter_number FROM chapters c \
         JOIN books b ON c.book_id = b.id \
         WHERE c.translation_id = $1 AND LOWER(b.name) = LOWER($2) \
         ORDER BY c.chapter_number",
    )
    .bind(&params.translation)
    .bind(&params.book)
    .fetch_all(&pool)
    .await
    .map_err(AppError::Database)?;

    let chapters: Vec<i32> = chapters.into_iter().map(|(n,)| n).collect();
    Ok(Json(chapters))
}
