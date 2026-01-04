use std::collections::HashMap;

use crate::models::candle::Candle;
use crate::models::interval::CandleInterval;
use crate::models::patterns::{
    PatternClassification, PatternLifecycleEntry, PatternLifecycleState, PatternSignalType,
};
use crate::business_logic::features::{FeatureSnapshot, Trendline, TrendlineKind};

use super::advanced::detect_advanced_patterns;
use super::candlesticks::detect_candlestick_patterns;
use super::gaps::detect_gap_patterns;
use super::chart_patterns::detect_chart_patterns;
use super::lifecycle_registry::{
    pattern_registry, PatternLifecycleCategory, PatternLifecycleDefinition,
};
use super::{AdvancedDetectedPattern, DetectedPattern};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct PatternLifecycleKey {
    coin: String,
    interval: CandleInterval,
    pattern: String,
    category: PatternLifecycleCategory,
    classification: PatternClassification,
}

#[derive(Debug)]
pub struct PatternLifecycleTracker {
    candlestick_defs: Vec<PatternLifecycleDefinition>,
    gap_defs: Vec<PatternLifecycleDefinition>,
    chart_defs: Vec<PatternLifecycleDefinition>,
    advanced_defs: Vec<PatternLifecycleDefinition>,
    entries: HashMap<PatternLifecycleKey, PatternLifecycleEntry>,
    gap_context: HashMap<PatternLifecycleKey, GapContext>,
}

