pub mod candlesticks;

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
