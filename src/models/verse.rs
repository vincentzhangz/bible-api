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

pub type SearchResult = VerseResponse;

impl From<(String, String, i32, i32, String)> for VerseResponse {
    fn from(tuple: (String, String, i32, i32, String)) -> Self {
        Self {
            translation: tuple.0,
            book: tuple.1,
            chapter: tuple.2,
            verse: tuple.3,
            text: tuple.4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verse_response_serialize() {
        let verse = VerseResponse {
            translation: "kjv".to_string(),
            book: "John".to_string(),
            chapter: 3,
            verse: 16,
            text: "For God so loved the world...".to_string(),
        };

        let json = serde_json::to_string(&verse).unwrap();
        assert!(json.contains("\"translation\":\"kjv\""));
        assert!(json.contains("\"book\":\"John\""));
        assert!(json.contains("\"chapter\":3"));
        assert!(json.contains("\"verse\":16"));
    }

    #[test]
    fn test_verse_response_deserialize() {
        let json = r#"{
            "translation": "niv",
            "book": "Genesis",
            "chapter": 1,
            "verse": 1,
            "text": "In the beginning..."
        }"#;

        let verse: VerseResponse = serde_json::from_str(json).unwrap();
        assert_eq!(verse.translation, "niv");
        assert_eq!(verse.book, "Genesis");
        assert_eq!(verse.chapter, 1);
        assert_eq!(verse.verse, 1);
        assert_eq!(verse.text, "In the beginning...");
    }

    #[test]
    fn test_search_result_is_verse_response() {
        let result: SearchResult = VerseResponse {
            translation: "esv".to_string(),
            book: "Romans".to_string(),
            chapter: 8,
            verse: 28,
            text: "And we know that all things work together...".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"translation\":\"esv\""));
        assert!(json.contains("\"book\":\"Romans\""));
    }

    #[test]
    fn test_verse_response_from_tuple() {
        let tuple = (
            "kjv".to_string(),
            "John".to_string(),
            3,
            16,
            "For God so loved...".to_string(),
        );
        let verse = VerseResponse::from(tuple);
        assert_eq!(verse.translation, "kjv");
        assert_eq!(verse.book, "John");
        assert_eq!(verse.chapter, 3);
        assert_eq!(verse.verse, 16);
        assert_eq!(verse.text, "For God so loved...");
    }
}
