use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct BookResponse {
    pub id: i32,
    pub name: String,
    pub testament: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BooksResponse {
    pub translation: String,
    pub books: Vec<BookResponse>,
}
