-- Books become a canonical, shared list: every translation indexes against the
-- same 66 books, and a translation that renames a book keeps its own name in
-- `translation_book_names`.

CREATE TABLE IF NOT EXISTS translation_book_names (
    translation_id TEXT NOT NULL REFERENCES translations(id) ON DELETE CASCADE,
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    PRIMARY KEY (translation_id, book_id)
);

CREATE INDEX IF NOT EXISTS idx_translation_book_names_book ON translation_book_names(book_id);

-- Clear the old per-translation book rows. The ingestion rebuilds chapters and
-- verses against the canonical books immediately after the migration runs.
DELETE FROM cross_references;
DELETE FROM verses;
DELETE FROM chapters;
DELETE FROM translation_book_names;
DELETE FROM books;

ALTER SEQUENCE books_id_seq RESTART WITH 1;
