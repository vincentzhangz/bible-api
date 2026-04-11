-- Bible Explore API - Initial Schema
-- Complete database schema with all tables and indexes

-- Create licenses table
CREATE TABLE IF NOT EXISTS licenses (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    url TEXT,
    attribution_required BOOLEAN DEFAULT FALSE,
    commercial_use BOOLEAN DEFAULT TRUE
);

-- Create translations table with json_hash for sync
CREATE TABLE IF NOT EXISTS translations (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    language TEXT NOT NULL,
    license_id TEXT REFERENCES licenses(id),
    source TEXT NOT NULL,
    json_hash TEXT,
    shortname TEXT,
    year TEXT,
    publisher TEXT,
    description TEXT,
    url TEXT,
    copyright_statement TEXT,
    italics BOOLEAN DEFAULT FALSE,
    strongs BOOLEAN DEFAULT FALSE,
    red_letter BOOLEAN DEFAULT FALSE,
    paragraph BOOLEAN DEFAULT FALSE,
    official BOOLEAN DEFAULT FALSE,
    research BOOLEAN DEFAULT FALSE,
    version TEXT
);

-- Create books table
CREATE TABLE IF NOT EXISTS books (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    testament TEXT NOT NULL CHECK (testament IN ('old', 'new')),
    ord INTEGER NOT NULL
);

-- Create chapters table
CREATE TABLE IF NOT EXISTS chapters (
    id SERIAL PRIMARY KEY,
    translation_id TEXT NOT NULL REFERENCES translations(id) ON DELETE CASCADE,
    book_id INTEGER NOT NULL REFERENCES books(id),
    chapter_number INTEGER NOT NULL,
    UNIQUE(translation_id, book_id, chapter_number)
);

-- Create verses table
CREATE TABLE IF NOT EXISTS verses (
    id SERIAL PRIMARY KEY,
    chapter_id INTEGER NOT NULL REFERENCES chapters(id) ON DELETE CASCADE,
    verse_number INTEGER NOT NULL,
    text TEXT NOT NULL,
    UNIQUE(chapter_id, verse_number)
);

-- Create cross_references table
CREATE TABLE IF NOT EXISTS cross_references (
    id SERIAL PRIMARY KEY,
    source_verse_id INTEGER NOT NULL REFERENCES verses(id) ON DELETE CASCADE,
    target_verse_id INTEGER NOT NULL REFERENCES verses(id) ON DELETE CASCADE,
    relationship_type TEXT
);

-- Create visualize_timeline table for i18n timeline events
CREATE TABLE IF NOT EXISTS visualize_timeline (
    id SERIAL PRIMARY KEY,
    language TEXT NOT NULL,
    event_key TEXT NOT NULL,
    event_text TEXT NOT NULL,
    reference TEXT NOT NULL,
    estimated_year INTEGER,
    category TEXT NOT NULL,
    UNIQUE(language, event_key)
);

-- Create visualize_relationships table for i18n character relationships
CREATE TABLE IF NOT EXISTS visualize_relationships (
    id SERIAL PRIMARY KEY,
    language TEXT NOT NULL,
    book_key TEXT NOT NULL,
    character_key TEXT NOT NULL,
    character_name TEXT NOT NULL,
    UNIQUE(language, book_key, character_key)
);

-- Create visualize_character_relations table for i18n relationship data
CREATE TABLE IF NOT EXISTS visualize_character_relations (
    id SERIAL PRIMARY KEY,
    language TEXT NOT NULL,
    book_key TEXT NOT NULL,
    rel_type TEXT NOT NULL,
    from_key TEXT NOT NULL,
    to_key TEXT NOT NULL,
    UNIQUE(language, book_key, rel_type, from_key, to_key)
);

-- Create indexes for visualize tables
CREATE INDEX IF NOT EXISTS idx_visualize_timeline_language ON visualize_timeline(language);
CREATE INDEX IF NOT EXISTS idx_visualize_relationships_lookup ON visualize_relationships(language, book_key);
CREATE INDEX IF NOT EXISTS idx_visualize_character_relations_lookup ON visualize_character_relations(language, book_key);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_verses_chapter_id ON verses(chapter_id);
CREATE INDEX IF NOT EXISTS idx_verses_text ON verses USING GIN (to_tsvector('english', text));
CREATE INDEX IF NOT EXISTS idx_chapters_lookup ON chapters(translation_id, book_id, chapter_number);
CREATE INDEX IF NOT EXISTS idx_translations_language ON translations(language);
CREATE INDEX IF NOT EXISTS idx_translations_hash ON translations(json_hash);
CREATE INDEX IF NOT EXISTS idx_cross_references_source ON cross_references(source_verse_id);
