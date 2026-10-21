use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use sqlx::PgPool;

use crate::db;
use crate::error::AppError;
use crate::models::{CrossReference, NeighborPassage, VerseContextResponse, VerseText};

/// How many thematic (shared-vocabulary) references to return.
const THEMATIC_LIMIT: i64 = 12;

/// How many curated references to return.
const CURATED_LIMIT: i64 = 12;

#[derive(Debug, Deserialize)]
pub struct VerseContextPath {
    pub translation: String,
    pub book: String,
    pub chapter: i32,
    pub verse: i32,
}

#[utoipa::path(
    get,
    path = "/api/v1/translations/{translation}/books/{book}/chapters/{chapter}/verses/{verse}/context",
    params(
        ("translation" = String, Path, description = "Translation ID"),
        ("book" = String, Path, description = "Book name or slug"),
        ("chapter" = i32, Path, description = "Chapter number"),
        ("verse" = i32, Path, description = "Verse number")
    ),
    responses(
        (status = 200, description = "Everything known about a verse", body = VerseContextResponse),
        (status = 404, description = "Verse not found")
    )
)]
pub async fn verse_context(
    State(pool): State<PgPool>,
    Path(params): Path<VerseContextPath>,
) -> Result<Json<VerseContextResponse>, AppError> {
    let resolved = db::resolve_book(&pool, &params.book)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("book {} not found", params.book)))?;

    let slug = resolved.slug.clone();

    let display_name = sqlx::query_scalar::<_, String>(
        "SELECT name FROM translation_book_names WHERE translation_id = $1 AND book_id = $2",
    )
    .bind(&params.translation)
    .bind(resolved.id)
    .fetch_optional(&pool)
    .await
    .map_err(AppError::Database)?;

    let book_name = display_name.unwrap_or_else(|| resolved.name.clone());

    let verse_row = sqlx::query_as::<_, (String, i32, i32)>(
        r#"
        SELECT v.text, c.chapter_number, v.verse_number
        FROM verses v
        JOIN chapters c ON v.chapter_id = c.id
        WHERE c.translation_id = $1
          AND c.book_id = $2
          AND c.chapter_number = $3
          AND v.verse_number = $4
        "#,
    )
    .bind(&params.translation)
    .bind(resolved.id)
    .bind(params.chapter)
    .bind(params.verse)
    .fetch_optional(&pool)
    .await
    .map_err(AppError::Database)?;

    let Some((text, chapter, verse)) = verse_row else {
        return Err(AppError::NotFound("Verse not found".to_string()));
    };

    let chapter_rows = sqlx::query_as::<_, (i32, String)>(
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
    .bind(&params.translation)
    .bind(resolved.id)
    .bind(chapter)
    .fetch_all(&pool)
    .await
    .map_err(AppError::Database)?;

    let chapter_verses: Vec<VerseText> = chapter_rows
        .into_iter()
        .map(|(verse, text)| VerseText { verse, text })
        .collect();

    let previous = neighbor(
        &pool,
        &params.translation,
        resolved.id,
        resolved.ord,
        chapter,
        verse,
        -1,
    )
    .await?;
    let next = neighbor(
        &pool,
        &params.translation,
        resolved.id,
        resolved.ord,
        chapter,
        verse,
        1,
    )
    .await?;

    let mut cross_references =
        curated_references(&pool, &params.translation, resolved.id, chapter, verse).await?;
    let thematic = thematic_references(&pool, &text, resolved.id, chapter, verse).await?;

    for reference in thematic {
        let already_present = cross_references.iter().any(|existing| {
            existing.book == reference.book
                && existing.chapter == reference.chapter
                && existing.verse == reference.verse
        });
        if !already_present {
            cross_references.push(reference);
        }
    }

    cross_references.truncate((CURATED_LIMIT + THEMATIC_LIMIT) as usize);

    Ok(Json(VerseContextResponse {
        translation: params.translation,
        book: book_name,
        book_slug: slug,
        chapter,
        verse: VerseText { verse, text },
        chapter_verse_count: chapter_verses.len() as i32,
        chapter_verses,
        previous,
        next,
        cross_references,
    }))
}

