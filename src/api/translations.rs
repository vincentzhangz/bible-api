use axum::extract::State;
use axum::Json;
use sqlx::PgPool;

use crate::models::{Translation, TranslationResponse};

pub async fn list_translations(State(pool): State<PgPool>) -> Json<Vec<TranslationResponse>> {
    let translations = sqlx::query_as::<_, Translation>(
        "SELECT id, name, language, license_id, source, json_hash FROM translations ORDER BY name"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let response: Vec<TranslationResponse> = translations
        .into_iter()
        .map(|t| TranslationResponse {
            id: t.id,
            name: t.name,
            language: t.language,
            license: t.license_id.map(|_| "unknown".to_string()).unwrap_or_else(|| "none".to_string()),
            source: t.source,
        })
        .collect();

    Json(response)
}
