use crate::models::candle::Candle;

#[derive(Debug, Clone)]
pub struct FeatureConfig {
    pub pivot_left: usize,
    pub pivot_right: usize,
    pub gap_min_pct: f64,
    pub range_window: usize,
    pub atr_period: usize,
    pub volatility_window: usize,
    pub trendline_min_points: usize,
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            pivot_left: 3,
            pivot_right: 3,
            gap_min_pct: 0.003,
            range_window: 20,
            atr_period: 14,
            volatility_window: 20,
            trendline_min_points: 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FeatureSnapshot {
    pub as_of_ms: u64,
    pub body_ratios: Vec<CandleBodyRatio>,
    pub gaps: Vec<Gap>,
    pub pivots: Vec<Pivot>,
    pub trendlines: Vec<Trendline>,
    pub ranges: Vec<PriceRange>,
    pub atr: Option<AtrSnapshot>,
    pub volatility: Option<VolatilitySnapshot>,
}

impl FeatureSnapshot {
    pub fn empty() -> Self {
        Self {
            as_of_ms: 0,
            body_ratios: Vec::new(),
            gaps: Vec::new(),
            pivots: Vec::new(),
            trendlines: Vec::new(),
            ranges: Vec::new(),
            atr: None,
            volatility: None,
        }
    }

    pub fn as_of(candles: &[Candle]) -> u64 {
        candles.last().map(|c| c.close_time).unwrap_or(0)
    }
}

#[derive(Debug, Clone)]
pub struct CandleBodyRatio {
    pub open_time: u64,
    pub close_time: u64,
    pub body: f64,
    pub range: f64,
    pub ratio: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GapDirection {
    Up,
    Down,
}

#[derive(Debug, Clone)]
pub struct Gap {
    pub open_time: u64,
    pub close_time: u64,
    pub previous_close: f64,
    pub gap_open: f64,
    pub size: f64,
    pub percent: f64,
    pub direction: GapDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PivotKind {
    High,
    Low,
}

#[derive(Debug, Clone)]
pub struct Pivot {
    pub index: usize,
    pub time: u64,
    pub price: f64,
    pub kind: PivotKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrendlineKind {
    Support,
    Resistance,
}

#[derive(Debug, Clone)]
pub struct Trendline {
    pub kind: TrendlineKind,
    pub start_time: u64,
    pub end_time: u64,
    pub start_price: f64,
    pub end_price: f64,
    pub slope: f64,
    pub intercept: f64,
}

#[derive(Debug, Clone)]
pub struct PriceRange {
    pub start_time: u64,
    pub end_time: u64,
    pub high: f64,
    pub low: f64,
    pub midpoint: f64,
}

#[derive(Debug, Clone)]
pub struct AtrPoint {
    pub close_time: u64,
    pub value: f64,
}

#[derive(Debug, Clone)]
pub struct AtrSnapshot {
    pub period: usize,
    pub values: Vec<AtrPoint>,
}

#[derive(Debug, Clone)]
pub struct VolatilityPoint {
    pub close_time: u64,
    pub value: f64,
}

#[derive(Debug, Clone)]
pub struct VolatilitySnapshot {
    pub window: usize,
    pub values: Vec<VolatilityPoint>,
}
