use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;

use crate::db;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub database: String,
    pub version: String,
}

pub async fn health_check(State(pool): State<PgPool>) -> Json<HealthResponse> {
    let db_status = match db::check_connection(&pool).await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    Json(HealthResponse {
        status: "healthy".to_string(),
        database: db_status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
