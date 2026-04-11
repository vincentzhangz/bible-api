#!/usr/bin/env python3
"""
Validate Bible JSON files in data/translations/ and data/visualize/
Checks structure, required fields, verse numbering, and license references.
Uses JSON Schema 2020-12 for structural validation.
"""

import json
import sys
from pathlib import Path

try:
    import jsonschema
    HAS_JSONSCHEMA = True
except ImportError:
    HAS_JSONSCHEMA = False


def load_json(path: Path) -> tuple[dict, list[str]]:
    """Load JSON file, return (data, errors)."""
    try:
        with open(path, 'r', encoding='utf-8') as f:
            return json.load(f), []
    except json.JSONDecodeError as e:
        return {}, [f"Invalid JSON: {e}"]
    except Exception as e:
        return {}, [f"Read error: {e}"]


def validate_translation(data: dict, path: Path, licenses: set[str]) -> list[str]:
    """Validate a single translation JSON file."""
    errors = []

    # Required top-level fields
    required = ['id', 'metadata', 'books']
    for field in required:
        if field not in data:
            errors.append(f"Missing required field: '{field}'")

    if 'id' not in data or 'metadata' not in data or 'books' not in data:
        return errors  # Can't continue without these

    translation_id = data['id']

    # Validate metadata
    metadata = data.get('metadata', {})
    meta_required = ['name', 'language', 'license']
    for field in meta_required:
        if field not in metadata:
            errors.append(f"[{translation_id}] Missing metadata field: '{field}'")

    # Check license exists
    if 'license' in metadata:
        if metadata['license'] not in licenses:
            errors.append(f"[{translation_id}] Unknown license: '{metadata['license']}' (not in licenses.json)")

    # Validate books
    books = data.get('books', [])
    if not books:
        errors.append(f"[{translation_id}] No books found")

    seen_books = set()
    book_num_to_name = {}

    for i, book in enumerate(books):
        book_errors = validate_book(book, translation_id, i, seen_books, book_num_to_name)
        errors.extend(book_errors)

    return errors


def validate_book(book: dict, translation_id: str, index: int, seen_books: set, book_num_to_name: dict) -> list[str]:
    """Validate a single book."""
    errors = []

    required = ['id', 'name', 'testament', 'chapters']
    for field in required:
        if field not in book:
            errors.append(f"[{translation_id}] Book[{index}] missing field: '{field}'")
            return errors  # Can't continue

    book_id = book['id']
    book_name = book['name']
    testament = book['testament']

    # Check for duplicate book ID
    if book_id in seen_books:
        errors.append(f"[{translation_id}] Duplicate book id: '{book_id}'")
    seen_books.add(book_id)

    # Validate testament
    if testament not in ('old', 'new'):
        errors.append(f"[{translation_id}] Book '{book_id}' invalid testament: '{testament}' (must be 'old' or 'new')")

    # Validate chapters
    chapters = book.get('chapters', [])
    if not chapters:
        errors.append(f"[{translation_id}] Book '{book_id}' has no chapters")

    prev_chapter = 0
    for j, chapter in enumerate(chapters):
        if 'chapter' not in chapter:
            errors.append(f"[{translation_id}] Book '{book_id}' chapter[{j}] missing 'chapter' number")
            continue
        if 'verses' not in chapter:
            errors.append(f"[{translation_id}] Book '{book_id}' chapter[{j}] missing 'verses'")
            continue

        chapter_num = chapter['chapter']
        verses = chapter['verses']
        if not verses:
            errors.append(f"[{translation_id}] Book '{book_id}' chapter {chapter_num} has no verses")
            continue

        prev_verse = 0
        for k, verse in enumerate(verses):
            if 'verse' not in verse:
                errors.append(f"[{translation_id}] Book '{book_id}' chapter {chapter_num} verse[{k}] missing 'verse' number")
                continue
            if 'text' not in verse:
                errors.append(f"[{translation_id}] Book '{book_id}' chapter {chapter_num} verse[{k}] missing 'text'")
                continue

            verse_num = verse['verse']
            # Check for duplicate verse numbers within chapter
            verse_key = (book_id, chapter_num, verse_num)
            if verse_key in book_num_to_name:
                errors.append(f"[{translation_id}] Book '{book_id}' chapter {chapter_num} duplicate verse: {verse_num}")

    return errors


def load_licenses(licenses_dir: Path) -> set[str]:
    """Load license IDs from licenses.json."""
    licenses_path = licenses_dir / "licenses.json"
    if not licenses_path.exists():
        print(f"Warning: {licenses_path} not found, skipping license validation")
        return set()

    try:
        with open(licenses_path, 'r', encoding='utf-8') as f:
            licenses_data = json.load(f)
        return {lic['id'] for lic in licenses_data}
    except Exception as e:
        print(f"Warning: Could not load licenses.json: {e}")
        return set()


def load_schema(schema_path: Path) -> dict | None:
    """Load JSON Schema file."""
    if not schema_path.exists():
        print(f"Warning: Schema not found at {schema_path}, skipping schema validation")
        return None
    try:
        with open(schema_path, 'r', encoding='utf-8') as f:
            return json.load(f)
    except Exception as e:
        print(f"Warning: Could not load schema: {e}")
        return None


