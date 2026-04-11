//! JSON validation tests for Bible data files.
//!
//! Validates structure, required fields, verse numbering, and license references.
//! Run with: `cargo test --test validate_json`

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Test data directory - resolved relative to the crate root
fn data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
}

/// Load a JSON file, returning (data, errors)
fn load_json(path: &Path) -> Result<serde_json::Value, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    serde_json::from_str(&content).map_err(|e| format!("Invalid JSON in {}: {}", path.display(), e))
}

/// Load license IDs from licenses.json
fn load_licenses() -> HashSet<String> {
    let licenses_path = data_dir().join("licenses").join("licenses.json");
    match load_json(&licenses_path) {
        Ok(licenses) => {
            if let Some(arr) = licenses.as_array() {
                arr.iter()
                    .filter_map(|v| v.get("id").and_then(|id| id.as_str()))
                    .map(String::from)
                    .collect()
            } else {
                HashSet::new()
            }
        }
        Err(e) => {
            eprintln!("Warning: Could not load licenses.json: {}", e);
            HashSet::new()
        }
    }
}

/// Validate a translation JSON file
fn validate_translation(
    data: &serde_json::Value,
    translation_id: &str,
    licenses: &HashSet<String>,
) -> Vec<String> {
    let mut errors = Vec::new();

    // Required top-level fields
    let required = ["id", "metadata", "books"];
    for field in required {
        if !data.get(field).is_some() {
            errors.push(format!(
                "[{}] Missing required field: '{}'",
                translation_id, field
            ));
        }
    }

    // Can't continue without these
    if !data.get("id").is_some() || !data.get("metadata").is_some() || !data.get("books").is_some()
    {
        return errors;
    }

    // Validate metadata
    if let Some(metadata) = data.get("metadata") {
        let meta_required = ["name", "language", "license"];
        for field in meta_required {
            if !metadata.get(field).is_some() {
                errors.push(format!(
                    "[{}] Missing metadata field: '{}'",
                    translation_id, field
                ));
            }
        }

        // Check license exists
        if let Some(license) = metadata.get("license").and_then(|l| l.as_str()) {
            if !licenses.contains(license) {
                errors.push(format!(
                    "[{}] Unknown license: '{}' (not in licenses.json)",
                    translation_id, license
                ));
            }
        }
    }

    // Validate books
    if let Some(books) = data.get("books").and_then(|b| b.as_array()) {
        if books.is_empty() {
            errors.push(format!("[{}] No books found", translation_id));
        }

        let mut seen_books = HashSet::new();

        for (i, book) in books.iter().enumerate() {
            let book_errors = validate_book(book, translation_id, i, &mut seen_books);
            errors.extend(book_errors);
        }
    }

    errors
}

