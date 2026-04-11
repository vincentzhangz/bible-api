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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chapter_response_serialize() {
        let chapter = ChapterResponse {
            translation: "kjv".to_string(),
            book: "John".to_string(),
            chapter: 1,
            verses: vec![
                VerseResponse {
                    translation: "kjv".to_string(),
                    book: "John".to_string(),
                    chapter: 1,
                    verse: 1,
                    text: "In the beginning...".to_string(),
                },
                VerseResponse {
                    translation: "kjv".to_string(),
                    book: "John".to_string(),
                    chapter: 1,
                    verse: 2,
                    text: "The same was in the beginning...".to_string(),
                },
            ],
        };

        let json = serde_json::to_string(&chapter).unwrap();
        assert!(json.contains("\"translation\":\"kjv\""));
        assert!(json.contains("\"book\":\"John\""));
        assert!(json.contains("\"chapter\":1"));
        assert!(json.contains("\"verses\""));
    }

    #[test]
    fn test_chapter_response_deserialize() {
        let json = r#"{
            "translation": "niv",
            "book": "Genesis",
            "chapter": 1,
            "verses": [
                {
                    "translation": "niv",
                    "book": "Genesis",
                    "chapter": 1,
                    "verse": 1,
                    "text": "In the beginning..."
                }
            ]
        }"#;

        let chapter: ChapterResponse = serde_json::from_str(json).unwrap();
        assert_eq!(chapter.translation, "niv");
        assert_eq!(chapter.book, "Genesis");
        assert_eq!(chapter.chapter, 1);
        assert_eq!(chapter.verses.len(), 1);
        assert_eq!(chapter.verses[0].verse, 1);
    }

    #[test]
    fn test_chapter_response_empty_verses() {
        let chapter = ChapterResponse {
            translation: "kjv".to_string(),
            book: " Esther".to_string(),
            chapter: 3,
            verses: vec![],
        };

        let json = serde_json::to_string(&chapter).unwrap();
        let deserialized: ChapterResponse = serde_json::from_str(&json).unwrap();
        assert!(deserialized.verses.is_empty());
    }
}
