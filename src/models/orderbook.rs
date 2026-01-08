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

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn l2_book_query_requires_non_empty_coin() {
        let query = L2BookQuery {
            coin: "".to_string(),
            n_sig_figs: None,
            mantissa: None,
        };
        assert!(query.validate().is_err());
    }

    #[test]
    fn l2_book_query_rejects_long_coin() {
        let query = L2BookQuery {
            coin: "A".repeat(21),
            n_sig_figs: None,
            mantissa: None,
        };
        assert!(query.validate().is_err());
    }

    #[test]
    fn l2_book_query_accepts_valid_coin() {
        let query = L2BookQuery {
            coin: "BTC".to_string(),
            n_sig_figs: Some(3),
            mantissa: Some(1),
        };
        assert!(query.validate().is_ok());
    }

    #[test]
    fn l2_book_level_serializes_correctly() {
        let level = L2BookLevel {
            px: "50000.00".to_string(),
            sz: "1.5".to_string(),
            n: 10,
        };
        let json = serde_json::to_string(&level).unwrap();
        assert!(json.contains("50000.00"));
        assert!(json.contains("1.5"));
        assert!(json.contains("10"));
    }

    #[test]
    fn l2_book_snapshot_serializes_levels_as_tuple() {
        let snapshot = L2BookSnapshot {
            coin: "ETH".to_string(),
            time: 1234567890,
            levels: (
                vec![L2BookLevel {
                    px: "3000.00".to_string(),
                    sz: "2.0".to_string(),
                    n: 5,
                }],
                vec![L2BookLevel {
                    px: "3001.00".to_string(),
                    sz: "3.0".to_string(),
                    n: 8,
                }],
            ),
        };
        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(json.contains("ETH"));
        assert!(json.contains("1234567890"));
        assert!(json.contains("3000.00"));
        assert!(json.contains("3001.00"));
    }

    #[test]
    fn l2_book_query_deserializes_with_optional_fields() {
        let json = r#"{"coin": "BTC"}"#;
        let query: L2BookQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.coin, "BTC");
        assert!(query.n_sig_figs.is_none());
        assert!(query.mantissa.is_none());
    }

    #[test]
    fn l2_book_query_deserializes_n_sig_figs_rename() {
        let json = r#"{"coin": "BTC", "nSigFigs": 4}"#;
        let query: L2BookQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.n_sig_figs, Some(4));
    }
}
