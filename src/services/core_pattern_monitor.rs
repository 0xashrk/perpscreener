use std::time::Duration;

use tokio::time::interval;

use crate::business_logic::patterns::aggregation::{summarize_detections, PatternScoreWeights};
use crate::business_logic::patterns::candlesticks::detect_candlestick_patterns;
use crate::business_logic::patterns::chart_patterns::detect_chart_patterns;
use crate::business_logic::patterns::gaps::detect_gap_patterns;
use crate::business_logic::patterns::DetectedPattern;
use crate::models::candle::Candle;
use crate::models::interval::CandleInterval;
use crate::models::patterns::{PatternDetection, PatternResponse};
use crate::services::candle_store::{CandleKey, SharedCandleStore};
use crate::services::core_pattern_state::SharedCorePatternState;
use crate::services::feature_store::SharedFeatureStore;

pub struct CorePatternMonitorConfig {
    pub coins: Vec<String>,
    pub intervals: Vec<CandleInterval>,
    pub poll_interval: Duration,
}

impl CorePatternMonitorConfig {
    pub fn new(coins: Vec<String>, intervals: Vec<CandleInterval>) -> Self {
        Self {
            coins,
            intervals,
            poll_interval: Duration::from_secs(60),
        }
    }
}

pub struct CorePatternMonitor {
    store: SharedCandleStore,
    feature_store: SharedFeatureStore,
    state: SharedCorePatternState,
    config: CorePatternMonitorConfig,
}

impl CorePatternMonitor {
    pub fn new(
        store: SharedCandleStore,
        feature_store: SharedFeatureStore,
        state: SharedCorePatternState,
        config: CorePatternMonitorConfig,
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
                if let Some(candles) = self.store.get(&key).await {
                    let features = self.feature_store.get(&key).await;
                    detections.extend(build_detections(
                        coin,
                        *interval,
                        &candles,
                        features.as_ref(),
                    ));
                }
            }
        }

        let as_of_ms = chrono::Utc::now().timestamp_millis() as u64;
        let summaries = summarize_detections(&detections, &PatternScoreWeights::default());
        let snapshot = PatternResponse {
            as_of_ms,
            detections: detections.clone(),
            summaries,
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
    features: Option<&crate::business_logic::features::FeatureSnapshot>,
) -> Vec<PatternDetection> {
    let mut patterns = detect_candlestick_patterns(candles);
    patterns.extend(detect_gap_patterns(candles));
    patterns.extend(detect_chart_patterns(candles, features, interval));

    patterns
        .into_iter()
        .filter_map(|pattern| to_detection(coin, interval, candles, pattern))
        .collect()
}

fn to_detection(
    coin: &str,
    interval: CandleInterval,
    candles: &[Candle],
    pattern: DetectedPattern,
) -> Option<PatternDetection> {
    if candles.len() < pattern.window {
        return None;
    }

    let start_idx = candles.len() - pattern.window;
    let window_start = candles.get(start_idx)?.open_time;
    let window_end = candles.last()?.close_time;

    Some(PatternDetection {
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
        notes: pattern.notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::patterns::{PatternClassification, PatternSignalType};

    fn candle(open_time: u64, close_time: u64) -> Candle {
        Candle {
            open_time,
            close_time,
            open: 1.0,
            high: 1.5,
            low: 0.5,
            close: 1.2,
            volume: 0.0,
            num_trades: 0,
            interval: None,
            symbol: None,
        }
    }

    #[test]
    fn to_detection_uses_window_bounds() {
        let candles = vec![candle(10, 20), candle(20, 30)];
        let pattern = DetectedPattern {
            pattern: "Hammer",
            category: "candlestick_reversal",
            classification: PatternClassification::Bullish,
            signal_type: PatternSignalType::Reversal,
            confidence: 0.7,
            window: 2,
            notes: None,
        };

        let detection = to_detection("BTC", CandleInterval::OneMinute, &candles, pattern)
            .expect("detection");
        assert_eq!(detection.window_start_ms, 10);
        assert_eq!(detection.window_end_ms, 30);
    }
}
