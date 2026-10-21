//! The canonical list of books.
//!
//! Every translation is indexed against the same 66 books, so that a passage
//! can be located in any translation. A translation may display a book under a
//! different name (the Indonesian Terjemahan Baru calls Genesis "Kejadian"),
//! so each canonical book also carries the aliases it is known by.

/// A book of the Bible, in canonical order.
pub struct CanonicalBook {
    pub name: &'static str,
    pub testament: &'static str,
    /// Other names this book is known by, in other translations.
    pub aliases: &'static [&'static str],
}

const OT: &str = "old";
const NT: &str = "new";

pub const CANONICAL_BOOKS: &[CanonicalBook] = &[
    CanonicalBook {
        name: "Genesis",
        testament: OT,
        aliases: &["Kejadian"],
    },
    CanonicalBook {
        name: "Exodus",
        testament: OT,
        aliases: &["Keluaran"],
    },
    CanonicalBook {
        name: "Leviticus",
        testament: OT,
        aliases: &["Imamat"],
    },
    CanonicalBook {
        name: "Numbers",
        testament: OT,
        aliases: &["Bilangan"],
    },
    CanonicalBook {
        name: "Deuteronomy",
        testament: OT,
        aliases: &["Ulangan"],
    },
    CanonicalBook {
        name: "Joshua",
        testament: OT,
        aliases: &["Yosua"],
    },
    CanonicalBook {
        name: "Judges",
        testament: OT,
        aliases: &["Hakim-hakim"],
    },
    CanonicalBook {
        name: "Ruth",
        testament: OT,
        aliases: &["Rut"],
    },
    CanonicalBook {
        name: "1 Samuel",
        testament: OT,
        aliases: &["I Samuel"],
    },
    CanonicalBook {
        name: "2 Samuel",
        testament: OT,
        aliases: &["II Samuel"],
    },
    CanonicalBook {
        name: "1 Kings",
        testament: OT,
        aliases: &["1 Raja-raja", "I Kings"],
    },
    CanonicalBook {
        name: "2 Kings",
        testament: OT,
        aliases: &["2 Raja-raja", "II Kings"],
    },
    CanonicalBook {
        name: "1 Chronicles",
        testament: OT,
        aliases: &["1 Tawarikh", "I Chronicles"],
    },
    CanonicalBook {
        name: "2 Chronicles",
        testament: OT,
        aliases: &["2 Tawarikh", "II Chronicles"],
    },
    CanonicalBook {
        name: "Ezra",
        testament: OT,
        aliases: &[],
    },
    CanonicalBook {
        name: "Nehemiah",
        testament: OT,
        aliases: &["Nehemia"],
    },
    CanonicalBook {
        name: "Esther",
        testament: OT,
        aliases: &["Ester"],
    },
    CanonicalBook {
        name: "Job",
        testament: OT,
        aliases: &["Ayub"],
    },
    CanonicalBook {
        name: "Psalms",
        testament: OT,
        aliases: &["Mazmur"],
    },
    CanonicalBook {
        name: "Proverbs",
        testament: OT,
        aliases: &["Amsal"],
    },
    CanonicalBook {
        name: "Ecclesiastes",
        testament: OT,
        aliases: &["Pengkhotbah"],
    },
    CanonicalBook {
        name: "Song of Solomon",
        testament: OT,
        aliases: &["Kidung Agung"],
    },
    CanonicalBook {
        name: "Isaiah",
        testament: OT,
        aliases: &["Yesaya"],
    },
    CanonicalBook {
        name: "Jeremiah",
        testament: OT,
        aliases: &["Yeremia"],
    },
    CanonicalBook {
        name: "Lamentations",
        testament: OT,
        aliases: &["Ratapan"],
    },
    CanonicalBook {
        name: "Ezekiel",
        testament: OT,
        aliases: &["Yehezkiel"],
    },
    CanonicalBook {
        name: "Daniel",
        testament: OT,
        aliases: &[],
    },
    CanonicalBook {
        name: "Hosea",
        testament: OT,
        aliases: &[],
    },
    CanonicalBook {
        name: "Joel",
        testament: OT,
        aliases: &["Yoel"],
    },
    CanonicalBook {
        name: "Amos",
        testament: OT,
        aliases: &[],
    },
    CanonicalBook {
        name: "Obadiah",
        testament: OT,
        aliases: &["Obaja"],
    },
    CanonicalBook {
        name: "Jonah",
        testament: OT,
        aliases: &["Yunus"],
    },
    CanonicalBook {
        name: "Micah",
        testament: OT,
        aliases: &["Mikha"],
    },
    CanonicalBook {
        name: "Nahum",
        testament: OT,
        aliases: &[],
    },
    CanonicalBook {
        name: "Habakkuk",
        testament: OT,
        aliases: &["Habakuk"],
    },
    CanonicalBook {
        name: "Zephaniah",
        testament: OT,
        aliases: &["Zefanya"],
    },
    CanonicalBook {
        name: "Haggai",
        testament: OT,
        aliases: &["Hagai"],
    },
    CanonicalBook {
        name: "Zechariah",
        testament: OT,
        aliases: &["Zakharia"],
    },
    CanonicalBook {
        name: "Malachi",
        testament: OT,
        aliases: &["Maleakhi"],
    },
    CanonicalBook {
        name: "Matthew",
        testament: NT,
        aliases: &["Matius"],
    },
    CanonicalBook {
        name: "Mark",
        testament: NT,
        aliases: &["Markus"],
    },
    CanonicalBook {
        name: "Luke",
        testament: NT,
        aliases: &["Lukas"],
    },
    CanonicalBook {
        name: "John",
        testament: NT,
        aliases: &["Yohanes"],
    },
    CanonicalBook {
        name: "Acts",
        testament: NT,
        aliases: &["Kisah Rasul-rasul"],
    },
    CanonicalBook {
        name: "Romans",
        testament: NT,
        aliases: &["Roma"],
    },
    CanonicalBook {
        name: "1 Corinthians",
        testament: NT,
        aliases: &["1 Korintus", "I Corinthians"],
    },
    CanonicalBook {
        name: "2 Corinthians",
        testament: NT,
        aliases: &["2 Korintus", "II Corinthians"],
    },
    CanonicalBook {
        name: "Galatians",
        testament: NT,
        aliases: &["Galatia"],
    },
    CanonicalBook {
        name: "Ephesians",
        testament: NT,
        aliases: &["Efesus"],
    },
    CanonicalBook {
        name: "Philippians",
        testament: NT,
        aliases: &["Filipi"],
    },
    CanonicalBook {
        name: "Colossians",
        testament: NT,
        aliases: &["Kolose"],
    },
    CanonicalBook {
        name: "1 Thessalonians",
        testament: NT,
        aliases: &["1 Tesalonika", "I Thessalonians"],
    },
    CanonicalBook {
        name: "2 Thessalonians",
        testament: NT,
        aliases: &["2 Tesalonika", "II Thessalonians"],
    },
    CanonicalBook {
        name: "1 Timothy",
        testament: NT,
        aliases: &["1 Timotius", "I Timothy"],
    },
    CanonicalBook {
        name: "2 Timothy",
        testament: NT,
        aliases: &["2 Timotius", "II Timothy"],
    },
    CanonicalBook {
        name: "Titus",
        testament: NT,
        aliases: &[],
    },
    CanonicalBook {
        name: "Philemon",
        testament: NT,
        aliases: &["Filemon"],
    },
    CanonicalBook {
        name: "Hebrews",
        testament: NT,
        aliases: &["Ibrani"],
    },
    CanonicalBook {
        name: "James",
        testament: NT,
        aliases: &["Yakobus"],
    },
    CanonicalBook {
        name: "1 Peter",
        testament: NT,
        aliases: &["1 Petrus", "I Peter"],
    },
    CanonicalBook {
        name: "2 Peter",
        testament: NT,
        aliases: &["2 Petrus", "II Peter"],
    },
    CanonicalBook {
        name: "1 John",
        testament: NT,
        aliases: &["1 Yohanes", "I John"],
    },
    CanonicalBook {
        name: "2 John",
        testament: NT,
        aliases: &["2 Yohanes", "II John"],
    },
    CanonicalBook {
        name: "3 John",
        testament: NT,
        aliases: &["3 Yohanes", "III John"],
    },
    CanonicalBook {
        name: "Jude",
        testament: NT,
        aliases: &["Yudas"],
    },
    CanonicalBook {
        name: "Revelation",
        testament: NT,
        aliases: &["Wahyu", "Revelation of John", "Apocalypse"],
    },
];

