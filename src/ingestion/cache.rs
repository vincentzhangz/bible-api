use std::collections::HashMap;
use std::path::Path;

use super::visualize::{BookRelationshipsJson, TimelineEventJson, VisualizeLocale};

#[derive(Debug, Clone)]
pub struct CachedLocale {
    pub timeline: Vec<TimelineEventJson>,
    pub books: HashMap<String, BookRelationshipsJson>,
}

#[derive(Debug)]
pub struct VisualizeCache {
    locales: HashMap<String, CachedLocale>,
}

impl VisualizeCache {
    pub async fn load(data_dir: &Path) -> Self {
        let visualize_dir = data_dir.join("visualize");
        let mut locales = HashMap::new();

        if let Ok(entries) = tokio::fs::read_dir(&visualize_dir).await {
            let mut entries = entries;
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json")
                    && let Some(lang) = path.file_stem().and_then(|s| s.to_str())
                    && let Ok(content) = tokio::fs::read_to_string(&path).await
                    && let Ok(locale) = serde_json::from_str::<VisualizeLocale>(&content)
                {
                    locales.insert(
                        lang.to_string(),
                        CachedLocale {
                            timeline: locale.timeline,
                            books: locale.books,
                        },
                    );
                }
            }
        }

        tracing::info!("Loaded {} visualize locale(s)", locales.len());
        Self { locales }
    }

    pub fn get_locale(&self, language: &str) -> Option<&CachedLocale> {
        self.locales
            .get(language)
            .or_else(|| {
                let base = language.split('-').next().unwrap_or(language);
                self.locales.get(base)
            })
            .or_else(|| self.locales.get("en"))
    }

    pub fn get_book_data(
        &self,
        language: &str,
        book: &str,
    ) -> Option<&BookRelationshipsJson> {
        self.get_locale(language)
            .and_then(|l| l.books.get(book))
            .or_else(|| self.locales.get("en").and_then(|l| l.books.get(book)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_cache_returns_none() {
        let cache = VisualizeCache {
            locales: HashMap::new(),
        };
        assert!(cache.get_locale("en").is_none());
        assert!(cache.get_book_data("en", "Genesis").is_none());
    }

    #[test]
    fn test_exact_language_match() {
        let mut locales = HashMap::new();
        locales.insert(
            "en".to_string(),
            CachedLocale {
                timeline: vec![],
                books: HashMap::new(),
            },
        );
        let cache = VisualizeCache { locales };
        assert!(cache.get_locale("en").is_some());
    }

    #[test]
    fn test_base_language_fallback() {
        let mut locales = HashMap::new();
        locales.insert(
            "en".to_string(),
            CachedLocale {
                timeline: vec![],
                books: HashMap::new(),
            },
        );
        let cache = VisualizeCache { locales };
        assert!(cache.get_locale("en-US").is_some());
    }

    #[test]
    fn test_english_fallback() {
        let mut locales = HashMap::new();
        locales.insert(
            "en".to_string(),
            CachedLocale {
                timeline: vec![],
                books: HashMap::new(),
            },
        );
        let cache = VisualizeCache { locales };
        assert!(cache.get_locale("fr").is_some());
    }

    #[test]
    fn test_no_fallback_returns_none() {
        let mut locales = HashMap::new();
        locales.insert(
            "fr".to_string(),
            CachedLocale {
                timeline: vec![],
                books: HashMap::new(),
            },
        );
        let cache = VisualizeCache { locales };
        assert!(cache.get_locale("de").is_none());
    }

    #[test]
    fn test_get_book_data_with_fallback() {
        let mut en_books = HashMap::new();
        en_books.insert(
            "Genesis".to_string(),
            BookRelationshipsJson {
                characters: vec![],
                relationships: vec![],
            },
        );
        let mut locales = HashMap::new();
        locales.insert(
            "en".to_string(),
            CachedLocale {
                timeline: vec![],
                books: en_books,
            },
        );
        locales.insert(
            "fr".to_string(),
            CachedLocale {
                timeline: vec![],
                books: HashMap::new(),
            },
        );
        let cache = VisualizeCache { locales };
        assert!(cache.get_book_data("fr", "Genesis").is_some());
    }
}
