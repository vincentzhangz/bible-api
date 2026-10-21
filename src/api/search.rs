use axum::{
    Json,
    extract::{Extension, Query, State},
};
use serde::Deserialize;
use sqlx::PgPool;
use sqlx::QueryBuilder;
use sqlx::postgres::Postgres;
use std::sync::Arc;
use utoipa::IntoParams;

use crate::config::env::AppConfig;
use crate::error::AppError;
use crate::models::{SearchHit, SearchResult, SearchResultsResponse};
use crate::util::slugify;

/// The maximum number of results a single page may contain.
const MAX_LIMIT: i64 = 100;
const DEFAULT_LIMIT: i64 = 20;

#[derive(Debug, Deserialize, IntoParams)]
pub struct SearchQuery {
    /// The text to look for. Quoted phrases are supported.
    pub q: String,
    /// Restrict results to a single translation.
    pub translation: Option<String>,
    /// Restrict results to a single book, by name or slug.
    pub book: Option<String>,
    /// Restrict results to a testament (`old` or `new`).
    pub testament: Option<String>,
    /// How many results to return (max 100).
    pub limit: Option<i64>,
    /// How many results to skip.
    pub offset: Option<i64>,
}

/// The filters shared by the result and count queries.
struct Filters<'a> {
    q: &'a str,
    translation: Option<&'a str>,
    book: Option<&'a str>,
    testament: Option<&'a str>,
}

impl<'a> Filters<'a> {
    fn from_query(query: &'a SearchQuery) -> Self {
        Self {
            q: &query.q,
            translation: query.translation.as_deref(),
            book: query.book.as_deref(),
            testament: query.testament.as_deref(),
        }
    }
}

