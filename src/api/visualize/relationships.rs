use axum::{
    Json,
    extract::{Extension, Path, Query},
};
use serde::{Deserialize, Serialize};
use std::path::Path as StdPath;
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};

use super::language_from_translation;
use crate::config::env::AppConfig;

#[derive(Debug, Serialize, ToSchema)]
pub struct CharacterRelationship {
    pub from: String,
    pub from_name: String,
    pub to: String,
    pub to_name: String,
    #[serde(rename = "type")]
    pub relationship_type: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RelationshipsResponse {
    pub language: String,
    pub book: String,
    pub characters: Vec<CharacterInfo>,
    pub relationships: Vec<CharacterRelationship>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CharacterInfo {
    pub key: String,
    pub name: String,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct RelationshipsQuery {
    pub lang: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct RelationshipsPath {
    pub translation: String,
    pub book: String,
}

/// Gets character relationships for a book.
#[utoipa::path(
    get,
    path = "/api/v1/visualize/relationships/{translation}/{book}",
    params(
        ("translation" = String, Path, description = "Translation ID"),
        ("book" = String, Path, description = "Book name")
    ),
    responses(
        (status = 200, description = "Character relationships", body = RelationshipsResponse)
    )
)]
pub async fn relationships(
    Extension(config): Extension<Arc<AppConfig>>,
    Path(params): Path<RelationshipsPath>,
    Query(query): Query<RelationshipsQuery>,
) -> Json<RelationshipsResponse> {
    let language = query
        .lang
        .clone()
        .unwrap_or_else(|| language_from_translation(&params.translation));

    let data_dir = StdPath::new(&config.data_dir);
    let locale = crate::ingestion::visualize::load_visualize_locale(data_dir, &language);

    let book_data = locale
        .as_ref()
        .and_then(|l| l.books.get(&params.book))
        .cloned()
        .or_else(|| {
            crate::ingestion::visualize::load_visualize_locale(data_dir, "en")
                .and_then(|l| l.books.get(&params.book).cloned())
        });

    match book_data {
        Some(book_data) => {
            let characters: Vec<CharacterInfo> = book_data
                .characters
                .into_iter()
                .map(|c| CharacterInfo {
                    key: c.key,
                    name: c.name,
                })
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
                    to_name: char_names
                        .get(&r.to)
                        .cloned()
                        .unwrap_or_else(|| r.to.clone()),
                    relationship_type: r.relationship_type,
                })
                .collect();

            Json(RelationshipsResponse {
                language,
                book: params.book,
                characters,
                relationships,
            })
        }
        None => Json(RelationshipsResponse {
            language,
            book: params.book,
            characters: vec![],
            relationships: vec![],
        }),
    }
}