/// Finds the canonical book a translation's book name refers to.
///
/// Matching is case-insensitive and considers the canonical name as well as
/// every known alias.
pub fn find_by_name(name: &str) -> Option<(usize, &'static CanonicalBook)> {
    let needle = name.trim().to_lowercase();

    CANONICAL_BOOKS.iter().enumerate().find(|(_, book)| {
        book.name.to_lowercase() == needle
            || book
                .aliases
                .iter()
                .any(|alias| alias.to_lowercase() == needle)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_the_full_protestant_canon() {
        assert_eq!(CANONICAL_BOOKS.len(), 66);
        assert_eq!(
            CANONICAL_BOOKS.iter().filter(|b| b.testament == OT).count(),
            39
        );
        assert_eq!(
            CANONICAL_BOOKS.iter().filter(|b| b.testament == NT).count(),
            27
        );
    }

    #[test]
    fn matches_english_names() {
        let (index, book) = find_by_name("Genesis").expect("Genesis is canonical");
        assert_eq!(index, 0);
        assert_eq!(book.name, "Genesis");
    }

    #[test]
    fn matches_indonesian_aliases() {
        let (_, book) = find_by_name("Kejadian").expect("Kejadian is an alias");
        assert_eq!(book.name, "Genesis");

        let (_, book) = find_by_name("Yohanes").expect("Yohanes is an alias");
        assert_eq!(book.name, "John");

        let (_, book) = find_by_name("Wahyu").expect("Wahyu is an alias");
        assert_eq!(book.name, "Revelation");
    }

    #[test]
    fn matching_is_case_insensitive_and_trims() {
        assert!(find_by_name("  john ").is_some());
        assert!(find_by_name("KEJADIAN").is_some());
    }

    #[test]
    fn unknown_names_do_not_match() {
        assert!(find_by_name("Book of Enoch").is_none());
        assert!(find_by_name("").is_none());
    }

    #[test]
    fn canonical_names_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for book in CANONICAL_BOOKS {
            assert!(
                seen.insert(book.name.to_lowercase()),
                "duplicate {}",
                book.name
            );
        }
    }

    #[test]
    fn aliases_do_not_collide_between_books() {
        let mut seen = std::collections::HashSet::new();
        for book in CANONICAL_BOOKS {
            for alias in book.aliases {
                assert!(
                    seen.insert(alias.to_lowercase()),
                    "duplicate alias {}",
                    alias
                );
            }
        }
    }
}
