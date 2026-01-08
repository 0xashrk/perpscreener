use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

/// A single price level in the order book.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct L2BookLevel {
    /// Price at this level.
    pub px: String,
    /// Total size at this level.
    pub sz: String,
    /// Number of orders at this level.
    pub n: u32,
}

/// L2 order book snapshot response.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct L2BookSnapshot {
    /// Coin symbol.
    pub coin: String,
    /// Snapshot timestamp in milliseconds.
    pub time: u64,
    /// Order book levels: [bids, asks].
    pub levels: (Vec<L2BookLevel>, Vec<L2BookLevel>),
}

/// Query parameters for L2 book endpoint.
#[derive(Debug, Clone, Deserialize, Validate, IntoParams, ToSchema)]
pub struct L2BookQuery {
    /// Coin symbol (e.g., "BTC", "ETH").
    #[validate(length(min = 1, max = 20))]
    #[param(example = "BTC")]
    pub coin: String,
    /// Optional significant figures for price aggregation (2-5, or null for full precision).
    #[serde(rename = "nSigFigs")]
    pub n_sig_figs: Option<u8>,
    /// Optional mantissa for aggregation (only valid when nSigFigs is 5). Accepts 1, 2, or 5.
    pub mantissa: Option<u8>,
}
