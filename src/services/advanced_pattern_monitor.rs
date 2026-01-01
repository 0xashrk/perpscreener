use std::time::Duration;

use tokio::time::interval;

use crate::business_logic::patterns::advanced::detect_advanced_patterns;
use crate::business_logic::patterns::AdvancedDetectedPattern;
use crate::models::candle::Candle;
use crate::models::interval::CandleInterval;
use crate::models::patterns::{
    AdvancedPatternDetection, AdvancedPatternResponse, PatternDetection,
};
use crate::services::advanced_pattern_state::SharedAdvancedPatternState;
use crate::services::candle_store::{CandleKey, SharedCandleStore};
use crate::services::feature_store::SharedFeatureStore;

pub struct AdvancedPatternMonitorConfig {
    pub coins: Vec<String>,
    pub intervals: Vec<CandleInterval>,
    pub poll_interval: Duration,
}

impl AdvancedPatternMonitorConfig {
    pub fn new(coins: Vec<String>, intervals: Vec<CandleInterval>) -> Self {
        Self {
            coins,
            intervals,
            poll_interval: Duration::from_secs(300),
        }
    }
}

pub struct AdvancedPatternMonitor {
    store: SharedCandleStore,
    feature_store: SharedFeatureStore,
    state: SharedAdvancedPatternState,
    config: AdvancedPatternMonitorConfig,
}

impl AdvancedPatternMonitor {
    pub fn new(
        store: SharedCandleStore,
        feature_store: SharedFeatureStore,
        state: SharedAdvancedPatternState,
        config: AdvancedPatternMonitorConfig,
    ) -> Self {
        Self {
            store,
            feature_store,
            state,
            config,
        }
    }

    pub async fn run(&self) {
        let mut ticker = interval(self.config.poll_interval);

        self.refresh().await;

        loop {
            ticker.tick().await;
            self.refresh().await;
        }
    }

    pub async fn refresh(&self) {
        let mut detections = Vec::new();

        for coin in &self.config.coins {
            for interval in &self.config.intervals {
                let key = CandleKey::new(coin.to_string(), *interval);
                if let (Some(candles), Some(features)) = (
                    self.store.get(&key).await,
                    self.feature_store.get(&key).await,
                ) {
                    detections.extend(build_detections(coin, *interval, &candles, &features));
                }
            }
        }

        let as_of_ms = chrono::Utc::now().timestamp_millis() as u64;
        let snapshot = AdvancedPatternResponse {
            as_of_ms,
            detections: detections.clone(),
        };

        let mut guard = self.state.detections.write().await;
        *guard = detections;
        let _ = self.state.broadcaster.send(snapshot);
    }
}

fn build_detections(
    coin: &str,
    interval: CandleInterval,
    candles: &[Candle],
    features: &crate::business_logic::features::FeatureSnapshot,
) -> Vec<AdvancedPatternDetection> {
    let patterns = detect_advanced_patterns(candles, Some(features));
    patterns
        .into_iter()
        .filter_map(|pattern| to_detection(coin, interval, candles, pattern))
        .collect()
}

fn to_detection(
    coin: &str,
    interval: CandleInterval,
    candles: &[Candle],
    pattern: AdvancedDetectedPattern,
) -> Option<AdvancedPatternDetection> {
    if candles.len() < pattern.window {
        return None;
    }

    let start_idx = candles.len() - pattern.window;
    let window_start = candles.get(start_idx)?.open_time;
    let window_end = candles.last()?.close_time;

    let detection = PatternDetection {
        coin: coin.to_string(),
        interval,
        pattern: pattern.pattern.to_string(),
        category: pattern.category.to_string(),
        classification: pattern.classification,
        signal_type: pattern.signal_type,
        confidence: pattern.confidence,
        detected_at_ms: window_end,
        window_start_ms: window_start,
        window_end_ms: window_end,
        notes: None,
    };

    Some(AdvancedPatternDetection {
        detection,
        method: pattern.method.to_string(),
        basis: pattern.basis.to_string(),
        assumptions: pattern.assumptions,
    })
}
