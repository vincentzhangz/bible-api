use axum::{Json, extract::State};
use sqlx::PgPool;

use crate::error::AppError;
use crate::models::{Translation, TranslationResponse};

/// Lists all available Bible translations.
pub async fn list_translations(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<TranslationResponse>>, AppError> {
    let translations = sqlx::query_as::<_, Translation>(
        "SELECT id, name, language, license_id, source, json_hash FROM translations ORDER BY name",
    )
    .fetch_all(&pool)
    .await
    .map_err(AppError::Database)?;

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
