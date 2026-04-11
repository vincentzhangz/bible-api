use axum::{extract::{Path, Query, State, Extension}, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::path::Path as StdPath;
use std::sync::Arc;

use crate::config::env::AppConfig;

#[derive(Debug, Serialize)]
pub struct WordFrequency {
    pub word: String,
    pub count: i64,
}

#[derive(Debug, Deserialize)]
pub struct WordFrequencyPath {
    pub translation: String,
    pub book: String,
}

pub async fn word_frequency(
    State(pool): State<PgPool>,
    Path(params): Path<WordFrequencyPath>,
) -> Json<Vec<WordFrequency>> {
    let results = sqlx::query_as::<_, (String, i64)>(
        r#"
        SELECT word, count
        FROM (
            SELECT unnest(string_to_array(lower(v.text), ' ')) as word
            FROM verses v
            JOIN chapters c ON v.chapter_id = c.id
            JOIN translations t ON c.translation_id = t.id
            JOIN books b ON c.book_id = b.id
            WHERE t.id = $1 AND LOWER(b.name) = LOWER($2)
        ) words
        WHERE length(word) > 3
        GROUP BY word
        ORDER BY count DESC
        LIMIT 100
        "#,
    )
    .bind(&params.translation)
    .bind(&params.book)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let frequency: Vec<WordFrequency> = results
        .into_iter()
        .map(|(word, count)| WordFrequency { word, count })
        .collect();

    Json(frequency)
}

#[derive(Debug, Serialize)]
pub struct CrossReferenceTarget {
    pub book: String,
    pub chapter: i32,
    pub verse: i32,
    pub relationship: String,
}

#[derive(Debug, Serialize)]
pub struct CrossReferenceResponse {
    pub source: CrossReferenceSource,
    pub references: Vec<CrossReferenceTarget>,
}

#[derive(Debug, Serialize)]
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

pub async fn cross_references(
    State(pool): State<PgPool>,
    Path(params): Path<CrossRefPath>,
) -> Json<CrossReferenceResponse> {
    let source_result = sqlx::query_as::<_, (String, i32, i32)>(
        r#"
        SELECT b.name, c.chapter_number, v.verse_number
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
    .unwrap_or_default();

    let (source_book, source_chapter, source_verse) = source_result.unwrap_or_else(|| {
        (params.book.clone(), params.chapter, params.verse)
    });

    let references_result = sqlx::query_as::<_, (String, i32, i32, String)>(
        r#"
        SELECT b.name, c.chapter_number, v.verse_number, COALESCE(cr.relationship_type, 'related')
        FROM cross_references cr
        JOIN verses v ON cr.target_verse_id = v.id
        JOIN chapters c ON v.chapter_id = c.id
        JOIN books b ON c.book_id = b.id
        WHERE cr.source_verse_id = (
            SELECT v.id FROM verses v
            JOIN chapters c ON v.chapter_id = c.id
            JOIN translations t ON c.translation_id = t.id
            JOIN books b ON c.book_id = b.id
            WHERE t.id = $1 AND LOWER(b.name) = LOWER($2) AND c.chapter_number = $3 AND v.verse_number = $4
        )
        "#,
    )
    .bind(&params.translation)
    .bind(&params.book)
    .bind(params.chapter)
    .bind(params.verse)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let references: Vec<CrossReferenceTarget> = references_result
        .into_iter()
        .map(|(book, chapter, verse, relationship)| CrossReferenceTarget {
            book,
            chapter,
            verse,
            relationship,
        })
        .collect();

    Json(CrossReferenceResponse {
        source: CrossReferenceSource {
            book: source_book,
            chapter: source_chapter,
            verse: source_verse,
        },
        references,
    })
}

#[derive(Debug, Serialize)]
pub struct TimelineEvent {
    pub key: String,
    pub event: String,
    pub reference: String,
    #[serde(rename = "estimatedYear")]
    pub estimated_year: Option<i32>,
    pub category: String,
}

#[derive(Debug, Deserialize)]
pub struct TimelineQuery {
    pub lang: Option<String>,
}

pub async fn timeline(
    State(_pool): State<PgPool>,
    Extension(config): Extension<Arc<AppConfig>>,
    Path(translation): Path<String>,
    Query(query): Query<TimelineQuery>,
) -> Json<Vec<TimelineEvent>> {
    let language = query.lang.clone().unwrap_or_else(|| {
        translation.split('-').next().unwrap_or("en").to_string()
    });

    let data_dir = StdPath::new(&config.data_dir);

    if let Some(locale) = crate::ingestion::visualize::load_visualize_locale(data_dir, &language) {
        let events: Vec<TimelineEvent> = locale
            .timeline
            .into_iter()
            .map(|e| TimelineEvent {
                key: e.key,
                event: e.event,
                reference: e.reference,
                estimated_year: e.estimated_year,
                category: e.category,
            })
            .collect();
        Json(events)
    } else {
        Json(vec![])
    }
}

#[derive(Debug, Serialize)]
pub struct CharacterRelationship {
    pub from: String,
    pub from_name: String,
    pub to: String,
    pub to_name: String,
    #[serde(rename = "type")]
    pub relationship_type: String,
}

#[derive(Debug, Serialize)]
pub struct RelationshipsResponse {
    pub language: String,
    pub book: String,
    pub characters: Vec<CharacterInfo>,
    pub relationships: Vec<CharacterRelationship>,
}

#[derive(Debug, Serialize)]
pub struct CharacterInfo {
    pub key: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct RelationshipsQuery {
    pub lang: Option<String>,
}

pub async fn relationships(
    State(_pool): State<PgPool>,
    Extension(config): Extension<Arc<AppConfig>>,
    Path((translation, book)): Path<(String, String)>,
    Query(query): Query<RelationshipsQuery>,
) -> Json<RelationshipsResponse> {
    let language = query.lang.clone().unwrap_or_else(|| {
        translation.split('-').next().unwrap_or("en").to_string()
    });

    let data_dir = StdPath::new(&config.data_dir);
    let locale = crate::ingestion::visualize::load_visualize_locale(data_dir, &language);

    // Try exact book match, then try mapping English book names to localized
    let book_data = locale.as_ref().and_then(|l| l.books.get(&book)).cloned()
        .or_else(|| {
            crate::ingestion::visualize::load_visualize_locale(data_dir, "en")
                .and_then(|l| l.books.get(&book).cloned())
        });

    match book_data {
        Some(book_data) => {
            let characters: Vec<CharacterInfo> = book_data
                .characters
                .into_iter()
                .map(|c| CharacterInfo { key: c.key, name: c.name })
                .collect();

            let char_names: std::collections::HashMap<_, _> = characters
                .iter()
                .map(|c| (c.key.clone(), c.name.clone()))
                .collect();

            let relationships: Vec<CharacterRelationship> = book_data
                .relationships
                .into_iter()
                .map(|r| CharacterRelationship {
                    from: r.from.clone(),
                    from_name: char_names.get(&r.from).cloned().unwrap_or_else(|| r.from.clone()),
                    to: r.to.clone(),
                    to_name: char_names.get(&r.to).cloned().unwrap_or_else(|| r.to.clone()),
                    relationship_type: r.relationship_type,
                })
                .collect();

            Json(RelationshipsResponse {
                language,
                book,
                characters,
                relationships,
            })
        }
        None => {
            Json(RelationshipsResponse {
                language,
                book,
                characters: vec![],
                relationships: vec![],
            })
        }
    }
}
