use sqlx::PgPool;

use crate::canonical::CANONICAL_BOOKS;
use crate::error::AppError;
use crate::util::slugify;

/// Seeds the shared `books` table with the canonical 66 books.
///
/// The id of a book is its canonical position (1-based), which is what the
/// ingestion uses to line translations up with each other.
pub async fn seed_canonical_books(pool: &PgPool) -> Result<(), sqlx::Error> {
    for (index, book) in CANONICAL_BOOKS.iter().enumerate() {
        let id = (index + 1) as i32;

        sqlx::query(
            "INSERT INTO books (id, name, slug, testament, ord) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT (id) DO UPDATE SET \
             name = EXCLUDED.name, slug = EXCLUDED.slug, \
             testament = EXCLUDED.testament, ord = EXCLUDED.ord",
        )
        .bind(id)
        .bind(book.name)
        .bind(slugify(book.name))
        .bind(book.testament)
        .bind(id)
        .execute(pool)
        .await?;
    }

    sqlx::query("SELECT setval('books_id_seq', $1)")
        .bind(CANONICAL_BOOKS.len() as i64)
        .execute(pool)
        .await?;

    Ok(())
}

/// A book resolved from either its display name or its slug.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedBook {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub testament: String,
    pub ord: i32,
}

/// Resolves a book by display name *or* slug, case-insensitively.
///
/// Clients may address books as `"John"`, `"john"` or `"1-samuel"`; the slug
/// form is what the UI puts in URLs. A translation that renames a book (the
/// Indonesian Terjemahan Baru calls Genesis "Kejadian") is also understood.
pub async fn resolve_book(pool: &PgPool, book: &str) -> Result<Option<ResolvedBook>, AppError> {
    let slug = slugify(book);

    let row = sqlx::query_as::<_, (i32, String, String, String, i32)>(
        r#"
        SELECT id, name, slug, testament, ord
        FROM books
        WHERE LOWER(name) = LOWER($1) OR slug = $2
        LIMIT 1
        "#,
    )
    .bind(book)
    .bind(&slug)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    if let Some((id, name, slug, testament, ord)) = row {
        return Ok(Some(ResolvedBook {
            id,
            name,
            slug,
            testament,
            ord,
        }));
    }

    // Fall back to the canonical aliases, which cover the names other
    // translations use for a book.
    let Some((_, canonical_book)) = crate::canonical::find_by_name(book) else {
        return Ok(None);
    };

    let canonical_slug = slugify(canonical_book.name);

    let row = sqlx::query_as::<_, (i32, String, String, String, i32)>(
        r#"
        SELECT id, name, slug, testament, ord
        FROM books
        WHERE slug = $1
        LIMIT 1
        "#,
    )
    .bind(&canonical_slug)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(row.map(|(id, name, slug, testament, ord)| ResolvedBook {
        id,
        name,
        slug,
        testament,
        ord,
    }))
}

/// Returns the slug for a book reference without touching the database.
pub fn book_slug(reference: &str) -> String {
    slugify(reference)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn book_slug_matches_display_names() {
        assert_eq!(book_slug("1 Samuel"), "1-samuel");
        assert_eq!(book_slug("John"), "john");
    }
}
