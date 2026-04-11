use axum::{extract::{Extension, Path, Query}, Json};
use serde::{Deserialize, Serialize};
use std::path::Path as StdPath;
use std::sync::Arc;

use crate::config::env::AppConfig;
use super::language_from_translation;

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
    Extension(config): Extension<Arc<AppConfig>>,
    Path((translation, book)): Path<(String, String)>,
    Query(query): Query<RelationshipsQuery>,
) -> Json<RelationshipsResponse> {
    let language = query
        .lang
        .clone()
        .unwrap_or_else(|| language_from_translation(&translation));

    let data_dir = StdPath::new(&config.data_dir);
    let locale = crate::ingestion::visualize::load_visualize_locale(data_dir, &language);

    let book_data = locale
        .as_ref()
        .and_then(|l| l.books.get(&book))
        .cloned()
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
                    from_name: char_names
                        .get(&r.from)
                        .cloned()
                        .unwrap_or_else(|| r.from.clone()),
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
        None => Json(RelationshipsResponse {
            language,
            book,
            characters: vec![],
            relationships: vec![],
        }),
    }
}
