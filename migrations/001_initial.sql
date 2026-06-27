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

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_verses_chapter_id ON verses(chapter_id);
CREATE INDEX IF NOT EXISTS idx_verses_text ON verses USING GIN (to_tsvector('english', text));
CREATE INDEX IF NOT EXISTS idx_chapters_lookup ON chapters(translation_id, book_id, chapter_number);
CREATE INDEX IF NOT EXISTS idx_translations_language ON translations(language);
CREATE INDEX IF NOT EXISTS idx_translations_hash ON translations(json_hash);
CREATE INDEX IF NOT EXISTS idx_cross_references_source ON cross_references(source_verse_id);
CREATE INDEX IF NOT EXISTS idx_cross_references_target ON cross_references(target_verse_id);
CREATE INDEX IF NOT EXISTS idx_books_lower_name ON books(LOWER(name));
