use axum::{
    Json,
    extract::{Extension, Path, Query},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};

use super::language_from_translation;
use crate::ingestion::cache::VisualizeCache;

#[derive(Debug, Serialize, ToSchema)]
pub struct TimelineEvent {
    pub key: String,
    pub event: String,
    pub reference: String,
    #[serde(rename = "estimatedYear")]
    pub estimated_year: Option<i32>,
    pub category: String,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct TimelineQuery {
    pub lang: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/visualize/timeline/{translation}",
    params(
        ("translation" = String, Path, description = "Translation ID")
    ),
    responses(
        (status = 200, description = "Timeline events", body = Vec<TimelineEvent>)
    )
)]
pub async fn timeline(
    Extension(cache): Extension<Arc<VisualizeCache>>,
    Path(translation): Path<String>,
    Query(query): Query<TimelineQuery>,
) -> Json<Vec<TimelineEvent>> {
    let language = query
        .lang
        .unwrap_or_else(|| language_from_translation(&translation));

    let events = cache
        .get_locale(&language)
        .map(|locale| {
            locale
                .timeline
                .iter()
                .map(|e| TimelineEvent {
                    key: e.key.clone(),
                    event: e.event.clone(),
                    reference: e.reference.clone(),
                    estimated_year: e.estimated_year,
                    category: e.category.clone(),
                })
                .collect()
        })
        .unwrap_or_default();

    Json(events)
}