/// Validate a single book
fn validate_book(
    book: &serde_json::Value,
    translation_id: &str,
    index: usize,
    seen_books: &mut HashSet<String>,
) -> Vec<String> {
    let mut errors = Vec::new();

    let required = ["id", "name", "testament", "chapters"];
    for field in required {
        if !book.get(field).is_some() {
            errors.push(format!(
                "[{}] Book[{}] missing field: '{}'",
                translation_id, index, field
            ));
            return errors; // Can't continue
        }
    }

    let book_id = book.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let _book_name = book.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let testament = book.get("testament").and_then(|v| v.as_str()).unwrap_or("");

    // Check for duplicate book ID
    if seen_books.contains(book_id) {
        errors.push(format!(
            "[{}] Duplicate book id: '{}'",
            translation_id, book_id
        ));
    }
    seen_books.insert(book_id.to_string());

    // Validate testament
    if !testament.is_empty() && testament != "old" && testament != "new" {
        errors.push(format!(
            "[{}] Book '{}' invalid testament: '{}' (must be 'old' or 'new')",
            translation_id, book_id, testament
        ));
    }

    // Validate chapters
    if let Some(chapters) = book.get("chapters").and_then(|c| c.as_array()) {
        if chapters.is_empty() {
            errors.push(format!(
                "[{}] Book '{}' has no chapters",
                translation_id, book_id
            ));
        }

        for (j, chapter) in chapters.iter().enumerate() {
            if !chapter.get("chapter").is_some() {
                errors.push(format!(
                    "[{}] Book '{}' chapter[{}] missing 'chapter' number",
                    translation_id, book_id, j
                ));
            }
            if !chapter.get("verses").is_some() {
                errors.push(format!(
                    "[{}] Book '{}' chapter[{}] missing 'verses'",
                    translation_id, book_id, j
                ));
                continue;
            }

            if let Some(verses) = chapter.get("verses").and_then(|v| v.as_array()) {
                if verses.is_empty() {
                    if let Some(chapter_num) = chapter.get("chapter").and_then(|c| c.as_i64()) {
                        errors.push(format!(
                            "[{}] Book '{}' chapter {} has no verses",
                            translation_id, book_id, chapter_num
                        ));
                    }
                }

                for (k, verse) in verses.iter().enumerate() {
                    if !verse.get("verse").is_some() {
                        errors.push(format!(
                            "[{}] Book '{}' chapter[{}] verse[{}] missing 'verse' number",
                            translation_id, book_id, j, k
                        ));
                    }
                    if !verse.get("text").is_some() {
                        errors.push(format!(
                            "[{}] Book '{}' chapter[{}] verse[{}] missing 'text'",
                            translation_id, book_id, j, k
                        ));
                    }
                }
            }
        }
    }

    errors
}

/// Validate a visualize JSON file
fn validate_visualize(data: &serde_json::Value, file_name: &str) -> Vec<String> {
    let mut errors = Vec::new();

    let required = ["language", "language_name", "timeline", "books"];
    for field in required {
        if !data.get(field).is_some() {
            errors.push(format!(
                "[{}] Missing required field: '{}'",
                file_name, field
            ));
            return errors;
        }
    }

    let language = data.get("language").and_then(|l| l.as_str()).unwrap_or("");

    // Validate timeline events
    if let Some(timeline) = data.get("timeline").and_then(|t| t.as_array()) {
        for (i, event) in timeline.iter().enumerate() {
            let event_errors = validate_timeline_event(event, language, i);
            errors.extend(event_errors);
        }
    }

    // Validate books
    if let Some(books) = data.get("books").and_then(|b| b.as_object()) {
        for (book_key, book_data) in books {
            let book_errors = validate_book_relationships(book_data, language, book_key);
            errors.extend(book_errors);
        }
    }

    errors
}

/// Validate a single timeline event
fn validate_timeline_event(event: &serde_json::Value, language: &str, index: usize) -> Vec<String> {
    let mut errors = Vec::new();

    let required = ["key", "event", "reference", "estimated_year", "category"];
    for field in required {
        if !event.get(field).is_some() {
            errors.push(format!(
                "[{}] Timeline[{}] missing field: '{}'",
                language, index, field
            ));
        }
    }

    errors
}

/// Validate book relationships data
fn validate_book_relationships(
    book_data: &serde_json::Value,
    language: &str,
    book_key: &str,
) -> Vec<String> {
    let mut errors = Vec::new();

    let required = ["characters", "relationships"];
    for field in required {
        if !book_data.get(field).is_some() {
            errors.push(format!(
                "[{}] Book '{}' missing field: '{}'",
                language, book_key, field
            ));
            return errors;
        }
    }

    // Validate characters
    if let Some(characters) = book_data.get("characters").and_then(|c| c.as_array()) {
        for (i, char) in characters.iter().enumerate() {
            if !char.get("key").is_some() {
                errors.push(format!(
                    "[{}] Book '{}' character[{}] missing 'key'",
                    language, book_key, i
                ));
            }
            if !char.get("name").is_some() {
                errors.push(format!(
                    "[{}] Book '{}' character[{}] missing 'name'",
                    language, book_key, i
                ));
            }
        }
    }

    // Validate relationships
    if let Some(relationships) = book_data.get("relationships").and_then(|r| r.as_array()) {
        for (i, rel) in relationships.iter().enumerate() {
            for field in &["type", "from", "to"] {
                if !rel.get(*field).is_some() {
                    errors.push(format!(
                        "[{}] Book '{}' relationship[{}] missing '{}'",
                        language, book_key, i, field
                    ));
                }
            }
        }
    }

    errors
}

