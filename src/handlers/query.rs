use axum::extract::{FromRequestParts, Query};
use axum::http::request::Parts;
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::errors::AppError;

/// Extractor that converts query parsing/validation failures into AppError responses.
pub struct ValidatedQuery<T>(pub T);

impl<S, T> FromRequestParts<S> for ValidatedQuery<T>
where
    T: DeserializeOwned + Validate + Send,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(value) = Query::<T>::from_request_parts(parts, state)
            .await
            .map_err(|error| AppError::Validation(error.to_string()))?;

        value
            .validate()
            .map_err(|error| AppError::Validation(error.to_string()))?;

        Ok(ValidatedQuery(value))
    }
}
