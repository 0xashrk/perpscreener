use std::collections::HashMap;
use std::time::Duration;

use tokio::time::{interval, sleep};

use crate::models::interval::CandleInterval;
use crate::services::candle_store::{CandleKey, SharedCandleStore};
use crate::services::candles::normalize_candles;
use crate::services::feature_store::SharedFeatureStore;
use crate::services::hyperliquid::HyperliquidClient;

#[derive(Clone)]
pub struct CandleIngestionConfig {
    pub coins: Vec<String>,
    pub intervals: Vec<CandleInterval>,
    pub warmup_candles: usize,
    pub max_candles: usize,
    pub poll_interval: Duration,
    pub request_delay: Duration,
}

impl CandleIngestionConfig {
    pub fn new(coins: Vec<String>) -> Self {
        Self {
            coins,
            intervals: vec![
                CandleInterval::OneMinute,
                CandleInterval::ThreeMinutes,
                CandleInterval::FiveMinutes,
                CandleInterval::FifteenMinutes,
                CandleInterval::ThirtyMinutes,
                CandleInterval::OneHour,
                CandleInterval::TwoHours,
                CandleInterval::FourHours,
                CandleInterval::EightHours,
                CandleInterval::TwelveHours,
                CandleInterval::OneDay,
                CandleInterval::ThreeDays,
                CandleInterval::OneWeek,
                CandleInterval::OneMonth,
            ],
            warmup_candles: 500,
            max_candles: 5000,
            poll_interval: Duration::from_secs(60),
            request_delay: Duration::from_millis(120),
        }
    }
}

pub struct CandleIngestionService {
    client: HyperliquidClient,
    store: SharedCandleStore,
    feature_store: SharedFeatureStore,
    config: CandleIngestionConfig,
    last_close_time: HashMap<CandleKey, u64>,
}

impl CandleIngestionService {
    pub fn new(
        client: HyperliquidClient,
        store: SharedCandleStore,
        feature_store: SharedFeatureStore,
        config: CandleIngestionConfig,
    ) -> Self {
        Self {
            client,
            store,
            feature_store,
            config,
            last_close_time: HashMap::new(),
        }
    }

    pub async fn warmup(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let coins = self.config.coins.clone();
        let intervals = self.config.intervals.clone();
        for coin in &coins {
            for interval in &intervals {
                self.warmup_key(coin, *interval).await?;
                if !self.config.request_delay.is_zero() {
                    sleep(self.config.request_delay).await;
                }
            }
        }
        Ok(())
    }

    pub async fn run(&mut self) {
        let mut ticker = interval(self.config.poll_interval);
        let coins = self.config.coins.clone();
        let intervals = self.config.intervals.clone();

        loop {
            ticker.tick().await;
            for coin in &coins {
                for candle_interval in &intervals {
                    if let Err(err) = self.refresh_key(coin, *candle_interval).await {
                        tracing::error!(
                            "Candle refresh failed for {} {}: {}",
                            coin,
                            candle_interval,
                            err
                        );
                    }
                    if !self.config.request_delay.is_zero() {
                        sleep(self.config.request_delay).await;
                    }
                }
            }
        }
    }

    async fn warmup_key(
        &mut self,
        coin: &str,
        interval: CandleInterval,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        let window_ms = interval.ms() * self.config.warmup_candles as u64;
        let start_time = now.saturating_sub(window_ms);

        let mut candles = self
            .client
            .fetch_candles(coin, interval.as_str(), start_time, now)
            .await?;
        normalize_candles(&mut candles, coin, interval.as_str());

        let key = CandleKey::new(coin.to_string(), interval);
        let summary = self.store.upsert(key.clone(), candles).await;
        self.update_features(&key).await;
        self.update_last_close_time(key, summary.newest_close_time);

        Ok(())
    }

    async fn refresh_key(
        &mut self,
        coin: &str,
        interval: CandleInterval,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        let key = CandleKey::new(coin.to_string(), interval);
        let start_time = self
            .last_close_time
            .get(&key)
            .copied()
            .unwrap_or(now.saturating_sub(interval.ms() * 2));

        let mut candles = self
            .client
            .fetch_candles(coin, interval.as_str(), start_time, now)
            .await?;
        normalize_candles(&mut candles, coin, interval.as_str());

        if candles.is_empty() {
            return Ok(());
        }

        let summary = self.store.upsert(key.clone(), candles).await;
        self.update_features(&key).await;
        self.update_last_close_time(key, summary.newest_close_time);

        Ok(())
    }

    async fn update_features(&self, key: &CandleKey) {
        if let Some(candles) = self.store.get(key).await {
            let _ = self.feature_store.recompute(key.clone(), &candles).await;
        }
    }

    fn update_last_close_time(&mut self, key: CandleKey, newest: Option<u64>) {
        if let Some(close_time) = newest {
            self.last_close_time.insert(key, close_time);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_defaults_match_supported_intervals() {
        let config = CandleIngestionConfig::new(vec!["BTC".to_string()]);
        assert_eq!(config.max_candles, 5000);
        assert_eq!(config.warmup_candles, 500);
        assert_eq!(config.intervals.len(), 14);
        assert!(config.intervals.contains(&CandleInterval::OneMinute));
        assert!(config.intervals.contains(&CandleInterval::OneMonth));
    }
}
