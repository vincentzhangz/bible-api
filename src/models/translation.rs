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
    #[sqlx(rename = "shortname")]
    pub shortname: Option<String>,
    pub year: Option<String>,
    pub publisher: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    #[sqlx(rename = "copyright_statement")]
    pub copyright_statement: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TranslationResponse {
    pub id: String,
    pub name: String,
    pub language: String,
    pub license: String,
    pub source: String,
    #[serde(rename = "shortName")]
    pub short_name: Option<String>,
    pub year: Option<String>,
    pub publisher: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub version: Option<String>,
    #[serde(rename = "copyrightStatement")]
    pub copyright_statement: Option<String>,
}

impl From<Translation> for TranslationResponse {
    fn from(t: Translation) -> Self {
        Self {
            id: t.id,
            name: t.name,
            language: t.language,
            license: t.license_id.unwrap_or_else(|| "none".to_string()),
            source: t.source,
            short_name: t.shortname,
            year: t.year,
            publisher: t.publisher,
            description: t.description,
            url: t.url,
            version: t.version,
            copyright_statement: t.copyright_statement,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Translation {
        Translation {
            id: "kjv".to_string(),
            name: "King James Version".to_string(),
            language: "en".to_string(),
            license_id: None,
            source: "av1611.com".to_string(),
            json_hash: Some("abc123".to_string()),
            shortname: Some("KJV".to_string()),
            year: Some("1611".to_string()),
            publisher: Some("Oxford".to_string()),
            description: Some("Authorised version".to_string()),
            url: Some("https://example.com".to_string()),
            copyright_statement: None,
            version: Some("1".to_string()),
        }
    }

    #[test]
    fn test_translation_response_from_with_license_id() {
        let mut translation = sample();
        translation.license_id = Some("cc-by".to_string());

        let response = TranslationResponse::from(translation);

        assert_eq!(response.id, "kjv");
        assert_eq!(response.name, "King James Version");
        assert_eq!(response.language, "en");
        assert_eq!(response.license, "cc-by");
        assert_eq!(response.source, "av1611.com");
    }

    #[test]
    fn test_translation_response_from_without_license_id() {
        let translation = sample();

        let response = TranslationResponse::from(translation);

        assert_eq!(response.license, "none");
    }

    #[test]
    fn test_translation_response_exposes_metadata() {
        let response = TranslationResponse::from(sample());

        assert_eq!(response.short_name.as_deref(), Some("KJV"));
        assert_eq!(response.year.as_deref(), Some("1611"));
        assert_eq!(response.publisher.as_deref(), Some("Oxford"));
        assert_eq!(response.description.as_deref(), Some("Authorised version"));
        assert_eq!(response.url.as_deref(), Some("https://example.com"));
    }

    #[test]
    fn test_translation_response_serializes_camel_case_metadata() {
        let response = TranslationResponse::from(sample());

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"shortName\":\"KJV\""));
        assert!(json.contains("\"copyrightStatement\""));
    }

    #[test]
    fn test_translation_response_fields_preserved() {
        let translation = sample();
        let response = TranslationResponse::from(translation.clone());

        assert_eq!(response.id, translation.id);
        assert_eq!(response.name, translation.name);
        assert_eq!(response.language, translation.language);
        assert_eq!(response.source, translation.source);
    }
}
