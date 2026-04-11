use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Translation {
    pub id: String,
    pub name: String,
    pub language: String,
    #[serde(rename = "licenseId")]
    #[sqlx(rename = "license_id")]
    pub license_id: Option<String>,
    pub source: String,
    #[serde(rename = "jsonHash")]
    #[sqlx(rename = "json_hash")]
    pub json_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TranslationResponse {
    pub id: String,
    pub name: String,
    pub language: String,
    pub license: String,
    pub source: String,
}

impl From<Translation> for TranslationResponse {
    fn from(t: Translation) -> Self {
        Self {
            id: t.id,
            name: t.name,
            language: t.language,
            license: t.license_id.map(|_| "unknown".to_string()).unwrap_or_else(|| "none".to_string()),
            source: t.source,
        }
    }
}
