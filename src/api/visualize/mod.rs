pub mod cross_references;
pub mod relationships;
pub mod timeline;
pub mod word_frequency;

pub use cross_references::cross_references;
pub use relationships::relationships;
pub use timeline::timeline;
pub use word_frequency::word_frequency;

/// Extracts the language code from a translation ID.
/// E.g., "en-kjv" -> "en", "id-tb" -> "id"
pub fn language_from_translation(translation: &str) -> String {
    translation.split('-').next().unwrap_or("en").to_string()
}
