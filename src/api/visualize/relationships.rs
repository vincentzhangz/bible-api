use axum::{
    Json,
    extract::{Extension, Path, Query},
};
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};

use super::language_from_translation;
use crate::ingestion::cache::VisualizeCache;
use crate::ingestion::visualize::BookRelationshipsJson;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct CharacterRelationship {
    pub from: String,
    pub from_name: String,
    pub to: String,
    pub to_name: String,
    #[serde(rename = "type")]
    pub relationship_type: String,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct RelationshipsResponse {
    pub language: String,
    pub book: String,
    pub characters: Vec<CharacterInfo>,
    pub relationships: Vec<CharacterRelationship>,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct CharacterInfo {
    pub key: String,
    pub name: String,
}

#[derive(Debug, serde::Deserialize, IntoParams)]
pub struct RelationshipsQuery {
    pub lang: Option<String>,
}

#[derive(Debug, serde::Deserialize, IntoParams)]
pub struct RelationshipsPath {
    pub translation: String,
    pub book: String,
}

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
    Extension(cache): Extension<Arc<VisualizeCache>>,
    Path(params): Path<RelationshipsPath>,
    Query(query): Query<RelationshipsQuery>,
) -> Json<RelationshipsResponse> {
    let language = query
        .lang
        .unwrap_or_else(|| language_from_translation(&params.translation));

    let empty = BookRelationshipsJson {
        characters: vec![],
        relationships: vec![],
    };
    let book_data = cache
        .get_book_data(&language, &params.book)
        .unwrap_or(&empty);

    let characters: Vec<CharacterInfo> = book_data
        .characters
        .iter()
        .map(|c| CharacterInfo {
            key: c.key.clone(),
            name: c.name.clone(),
        })
        .collect();

    let char_names: std::collections::HashMap<_, _> = characters
        .iter()
        .map(|c| (c.key.clone(), c.name.clone()))
        .collect();

    let relationships: Vec<CharacterRelationship> = book_data
        .relationships
        .iter()
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
            relationship_type: r.relationship_type.clone(),
        })
        .collect();

    Json(RelationshipsResponse {
        language,
        book: params.book,
        characters,
        relationships,
    })
}
