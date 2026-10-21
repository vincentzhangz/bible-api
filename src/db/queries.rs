use sqlx::PgPool;

use crate::error::AppError;

pub async fn find_verse_id(
    pool: &PgPool,
    translation_id: &str,
    book: &str,
    chapter: i32,
    verse: i32,
) -> Result<Option<i32>, AppError> {
    let Some(resolved) = crate::db::resolve_book(pool, book).await? else {
        return Ok(None);
    };

    sqlx::query_scalar::<_, i32>(
        r#"
        SELECT v.id FROM verses v
        JOIN chapters c ON v.chapter_id = c.id
        WHERE c.translation_id = $1 AND c.book_id = $2
          AND c.chapter_number = $3 AND v.verse_number = $4
        "#,
    )
    .bind(translation_id)
    .bind(resolved.id)
    .bind(chapter)
    .bind(verse)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn find_verse_info(
    pool: &PgPool,
    translation_id: &str,
    book: &str,
    chapter: i32,
    verse: i32,
) -> Result<Option<(String, i32, i32)>, AppError> {
    let Some(resolved) = crate::db::resolve_book(pool, book).await? else {
        return Ok(None);
    };

    sqlx::query_as::<_, (String, i32, i32)>(
        r#"
        SELECT b.name, c.chapter_number, v.verse_number
        FROM verses v
        JOIN chapters c ON v.chapter_id = c.id
        JOIN books b ON c.book_id = b.id
        WHERE c.translation_id = $1 AND c.book_id = $2
          AND c.chapter_number = $3 AND v.verse_number = $4
        "#,
    )
    .bind(translation_id)
    .bind(resolved.id)
    .bind(chapter)
    .bind(verse)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
}
