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
