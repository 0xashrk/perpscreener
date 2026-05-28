#[allow(unused_imports)]
use std::collections::HashSet;

use crate::models::patterns::{PatternClassification, PatternSignalType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PatternLifecycleCategory {
    Candlestick,
    Gap,
    Chart,
    Advanced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatternLifecycleDefinition {
    pub name: &'static str,
    pub detector_name: &'static str,
    pub category: PatternLifecycleCategory,
    pub category_label: &'static str,
    pub classification: PatternClassification,
    pub signal_type: PatternSignalType,
    pub window: usize,
    pub max_age_bars: usize,
}

pub fn pattern_registry() -> Vec<PatternLifecycleDefinition> {
    let mut entries = Vec::new();

    for name in CANDLESTICK_BULLISH {
        let window = candlestick_window(name);
        entries.push(PatternLifecycleDefinition {
            name,
            detector_name: candlestick_detector_name(name),
            category: PatternLifecycleCategory::Candlestick,
            category_label: "candlestick_reversal",
            classification: PatternClassification::Bullish,
            signal_type: PatternSignalType::Reversal,
            window,
            max_age_bars: window,
        });
    }

    for name in CANDLESTICK_BEARISH {
        let window = candlestick_window(name);
        entries.push(PatternLifecycleDefinition {
            name,
            detector_name: candlestick_detector_name(name),
            category: PatternLifecycleCategory::Candlestick,
            category_label: "candlestick_reversal",
            classification: PatternClassification::Bearish,
            signal_type: PatternSignalType::Reversal,
            window,
            max_age_bars: window,
        });
    }

    for &(name, classification, signal_type) in GAP_PATTERNS {
        entries.push(PatternLifecycleDefinition {
            name,
            detector_name: name,
            category: PatternLifecycleCategory::Gap,
            category_label: "gap",
            classification,
            signal_type,
            window: 2,
            max_age_bars: 3,
        });
    }

    for &(name, classification, signal_type, category_label, window) in CHART_PATTERNS {
        entries.push(PatternLifecycleDefinition {
            name,
            detector_name: name,
            category: PatternLifecycleCategory::Chart,
            category_label,
            classification,
            signal_type,
            window,
            max_age_bars: window.saturating_mul(2),
        });
    }

    for &(name, classification, signal_type, category_label, window) in ADVANCED_PATTERNS {
        entries.push(PatternLifecycleDefinition {
            name,
            detector_name: name,
            category: PatternLifecycleCategory::Advanced,
            category_label,
            classification,
            signal_type,
            window,
            max_age_bars: window.saturating_mul(2),
        });
    }

    entries
}

fn candlestick_window(name: &str) -> usize {
    match name {
        "Abandoned Baby" => 3,
        "Advance Block" => 3,
        "Belt Hold" => 4,
        "Breakaway" => 5,
        "Concealing Baby Swallow" => 4,
        "Dark Cloud Cover" => 2,
        "Deliberation" => 3,
        "Doji (Dragonfly)" => 1,
        "Doji (Gravestone)" => 1,
        "Doji Star" => 2,
        "Dragonfly Doji / Hanging Man" => 1,
        "Engulfing" => 2,
        "Evening Doji Star" => 3,
        "Evening Star" => 3,
        "Falling Three Methods" => 5,
        "Grave Stone Doji / Shooting Star" => 1,
        "Hammer / Dragonfly Doji" => 1,
        "Harami" => 2,
        "Harami (Bearish)" => 2,
        "Harami Cross" => 2,
        "Hanging Man" => 1,
        "Homing Pigeon" => 2,
        "Identical Three Crows" => 3,
        "In Neck" => 2,
        "Inverted Hammer" => 2,
        "Kicking" => 4,
        "Ladder Bottom" => 5,
        "Mat Hold" => 5,
        "Matching Low" => 2,
        "Meeting Lines" => 2,
        "Morning Doji Star" => 3,
        "Morning Star" => 3,
        "On Neck" => 2,
        "Piercing Line" => 2,
        "Rising Three Methods" => 5,
        "Separating Lines" => 2,
        "Shooting Star" => 1,
        "Side by Side White Lines" => 3,
        "Side-by-side White Lines" => 3,
        "Stick Sandwich" => 3,
        "Three Black Crows" => 3,
        "Three Inside Down" => 3,
        "Three Inside Up" => 3,
        "Three Line Strike" => 4,
        "Three Outside Down" => 3,
        "Three Outside Up" => 3,
        "Three Stars in the South" => 4,
        "Three White Soldiers" => 4,
        "Thrusting" => 2,
        "Tri Star" => 3,
        "Tweezer Bottom" => 2,
        "Tweezer Top" => 2,
        "Two Crows" => 3,
        "Unique Three River Bottom" => 3,
        "Upside Gap Three Methods" => 3,
        "Upside Gap Two Crows" => 3,
        "Upside Tasuki Gap" => 3,
        "Downside Gap Three Methods" => 3,
        "Downside Tasuki Gap" => 3,
        _ => 3,
    }
}

fn candlestick_detector_name(name: &'static str) -> &'static str {
    match name {
        "Hammer / Dragonfly Doji" => "Hammer",
        "Dragonfly Doji / Hanging Man" => "Hanging Man",
        "Grave Stone Doji / Shooting Star" => "Shooting Star",
        "Side-by-side White Lines" => "Side by Side White Lines",
        _ => name,
    }
}

const CANDLESTICK_BULLISH: &[&str] = &[
    "Abandoned Baby",
    "Belt Hold",
    "Breakaway",
    "Concealing Baby Swallow",
    "Doji (Dragonfly)",
    "Doji (Gravestone)",
    "Doji Star",
    "Engulfing",
    "Hammer / Dragonfly Doji",
    "Harami",
    "Harami Cross",
    "Homing Pigeon",
    "Inverted Hammer",
    "Kicking",
    "Ladder Bottom",
    "Mat Hold",
    "Matching Low",
    "Meeting Lines",
    "Morning Doji Star",
    "Morning Star",
    "Piercing Line",
    "Rising Three Methods",
    "Separating Lines",
    "Side by Side White Lines",
    "Stick Sandwich",
    "Three Inside Up",
    "Three Line Strike",
    "Three Outside Up",
    "Three Stars in the South",
    "Three White Soldiers",
    "Tri Star",
    "Tweezer Bottom",
    "Unique Three River Bottom",
    "Upside Gap Three Methods",
    "Upside Tasuki Gap",
];

const CANDLESTICK_BEARISH: &[&str] = &[
    "Abandoned Baby",
    "Advance Block",
    "Belt Hold",
    "Breakaway",
    "Dark Cloud Cover",
    "Deliberation",
    "Downside Gap Three Methods",
    "Downside Tasuki Gap",
    "Doji Star",
    "Doji (Gravestone)",
    "Dragonfly Doji / Hanging Man",
    "Engulfing",
    "Evening Doji Star",
    "Evening Star",
    "Falling Three Methods",
    "Grave Stone Doji / Shooting Star",
    "Hanging Man",
    "Harami (Bearish)",
    "Harami Cross",
    "Identical Three Crows",
    "In Neck",
    "Kicking",
    "Meeting Lines",
    "On Neck",
    "Separating Lines",
    "Shooting Star",
    "Side-by-side White Lines",
    "Three Black Crows",
    "Three Inside Down",
    "Three Line Strike",
    "Three Outside Down",
    "Thrusting",
    "Tri Star",
    "Tweezer Top",
    "Two Crows",
    "Upside Gap Two Crows",
];

const GAP_PATTERNS: &[(&str, PatternClassification, PatternSignalType)] = &[
    (
        "Breakaway Gap (Up)",
        PatternClassification::Bullish,
        PatternSignalType::Trend,
    ),
    (
        "Breakaway Gap (Down)",
        PatternClassification::Bearish,
        PatternSignalType::Trend,
    ),
    (
        "Runaway Gap (Up)",
        PatternClassification::Bullish,
        PatternSignalType::Continuation,
    ),
    (
        "Runaway Gap (Down)",
        PatternClassification::Bearish,
        PatternSignalType::Continuation,
    ),
    (
        "Exhaustion Gap (Up)",
        PatternClassification::Bearish,
        PatternSignalType::Reversal,
    ),
    (
        "Exhaustion Gap (Down)",
        PatternClassification::Bullish,
        PatternSignalType::Reversal,
    ),
    (
        "Common Gap",
        PatternClassification::Neutral,
        PatternSignalType::Range,
    ),
];

const CHART_PATTERNS: &[(&str, PatternClassification, PatternSignalType, &str, usize)] = &[
    (
        "Ascending Triangle",
        PatternClassification::Bullish,
        PatternSignalType::Continuation,
        "chart_continuation",
        10,
    ),
    (
        "Descending Triangle",
        PatternClassification::Bearish,
        PatternSignalType::Continuation,
        "chart_continuation",
        10,
    ),
    (
        "Symmetrical Triangle",
        PatternClassification::Neutral,
        PatternSignalType::Continuation,
        "chart_continuation",
        10,
    ),
    (
        "Bull Flag",
        PatternClassification::Bullish,
        PatternSignalType::Continuation,
        "chart_continuation",
        10,
    ),
    (
        "Bear Flag",
        PatternClassification::Bearish,
        PatternSignalType::Continuation,
        "chart_continuation",
        10,
    ),
    (
        "Bull Pennant",
        PatternClassification::Bullish,
        PatternSignalType::Continuation,
        "chart_continuation",
        10,
    ),
    (
        "Bear Pennant",
        PatternClassification::Bearish,
        PatternSignalType::Continuation,
        "chart_continuation",
        10,
    ),
    (
        "Rising Wedge",
        PatternClassification::Bearish,
        PatternSignalType::Reversal,
        "chart_reversal",
        10,
    ),
    (
        "Falling Wedge",
        PatternClassification::Bullish,
        PatternSignalType::Reversal,
        "chart_reversal",
        10,
    ),
    (
        "Ascending Channel",
        PatternClassification::Bullish,
        PatternSignalType::Trend,
        "channel",
        10,
    ),
    (
        "Descending Channel",
        PatternClassification::Bearish,
        PatternSignalType::Trend,
        "channel",
        10,
    ),
    (
        "Horizontal Channel",
        PatternClassification::Neutral,
        PatternSignalType::Range,
        "channel",
        10,
    ),
    (
        "Head and Shoulders",
        PatternClassification::Bearish,
        PatternSignalType::Reversal,
        "chart_reversal",
        20,
    ),
    (
        "Inverse Head and Shoulders",
        PatternClassification::Bullish,
        PatternSignalType::Reversal,
        "chart_reversal",
        20,
    ),
    (
        "Double Top",
        PatternClassification::Bearish,
        PatternSignalType::Reversal,
        "chart_reversal",
        15,
    ),
    (
        "Double Bottom",
        PatternClassification::Bullish,
        PatternSignalType::Reversal,
        "chart_reversal",
        15,
    ),
    (
        "Triple Top",
        PatternClassification::Bearish,
        PatternSignalType::Reversal,
        "chart_reversal",
        20,
    ),
    (
        "Triple Bottom",
        PatternClassification::Bullish,
        PatternSignalType::Reversal,
        "chart_reversal",
        20,
    ),
    (
        "Cup and Handle",
        PatternClassification::Bullish,
        PatternSignalType::Continuation,
        "chart_continuation",
        25,
    ),
];

const ADVANCED_PATTERNS: &[(&str, PatternClassification, PatternSignalType, &str, usize)] = &[
    (
        "Fibonacci 38.2% Retracement",
        PatternClassification::Neutral,
        PatternSignalType::KeyLevel,
        "fibonacci_retracement",
        10,
    ),
    (
        "Fibonacci 50% Retracement",
        PatternClassification::Neutral,
        PatternSignalType::KeyLevel,
        "fibonacci_retracement",
        10,
    ),
    (
        "Fibonacci 61.8% Retracement",
        PatternClassification::Neutral,
        PatternSignalType::KeyLevel,
        "fibonacci_retracement",
        10,
    ),
    (
        "Elliott Wave 1-2-3-4-5 (Up)",
        PatternClassification::Bullish,
        PatternSignalType::Impulse,
        "elliott_wave",
        30,
    ),
    (
        "Elliott Wave 1-2-3-4-5 (Down)",
        PatternClassification::Bearish,
        PatternSignalType::Impulse,
        "elliott_wave",
        30,
    ),
    (
        "Elliott Wave A-B-C",
        PatternClassification::Neutral,
        PatternSignalType::Correction,
        "elliott_wave",
        20,
    ),
    (
        "Williams Fractal (Up)",
        PatternClassification::Bearish,
        PatternSignalType::Reversal,
        "williams_fractal",
        5,
    ),
    (
        "Williams Fractal (Down)",
        PatternClassification::Bullish,
        PatternSignalType::Reversal,
        "williams_fractal",
        5,
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_expected_counts() {
        let registry = pattern_registry();
        assert_eq!(registry.len(), 105);
    }

    #[test]
    fn registry_has_unique_pattern_keys() {
        let registry = pattern_registry();
        let mut seen = HashSet::new();
        for entry in registry {
            let key = (entry.name, entry.category, entry.classification);
            assert!(seen.insert(key), "duplicate pattern key: {:?}", key);
            assert!(entry.window > 0, "window missing for {}", entry.name);
            assert!(
                entry.max_age_bars >= entry.window,
                "max_age_bars must be >= window for {}",
                entry.name
            );
        }
    }
}