// ============ TESTS ============

#[test]
fn test_load_licenses() {
    let licenses = load_licenses();
    assert!(!licenses.is_empty(), "Should have loaded some licenses");
    assert!(
        licenses.contains("public-domain"),
        "Should contain public-domain license"
    );
    assert!(
        licenses.contains("cc-by-4.0"),
        "Should contain cc-by-4.0 license"
    );
}

#[test]
fn test_validate_translation_kjv() {
    let licenses = load_licenses();
    let path = data_dir().join("translations").join("en-kjv.json");
    let data = load_json(&path).expect("Should load en-kjv.json");

    let errors = validate_translation(&data, "en-kjv", &licenses);
    assert!(
        errors.is_empty(),
        "en-kjv.json should have no validation errors: {:?}",
        errors
    );
}

#[test]
fn test_validate_translation_id_tb() {
    let licenses = load_licenses();
    let path = data_dir().join("translations").join("id-tb.json");
    let data = load_json(&path).expect("Should load id-tb.json");

    let errors = validate_translation(&data, "id-tb", &licenses);
    assert!(
        errors.is_empty(),
        "id-tb.json should have no validation errors: {:?}",
        errors
    );
}

#[test]
fn test_validate_all_translations() {
    let licenses = load_licenses();
    let translations_dir = data_dir().join("translations");

    let mut all_errors = Vec::new();

    for entry in fs::read_dir(translations_dir).expect("Should read translations dir") {
        let entry = entry.expect("Should read entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let data = match load_json(&path) {
                Ok(d) => d,
                Err(e) => {
                    all_errors.push(e);
                    continue;
                }
            };
            let translation_id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            let errors = validate_translation(&data, translation_id, &licenses);
            all_errors.extend(errors);
        }
    }

    assert!(
        all_errors.is_empty(),
        "All translations should be valid: {:?}",
        all_errors
    );
}

#[test]
fn test_validate_visualize_en() {
    let path = data_dir().join("visualize").join("en.json");
    let data = load_json(&path).expect("Should load en.json");

    let errors = validate_visualize(&data, "en.json");
    assert!(
        errors.is_empty(),
        "en.json should have no validation errors: {:?}",
        errors
    );
}

#[test]
fn test_validate_visualize_id() {
    let path = data_dir().join("visualize").join("id.json");
    let data = load_json(&path).expect("Should load id.json");

    let errors = validate_visualize(&data, "id.json");
    assert!(
        errors.is_empty(),
        "id.json should have no validation errors: {:?}",
        errors
    );
}

#[test]
fn test_validate_all_visualize_files() {
    let visualize_dir = data_dir().join("visualize");

    let mut all_errors = Vec::new();

    for entry in fs::read_dir(visualize_dir).expect("Should read visualize dir") {
        let entry = entry.expect("Should read entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let data = match load_json(&path) {
                Ok(d) => d,
                Err(e) => {
                    all_errors.push(e);
                    continue;
                }
            };
            let file_name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            let errors = validate_visualize(&data, file_name);
            all_errors.extend(errors);
        }
    }

    assert!(
        all_errors.is_empty(),
        "All visualize files should be valid: {:?}",
        all_errors
    );
}

#[test]
fn test_licenses_file_exists() {
    let licenses_path = data_dir().join("licenses").join("licenses.json");
    assert!(
        licenses_path.exists(),
        "licenses.json should exist at {}",
        licenses_path.display()
    );
}

#[test]
fn test_translations_directory_exists() {
    let translations_dir = data_dir().join("translations");
    assert!(
        translations_dir.is_dir(),
        "translations directory should exist"
    );
}

#[test]
fn test_visualize_directory_exists() {
    let visualize_dir = data_dir().join("visualize");
    assert!(visualize_dir.is_dir(), "visualize directory should exist");
}
