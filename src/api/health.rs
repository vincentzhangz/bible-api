use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
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

#[derive(Debug, Serialize, ToSchema)]
pub struct LivenessResponse {
    pub status: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReadinessResponse {
    pub status: String,
    pub database: String,
}

async fn check_db_status(pool: &PgPool) -> &'static str {
    match db::check_connection(pool).await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    }
}

pub async fn liveness() -> Json<LivenessResponse> {
    Json(LivenessResponse {
        status: "alive".to_string(),
    })
}

pub async fn readiness(
    State(pool): State<PgPool>,
) -> (StatusCode, Json<ReadinessResponse>) {
    let db_status = check_db_status(&pool).await;
    let is_ready = db_status == "connected";

    let status_code = if is_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(ReadinessResponse {
            status: if is_ready { "ready" } else { "not_ready" }.to_string(),
            database: db_status.to_string(),
        }),
    )
}

pub async fn health_check(State(pool): State<PgPool>) -> Json<HealthResponse> {
    let db_status = check_db_status(&pool).await;

    Json(HealthResponse {
        status: "healthy".to_string(),
        database: db_status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
