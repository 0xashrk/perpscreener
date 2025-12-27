use axum::Json;

use crate::errors::AppError;
use crate::models::health::HealthResponse;

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Health check", body = HealthResponse)
    )
)]
pub async fn health() -> Result<Json<HealthResponse>, AppError> {
    Ok(Json(HealthResponse {
        status: "healthy".to_string(),
    }))
}
