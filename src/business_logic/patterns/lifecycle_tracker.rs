use std::collections::HashMap;

use crate::models::candle::Candle;
use crate::models::interval::CandleInterval;
use crate::models::patterns::{
    PatternClassification, PatternLifecycleEntry, PatternLifecycleState, PatternSignalType,
};

use super::candlesticks::detect_candlestick_patterns;
use super::lifecycle_registry::{pattern_registry, PatternLifecycleCategory, PatternLifecycleDefinition};
use super::DetectedPattern;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct PatternLifecycleKey {
    coin: String,
    interval: CandleInterval,
    pattern: String,
    classification: PatternClassification,
}

#[derive(Debug)]
pub struct PatternLifecycleTracker {
    candlestick_defs: Vec<PatternLifecycleDefinition>,
    entries: HashMap<PatternLifecycleKey, PatternLifecycleEntry>,
}

impl PatternLifecycleTracker {
    pub fn new() -> Self {
        let candlestick_defs = pattern_registry()
            .into_iter()
            .filter(|def| def.category == PatternLifecycleCategory::Candlestick)
            .collect();
        Self {
            candlestick_defs,
            entries: HashMap::new(),
        }
    }

    pub fn update_candlesticks(
        &mut self,
        coin: &str,
        interval: CandleInterval,
        candles: &[Candle],
    ) -> Vec<PatternLifecycleEntry> {
        let detections = detect_candlestick_patterns(candles);
        self.apply_candlestick_detections(coin, interval, candles, &detections)
    }

    fn apply_candlestick_detections(
        &mut self,
        coin: &str,
        interval: CandleInterval,
        candles: &[Candle],
        detections: &[DetectedPattern],
    ) -> Vec<PatternLifecycleEntry> {
        let now_ms = candles.last().map(|c| c.close_time).unwrap_or(0);
        let detection_map = candlestick_detection_map(detections);
        let mut updated = Vec::new();

        for def in &self.candlestick_defs {
            let key = PatternLifecycleKey {
                coin: coin.to_string(),
                interval,
                pattern: def.name.to_string(),
                classification: def.classification,
            };

            let detection = detection_map
                .get(&(def.detector_name, def.classification))
                .copied();
            let entry = self.entries.get(&key).cloned();
            let next = next_entry(
                &key,
                def,
                interval,
                now_ms,
                candles,
                entry,
                detection,
            );
            self.entries.insert(key, next.clone());
            updated.push(next);
        }

        updated
    }
}

fn candlestick_detection_map(
    detections: &[DetectedPattern],
) -> HashMap<(&'static str, PatternClassification), &DetectedPattern> {
    let mut map = HashMap::new();
    for detection in detections {
        map.insert((detection.pattern, detection.classification), detection);
    }
    map
}

fn next_entry(
    key: &PatternLifecycleKey,
    def: &PatternLifecycleDefinition,
    interval: CandleInterval,
    now_ms: u64,
    candles: &[Candle],
    previous: Option<PatternLifecycleEntry>,
    detection: Option<&DetectedPattern>,
) -> PatternLifecycleEntry {
    if candles.len() < def.window {
        return build_entry(
            key,
            def,
            PatternLifecycleState::Warming,
            previous,
            now_ms,
            0.0,
            0,
            0,
            None,
        );
    }

    if let Some(detection) = detection {
        let (window_start_ms, window_end_ms) = window_bounds(candles, detection.window);
        let confidence = detection.confidence;
        let notes = detection.notes.clone();
        let state_since_ms = match previous.as_ref() {
            Some(prev) if prev.state == PatternLifecycleState::Confirmed => prev.state_since_ms,
            _ => now_ms,
        };

        return PatternLifecycleEntry {
            coin: key.coin.clone(),
            interval: key.interval,
            pattern: key.pattern.clone(),
            category: def.category_label.to_string(),
            classification: def.classification,
            signal_type: def.signal_type,
            state: PatternLifecycleState::Confirmed,
            confidence,
            state_since_ms,
            last_updated_ms: now_ms,
            window_start_ms,
            window_end_ms,
            notes,
        };
    }

    let Some(previous) = previous else {
        return build_entry(
            key,
            def,
            PatternLifecycleState::Watching,
            None,
            now_ms,
            0.0,
            0,
            0,
            None,
        );
    };

    let bars_since = bars_since(previous.state_since_ms, now_ms, interval);
    let next_state = match previous.state {
        PatternLifecycleState::Confirmed
            if bars_since >= def.max_age_bars =>
        {
            PatternLifecycleState::Expired
        }
        PatternLifecycleState::Expired if bars_since >= def.max_age_bars => {
            PatternLifecycleState::Watching
        }
        PatternLifecycleState::Warming => PatternLifecycleState::Watching,
        PatternLifecycleState::Forming => PatternLifecycleState::Expired,
        PatternLifecycleState::Invalidated => PatternLifecycleState::Watching,
        state => state,
    };

    let state_since_ms = if next_state == previous.state {
        previous.state_since_ms
    } else {
        now_ms
    };

    PatternLifecycleEntry {
        state: next_state,
        state_since_ms,
        last_updated_ms: now_ms,
        ..previous
    }
}

