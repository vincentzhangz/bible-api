use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::IntoParams;

use crate::api::books::book_summaries;
use crate::error::AppError;
use crate::models::{DailyReading, VerseText};

/// How many verses the daily passage contains.
const PASSAGE_LENGTH: usize = 5;

#[derive(Debug, Deserialize, IntoParams)]
pub struct DailyQuery {
    /// Date to build the passage for, formatted `YYYY-MM-DD`. Defaults to today.
    pub date: Option<String>,
    /// Translation to render the passage in. Defaults to the first available.
    pub translation: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/reading/daily",
    params(DailyQuery),
    responses(
        (status = 200, description = "The passage for a given day", body = DailyReading)
    )
)]
pub async fn daily(
    State(pool): State<PgPool>,
    Query(query): Query<DailyQuery>,
) -> Result<Json<DailyReading>, AppError> {
    let date = query.date.unwrap_or_else(today_utc);
    validate_date(&date)?;

    let (translation_id, translation_name) = match query.translation {
        Some(id) => {
            let meta = sqlx::query_as::<_, (String, String)>(
                "SELECT id, name FROM translations WHERE id = $1",
            )
            .bind(&id)
            .fetch_optional(&pool)
            .await
            .map_err(AppError::Database)?;

            match meta {
                Some(row) => row,
                None => return Err(AppError::NotFound(format!("translation {} not found", id))),
            }
        }
        None => {
            let row = sqlx::query_as::<_, (String, String)>(
                "SELECT id, name FROM translations ORDER BY id LIMIT 1",
            )
            .fetch_optional(&pool)
            .await
            .map_err(AppError::Database)?;

            match row {
                Some(row) => row,
                None => return Err(AppError::NotFound("no translations available".to_string())),
            }
        }
    };

    let books = book_summaries(&pool).await?;
    if books.is_empty() {
        return Err(AppError::NotFound("no books available".to_string()));
    }

    // Deterministic selection: the same date always yields the same passage.
    let seed = fnv1a(&format!("{}:{}", date, translation_id));
    let book = &books[(seed as usize) % books.len()];

    let display_name = sqlx::query_scalar::<_, String>(
        "SELECT name FROM translation_book_names WHERE translation_id = $1 AND book_id = $2",
    )
    .bind(&translation_id)
    .bind(book.id)
    .fetch_optional(&pool)
    .await
    .map_err(AppError::Database)?;

    let book_name = display_name.unwrap_or_else(|| book.name.clone());

    let chapters = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT DISTINCT c.chapter_number
        FROM chapters c
        WHERE c.translation_id = $1 AND c.book_id = $2
        ORDER BY c.chapter_number
        "#,
    )
    .bind(&translation_id)
    .bind(book.id)
    .fetch_all(&pool)
    .await
    .map_err(AppError::Database)?;

    if chapters.is_empty() {
        return Err(AppError::NotFound(format!(
            "no chapters for {} in {}",
            book.name, translation_id
        )));
    }

    let chapter_seed = fnv1a(&format!("{}:{}:chapter", date, book.slug));
    let chapter = chapters[(chapter_seed as usize) % chapters.len()];

    let rows = sqlx::query_as::<_, (i32, String)>(
        r#"
        SELECT v.verse_number, v.text
        FROM verses v
        JOIN chapters c ON v.chapter_id = c.id
        WHERE c.translation_id = $1 AND c.book_id = $2 AND c.chapter_number = $3
        ORDER BY v.verse_number
        "#,
    )
    .bind(&translation_id)
    .bind(book.id)
    .bind(chapter)
    .fetch_all(&pool)
    .await
    .map_err(AppError::Database)?;

    if rows.is_empty() {
        return Err(AppError::NotFound(format!(
            "no verses for {} {} in {}",
            book.name, chapter, translation_id
        )));
    }

    let all_verses: Vec<VerseText> = rows
        .into_iter()
        .map(|(verse, text)| VerseText { verse, text })
        .collect();

    // Start the passage somewhere in the chapter, but always leave room for
    // the whole window.
    let window = PASSAGE_LENGTH.min(all_verses.len());
    let start_seed = fnv1a(&format!("{}:{}:start", date, book.slug));
    let max_start = all_verses.len() - window;
    let start = if max_start == 0 {
        0
    } else {
        (start_seed as usize) % (max_start + 1)
    };

    let verses: Vec<VerseText> = all_verses[start..start + window].to_vec();
    let featured_seed = fnv1a(&format!("{}:{}:featured", date, book.slug));
    let featured_index = (featured_seed as usize) % verses.len();

    Ok(Json(DailyReading {
        date,
        translation: translation_id,
        translation_name,
        book: book_name.clone(),
        book_slug: book.slug.clone(),
        chapter,
        reference: format!("{} {}", book_name, chapter),
        featured: verses[featured_index].clone(),
        verses,
    }))
}

/// Today's date in UTC, formatted `YYYY-MM-DD`.
fn today_utc() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let days = (now / 86_400) as i64;
    format_ordinal_date(days)
}

/// Converts days since the Unix epoch into `YYYY-MM-DD`.
fn format_ordinal_date(days: i64) -> String {
    // Days from 1970-01-01 to a civil date, using the standard
    // days-from-civil algorithm (Howard Hinnant, public domain).
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };

    let year = if m <= 2 { y + 1 } else { y };

    format!("{:04}-{:02}-{:02}", year, m, d)
}

/// Rejects anything that is not a plain `YYYY-MM-DD` date.
fn validate_date(date: &str) -> Result<(), AppError> {
    let bytes = date.as_bytes();
    let shaped = bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[..4].iter().all(u8::is_ascii_digit)
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[8..].iter().all(u8::is_ascii_digit);

    if shaped {
        Ok(())
    } else {
        Err(AppError::BadRequest(
            "date must be formatted as YYYY-MM-DD".to_string(),
        ))
    }
}

/// FNV-1a, used to pick a deterministic passage for a date.
fn fnv1a(input: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_epoch_dates() {
        assert_eq!(format_ordinal_date(0), "1970-01-01");
        assert_eq!(format_ordinal_date(19_723), "2024-01-01");
        assert_eq!(format_ordinal_date(20_746), "2026-10-20");
    }

    #[test]
    fn hash_is_stable_and_distinct() {
        assert_eq!(fnv1a("2026-10-20"), fnv1a("2026-10-20"));
        assert_ne!(fnv1a("2026-10-20"), fnv1a("2026-10-21"));
    }

    #[test]
    fn validates_date_shape() {
        assert!(validate_date("2026-10-20").is_ok());
        assert!(validate_date("2026-10-2").is_err());
        assert!(validate_date("not-a-date").is_err());
        assert!(validate_date("2026/10/20").is_err());
    }
}
