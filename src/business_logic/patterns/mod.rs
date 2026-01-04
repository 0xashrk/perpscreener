pub mod advanced;
pub mod aggregation;
pub mod candlesticks;
pub mod candlesticks_bearish;
pub mod candlesticks_bullish;
pub mod chart_patterns;
pub mod gaps;
pub mod lifecycle_registry;
pub mod lifecycle_tracker;

use crate::models::patterns::{PatternClassification, PatternSignalType};

#[derive(Debug, Clone)]
pub struct DetectedPattern {
    pub pattern: &'static str,
    pub category: &'static str,
    pub classification: PatternClassification,
    pub signal_type: PatternSignalType,
    pub confidence: f64,
    pub window: usize,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AdvancedDetectedPattern {
    pub pattern: &'static str,
    pub category: &'static str,
    pub classification: PatternClassification,
    pub signal_type: PatternSignalType,
    pub confidence: f64,
    pub window: usize,
    pub method: &'static str,
    pub basis: &'static str,
    pub assumptions: Vec<String>,
}