fn build_entry(
    key: &PatternLifecycleKey,
    def: &PatternLifecycleDefinition,
    state: PatternLifecycleState,
    previous: Option<PatternLifecycleEntry>,
    now_ms: u64,
    confidence: f64,
    window_start_ms: u64,
    window_end_ms: u64,
    notes: Option<String>,
) -> PatternLifecycleEntry {
    let state_since_ms = match previous.as_ref() {
        Some(prev) if prev.state == state => prev.state_since_ms,
        _ => now_ms,
    };

    PatternLifecycleEntry {
        coin: key.coin.clone(),
        interval: key.interval,
        pattern: key.pattern.clone(),
        category: def.category_label.to_string(),
        classification: def.classification,
        signal_type: def.signal_type,
        state,
        confidence: if confidence > 0.0 {
            confidence
        } else {
            previous.as_ref().map(|prev| prev.confidence).unwrap_or(0.0)
        },
        state_since_ms,
        last_updated_ms: now_ms,
        window_start_ms: if window_start_ms > 0 {
            window_start_ms
        } else {
            previous
                .as_ref()
                .map(|prev| prev.window_start_ms)
                .unwrap_or(0)
        },
        window_end_ms: if window_end_ms > 0 {
            window_end_ms
        } else {
            previous
                .as_ref()
                .map(|prev| prev.window_end_ms)
                .unwrap_or(0)
        },
        notes: notes.or_else(|| previous.and_then(|prev| prev.notes)),
    }
}

fn window_bounds(candles: &[Candle], window: usize) -> (u64, u64) {
    if window == 0 || candles.len() < window {
        return (0, 0);
    }
    let start_idx = candles.len() - window;
    let window_start_ms = candles.get(start_idx).map(|c| c.open_time).unwrap_or(0);
    let window_end_ms = candles.last().map(|c| c.close_time).unwrap_or(0);
    (window_start_ms, window_end_ms)
}

fn bars_since(state_since_ms: u64, now_ms: u64, interval: CandleInterval) -> usize {
    if state_since_ms == 0 || now_ms <= state_since_ms {
        return 0;
    }
    let interval_ms = interval.ms();
    if interval_ms == 0 {
        return 0;
    }
    ((now_ms - state_since_ms) / interval_ms) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle_series(start_ms: u64, interval: CandleInterval, count: usize) -> Vec<Candle> {
        let mut candles = Vec::new();
        for idx in 0..count {
            let open_time = start_ms + interval.ms() * idx as u64;
            let close_time = open_time + interval.ms();
            candles.push(Candle {
                open_time,
                close_time,
                open: 100.0,
                high: 110.0,
                low: 90.0,
                close: 105.0,
                volume: 1.0,
                num_trades: 1,
                interval: None,
                symbol: None,
            });
        }
        candles
    }

    fn detection(pattern: &'static str, classification: PatternClassification) -> DetectedPattern {
        DetectedPattern {
            pattern,
            category: "candlestick_reversal",
            classification,
            signal_type: PatternSignalType::Reversal,
            confidence: 0.7,
            window: 1,
            notes: None,
        }
    }

    #[test]
    fn candlestick_entries_start_warming() {
        let mut tracker = PatternLifecycleTracker::new();
        let candles = candle_series(0, CandleInterval::OneMinute, 1);
        let entries = tracker.apply_candlestick_detections(
            "BTC",
            CandleInterval::OneMinute,
            &candles,
            &[],
        );

        let abandoned = entries
            .iter()
            .find(|entry| entry.pattern == "Abandoned Baby" && entry.classification == PatternClassification::Bullish)
            .expect("entry");
        assert_eq!(abandoned.state, PatternLifecycleState::Warming);
    }

    #[test]
    fn candlestick_entries_confirm_on_detection() {
        let mut tracker = PatternLifecycleTracker::new();
        let candles = candle_series(0, CandleInterval::OneMinute, 5);
        let detections = vec![detection("Hammer", PatternClassification::Bullish)];

        let entries = tracker.apply_candlestick_detections(
            "BTC",
            CandleInterval::OneMinute,
            &candles,
            &detections,
        );

        let hammer = entries
            .iter()
            .find(|entry| entry.pattern == "Hammer / Dragonfly Doji" && entry.classification == PatternClassification::Bullish)
            .expect("entry");
        assert_eq!(hammer.state, PatternLifecycleState::Confirmed);
        assert!((hammer.confidence - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn candlestick_entries_expire_after_age() {
        let mut tracker = PatternLifecycleTracker::new();
        let candles = candle_series(0, CandleInterval::OneMinute, 5);
        let detections = vec![detection("Hammer", PatternClassification::Bullish)];
        tracker.apply_candlestick_detections(
            "BTC",
            CandleInterval::OneMinute,
            &candles,
            &detections,
        );

        let later_candles = candle_series(120_000, CandleInterval::OneMinute, 5);
        let entries = tracker.apply_candlestick_detections(
            "BTC",
            CandleInterval::OneMinute,
            &later_candles,
            &[],
        );

        let hammer = entries
            .iter()
            .find(|entry| entry.pattern == "Hammer / Dragonfly Doji" && entry.classification == PatternClassification::Bullish)
            .expect("entry");
        assert_eq!(hammer.state, PatternLifecycleState::Expired);
    }
}
