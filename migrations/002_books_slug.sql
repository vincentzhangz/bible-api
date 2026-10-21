-- Add URL-safe slugs to books so clients can address a book by slug
-- (e.g. "1-samuel") instead of only by its display name ("1 Samuel").

ALTER TABLE books ADD COLUMN IF NOT EXISTS slug TEXT;

UPDATE books
SET slug = TRIM(BOTH '-' FROM LOWER(REGEXP_REPLACE(name, '[^a-zA-Z0-9]+', '-', 'g')))
WHERE slug IS NULL OR slug = '';

CREATE UNIQUE INDEX IF NOT EXISTS idx_books_slug ON books(slug);

ALTER TABLE books ALTER COLUMN slug SET NOT NULL;
