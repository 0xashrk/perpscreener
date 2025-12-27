use std::sync::Arc;

use anyhow::Context;

use crate::models::chart::ChartSnapshot;
use crate::models::interval::interval_ms;
use crate::services::candles::normalize_candles;
use crate::services::hyperliquid::HyperliquidClient;

/// Service for fetching chart snapshots from Hyperliquid.
pub struct ChartService {
    client: Arc<HyperliquidClient>,
}

impl ChartService {
    /// Create a new chart service.
    pub fn new(client: Arc<HyperliquidClient>) -> Self {
        Self { client }
    }

    /// Fetch a candle snapshot for the given coin and interval.
    pub async fn fetch_snapshot(
        &self,
        coin: &str,
        interval: &str,
        limit: usize,
    ) -> anyhow::Result<ChartSnapshot> {
        let now_ms = chrono::Utc::now().timestamp_millis() as u64;
        let interval_ms = interval_ms(interval).context("unsupported interval")?;
        let (start_time, end_time) = build_time_range(now_ms, interval_ms, limit);

        let mut candles = self
            .client
            .fetch_candles(coin, interval, start_time, end_time)
            .await
            .context("failed to fetch candle snapshot")?;
        normalize_candles(&mut candles, coin, interval);

        Ok(ChartSnapshot {
            as_of_ms: now_ms,
            coin: coin.to_string(),
            interval: interval.to_string(),
            candles,
        })
    }
}

fn build_time_range(now_ms: u64, interval_ms: u64, limit: usize) -> (u64, u64) {
    let span = interval_ms.saturating_mul(limit as u64);
    let start_time = now_ms.saturating_sub(span);
    (start_time, now_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_time_range_uses_limit_and_interval() {
        let now_ms = 1_000_000;
        let interval_ms = 60_000;
        let (start_time, end_time) = build_time_range(now_ms, interval_ms, 5);

        assert_eq!(end_time, now_ms);
        assert_eq!(start_time, now_ms - (interval_ms * 5));
    }

    #[test]
    fn build_time_range_saturates_on_overflow() {
        let now_ms = 1_000;
        let interval_ms = u64::MAX;
        let (start_time, end_time) = build_time_range(now_ms, interval_ms, 2);

        assert_eq!(end_time, now_ms);
        assert_eq!(start_time, 0);
    }
}
