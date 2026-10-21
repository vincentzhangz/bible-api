//! Small shared helpers that are used by both the ingestion pipeline and the
//! HTTP layer.

/// Converts a display name into a URL-safe slug.
///
/// `"1 Samuel"` -> `"1-samuel"`, `"Song of Solomon"` -> `"song-of-solomon"`.
/// The result always matches the slug stored on the `books` table, so clients
/// can address a book by either its name or its slug.
pub fn slugify(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut pending_separator = false;

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_separator && !out.is_empty() {
                out.push('-');
            }
            pending_separator = false;
            out.push(ch.to_ascii_lowercase());
        } else {
            pending_separator = true;
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugifies_names() {
        assert_eq!(slugify("Genesis"), "genesis");
        assert_eq!(slugify("1 Samuel"), "1-samuel");
        assert_eq!(slugify("Song of Solomon"), "song-of-solomon");
        assert_eq!(slugify("Psalms"), "psalms");
    }

    #[test]
    fn collapses_separators_and_trims() {
        assert_eq!(slugify("  John  "), "john");
        assert_eq!(slugify("Song  of   Solomon"), "song-of-solomon");
        assert_eq!(slugify("Mark---Luke"), "mark-luke");
    }

    #[test]
    fn keeps_only_ascii_alphanumerics() {
        // Must stay in lockstep with the SQL backfill in 002_books_slug.sql,
        // which replaces every non `[a-zA-Z0-9]` run with a single dash.
        assert_eq!(slugify("Yohanan"), "yohanan");
        assert_eq!(slugify("1 Corínthians"), "1-cor-nthians");
    }
}