impl PatternLifecycleTracker {
    pub fn new() -> Self {
        let definitions = pattern_registry();
        let candlestick_defs = definitions
            .iter()
            .copied()
            .filter(|def| def.category == PatternLifecycleCategory::Candlestick)
            .collect();
        let gap_defs = definitions
            .iter()
            .copied()
            .filter(|def| def.category == PatternLifecycleCategory::Gap)
            .collect();
        let chart_defs = definitions
            .iter()
            .copied()
            .filter(|def| def.category == PatternLifecycleCategory::Chart)
            .collect();
        let advanced_defs = definitions
            .iter()
            .copied()
            .filter(|def| def.category == PatternLifecycleCategory::Advanced)
            .collect();
        Self {
            candlestick_defs,
            gap_defs,
            chart_defs,
            advanced_defs,
            entries: HashMap::new(),
            gap_context: HashMap::new(),
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

    pub fn update_gaps(
        &mut self,
        coin: &str,
        interval: CandleInterval,
        candles: &[Candle],
    ) -> Vec<PatternLifecycleEntry> {
        let detections = detect_gap_patterns(candles);
        self.apply_gap_detections(coin, interval, candles, &detections)
    }

    pub fn update_chart_patterns(
        &mut self,
        coin: &str,
        interval: CandleInterval,
        candles: &[Candle],
        features: Option<&FeatureSnapshot>,
    ) -> Vec<PatternLifecycleEntry> {
        let detections = detect_chart_patterns(candles, features, interval);
        self.apply_chart_detections(coin, interval, candles, features, &detections)
    }

    pub fn update_advanced_patterns(
        &mut self,
        coin: &str,
        interval: CandleInterval,
        candles: &[Candle],
        features: Option<&FeatureSnapshot>,
    ) -> Vec<PatternLifecycleEntry> {
        let detections = detect_advanced_patterns(candles, features);
        self.apply_advanced_detections(coin, interval, candles, &detections)
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
                category: def.category,
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

    fn apply_gap_detections(
        &mut self,
        coin: &str,
        interval: CandleInterval,
        candles: &[Candle],
        detections: &[DetectedPattern],
    ) -> Vec<PatternLifecycleEntry> {
        let now_ms = candles.last().map(|c| c.close_time).unwrap_or(0);
        let gap_snapshot = gap_snapshot(candles);
        let active_detection = detections.first();
        let mut updated = Vec::new();

        for def in &self.gap_defs {
            let key = PatternLifecycleKey {
                coin: coin.to_string(),
                interval,
                pattern: def.name.to_string(),
                category: def.category,
                classification: def.classification,
            };

            let detection = active_detection
                .filter(|det| det.pattern == def.detector_name && det.classification == def.classification);
            let entry = self.entries.get(&key).cloned();
            let context = self.gap_context.get(&key).cloned();

            let (next, next_context) = next_gap_entry(
                &key,
                def,
                interval,
                now_ms,
                candles,
                entry,
                context,
                detection,
                gap_snapshot.as_ref(),
            );

            self.entries.insert(key.clone(), next.clone());
            match next_context {
                Some(ctx) => {
                    self.gap_context.insert(key, ctx);
                }
                None => {
                    self.gap_context.remove(&key);
                }
            }
            updated.push(next);
        }

        updated
    }

    fn apply_chart_detections(
        &mut self,
        coin: &str,
        interval: CandleInterval,
        candles: &[Candle],
        features: Option<&FeatureSnapshot>,
        detections: &[DetectedPattern],
    ) -> Vec<PatternLifecycleEntry> {
        let now_ms = candles.last().map(|c| c.close_time).unwrap_or(0);
        let detection_map = chart_detection_map(detections);
        let breakout = breakout_context(features, candles);
        let mut updated = Vec::new();

        for def in &self.chart_defs {
            let key = PatternLifecycleKey {
                coin: coin.to_string(),
                interval,
                pattern: def.name.to_string(),
                category: def.category,
                classification: def.classification,
            };

            let detection = detection_map
                .get(&(def.detector_name, def.classification))
                .copied();
            let entry = self.entries.get(&key).cloned();

            let next = next_chart_entry(
                &key,
                def,
                interval,
                now_ms,
                candles,
                entry,
                detection,
                breakout.as_ref(),
            );

            self.entries.insert(key, next.clone());
            updated.push(next);
        }

        updated
    }

    fn apply_advanced_detections(
        &mut self,
        coin: &str,
        interval: CandleInterval,
        candles: &[Candle],
        detections: &[AdvancedDetectedPattern],
    ) -> Vec<PatternLifecycleEntry> {
        let now_ms = candles.last().map(|c| c.close_time).unwrap_or(0);
        let detection_map = advanced_detection_map(detections);
        let mut updated = Vec::new();

        for def in &self.advanced_defs {
            let key = PatternLifecycleKey {
                coin: coin.to_string(),
                interval,
                pattern: def.name.to_string(),
                category: def.category,
                classification: def.classification,
            };

            let detection = detection_map
                .get(&(def.detector_name, def.classification))
                .copied();
            let entry = self.entries.get(&key).cloned();

            let next = next_advanced_entry(
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

fn chart_detection_map(
    detections: &[DetectedPattern],
) -> HashMap<(&'static str, PatternClassification), &DetectedPattern> {
    let mut map = HashMap::new();
    for detection in detections {
        map.insert((detection.pattern, detection.classification), detection);
    }
    map
}

fn advanced_detection_map(
    detections: &[AdvancedDetectedPattern],
) -> HashMap<(&'static str, PatternClassification), &AdvancedDetectedPattern> {
    let mut map = HashMap::new();
    for detection in detections {
        map.insert((detection.pattern, detection.classification), detection);
    }
    map
}

#[derive(Debug, Clone, Copy)]
enum GapDirection {
    Up,
    Down,
}

#[derive(Debug, Clone)]
struct GapContext {
    direction: GapDirection,
    gap_low: f64,
    gap_high: f64,
}

#[derive(Debug, Clone, Copy)]
enum BreakoutDirection {
    Up,
    Down,
}

#[derive(Debug, Clone)]
struct BreakoutContext {
    support: Option<f64>,
    resistance: Option<f64>,
    last_close: f64,
}

#[derive(Debug, Clone, Copy)]
enum BreakoutSignal {
    None,
    Confirmed(BreakoutDirection),
    Invalidated,
}

const BREAKOUT_TOLERANCE_PCT: f64 = 0.002;

fn breakout_context(
    features: Option<&FeatureSnapshot>,
    candles: &[Candle],
) -> Option<BreakoutContext> {
    let features = features?;
    let last = candles.last()?;
    let (support, resistance) = trendline_pair(features);
    let time = last.close_time as f64;
    let support_price = support.and_then(|line| trendline_price_at(line, time));
    let resistance_price = resistance.and_then(|line| trendline_price_at(line, time));

    Some(BreakoutContext {
        support: support_price,
        resistance: resistance_price,
        last_close: last.close,
    })
}

fn breakout_signal(def: &PatternLifecycleDefinition, context: &BreakoutContext) -> BreakoutSignal {
    match def.classification {
        PatternClassification::Bullish => {
            if let Some(resistance) = context.resistance {
                if context.last_close >= resistance * (1.0 + BREAKOUT_TOLERANCE_PCT) {
                    return BreakoutSignal::Confirmed(BreakoutDirection::Up);
                }
            }
            if let Some(support) = context.support {
                if context.last_close <= support * (1.0 - BREAKOUT_TOLERANCE_PCT) {
                    return BreakoutSignal::Invalidated;
                }
            }
        }
        PatternClassification::Bearish => {
            if let Some(support) = context.support {
                if context.last_close <= support * (1.0 - BREAKOUT_TOLERANCE_PCT) {
                    return BreakoutSignal::Confirmed(BreakoutDirection::Down);
                }
            }
            if let Some(resistance) = context.resistance {
                if context.last_close >= resistance * (1.0 + BREAKOUT_TOLERANCE_PCT) {
                    return BreakoutSignal::Invalidated;
                }
            }
        }
        PatternClassification::Neutral => {
            if let Some(resistance) = context.resistance {
                if context.last_close >= resistance * (1.0 + BREAKOUT_TOLERANCE_PCT) {
                    return BreakoutSignal::Confirmed(BreakoutDirection::Up);
                }
            }
            if let Some(support) = context.support {
                if context.last_close <= support * (1.0 - BREAKOUT_TOLERANCE_PCT) {
                    return BreakoutSignal::Confirmed(BreakoutDirection::Down);
                }
            }
        }
    }

    BreakoutSignal::None
}

fn trendline_pair(features: &FeatureSnapshot) -> (Option<&Trendline>, Option<&Trendline>) {
    let mut support = None;
    let mut resistance = None;
    for line in &features.trendlines {
        match line.kind {
            TrendlineKind::Support => support = Some(line),
            TrendlineKind::Resistance => resistance = Some(line),
        }
    }
    (support, resistance)
}

fn trendline_price_at(line: &Trendline, time: f64) -> Option<f64> {
    let price = line.slope * time + line.intercept;
    if price.is_finite() {
        Some(price)
    } else {
        None
    }
}

fn next_gap_entry(
    key: &PatternLifecycleKey,
    def: &PatternLifecycleDefinition,
    interval: CandleInterval,
    now_ms: u64,
    candles: &[Candle],
    previous: Option<PatternLifecycleEntry>,
    previous_context: Option<GapContext>,
    detection: Option<&DetectedPattern>,
    gap_snapshot: Option<&GapContext>,
) -> (PatternLifecycleEntry, Option<GapContext>) {
    if candles.len() < def.window {
        return (
            build_entry(
                key,
                def,
                PatternLifecycleState::Warming,
                previous,
                now_ms,
                0.0,
                0,
                0,
                None,
            ),
            None,
        );
    }

    let last_candle = candles.last();
    let gap_filled = previous_context
        .as_ref()
        .and_then(|ctx| last_candle.map(|c| is_gap_filled(ctx, c)))
        .unwrap_or(false);

    if let Some(previous) = previous.clone() {
        let bars_since = bars_since(previous.state_since_ms, now_ms, interval);
        match previous.state {
            PatternLifecycleState::Forming => {
                if gap_filled {
                    let next = build_entry(
                        key,
                        def,
                        PatternLifecycleState::Invalidated,
                        Some(previous),
                        now_ms,
                        0.0,
                        0,
                        0,
                        None,
                    );
                    return (next, None);
                }
                if bars_since >= 1 {
                    let next = PatternLifecycleEntry {
                        state: PatternLifecycleState::Confirmed,
                        state_since_ms: now_ms,
                        last_updated_ms: now_ms,
                        ..previous
                    };
                    return (next, previous_context);
                }
            }
            PatternLifecycleState::Confirmed => {
                if gap_filled {
                    let next = build_entry(
                        key,
                        def,
                        PatternLifecycleState::Invalidated,
                        Some(previous),
                        now_ms,
                        0.0,
                        0,
                        0,
                        None,
                    );
                    return (next, None);
                }
                if bars_since >= def.max_age_bars {
                    let next = PatternLifecycleEntry {
                        state: PatternLifecycleState::Expired,
                        state_since_ms: now_ms,
                        last_updated_ms: now_ms,
                        ..previous
                    };
                    return (next, previous_context);
                }
            }
            PatternLifecycleState::Expired if bars_since >= def.max_age_bars => {
                let next = build_entry(
                    key,
                    def,
                    PatternLifecycleState::Watching,
                    Some(previous),
                    now_ms,
                    0.0,
                    0,
                    0,
                    None,
                );
                return (next, None);
            }
            PatternLifecycleState::Invalidated => {
                let next = build_entry(
                    key,
                    def,
                    PatternLifecycleState::Watching,
                    Some(previous),
                    now_ms,
                    0.0,
                    0,
                    0,
                    None,
                );
                return (next, None);
            }
            _ => {}
        }
    }

    if let (Some(detection), Some(gap_snapshot)) = (detection, gap_snapshot) {
        let (window_start_ms, window_end_ms) = window_bounds(candles, detection.window);
        let state_since_ms = match previous.as_ref() {
            Some(prev) if prev.state == PatternLifecycleState::Forming => prev.state_since_ms,
            _ => now_ms,
        };
        let entry = PatternLifecycleEntry {
            coin: key.coin.clone(),
            interval: key.interval,
            pattern: key.pattern.clone(),
            category: def.category_label.to_string(),
            classification: def.classification,
            signal_type: def.signal_type,
            state: PatternLifecycleState::Forming,
            confidence: detection.confidence,
            state_since_ms,
            last_updated_ms: now_ms,
            window_start_ms,
            window_end_ms,
            notes: detection.notes.clone(),
        };
        return (entry, Some(gap_snapshot.clone()));
    }

    let entry = build_entry(
        key,
        def,
        PatternLifecycleState::Watching,
        previous,
        now_ms,
        0.0,
        0,
        0,
        None,
    );
    (entry, None)
}

fn next_chart_entry(
    key: &PatternLifecycleKey,
    def: &PatternLifecycleDefinition,
    interval: CandleInterval,
    now_ms: u64,
    candles: &[Candle],
    previous: Option<PatternLifecycleEntry>,
    detection: Option<&DetectedPattern>,
    breakout: Option<&BreakoutContext>,
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
        let signal = breakout
            .map(|context| breakout_signal(def, context))
            .unwrap_or(BreakoutSignal::None);

        let (state, note) = match signal {
            BreakoutSignal::Confirmed(direction) => (
                PatternLifecycleState::Confirmed,
                Some(match direction {
                    BreakoutDirection::Up => "breakout=up".to_string(),
                    BreakoutDirection::Down => "breakout=down".to_string(),
                }),
            ),
            BreakoutSignal::Invalidated => (PatternLifecycleState::Invalidated, None),
            BreakoutSignal::None => (PatternLifecycleState::Forming, None),
        };

        let state_since_ms = match previous.as_ref() {
            Some(prev) if prev.state == state => prev.state_since_ms,
            _ => now_ms,
        };

        return PatternLifecycleEntry {
            coin: key.coin.clone(),
            interval: key.interval,
            pattern: key.pattern.clone(),
            category: def.category_label.to_string(),
            classification: def.classification,
            signal_type: def.signal_type,
            state,
            confidence: detection.confidence,
            state_since_ms,
            last_updated_ms: now_ms,
            window_start_ms,
            window_end_ms,
            notes: note.or_else(|| detection.notes.clone()),
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
        PatternLifecycleState::Forming if bars_since >= 1 => PatternLifecycleState::Expired,
        PatternLifecycleState::Confirmed if bars_since >= def.max_age_bars => {
            PatternLifecycleState::Expired
        }
        PatternLifecycleState::Expired if bars_since >= def.max_age_bars => {
            PatternLifecycleState::Watching
        }
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

fn next_advanced_entry(
    key: &PatternLifecycleKey,
    def: &PatternLifecycleDefinition,
    interval: CandleInterval,
    now_ms: u64,
    candles: &[Candle],
    previous: Option<PatternLifecycleEntry>,
    detection: Option<&AdvancedDetectedPattern>,
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
        let note = Some(format!(
            "method={}, basis={}",
            detection.method, detection.basis
        ));
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
            confidence: detection.confidence,
            state_since_ms,
            last_updated_ms: now_ms,
            window_start_ms,
            window_end_ms,
            notes: note,
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
        PatternLifecycleState::Confirmed if bars_since >= def.max_age_bars => {
            PatternLifecycleState::Expired
        }
        PatternLifecycleState::Expired if bars_since >= def.max_age_bars => {
            PatternLifecycleState::Watching
        }
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

fn gap_snapshot(candles: &[Candle]) -> Option<GapContext> {
    if candles.len() < 2 {
        return None;
    }
    let prev = candles.get(candles.len() - 2)?;
    let current = candles.last()?;

    if current.low > prev.high {
        return Some(GapContext {
            direction: GapDirection::Up,
            gap_low: prev.high,
            gap_high: current.low,
        });
    }
    if current.high < prev.low {
        return Some(GapContext {
            direction: GapDirection::Down,
            gap_low: current.high,
            gap_high: prev.low,
        });
    }
    None
}

fn is_gap_filled(context: &GapContext, candle: &Candle) -> bool {
    match context.direction {
        GapDirection::Up => candle.low <= context.gap_low,
        GapDirection::Down => candle.high >= context.gap_high,
    }
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

    fn chart_detection(pattern: &'static str, classification: PatternClassification) -> DetectedPattern {
        DetectedPattern {
            pattern,
            category: "chart_continuation",
            classification,
            signal_type: PatternSignalType::Continuation,
            confidence: 0.62,
            window: 10,
            notes: None,
        }
    }

    fn advanced_detection(
        pattern: &'static str,
        classification: PatternClassification,
    ) -> AdvancedDetectedPattern {
        AdvancedDetectedPattern {
            pattern,
            category: "williams_fractal",
            classification,
            signal_type: PatternSignalType::Reversal,
            confidence: 0.73,
            window: 5,
            method: "five_bar",
            basis: "pivot_window",
            assumptions: vec!["window=5".to_string()],
        }
    }

    fn chart_features(resistance: f64, support: f64) -> FeatureSnapshot {
        FeatureSnapshot {
            as_of_ms: 0,
            body_ratios: Vec::new(),
            gaps: Vec::new(),
            pivots: Vec::new(),
            trendlines: vec![
                Trendline {
                    kind: TrendlineKind::Resistance,
                    start_time: 0,
                    end_time: 0,
                    start_price: resistance,
                    end_price: resistance,
                    slope: 0.0,
                    intercept: resistance,
                },
                Trendline {
                    kind: TrendlineKind::Support,
                    start_time: 0,
                    end_time: 0,
                    start_price: support,
                    end_price: support,
                    slope: 0.0,
                    intercept: support,
                },
            ],
            ranges: Vec::new(),
            atr: None,
            volatility: None,
        }
    }

    fn common_gap_up_series(
        start_ms: u64,
        interval: CandleInterval,
    ) -> (Vec<Candle>, f64) {
        let mut candles = candle_series(start_ms, interval, 20);
        for candle in &mut candles {
            candle.open = 95.0;
            candle.high = 100.0;
            candle.low = 90.0;
            candle.close = 95.0;
            candle.volume = 0.0;
        }

        let gap_low = 100.0;
        let gap_high = 110.0;
        let last_idx = candles.len() - 1;
        let gap_candle = &mut candles[last_idx];
        gap_candle.open = gap_high + 2.0;
        gap_candle.high = gap_high + 10.0;
        gap_candle.low = gap_high;
        gap_candle.close = gap_high + 5.0;

        (candles, gap_low)
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

    #[test]
    fn gap_entries_form_and_confirm() {
        let mut tracker = PatternLifecycleTracker::new();
        let (candles, gap_low) = common_gap_up_series(0, CandleInterval::OneMinute);

        let entries = tracker.apply_gap_detections(
            "BTC",
            CandleInterval::OneMinute,
            &candles,
            &detect_gap_patterns(&candles),
        );
        let common_gap = entries
            .iter()
            .find(|entry| entry.pattern == "Common Gap")
            .expect("entry");
        assert_eq!(common_gap.state, PatternLifecycleState::Forming);

        let mut next_candles = candles.clone();
        let next_open = next_candles.last().unwrap().close_time;
        let interval_ms = CandleInterval::OneMinute.ms();
        next_candles.push(Candle {
            open_time: next_open,
            close_time: next_open + interval_ms,
            open: gap_low + 15.0,
            high: gap_low + 18.0,
            low: gap_low + 5.0,
            close: gap_low + 12.0,
            volume: 0.0,
            num_trades: 1,
            interval: None,
            symbol: None,
        });

        let entries = tracker.apply_gap_detections(
            "BTC",
            CandleInterval::OneMinute,
            &next_candles,
            &detect_gap_patterns(&next_candles),
        );
        let common_gap = entries
            .iter()
            .find(|entry| entry.pattern == "Common Gap")
            .expect("entry");
        assert_eq!(common_gap.state, PatternLifecycleState::Confirmed);
    }

    #[test]
    fn gap_entries_invalidate_on_fill() {
        let mut tracker = PatternLifecycleTracker::new();
        let (candles, gap_low) = common_gap_up_series(0, CandleInterval::OneMinute);

        tracker.apply_gap_detections(
            "BTC",
            CandleInterval::OneMinute,
            &candles,
            &detect_gap_patterns(&candles),
        );

        let mut next_candles = candles.clone();
        let next_open = next_candles.last().unwrap().close_time;
        let interval_ms = CandleInterval::OneMinute.ms();
        next_candles.push(Candle {
            open_time: next_open,
            close_time: next_open + interval_ms,
            open: gap_low + 2.0,
            high: gap_low + 3.0,
            low: gap_low - 1.0,
            close: gap_low - 0.5,
            volume: 0.0,
            num_trades: 1,
            interval: None,
            symbol: None,
        });

        let entries = tracker.apply_gap_detections(
            "BTC",
            CandleInterval::OneMinute,
            &next_candles,
            &detect_gap_patterns(&next_candles),
        );
        let common_gap = entries
            .iter()
            .find(|entry| entry.pattern == "Common Gap")
            .expect("entry");
        assert_eq!(common_gap.state, PatternLifecycleState::Invalidated);
    }

    #[test]
    fn chart_entries_form_when_no_breakout() {
        let mut tracker = PatternLifecycleTracker::new();
        let candles = candle_series(0, CandleInterval::OneMinute, 20);
        let detection = chart_detection("Ascending Triangle", PatternClassification::Bullish);

        let entries = tracker.apply_chart_detections(
            "BTC",
            CandleInterval::OneMinute,
            &candles,
            None,
            &[detection],
        );

        let triangle = entries
            .iter()
            .find(|entry| entry.pattern == "Ascending Triangle")
            .expect("entry");
        assert_eq!(triangle.state, PatternLifecycleState::Forming);
    }

    #[test]
    fn chart_entries_confirm_on_breakout() {
        let mut tracker = PatternLifecycleTracker::new();
        let mut candles = candle_series(0, CandleInterval::OneMinute, 20);
        if let Some(last) = candles.last_mut() {
            last.close = 120.0;
        }
        let detection = chart_detection("Ascending Triangle", PatternClassification::Bullish);
        let features = chart_features(100.0, 90.0);

        let entries = tracker.apply_chart_detections(
            "BTC",
            CandleInterval::OneMinute,
            &candles,
            Some(&features),
            &[detection],
        );

        let triangle = entries
            .iter()
            .find(|entry| entry.pattern == "Ascending Triangle")
            .expect("entry");
        assert_eq!(triangle.state, PatternLifecycleState::Confirmed);
    }

    #[test]
    fn advanced_entries_confirm_on_detection() {
        let mut tracker = PatternLifecycleTracker::new();
        let candles = candle_series(0, CandleInterval::OneMinute, 10);
        let detection = advanced_detection("Williams Fractal (Up)", PatternClassification::Bearish);

        let entries = tracker.apply_advanced_detections(
            "BTC",
            CandleInterval::OneMinute,
            &candles,
            &[detection],
        );

        let fractal = entries
            .iter()
            .find(|entry| entry.pattern == "Williams Fractal (Up)")
            .expect("entry");
        assert_eq!(fractal.state, PatternLifecycleState::Confirmed);
    }

    #[test]
    fn advanced_entries_expire_after_age() {
        let mut tracker = PatternLifecycleTracker::new();
        let candles = candle_series(0, CandleInterval::OneMinute, 10);
        let detection = advanced_detection("Williams Fractal (Up)", PatternClassification::Bearish);
        tracker.apply_advanced_detections(
            "BTC",
            CandleInterval::OneMinute,
            &candles,
            &[detection],
        );

        let later = candle_series(600_000, CandleInterval::OneMinute, 10);
        let entries =
            tracker.apply_advanced_detections("BTC", CandleInterval::OneMinute, &later, &[]);

        let fractal = entries
            .iter()
            .find(|entry| entry.pattern == "Williams Fractal (Up)")
            .expect("entry");
        assert_eq!(fractal.state, PatternLifecycleState::Expired);
    }
}
