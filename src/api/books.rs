use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use sqlx::PgPool;

use crate::db;
use crate::error::AppError;
use crate::models::{BookDetailResponse, BookSummary, BooksListResponse};

/// Canonical counts, computed per translation and then collapsed with MAX so
/// that a book never gains chapters just because more than one translation
/// happens to be loaded.
const BOOK_COUNTS: &str = r#"
WITH per_translation AS (
    SELECT
        c.translation_id,
        c.book_id,
        COUNT(DISTINCT c.chapter_number) AS chapters,
        COUNT(DISTINCT (c.chapter_number, v.verse_number)) AS verses
    FROM chapters c
    JOIN verses v ON v.chapter_id = c.id
    GROUP BY c.translation_id, c.book_id
)
SELECT
    b.id,
    b.name,
    b.slug,
    b.testament,
    b.ord,
    COALESCE(MAX(pt.chapters), 0)::int AS chapter_count,
    COALESCE(MAX(pt.verses), 0)::int AS verse_count
FROM books b
LEFT JOIN per_translation pt ON pt.book_id = b.id
GROUP BY b.id, b.name, b.slug, b.testament, b.ord
ORDER BY b.ord
"#;

#[utoipa::path(
    get,
    path = "/api/v1/books",
    responses(
        (status = 200, description = "The canonical list of books", body = BooksListResponse)
    )
)]
pub async fn list_books(State(pool): State<PgPool>) -> Result<Json<BooksListResponse>, AppError> {
    let rows = sqlx::query_as::<_, (i32, String, String, String, i32, i32, i32)>(BOOK_COUNTS)
        .fetch_all(&pool)
        .await
        .map_err(AppError::Database)?;

    let books = rows
        .into_iter()
        .map(
            |(id, name, slug, testament, ord, chapter_count, verse_count)| BookSummary {
                id,
                name,
                slug,
                testament,
                ord,
                chapter_count,
                verse_count,
            },
        )
        .collect();

    Ok(Json(BooksListResponse { books }))
}

#[derive(Debug, Deserialize)]
pub struct BookPath {
    pub book: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/books/{book}",
    params(
        ("book" = String, Path, description = "Book name or slug")
    ),
    responses(
        (status = 200, description = "A single book", body = BookDetailResponse),
        (status = 404, description = "Book not found")
    )
)]
pub async fn get_book(
    State(pool): State<PgPool>,
    Path(params): Path<BookPath>,
) -> Result<Json<BookDetailResponse>, AppError> {
    let resolved = db::resolve_book(&pool, &params.book)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("book {} not found", params.book)))?;

    let chapters = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT DISTINCT c.chapter_number
        FROM chapters c
        WHERE c.book_id = $1
        ORDER BY c.chapter_number
        "#,
    )
    .bind(resolved.id)
    .fetch_all(&pool)
    .await
    .map_err(AppError::Database)?;

    let counts = sqlx::query_as::<_, (i32, i32)>(
        r#"
        SELECT
            COUNT(DISTINCT c.chapter_number)::int,
            COUNT(DISTINCT (c.chapter_number, v.verse_number))::int
        FROM chapters c
        JOIN verses v ON v.chapter_id = c.id
        WHERE c.book_id = $1
        "#,
    )
    .bind(resolved.id)
    .fetch_one(&pool)
    .await
    .map_err(AppError::Database)?;

    Ok(Json(BookDetailResponse {
        id: resolved.id,
        name: resolved.name,
        slug: resolved.slug,
        testament: resolved.testament,
        ord: resolved.ord,
        chapter_count: counts.0,
        verse_count: counts.1,
        chapters,
    }))
}

/// Shared helper so other handlers can reuse the canonical counts query.
pub async fn book_summaries(pool: &PgPool) -> Result<Vec<BookSummary>, AppError> {
    let rows = sqlx::query_as::<_, (i32, String, String, String, i32, i32, i32)>(BOOK_COUNTS)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

    Ok(rows
        .into_iter()
        .map(
            |(id, name, slug, testament, ord, chapter_count, verse_count)| BookSummary {
                id,
                name,
                slug,
                testament,
                ord,
                chapter_count,
                verse_count,
            },
        )
        .collect())
}
