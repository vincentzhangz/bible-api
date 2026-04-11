use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VerseResponse {
    pub translation: String,
    pub book: String,
    pub chapter: i32,
    pub verse: i32,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchResult {
    pub translation: String,
    pub book: String,
    pub chapter: i32,
    pub verse: i32,
    pub text: String,
}
