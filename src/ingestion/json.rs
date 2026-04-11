use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::fs;
use std::path::Path;
use tracing::{error, info};

use super::hash_sync::{calculate_hash, SyncStatus, check_and_sync_translation};

#[derive(Debug, Serialize, Deserialize)]
pub struct TranslationJson {
    pub id: String,
    pub metadata: MetadataJson,
    pub books: Vec<BookJson>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetadataJson {
    pub name: String,
    #[serde(default)]
    pub shortname: Option<String>,
    pub language: String,
    pub license: String,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub year: Option<String>,
    #[serde(default)]
    pub publisher: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub copyright_statement: Option<String>,
    #[serde(default)]
    pub italics: Option<bool>,
    #[serde(default)]
    pub strongs: Option<bool>,
    #[serde(default)]
    pub red_letter: Option<bool>,
    #[serde(default)]
    pub paragraph: Option<bool>,
    #[serde(default)]
    pub official: Option<bool>,
    #[serde(default)]
    pub research: Option<bool>,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BookJson {
    pub id: String,
    pub name: String,
    pub testament: String,
    pub chapters: Vec<ChapterJson>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChapterJson {
    pub chapter: i32,
    pub verses: Vec<VerseJson>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerseJson {
    pub verse: i32,
    pub text: String,
}

pub async fn ingest_json_file(pool: &PgPool, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read(path)?;
    let hash = calculate_hash(&content);

    let translation: TranslationJson = serde_json::from_slice(&content)?;
    let source = translation.metadata.source.clone().unwrap_or_else(|| "community".to_string());

    let sync_status = check_and_sync_translation(pool, &translation.id, &hash).await?;

    match sync_status {
        SyncStatus::Skipped => {
            return Ok(());
        }
        SyncStatus::Updated => {
            truncate_translation(pool, &translation.id).await?;
        }
        SyncStatus::Ingested => {}
    }

    let license_id = get_or_create_license(pool, &translation.metadata.license).await?;

    sqlx::query(
        r#"
        INSERT INTO translations (id, name, language, license_id, source, json_hash, shortname, year, publisher, description, url, copyright_statement, italics, strongs, red_letter, paragraph, official, research, version)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
        ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            language = EXCLUDED.language,
            license_id = EXCLUDED.license_id,
            source = EXCLUDED.source,
            json_hash = EXCLUDED.json_hash,
            shortname = EXCLUDED.shortname,
            year = EXCLUDED.year,
            publisher = EXCLUDED.publisher,
            description = EXCLUDED.description,
            url = EXCLUDED.url,
            copyright_statement = EXCLUDED.copyright_statement,
            italics = EXCLUDED.italics,
            strongs = EXCLUDED.strongs,
            red_letter = EXCLUDED.red_letter,
            paragraph = EXCLUDED.paragraph,
            official = EXCLUDED.official,
            research = EXCLUDED.research,
            version = EXCLUDED.version
        "#,
    )
    .bind(&translation.id)
    .bind(&translation.metadata.name)
    .bind(&translation.metadata.language)
    .bind(license_id)
    .bind(&source)
    .bind(&hash)
    .bind(&translation.metadata.shortname)
    .bind(&translation.metadata.year)
    .bind(&translation.metadata.publisher)
    .bind(&translation.metadata.description)
    .bind(&translation.metadata.url)
    .bind(&translation.metadata.copyright_statement)
    .bind(translation.metadata.italics.unwrap_or(false))
    .bind(translation.metadata.strongs.unwrap_or(false))
    .bind(translation.metadata.red_letter.unwrap_or(false))
    .bind(translation.metadata.paragraph.unwrap_or(false))
    .bind(translation.metadata.official.unwrap_or(false))
    .bind(translation.metadata.research.unwrap_or(false))
    .bind(&translation.metadata.version)
    .execute(pool)
    .await?;

    for book_json in &translation.books {
        let book_id = get_or_create_book(pool, &book_json.name, &book_json.testament).await?;

        for chapter_json in &book_json.chapters {
            let chapter_id = insert_chapter(pool, &translation.id, book_id, chapter_json.chapter).await?;

            for verse_json in &chapter_json.verses {
                insert_verse(pool, chapter_id, verse_json.verse, &verse_json.text).await?;
            }
        }
    }

    info!("Ingested translation: {}", translation.id);
    Ok(())
}

async fn get_or_create_license(pool: &PgPool, license_id: &str) -> Result<Option<String>, sqlx::Error> {
    // Look up existing license by id
    let existing: Option<String> = sqlx::query_scalar(
        "SELECT id FROM licenses WHERE id = $1",
    )
    .bind(license_id)
    .fetch_optional(pool)
    .await?;

    if let Some(id) = existing {
        return Ok(Some(id));
    }

    // License doesn't exist - create it with the given id
    sqlx::query(
        r#"
        INSERT INTO licenses (id, name, attribution_required, commercial_use)
        VALUES ($1, $2, false, true)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(license_id)
    .bind(license_id)
    .execute(pool)
    .await?;

    Ok(Some(license_id.to_string()))
}

async fn get_or_create_book(pool: &PgPool, name: &str, testament: &str) -> Result<i32, sqlx::Error> {
    let existing = sqlx::query_scalar::<_, Option<i32>>(
        "SELECT id FROM books WHERE LOWER(name) = LOWER($1)",
    )
    .bind(name)
    .fetch_optional(pool)
    .await?;

    if let Some(id) = existing.flatten() {
        return Ok(id);
    }

    let max_order = sqlx::query_scalar::<_, Option<i32>>(
        "SELECT MAX(ord) FROM books",
    )
    .fetch_optional(pool)
    .await?
    .flatten()
    .unwrap_or(0);

    let new_order = max_order + 1;

    let id = sqlx::query_scalar::<_, i32>(
        r#"
        INSERT INTO books (name, testament, ord)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(name)
    .bind(testament)
    .bind(new_order)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

async fn insert_chapter(pool: &PgPool, translation_id: &str, book_id: i32, chapter_number: i32) -> Result<i32, sqlx::Error> {
    let id = sqlx::query_scalar::<_, i32>(
        r#"
        INSERT INTO chapters (translation_id, book_id, chapter_number)
        VALUES ($1, $2, $3)
        ON CONFLICT (translation_id, book_id, chapter_number) DO UPDATE SET chapter_number = EXCLUDED.chapter_number
        RETURNING id
        "#,
    )
    .bind(translation_id)
    .bind(book_id)
    .bind(chapter_number)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

async fn insert_verse(pool: &PgPool, chapter_id: i32, verse_number: i32, text: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO verses (chapter_id, verse_number, text)
        VALUES ($1, $2, $3)
        ON CONFLICT (chapter_id, verse_number) DO UPDATE SET text = EXCLUDED.text
        "#,
    )
    .bind(chapter_id)
    .bind(verse_number)
    .bind(text)
    .execute(pool)
    .await?;

    Ok(())
}

async fn truncate_translation(pool: &PgPool, translation_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        DELETE FROM verses WHERE chapter_id IN (
            SELECT id FROM chapters WHERE translation_id = $1
        )
        "#,
    )
    .bind(translation_id)
    .execute(pool)
    .await?;

    sqlx::query("DELETE FROM chapters WHERE translation_id = $1")
        .bind(translation_id)
        .execute(pool)
        .await?;

    sqlx::query("DELETE FROM translations WHERE id = $1")
        .bind(translation_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn ingest_all_json_files(pool: &PgPool, data_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let translations_dir = data_dir.join("translations");

    if !translations_dir.exists() {
        error!("Translations directory not found: {:?}", translations_dir);
        return Ok(());
    }

    let mut ingested = 0;

    for entry in fs::read_dir(translations_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            match ingest_json_file(pool, &path).await {
                Ok(_) => {
                    ingested += 1;
                }
                Err(e) => {
                    error!("Failed to ingest {:?}: {}", path, e);
                }
            }
        }
    }

    info!("Ingestion complete: {} ingested", ingested);
    Ok(())
}
