use axum::{extract::State, Json};

use crate::errors::AppError;
use crate::handlers::query::ValidatedQuery;
use crate::models::orderbook::{L2BookQuery, L2BookSnapshot};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/orderbook",
    params(L2BookQuery),
    responses(
        (status = 200, description = "L2 order book snapshot", body = L2BookSnapshot),
        (status = 400, description = "Invalid request", body = crate::errors::ErrorResponse)
    )
)]
/// Return the L2 order book snapshot for the requested coin.
pub async fn get_orderbook(
    State(state): State<AppState>,
    ValidatedQuery(query): ValidatedQuery<L2BookQuery>,
) -> Result<Json<L2BookSnapshot>, AppError> {
    let snapshot = state
        .hyperliquid
        .fetch_l2_book(&query.coin, query.n_sig_figs, query.mantissa)
        .await
        .map_err(|error| AppError::Upstream(error.to_string()))?;

    Ok(Json(snapshot))
}
