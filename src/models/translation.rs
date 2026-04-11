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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_response_from_with_license_id() {
        let translation = Translation {
            id: "tst".to_string(),
            name: "Translation Name".to_string(),
            language: "en".to_string(),
            license_id: Some("cc-by".to_string()),
            source: "example.com".to_string(),
            json_hash: None,
        };

        let response = TranslationResponse::from(translation);

        assert_eq!(response.id, "tst");
        assert_eq!(response.name, "Translation Name");
        assert_eq!(response.language, "en");
        assert_eq!(response.license, "unknown");
        assert_eq!(response.source, "example.com");
    }

    #[test]
    fn test_translation_response_from_without_license_id() {
        let translation = Translation {
            id: "tst".to_string(),
            name: "Translation Name".to_string(),
            language: "en".to_string(),
            license_id: None,
            source: "example.com".to_string(),
            json_hash: None,
        };

        let response = TranslationResponse::from(translation);

        assert_eq!(response.id, "tst");
        assert_eq!(response.name, "Translation Name");
        assert_eq!(response.language, "en");
        assert_eq!(response.license, "none");
        assert_eq!(response.source, "example.com");
    }

    #[test]
    fn test_translation_response_fields_preserved() {
        let translation = Translation {
            id: "kjv".to_string(),
            name: "King James Version".to_string(),
            language: "en".to_string(),
            license_id: None,
            source: "av1611.com".to_string(),
            json_hash: Some("abc123".to_string()),
        };

        let response = TranslationResponse::from(translation.clone());

        assert_eq!(response.id, translation.id);
        assert_eq!(response.name, translation.name);
        assert_eq!(response.language, translation.language);
        assert_eq!(response.source, translation.source);
    }
}
