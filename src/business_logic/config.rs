/// Configuration parameters for double top detection
#[derive(Debug, Clone)]
pub struct DoubleTopConfig {
    /// Historical candles to fetch on startup
    pub warmup_candles: usize,
    /// Rolling candle window size for detection
    pub history_window: usize,
    /// Max candles between two peaks
    pub max_peak_distance: usize,
    /// Max % difference between peak prices
    pub peak_tolerance: f64,
    /// Min % drop to trough from first peak
    pub min_pullback_pct: f64,
    /// Min % from peaks to neckline (validates trough depth)
    pub min_pattern_height: f64,
    /// % distance to Peak 1 to flag early warning
    pub approach_threshold: f64,
    /// ATR window for volatility scaling
    pub atr_period: usize,
    /// Swing reversal size (ATR multiplier)
    pub rev_atr: f64,
    /// Buffer below neckline in ATR units
    pub breakdown_buffer: f64,
    /// % above Peak 1 that invalidates pattern
    pub peak_fail_pct: f64,
    /// Candles to check for uptrend in early warning
    pub trend_lookback: usize,
}

impl Default for DoubleTopConfig {
    fn default() -> Self {
        Self {
            warmup_candles: 200,
            history_window: 300,
            max_peak_distance: 60,
            peak_tolerance: 1.5,
            min_pullback_pct: 2.0,
            min_pattern_height: 2.0,
            approach_threshold: 1.0,
            atr_period: 14,
            rev_atr: 1.0,
            breakdown_buffer: 0.3,
            peak_fail_pct: 1.5,
            trend_lookback: 3,
        }
    }
}
