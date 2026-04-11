use axum::{extract::State, Json};
use sqlx::PgPool;

use crate::models::{Translation, TranslationResponse};

pub async fn list_translations(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<TranslationResponse>>, (axum::http::StatusCode, String)> {
    let translations = sqlx::query_as::<_, Translation>(
        "SELECT id, name, language, license_id, source, json_hash FROM translations ORDER BY name",
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Database error".to_string(),
        )
    })?;

    let response: Vec<TranslationResponse> = translations
        .into_iter()
        .map(|t| TranslationResponse {
            id: t.id,
            name: t.name,
            language: t.language,
            license: t.license_id.unwrap_or_else(|| "none".to_string()),
            source: t.source,
        })
        .collect();

    Ok(Json(response))
}