/// The nearest verse before (`direction = -1`) or after (`direction = 1`) the
/// one under study, crossing chapter and book boundaries when needed.
async fn neighbor(
    pool: &PgPool,
    translation: &str,
    book_id: i32,
    book_ord: i32,
    chapter: i32,
    verse: i32,
    direction: i32,
) -> Result<Option<NeighborPassage>, AppError> {
    let forward = direction > 0;

    // 1. Within the same book: the next or previous verse, or the first verse
    //    of the next chapter / last verse of the previous chapter.
    let same_book = sqlx::query_as::<_, (String, String, i32, i32, String)>(
        r#"
        SELECT b.name, b.slug, c.chapter_number, v.verse_number, v.text
        FROM verses v
        JOIN chapters c ON v.chapter_id = c.id
        JOIN books b ON c.book_id = b.id
        WHERE c.translation_id = $1
          AND c.book_id = $2
          AND (
              (c.chapter_number = $3 AND v.verse_number > $4)
              OR (c.chapter_number > $3)
          )
        ORDER BY c.chapter_number, v.verse_number
        LIMIT 1
        "#,
    );

    let same_book = if forward {
        same_book
            .bind(translation)
            .bind(book_id)
            .bind(chapter)
            .bind(verse)
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)?
    } else {
        sqlx::query_as::<_, (String, String, i32, i32, String)>(
            r#"
            SELECT b.name, b.slug, c.chapter_number, v.verse_number, v.text
            FROM verses v
            JOIN chapters c ON v.chapter_id = c.id
            JOIN books b ON c.book_id = b.id
            LEFT JOIN translation_book_names tbn
                   ON tbn.book_id = b.id AND tbn.translation_id = c.translation_id
            WHERE c.translation_id = $1
              AND c.book_id = $2
              AND (
                  (c.chapter_number = $3 AND v.verse_number < $4)
                  OR (c.chapter_number < $3)
              )
            ORDER BY c.chapter_number DESC, v.verse_number DESC
            LIMIT 1
            "#,
        )
        .bind(translation)
        .bind(book_id)
        .bind(chapter)
        .bind(verse)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?
    };

    if let Some(row) = same_book {
        return Ok(Some(NeighborPassage {
            book: row.0,
            book_slug: row.1,
            chapter: row.2,
            verse: row.3,
            text: row.4,
        }));
    }

    // 2. The verse is at a book boundary: step to the adjacent book and take
    //    its first (or last) verse.
    let adjacent_ord = book_ord + direction;

    let adjacent = sqlx::query_as::<_, (String, String, i32, i32, String)>(
        r#"
        SELECT b.name, b.slug, c.chapter_number, v.verse_number, v.text
        FROM verses v
        JOIN chapters c ON v.chapter_id = c.id
        JOIN books b ON c.book_id = b.id
        WHERE c.translation_id = $1
          AND b.ord = $2
        ORDER BY c.chapter_number, v.verse_number
        LIMIT 1
        "#,
    )
    .bind(translation)
    .bind(adjacent_ord)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    let Some(row) = adjacent else {
        return Ok(None);
    };

    // For the passage before, the last verse of the previous book is wanted.
    let text = if forward {
        row.4
    } else {
        sqlx::query_scalar::<_, String>(
            r#"
            SELECT v.text
            FROM verses v
            JOIN chapters c ON v.chapter_id = c.id
            JOIN books b ON c.book_id = b.id
            WHERE c.translation_id = $1
              AND b.ord = $2
            ORDER BY c.chapter_number DESC, v.verse_number DESC
            LIMIT 1
            "#,
        )
        .bind(translation)
        .bind(adjacent_ord)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?
        .unwrap_or_default()
    };

    Ok(Some(NeighborPassage {
        book: row.0,
        book_slug: row.1,
        chapter: row.2,
        verse: row.3,
        text,
    }))
}

/// Curated links stored in the `cross_references` table.
///
/// The table is populated by a separate dataset, so it is often empty; the
/// thematic links below fill the gap in that case.
async fn curated_references(
    pool: &PgPool,
    translation: &str,
    book_id: i32,
    chapter: i32,
    verse: i32,
) -> Result<Vec<CrossReference>, AppError> {
    let rows = sqlx::query_as::<_, (String, String, i32, i32, String, String)>(
        r#"
        SELECT b.name, b.slug, c.chapter_number, v.verse_number, v.text,
               COALESCE(cr.relationship_type, 'related')
        FROM cross_references cr
        JOIN verses v ON cr.target_verse_id = v.id
        JOIN chapters c ON v.chapter_id = c.id
        JOIN books b ON c.book_id = b.id
        WHERE cr.source_verse_id = (
            SELECT v2.id
            FROM verses v2
            JOIN chapters c2 ON v2.chapter_id = c2.id
            WHERE c2.translation_id = $1
              AND c2.book_id = $2
              AND c2.chapter_number = $3
              AND v2.verse_number = $4
        )
        LIMIT $5
        "#,
    )
    .bind(translation)
    .bind(book_id)
    .bind(chapter)
    .bind(verse)
    .bind(CURATED_LIMIT)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(rows
        .into_iter()
        .map(
            |(book, book_slug, chapter, verse, text, relationship)| CrossReference {
                book,
                book_slug,
                chapter,
                verse,
                text,
                relationship,
                source: "curated".to_string(),
            },
        )
        .collect())
}