def validate_with_schema(data: dict, schema: dict, file_id: str, schema_name: str) -> list[str]:
    """Validate data against JSON Schema 2020-12."""
    errors = []
    if HAS_JSONSCHEMA:
        validator = jsonschema.Draft202012Validator(schema)
        for error in validator.iter_errors(data):
            path = '.'.join(str(p) for p in error.path) if error.path else 'root'
            errors.append(f"[{file_id}] {schema_name} schema validation failed at '{path}': {error.message}")
    return errors


def validate_visualize(data: dict, path: Path) -> list[str]:
    """Validate a single visualize JSON file."""
    errors = []

    required = ['language', 'language_name', 'timeline', 'books']
    for field in required:
        if field not in data:
            errors.append(f"[{path.name}] Missing required field: '{field}'")
            return errors

    language = data['language']

    # Validate timeline events
    for i, event in enumerate(data.get('timeline', [])):
        event_errors = validate_timeline_event(event, language, i)
        errors.extend(event_errors)

    # Validate books
    books = data.get('books', {})
    for book_key, book_data in books.items():
        book_errors = validate_book_relationships(book_data, language, book_key)
        errors.extend(book_errors)

    return errors


def validate_timeline_event(event: dict, language: str, index: int) -> list[str]:
    """Validate a single timeline event."""
    errors = []

    required = ['key', 'event', 'reference', 'estimated_year', 'category']
    for field in required:
        if field not in event:
            errors.append(f"[{language}] Timeline[{index}] missing field: '{field}'")

    return errors


def validate_book_relationships(book_data: dict, language: str, book_key: str) -> list[str]:
    """Validate book relationships data."""
    errors = []

    required = ['characters', 'relationships']
    for field in required:
        if field not in book_data:
            errors.append(f"[{language}] Book '{book_key}' missing field: '{field}'")
            return errors

    # Validate characters have required fields
    for i, char in enumerate(book_data.get('characters', [])):
        if 'key' not in char:
            errors.append(f"[{language}] Book '{book_key}' character[{i}] missing 'key'")
        if 'name' not in char:
            errors.append(f"[{language}] Book '{book_key}' character[{i}] missing 'name'")

    # Validate relationships have required fields
    for i, rel in enumerate(book_data.get('relationships', [])):
        for field in ['type', 'from', 'to']:
            if field not in rel:
                errors.append(f"[{language}] Book '{book_key}' relationship[{i}] missing '{field}'")

    return errors


def main():
    base_dir = Path(__file__).parent.parent
    translations_dir = base_dir / "data" / "translations"
    visualize_dir = base_dir / "data" / "visualize"
    licenses_dir = base_dir / "data" / "licenses"
    translation_schema_path = base_dir / "schema" / "translation.schema.json"
    visualize_schema_path = base_dir / "schema" / "visualize.schema.json"

    all_errors = []
    files_validated = 0

    # Load schemas
    translation_schema = load_schema(translation_schema_path)
    visualize_schema = load_schema(visualize_schema_path)

    if translation_schema:
        print(f"Loaded translation schema from {translation_schema_path}")
    if visualize_schema:
        print(f"Loaded visualize schema from {visualize_schema_path}")

    # Load licenses for translation validation
    licenses = load_licenses(licenses_dir)
    print(f"Loaded {len(licenses)} licenses from licenses.json")

    # Validate translations
    if translations_dir.exists():
        print(f"\n=== Validating translations ===")
        for json_file in sorted(translations_dir.glob("*.json")):
            data, read_errors = load_json(json_file)
            if read_errors:
                all_errors.extend([f"[{json_file.name}] {e}" for e in read_errors])
                files_validated += 1
                continue

            translation_id = data.get('id', json_file.stem)

            # Validate with schema if available
            if translation_schema:
                schema_errors = validate_with_schema(data, translation_schema, translation_id, "translation")
                all_errors.extend(schema_errors)

            errors = validate_translation(data, json_file, licenses)
            if errors:
                all_errors.extend(errors)
            else:
                book_count = len(data.get('books', []))
                verse_count = sum(len(ch['verses']) for b in data['books'] for ch in b['chapters'])
                print(f"[OK] {json_file.name} ({book_count} books, {verse_count} verses)")

            files_validated += 1
    else:
        print(f"Warning: Translations directory not found: {translations_dir}")

    # Validate visualize files
    if visualize_dir.exists():
        print(f"\n=== Validating visualize ===")
        for json_file in sorted(visualize_dir.glob("*.json")):
            data, read_errors = load_json(json_file)
            if read_errors:
                all_errors.extend([f"[{json_file.name}] {e}" for e in read_errors])
                files_validated += 1
                continue

            lang = data.get('language', json_file.stem)

            # Validate with schema if available
            if visualize_schema:
                schema_errors = validate_with_schema(data, visualize_schema, lang, "visualize")
                all_errors.extend(schema_errors)

            errors = validate_visualize(data, json_file)
            if errors:
                all_errors.extend(errors)
            else:
                timeline_count = len(data.get('timeline', []))
                book_count = len(data.get('books', {}))
                print(f"[OK] {json_file.name} ({timeline_count} events, {book_count} books)")

            files_validated += 1
    else:
        print(f"Warning: Visualize directory not found: {visualize_dir}")

    print(f"\nValidated {files_validated} files")

    if all_errors:
        print(f"\nFAILED: {len(all_errors)} validation error(s):")
        for error in all_errors:
            print(f"  - {error}")
        sys.exit(1)
    else:
        print(f"\nAll files valid!")
        sys.exit(0)


if __name__ == "__main__":
    main()
