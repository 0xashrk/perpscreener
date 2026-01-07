use std::cmp;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use futures::stream::{self, StreamExt};
use tokio::sync::RwLock;
use tokio::time::{interval, sleep};

use crate::models::interval::CandleInterval;
use crate::services::candle_store::{CandleKey, SharedCandleStore};
use crate::services::candles::normalize_candles;
use crate::services::feature_store::SharedFeatureStore;
use crate::services::hyperliquid::HyperliquidClient;

const CONCURRENT_REQUESTS: usize = 8;

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
    client: Arc<HyperliquidClient>,
    store: SharedCandleStore,
    feature_store: SharedFeatureStore,
    config: CandleIngestionConfig,
    last_close_time: RwLock<HashMap<CandleKey, u64>>,
}

impl CandleIngestionService {
    pub fn new(
        client: Arc<HyperliquidClient>,
        store: SharedCandleStore,
        feature_store: SharedFeatureStore,
        config: CandleIngestionConfig,
    ) -> Self {
        Self {
            client,
            store,
            feature_store,
            config,
            last_close_time: RwLock::new(HashMap::new()),
        }
    }

    pub async fn warmup(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request_delay = self.config.request_delay;
        let tasks = self.config.coins.iter().cloned().flat_map(|coin| {
            self.config
                .intervals
                .iter()
                .copied()
                .map(move |interval| (coin.clone(), interval))
        });

        let results = stream::iter(tasks)
            .map(|(coin, interval)| {
                let delay = request_delay;
                async move {
                    let result = self.warmup_key(&coin, interval).await;
                    if !delay.is_zero() {
                        sleep(delay).await;
                    }
                    result.map_err(|err| (coin, interval, err))
                }
            })
            .buffer_unordered(CONCURRENT_REQUESTS)
            .collect::<Vec<_>>()
            .await;

        for result in results {
            if let Err((coin, interval, err)) = result {
                tracing::error!("Candle warmup failed for {} {}: {}", coin, interval, err);
                return Err(err);
            }
        }

        Ok(())
    }

    pub async fn run(&self) {
        let mut ticker = interval(self.config.poll_interval);

        loop {
            ticker.tick().await;

            let request_delay = self.config.request_delay;
            let tasks = self.config.coins.iter().cloned().flat_map(|coin| {
                self.config
                    .intervals
                    .iter()
                    .copied()
                    .map(move |interval| (coin.clone(), interval))
            });

            stream::iter(tasks)
                .for_each_concurrent(
                    Some(cmp::max(1, CONCURRENT_REQUESTS)),
                    |(coin, interval)| async move {
                        if let Err(err) = self.refresh_key(&coin, interval).await {
                            tracing::error!(
                                "Candle refresh failed for {} {}: {}",
                                coin,
                                interval,
                                err
                            );
                        }

                        if !request_delay.is_zero() {
                            sleep(request_delay).await;
                        }
                    },
                )
                .await;
        }
    }

    async fn warmup_key(
        &self,
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
        self.update_last_close_time(key, summary.newest_close_time)
            .await;

        Ok(())
    }

    async fn refresh_key(
        &self,
        coin: &str,
        interval: CandleInterval,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        let key = CandleKey::new(coin.to_string(), interval);
        let start_time = {
            let guard = self.last_close_time.read().await;
            guard
                .get(&key)
                .copied()
                .unwrap_or(now.saturating_sub(interval.ms() * 2))
        };

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
        self.update_last_close_time(key, summary.newest_close_time)
            .await;

        Ok(())
    }

    async fn update_features(&self, key: &CandleKey) {
        if let Some(candles) = self.store.get(key).await {
            let _ = self.feature_store.recompute(key.clone(), &candles).await;
        }
    }

    async fn update_last_close_time(&self, key: CandleKey, newest: Option<u64>) {
        if let Some(close_time) = newest {
            let mut guard = self.last_close_time.write().await;
            guard.insert(key, close_time);
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
