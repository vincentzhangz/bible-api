use std::fs;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct VisualizeLocale {
    pub language: String,
    pub language_name: String,
    pub timeline: Vec<TimelineEventJson>,
    pub books: std::collections::HashMap<String, BookRelationshipsJson>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TimelineEventJson {
    pub key: String,
    pub event: String,
    pub reference: String,
    pub estimated_year: Option<i32>,
    pub category: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct BookRelationshipsJson {
    pub characters: Vec<CharacterJson>,
    pub relationships: Vec<RelationshipJson>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct CharacterJson {
    pub key: String,
    pub name: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct RelationshipJson {
    #[serde(rename = "type")]
    pub relationship_type: String,
    pub from: String,
    pub to: String,
}

pub fn load_visualize_locale(data_dir: &std::path::Path, language: &str) -> Option<VisualizeLocale> {
    let visualize_dir = data_dir.join("visualize");

    // Try exact language match first
    let file_path = visualize_dir.join(format!("{}.json", language));
    if let Ok(content) = fs::read_to_string(&file_path)
        && let Ok(locale) = serde_json::from_str(&content)
    {
        return Some(locale);
    }

    // Try base language (e.g., "en" from "en-US")
    let base_lang = language.split('-').next().unwrap_or(language);
    let file_path = visualize_dir.join(format!("{}.json", base_lang));
    if let Ok(content) = fs::read_to_string(&file_path)
        && let Ok(locale) = serde_json::from_str(&content)
    {
        return Some(locale);
    }

    // Fall back to English
    let fallback_path = visualize_dir.join("en.json");
    if let Ok(content) = fs::read_to_string(&fallback_path)
        && let Ok(locale) = serde_json::from_str(&content)
    {
        return Some(locale);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_load_visualize_locale_exact_match() {
        let dir = tempdir().unwrap();
        let visualize_dir = dir.path().join("visualize");
        fs::create_dir(&visualize_dir).unwrap();

        let locale_content = r#"{
            "language": "en-US",
            "language_name": "English (US)",
            "timeline": [],
            "books": {}
        }"#;
        fs::write(visualize_dir.join("en-US.json"), locale_content).unwrap();

        let result = load_visualize_locale(dir.path(), "en-US");
        assert!(result.is_some());
        assert_eq!(result.unwrap().language, "en-US");
    }

    #[test]
    fn test_load_visualize_locale_base_language_fallback() {
        let dir = tempdir().unwrap();
        let visualize_dir = dir.path().join("visualize");
        fs::create_dir(&visualize_dir).unwrap();

        let locale_content = r#"{
            "language": "en",
            "language_name": "English",
            "timeline": [],
            "books": {}
        }"#;
        fs::write(visualize_dir.join("en.json"), locale_content).unwrap();

        let result = load_visualize_locale(dir.path(), "en-US");
        assert!(result.is_some());
        assert_eq!(result.unwrap().language, "en");
    }

    #[test]
    fn test_load_visualize_locale_english_fallback() {
        let dir = tempdir().unwrap();
        let visualize_dir = dir.path().join("visualize");
        fs::create_dir(&visualize_dir).unwrap();

        let locale_content = r#"{
            "language": "en",
            "language_name": "English",
            "timeline": [],
            "books": {}
        }"#;
        fs::write(visualize_dir.join("en.json"), locale_content).unwrap();

        let result = load_visualize_locale(dir.path(), "fr-FR");
        assert!(result.is_some());
        assert_eq!(result.unwrap().language, "en");
    }

    #[test]
    fn test_load_visualize_locale_none_when_no_fallback() {
        let dir = tempdir().unwrap();
        let visualize_dir = dir.path().join("visualize");
        fs::create_dir(&visualize_dir).unwrap();
        // Create only en.json, but we're looking for fr-FR which will fall back to en, which exists
        // Let's create a locale without en to test none

        let result = load_visualize_locale(dir.path(), "fr-FR");
        assert!(result.is_none());
    }

    #[test]
    fn test_load_visualize_locale_empty_dir() {
        let dir = tempdir().unwrap();
        let result = load_visualize_locale(dir.path(), "en-US");
        assert!(result.is_none());
    }
}
