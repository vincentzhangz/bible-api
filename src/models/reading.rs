use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A single verse of scripture with its number.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VerseText {
    pub verse: i32,
    pub text: String,
}

/// The passage immediately before or after the verse under study.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NeighborPassage {
    pub book: String,
    pub book_slug: String,
    pub chapter: i32,
    pub verse: i32,
    pub text: String,
}

/// A passage that speaks to the verse under study.
///
/// `source` distinguishes a curated link stored in the `cross_references`
/// table from a thematic link derived from shared vocabulary.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CrossReference {
    pub book: String,
    pub book_slug: String,
    pub chapter: i32,
    pub verse: i32,
    pub text: String,
    pub relationship: String,
    pub source: String,
}

/// Everything the study panel needs about one verse.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerseContextResponse {
    pub translation: String,
    pub book: String,
    pub book_slug: String,
    pub chapter: i32,
    pub verse: VerseText,
    pub chapter_verse_count: i32,
    pub chapter_verses: Vec<VerseText>,
    pub previous: Option<NeighborPassage>,
    pub next: Option<NeighborPassage>,
    pub cross_references: Vec<CrossReference>,
}

/// One column of a parallel reading.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ParallelTranslation {
    pub id: String,
    pub name: String,
    pub language: String,
    /// The name this translation uses for the book, when it differs.
    pub book_name: Option<String>,
    pub verses: Vec<VerseText>,
}

/// The same chapter rendered in several translations.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ParallelResponse {
    pub book: String,
    pub book_slug: String,
    pub chapter: i32,
    pub reference_verses: Vec<i32>,
    pub translations: Vec<ParallelTranslation>,
}

/// A short, deterministic passage for a given day.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DailyReading {
    pub date: String,
    pub translation: String,
    pub translation_name: String,
    pub book: String,
    pub book_slug: String,
    pub chapter: i32,
    pub reference: String,
    pub verses: Vec<VerseText>,
    pub featured: VerseText,
}

/// A search hit with enough context to render and link to it.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub translation: String,
    pub translation_name: String,
    pub book: String,
    pub book_slug: String,
    pub chapter: i32,
    pub verse: i32,
    pub text: String,
    pub rank: f32,
    /// How many versions carry this passage.
    pub versions: i64,
}

/// A page of search results plus the total number of matches.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultsResponse {
    pub results: Vec<SearchHit>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_more: bool,
}

/// A canonical book of the Bible with its structure.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BookSummary {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub testament: String,
    pub ord: i32,
    pub chapter_count: i32,
    pub verse_count: i32,
}

/// The canonical book list.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BooksListResponse {
    pub books: Vec<BookSummary>,
}

/// A single book with the chapters it contains.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BookDetailResponse {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub testament: String,
    pub ord: i32,
    pub chapter_count: i32,
    pub verse_count: i32,
    pub chapters: Vec<i32>,
}
