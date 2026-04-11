use axum::Json;
use axum::extract::State;
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

/// Liveness probe - returns 200 if the service is alive
pub async fn liveness() -> Json<LivenessResponse> {
    Json(LivenessResponse {
        status: "alive".to_string(),
    })
}

/// Readiness probe - returns 200 if the service is ready to accept traffic
pub async fn readiness(State(pool): State<PgPool>) -> Json<ReadinessResponse> {
    let db_status = match db::check_connection(&pool).await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    let status = if db_status == "connected" {
        "ready"
    } else {
        "not_ready"
    };

    Json(ReadinessResponse {
        status: status.to_string(),
        database: db_status.to_string(),
    })
}

/// Combined health check (backward compatible)
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