/// Words that carry no signal when looking for related passages.
const STOP_WORDS: &[&str] = &[
    "about",
    "above",
    "after",
    "again",
    "against",
    "because",
    "been",
    "before",
    "being",
    "below",
    "between",
    "both",
    "cannot",
    "could",
    "does",
    "doing",
    "done",
    "down",
    "during",
    "each",
    "even",
    "every",
    "from",
    "further",
    "have",
    "having",
    "here",
    "hers",
    "herself",
    "himself",
    "into",
    "itself",
    "just",
    "many",
    "more",
    "most",
    "much",
    "must",
    "myself",
    "neither",
    "never",
    "next",
    "only",
    "other",
    "ought",
    "ours",
    "ourselves",
    "over",
    "own",
    "same",
    "shall",
    "should",
    "some",
    "such",
    "than",
    "that",
    "the",
    "their",
    "theirs",
    "them",
    "themselves",
    "then",
    "there",
    "these",
    "they",
    "this",
    "those",
    "through",
    "under",
    "until",
    "very",
    "were",
    "what",
    "when",
    "where",
    "which",
    "while",
    "who",
    "whom",
    "whose",
    "with",
    "within",
    "without",
    "would",
    "your",
    "yours",
    "yourself",
    "yourselves",
    "unto",
    "unto",
    "thou",
    "thee",
    "thy",
    "thine",
    "ye",
    "hath",
    "doth",
    "said",
    "say",
    "saying",
    "came",
    "come",
    "shall",
    "wilt",
    "shalt",
    "therefore",
    "thus",
    "wherefore",
    "because",
    "indeed",
    "though",
    "although",
    "let",
    "may",
    "might",
    "must",
    "need",
    "ought",
    "will",
    "shall",
];

/// Derives related passages by looking for other verses that share the
/// distinctive vocabulary of the verse under study.
async fn thematic_references(
    pool: &PgPool,
    verse_text: &str,
    book_id: i32,
    chapter: i32,
    verse: i32,
) -> Result<Vec<CrossReference>, AppError> {
    let candidates = distinctive_words(verse_text);
    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    let total_verses = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM verses")
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

    // A word that appears in almost every verse carries no signal, and a word
    // that appears in only one or two verses will only ever match the source
    // verse itself.
    let max_df = (total_verses / 50).max(5);

    let rows = sqlx::query_as::<_, (String, i64)>(
        r#"
        SELECT word.word, (
            SELECT COUNT(*)::bigint
            FROM verses v
            WHERE to_tsvector('english', v.text) @@ plainto_tsquery('english', word.word)
        ) AS df
        FROM unnest($1::text[]) AS word(word)
        "#,
    )
    .bind(&candidates)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let mut scored: Vec<(String, i64)> = rows
        .into_iter()
        .filter(|(_, df)| *df >= 3 && *df <= max_df)
        .collect();

    // Rarest words first: they carry the most signal.
    scored.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.len().cmp(&a.0.len())));

    let terms: Vec<String> = scored
        .iter()
        .take(8)
        .map(|(word, _)| word.clone())
        .collect();

    if terms.is_empty() {
        return Ok(Vec::new());
    }

    let tsquery = terms.join(" | ");

    // Every version carries the same passage, so the matches are reduced to
    // one row per passage before they are returned.
    let matches = sqlx::query_as::<_, (String, String, i32, i32, String, f32)>(
        r#"
        SELECT * FROM (
            SELECT DISTINCT ON (c.book_id, c.chapter_number, v.verse_number)
                b.name AS book, b.slug AS book_slug,
                c.chapter_number, v.verse_number, v.text,
                ts_rank_cd(to_tsvector('english', v.text), to_tsquery('english', $1)) AS rank
            FROM verses v
            JOIN chapters c ON v.chapter_id = c.id
            JOIN books b ON c.book_id = b.id
            WHERE to_tsvector('english', v.text) @@ to_tsquery('english', $1)
              AND NOT (c.book_id = $2 AND c.chapter_number = $3 AND v.verse_number = $4)
            ORDER BY c.book_id, c.chapter_number, v.verse_number, rank DESC
        ) passages
        ORDER BY rank DESC, book_slug, chapter_number, verse_number
        LIMIT $5
        "#,
    )
    .bind(&tsquery)
    .bind(book_id)
    .bind(chapter)
    .bind(verse)
    .bind(THEMATIC_LIMIT)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(matches
        .into_iter()
        .map(
            |(book, book_slug, chapter, verse, text, _rank)| CrossReference {
                book,
                book_slug,
                chapter,
                verse,
                text,
                relationship: "thematic".to_string(),
                source: "thematic".to_string(),
            },
        )
        .collect())
}

/// The distinctive, non-stop words of a verse.
fn distinctive_words(text: &str) -> Vec<String> {
    let mut seen = Vec::new();

    for raw in text.split(|c: char| !c.is_ascii_alphabetic()) {
        if raw.len() < 4 || raw.len() > 24 {
            continue;
        }

        let lower = raw.to_ascii_lowercase();
        if STOP_WORDS.contains(&lower.as_str()) {
            continue;
        }

        if !seen.contains(&lower) {
            seen.push(lower);
        }
    }

    seen
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_distinctive_words() {
        let words =
            distinctive_words("For God so loved the world, that he gave his only begotten Son");
        assert!(words.contains(&"loved".to_string()));
        assert!(words.contains(&"begotten".to_string()));
        assert!(!words.contains(&"the".to_string()));
        assert!(!words.contains(&"god".to_string()));
    }

    #[test]
    fn deduplicates_words() {
        let words = distinctive_words("love love love mercy");
        assert_eq!(words, vec!["love".to_string(), "mercy".to_string()]);
    }

    #[test]
    fn ignores_short_tokens() {
        let words = distinctive_words("a of the is was");
        assert!(words.is_empty());
    }
}
