use axum::{
    Json,
    extract::{Extension, Path, Query},
};
use serde::{Deserialize, Serialize};
use std::path::Path as StdPath;
use std::sync::Arc;

use super::language_from_translation;
use crate::config::env::AppConfig;

#[derive(Debug, Serialize)]
pub struct TimelineEvent {
    pub key: String,
    pub event: String,
    pub reference: String,
    #[serde(rename = "estimatedYear")]
    pub estimated_year: Option<i32>,
    pub category: String,
}

#[derive(Debug, Deserialize)]
pub struct TimelineQuery {
    pub lang: Option<String>,
}

/// Gets timeline events for a translation.
pub async fn timeline(
    Extension(config): Extension<Arc<AppConfig>>,
    Path(translation): Path<String>,
    Query(query): Query<TimelineQuery>,
) -> Json<Vec<TimelineEvent>> {
    let language = query
        .lang
        .clone()
        .unwrap_or_else(|| language_from_translation(&translation));

    let data_dir = StdPath::new(&config.data_dir);

    if let Some(locale) = crate::ingestion::visualize::load_visualize_locale(data_dir, &language) {
        let events: Vec<TimelineEvent> = locale
            .timeline
            .into_iter()
            .map(|e| TimelineEvent {
                key: e.key,
                event: e.event,
                reference: e.reference,
                estimated_year: e.estimated_year,
                category: e.category,
            })
            .collect();
        Json(events)
    } else {
        Json(vec![])
    }
}
