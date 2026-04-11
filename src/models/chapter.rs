use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::verse::VerseResponse;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChapterResponse {
    pub translation: String,
    pub book: String,
    pub chapter: i32,
    pub verses: Vec<VerseResponse>,
}
