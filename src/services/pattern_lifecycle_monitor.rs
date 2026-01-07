use std::time::Duration;

use tokio::time::interval;

use crate::business_logic::patterns::lifecycle_tracker::PatternLifecycleTracker;
use crate::models::interval::CandleInterval;
use crate::models::patterns::{PatternLifecycleSnapshot, PatternLifecycleState};
use crate::services::candle_store::{CandleKey, SharedCandleStore};
use crate::services::feature_store::SharedFeatureStore;
use crate::services::pattern_lifecycle_state::SharedPatternLifecycleState;

pub struct PatternLifecycleMonitorConfig {
    pub coins: Vec<String>,
    pub intervals: Vec<CandleInterval>,
    pub poll_interval: Duration,
}

impl PatternLifecycleMonitorConfig {
    pub fn new(coins: Vec<String>, intervals: Vec<CandleInterval>) -> Self {
        Self {
            coins,
            intervals,
            poll_interval: Duration::from_secs(60),
        }
    }
}

pub struct PatternLifecycleMonitor {
    store: SharedCandleStore,
    feature_store: SharedFeatureStore,
    state: SharedPatternLifecycleState,
    tracker: PatternLifecycleTracker,
    config: PatternLifecycleMonitorConfig,
}

impl PatternLifecycleMonitor {
    pub fn new(
        store: SharedCandleStore,
        feature_store: SharedFeatureStore,
        state: SharedPatternLifecycleState,
        config: PatternLifecycleMonitorConfig,
    ) -> Self {
        Self {
            store,
            feature_store,
            state,
            tracker: PatternLifecycleTracker::new(),
            config,
        }
    }

    pub async fn run(mut self) {
        let mut ticker = interval(self.config.poll_interval);
        self.refresh().await;
        loop {
            ticker.tick().await;
            self.refresh().await;
        }
    }

    async fn refresh(&mut self) {
        let mut entries = Vec::new();

        for coin in &self.config.coins {
            for interval in &self.config.intervals {
                let key = CandleKey::new(coin.to_string(), *interval);
                if let Some(candles) = self.store.get(&key).await {
                    let features = self.feature_store.get(&key).await;
                    entries.extend(self.tracker.update_candlesticks(coin, *interval, &candles));
                    entries.extend(self.tracker.update_gaps(coin, *interval, &candles));
                    entries.extend(self.tracker.update_chart_patterns(
                        coin,
                        *interval,
                        &candles,
                        features.as_ref(),
                    ));
                    entries.extend(self.tracker.update_advanced_patterns(
                        coin,
                        *interval,
                        &candles,
                        features.as_ref(),
                    ));
                }
            }
        }

        let snapshot = PatternLifecycleSnapshot {
            as_of_ms: chrono::Utc::now().timestamp_millis() as u64,
            entries: entries.clone(),
        };

        log_confirmed_alerts(&entries, snapshot.as_of_ms);

        let mut guard = self.state.entries.write().await;
        *guard = entries;
        let _ = self.state.broadcaster.send(snapshot);
    }
}

fn log_confirmed_alerts(entries: &[crate::models::patterns::PatternLifecycleEntry], now_ms: u64) {
    for entry in entries {
        if is_new_confirmation(entry, now_ms) {
            tracing::warn!(
                "🔔 PATTERN CONFIRMED: {} {} {} ({:?}) confidence {:.1}%",
                entry.coin,
                entry.interval.as_str(),
                entry.pattern,
                entry.classification,
                entry.confidence * 100.0
            );
        }
    }
}

fn is_new_confirmation(
    entry: &crate::models::patterns::PatternLifecycleEntry,
    now_ms: u64,
) -> bool {
    entry.state == PatternLifecycleState::Confirmed && entry.state_since_ms == now_ms
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::candle::Candle;
    use crate::models::patterns::{
        PatternClassification, PatternLifecycleEntry, PatternSignalType,
    };
    use crate::services::candle_store::CandleStoreInner;
    use crate::services::feature_store::FeatureStoreInner;
    use crate::services::pattern_lifecycle_state::PatternLifecycleStateInner;
    use std::sync::Arc;

    fn candle(close_time: u64) -> Candle {
        Candle {
            open_time: close_time.saturating_sub(60_000),
            close_time,
            open: 100.0,
            high: 110.0,
            low: 90.0,
            close: 105.0,
            volume: 0.0,
            num_trades: 1,
            interval: None,
            symbol: None,
        }
    }

    #[tokio::test]
    async fn refresh_populates_entries() {
        let store = CandleStoreInner::new(100);
        let features = FeatureStoreInner::new(Default::default());
        let state = PatternLifecycleStateInner::new();
        let config = PatternLifecycleMonitorConfig::new(
            vec!["BTC".to_string()],
            vec![CandleInterval::OneMinute],
        );

        let monitor = PatternLifecycleMonitor::new(
            Arc::new(store),
            Arc::new(features),
            Arc::new(state),
            config,
        );

        let key = CandleKey::new("BTC".to_string(), CandleInterval::OneMinute);
        monitor
            .store
            .upsert(key, vec![candle(60_000), candle(120_000), candle(180_000)])
            .await;

        let mut monitor = monitor;
        monitor.refresh().await;

        let guard = monitor.state.entries.read().await;
        assert!(!guard.is_empty());
    }

    #[test]
    fn confirmation_filter_only_flags_new_confirmations() {
        let entry = PatternLifecycleEntry {
            coin: "BTC".to_string(),
            interval: CandleInterval::OneMinute,
            pattern: "Ascending Triangle".to_string(),
            category: "chart_continuation".to_string(),
            classification: PatternClassification::Bullish,
            signal_type: PatternSignalType::Continuation,
            state: PatternLifecycleState::Confirmed,
            confidence: 0.7,
            state_since_ms: 100,
            last_updated_ms: 100,
            window_start_ms: 0,
            window_end_ms: 0,
            notes: None,
        };

        assert!(is_new_confirmation(&entry, 100));
        assert!(!is_new_confirmation(&entry, 200));
    }
}
