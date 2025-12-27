use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub message: String,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("upstream error: {0}")]
    Upstream(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Validation(message) => (StatusCode::BAD_REQUEST, message.clone()),
            AppError::Upstream(message) => (StatusCode::BAD_GATEWAY, message.clone()),
            AppError::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message.clone()),
        };

        let body = Json(ErrorResponse { message });
        (status, body).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        AppError::Internal(error.to_string())
    }
}