/// Appends the shared `WHERE` clause. Using a `QueryBuilder` keeps the
/// parameter numbering correct no matter which filters are present.
fn apply_filters<'a>(builder: &mut QueryBuilder<Postgres>, filters: &Filters<'a>) {
    builder.push(" WHERE to_tsvector('english', v.text) @@ websearch_to_tsquery('english', ");
    builder.push_bind(filters.q);
    builder.push(")");

    if let Some(translation) = filters.translation {
        builder.push(" AND t.id = ");
        builder.push_bind(translation);
    }

    if let Some(book) = filters.book {
        builder.push(" AND (LOWER(b.name) = LOWER(");
        builder.push_bind(book);
        builder.push(") OR b.slug = ");
        builder.push_bind(slugify(book));
        builder.push(")");
    }

    if let Some(testament) = filters.testament {
        builder.push(" AND b.testament = ");
        builder.push_bind(testament);
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/search",
    params(SearchQuery),
    responses(
        (status = 200, description = "Matching verses", body = [SearchResult])
    )
)]
pub async fn search(
    State(pool): State<PgPool>,
    Extension(config): Extension<Arc<AppConfig>>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<SearchResult>>, AppError> {
    let filters = Filters::from_query(&params);
    let limit = params
        .limit
        .unwrap_or(i64::from(config.search_limit))
        .clamp(1, MAX_LIMIT);

    // A passage is returned once, however many versions carry it.
    let mut builder = QueryBuilder::new(
        r#"
        WITH matched AS (
            SELECT t.id AS translation, b.name AS book, b.ord AS book_ord,
                   c.book_id, c.chapter_number, v.verse_number, v.text,
                   ts_rank(to_tsvector('english', v.text), websearch_to_tsquery('english', "#,
    );
    builder.push_bind(filters.q);
    builder.push(")) AS rank ");
    builder.push(
        r#"
            FROM verses v
            JOIN chapters c ON v.chapter_id = c.id
            JOIN translations t ON c.translation_id = t.id
            JOIN books b ON c.book_id = b.id
        "#,
    );
    apply_filters(&mut builder, &filters);
    builder.push(
        " ) SELECT * FROM ( \
             SELECT DISTINCT ON (book_id, chapter_number, verse_number) \
                 translation, book, book_ord, chapter_number, verse_number, text, rank \
             FROM matched \
             ORDER BY book_id, chapter_number, verse_number, rank DESC, translation \
         ) passages \
         ORDER BY rank DESC, book_ord, chapter_number, verse_number LIMIT ",
    );
    builder.push_bind(limit);

    let rows = builder
        .build_query_as::<(String, String, i32, i32, i32, String, f32)>()
        .fetch_all(&pool)
        .await
        .map_err(AppError::Database)?;

    let results: Vec<SearchResult> = rows
        .into_iter()
        .map(
            |(translation, book, _book_ord, chapter, verse, text, _rank)| SearchResult {
                translation,
                book,
                chapter,
                verse,
                text,
            },
        )
        .collect();

    Ok(Json(results))
}

#[utoipa::path(
    get,
    path = "/api/v1/search/results",
    params(SearchQuery),
    responses(
        (status = 200, description = "A page of matching verses plus the total match count", body = SearchResultsResponse)
    )
)]
pub async fn search_results(
    State(pool): State<PgPool>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<SearchResultsResponse>, AppError> {
    let filters = Filters::from_query(&params);
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let offset = params.offset.unwrap_or(0).max(0);

    // A passage is counted once, however many versions carry it.
    let mut count_builder = QueryBuilder::new(
        r#"
        WITH matched AS (
            SELECT c.book_id, c.chapter_number, v.verse_number
            FROM verses v
            JOIN chapters c ON v.chapter_id = c.id
            JOIN translations t ON c.translation_id = t.id
            JOIN books b ON c.book_id = b.id
        "#,
    );
    apply_filters(&mut count_builder, &filters);
    count_builder.push(
        " ) SELECT COUNT(*) FROM (
               SELECT DISTINCT book_id, chapter_number, verse_number FROM matched
           ) passages",
    );

    let total: i64 = count_builder
        .build_query_scalar()
        .fetch_one(&pool)
        .await
        .map_err(AppError::Database)?;

    // The same passage is written out in every version, so the matches are
    // gathered first and then reduced to one row per passage, keeping the
    // version that reads best and recording how many others carry it.
    let mut builder = QueryBuilder::new(
        "WITH matched AS ( \
         SELECT t.id AS translation, t.name AS translation_name, \
                b.name AS book, b.slug AS book_slug, b.ord AS book_ord, \
                c.book_id, c.chapter_number, \
                v.verse_number, v.text, \
         ts_rank(to_tsvector('english', v.text), websearch_to_tsquery('english', ",
    );
    builder.push_bind(filters.q);
    builder.push(")) AS rank ");
    builder.push(
        r#"
        FROM verses v
        JOIN chapters c ON v.chapter_id = c.id
        JOIN translations t ON c.translation_id = t.id
        JOIN books b ON c.book_id = b.id
        "#,
    );
    apply_filters(&mut builder, &filters);
    builder.push(
        " ) SELECT * FROM ( \
             SELECT DISTINCT ON (book_id, chapter_number, verse_number) \
                 translation, translation_name, book, book_slug, book_ord, \
                 chapter_number, verse_number, text, rank, \
                 COUNT(*) OVER (PARTITION BY book_id, chapter_number, verse_number) AS versions \
             FROM matched \
             ORDER BY book_id, chapter_number, verse_number, rank DESC, translation \
         ) passages \
         ORDER BY rank DESC, book_ord, chapter_number, verse_number LIMIT ",
    );
    builder.push_bind(limit);
    builder.push(" OFFSET ");
    builder.push_bind(offset);

    let rows = builder
        .build_query_as::<(
            String,
            String,
            String,
            String,
            i32,
            i32,
            i32,
            String,
            f32,
            i64,
        )>()
        .fetch_all(&pool)
        .await
        .map_err(AppError::Database)?;

    let results: Vec<SearchHit> = rows
        .into_iter()
        .map(
            |(
                translation,
                translation_name,
                book,
                book_slug,
                _book_ord,
                chapter,
                verse,
                text,
                rank,
                versions,
            )| SearchHit {
                translation,
                translation_name,
                book,
                book_slug,
                chapter,
                verse,
                text,
                rank,
                versions,
            },
        )
        .collect();

    Ok(Json(SearchResultsResponse {
        total,
        limit,
        offset,
        has_more: offset + limit < total,
        results,
    }))
}
