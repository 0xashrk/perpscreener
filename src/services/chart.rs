use std::sync::Arc;

use anyhow::Context;

use crate::models::candle::Candle;
use crate::models::chart::{interval_ms, ChartSnapshot};
use crate::services::hyperliquid::HyperliquidClient;

pub struct ChartService {
    client: Arc<HyperliquidClient>,
}

impl ChartService {
    pub fn new(client: Arc<HyperliquidClient>) -> Self {
        Self { client }
    }

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

fn normalize_candles(candles: &mut [Candle], coin: &str, interval: &str) {
    for candle in candles {
        if candle.interval.is_none() {
            candle.interval = Some(interval.to_string());
        }
        if candle.symbol.is_none() {
            candle.symbol = Some(coin.to_string());
        }
    }
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
    fn normalize_candles_sets_missing_fields() {
        let mut candles = vec![Candle {
            open_time: 1,
            close_time: 2,
            open: 1.0,
            high: 2.0,
            low: 0.5,
            close: 1.5,
            volume: 10.0,
            num_trades: 5,
            interval: None,
            symbol: None,
        }];

        normalize_candles(&mut candles, "BTC", "1m");

        assert_eq!(candles[0].interval.as_deref(), Some("1m"));
        assert_eq!(candles[0].symbol.as_deref(), Some("BTC"));
    }
}
