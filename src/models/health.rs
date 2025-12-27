use serde::Serialize;
use utoipa::ToSchema;

/// Health check response payload.
#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
}
