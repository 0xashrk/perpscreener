use std::collections::HashMap;

use crate::models::interval::CandleInterval;
use crate::models::patterns::{
    PatternClassification, PatternDetection, PatternSignalType, PatternSummary,
    PatternSummarySignal,
};

#[derive(Debug, Clone)]
pub struct PatternScoreWeights {
    timeframe_weights: HashMap<CandleInterval, f64>,
    signal_type_weights: HashMap<PatternSignalType, f64>,
}

impl PatternScoreWeights {
    pub fn new(
        timeframe_weights: HashMap<CandleInterval, f64>,
        signal_type_weights: HashMap<PatternSignalType, f64>,
    ) -> Self {
        Self {
            timeframe_weights,
            signal_type_weights,
        }
    }

    pub fn timeframe_weight(&self, interval: CandleInterval) -> f64 {
        self.timeframe_weights
            .get(&interval)
            .copied()
            .unwrap_or(1.0)
    }

    pub fn signal_weight(&self, signal_type: PatternSignalType) -> f64 {
        self.signal_type_weights
            .get(&signal_type)
            .copied()
            .unwrap_or(1.0)
    }
}

impl Default for PatternScoreWeights {
    fn default() -> Self {
        let mut timeframe_weights = HashMap::new();
        timeframe_weights.insert(CandleInterval::OneMinute, 0.5);
        timeframe_weights.insert(CandleInterval::ThreeMinutes, 0.6);
        timeframe_weights.insert(CandleInterval::FiveMinutes, 0.7);
        timeframe_weights.insert(CandleInterval::FifteenMinutes, 0.8);
        timeframe_weights.insert(CandleInterval::ThirtyMinutes, 0.9);
        timeframe_weights.insert(CandleInterval::OneHour, 1.0);
        timeframe_weights.insert(CandleInterval::TwoHours, 1.1);
        timeframe_weights.insert(CandleInterval::FourHours, 1.2);
        timeframe_weights.insert(CandleInterval::EightHours, 1.3);
        timeframe_weights.insert(CandleInterval::TwelveHours, 1.4);
        timeframe_weights.insert(CandleInterval::OneDay, 1.6);
        timeframe_weights.insert(CandleInterval::ThreeDays, 1.8);
        timeframe_weights.insert(CandleInterval::OneWeek, 2.0);
        timeframe_weights.insert(CandleInterval::OneMonth, 2.2);

        let mut signal_type_weights = HashMap::new();
        signal_type_weights.insert(PatternSignalType::Reversal, 0.9);
        signal_type_weights.insert(PatternSignalType::Continuation, 1.1);
        signal_type_weights.insert(PatternSignalType::Trend, 1.2);
        signal_type_weights.insert(PatternSignalType::Range, 0.7);
        signal_type_weights.insert(PatternSignalType::KeyLevel, 1.0);
        signal_type_weights.insert(PatternSignalType::Impulse, 1.3);
        signal_type_weights.insert(PatternSignalType::Correction, 0.8);

        Self::new(timeframe_weights, signal_type_weights)
    }
}

pub fn summarize_detections(
    detections: &[PatternDetection],
    weights: &PatternScoreWeights,
) -> Vec<PatternSummary> {
    let mut grouped: HashMap<(String, CandleInterval), Vec<&PatternDetection>> = HashMap::new();

    for detection in detections {
        grouped
            .entry((detection.coin.clone(), detection.interval))
            .or_default()
            .push(detection);
    }

    let mut summaries = Vec::new();

    for ((coin, interval), group) in grouped {
        let mut bullish = 0.0;
        let mut bearish = 0.0;
        let mut neutral = 0.0;
        let mut scored: Vec<(f64, &PatternDetection)> = Vec::new();

        for detection in group {
            let score = detection.confidence
                * weights.timeframe_weight(detection.interval)
                * weights.signal_weight(detection.signal_type);

            match detection.classification {
                PatternClassification::Bullish => bullish += score,
                PatternClassification::Bearish => bearish += score,
                PatternClassification::Neutral => neutral += score,
            }

            scored.push((score, detection));
        }

        let total = bullish + bearish + neutral;
        let (bullish_score, bearish_score, neutral_score) = if total > 0.0 {
            (bullish / total, bearish / total, neutral / total)
        } else {
            (0.0, 0.0, 0.0)
        };

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let top_signals = scored
            .into_iter()
            .take(3)
            .map(|(_, detection)| PatternSummarySignal {
                pattern: detection.pattern.clone(),
                classification: detection.classification,
                confidence: detection.confidence,
            })
            .collect();

        summaries.push(PatternSummary {
            coin,
            interval,
            bullish_score,
            bearish_score,
            neutral_score,
            top_signals,
        });
    }

    summaries.sort_by(|a, b| {
        a.coin
            .cmp(&b.coin)
            .then_with(|| a.interval.as_str().cmp(b.interval.as_str()))
    });

    summaries
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::patterns::{PatternClassification, PatternSignalType};

    fn detection(
        pattern: &str,
        classification: PatternClassification,
        signal_type: PatternSignalType,
        confidence: f64,
    ) -> PatternDetection {
        PatternDetection {
            coin: "BTC".to_string(),
            interval: CandleInterval::OneHour,
            pattern: pattern.to_string(),
            category: "candlestick".to_string(),
            classification,
            signal_type,
            confidence,
            detected_at_ms: 0,
            window_start_ms: 0,
            window_end_ms: 0,
            notes: None,
        }
    }

    #[test]
    fn summarize_detections_normalizes_scores() {
        let weights = PatternScoreWeights::new(HashMap::new(), HashMap::new());
        let detections = vec![
            detection(
                "A",
                PatternClassification::Bullish,
                PatternSignalType::Reversal,
                0.8,
            ),
            detection(
                "B",
                PatternClassification::Bullish,
                PatternSignalType::Trend,
                0.6,
            ),
            detection(
                "C",
                PatternClassification::Bearish,
                PatternSignalType::Reversal,
                0.4,
            ),
            detection(
                "D",
                PatternClassification::Neutral,
                PatternSignalType::Range,
                0.2,
            ),
        ];

        let summaries = summarize_detections(&detections, &weights);
        let summary = summaries.first().expect("summary");

        assert!((summary.bullish_score - 0.7).abs() < 1e-6);
        assert!((summary.bearish_score - 0.2).abs() < 1e-6);
        assert!((summary.neutral_score - 0.1).abs() < 1e-6);
    }

    #[test]
    fn summarize_detections_orders_top_signals_by_weighted_score() {
        let mut signal_weights = HashMap::new();
        signal_weights.insert(PatternSignalType::Trend, 2.0);
        let weights = PatternScoreWeights::new(HashMap::new(), signal_weights);

        let detections = vec![
            detection(
                "Reversal",
                PatternClassification::Bullish,
                PatternSignalType::Reversal,
                0.9,
            ),
            detection(
                "Trend",
                PatternClassification::Bullish,
                PatternSignalType::Trend,
                0.6,
            ),
        ];

        let summaries = summarize_detections(&detections, &weights);
        let summary = summaries.first().expect("summary");

        assert_eq!(summary.top_signals[0].pattern, "Trend");
    }
}
