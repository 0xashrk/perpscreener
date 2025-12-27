use crate::models::candle::Candle;

/// Average True Range (ATR) calculator
#[derive(Debug, Clone)]
pub struct AtrCalculator {
    period: usize,
    values: Vec<f64>,
    prev_close: Option<f64>,
}

impl AtrCalculator {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            values: Vec::with_capacity(period),
            prev_close: None,
        }
    }

    /// Calculate True Range for a candle
    fn true_range(&self, candle: &Candle) -> f64 {
        let hl = candle.high - candle.low;
        match self.prev_close {
            Some(pc) => {
                let hpc = (candle.high - pc).abs();
                let lpc = (candle.low - pc).abs();
                hl.max(hpc).max(lpc)
            }
            None => hl,
        }
    }

    /// Update ATR with a new candle, returns current ATR if available
    pub fn update(&mut self, candle: &Candle) -> Option<f64> {
        let tr = self.true_range(candle);
        self.prev_close = Some(candle.close);

        if self.values.len() < self.period {
            self.values.push(tr);
            if self.values.len() == self.period {
                // Initial ATR is simple average
                Some(self.values.iter().sum::<f64>() / self.period as f64)
            } else {
                None
            }
        } else {
            // Smoothed ATR: ((prev_atr * (period - 1)) + tr) / period
            let prev_atr = self.values.iter().sum::<f64>() / self.period as f64;
            let new_atr = (prev_atr * (self.period - 1) as f64 + tr) / self.period as f64;
            self.values.remove(0);
            self.values.push(tr);
            Some(new_atr)
        }
    }
}

/// Swing detector for real-time peak/trough identification (no look-ahead)
#[derive(Debug, Clone)]
pub struct SwingDetector {
    rev_atr_mult: f64,
    trend: Option<Trend>,
    swing_high: f64,
    swing_high_idx: usize,
    swing_low: f64,
    swing_low_idx: usize,
    candle_idx: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trend {
    Up,
    Down,
}

#[derive(Debug, Clone)]
pub struct SwingPoint {
    pub price: f64,
    pub is_peak: bool,
}

impl SwingDetector {
    pub fn new(rev_atr_mult: f64) -> Self {
        Self {
            rev_atr_mult,
            trend: None,
            swing_high: 0.0,
            swing_high_idx: 0,
            swing_low: f64::MAX,
            swing_low_idx: 0,
            candle_idx: 0,
        }
    }

    /// Update swing detector with a new candle
    /// Returns a confirmed swing point if a reversal occurred
    pub fn update(&mut self, candle: &Candle, atr: f64) -> Option<SwingPoint> {
        let rev = self.rev_atr_mult * atr;
        self.candle_idx += 1;

        // Initialize trend on first candle
        if self.trend.is_none() {
            self.swing_high = candle.high;
            self.swing_high_idx = self.candle_idx;
            self.swing_low = candle.low;
            self.swing_low_idx = self.candle_idx;
            // Default to up trend
            self.trend = Some(Trend::Up);
            return None;
        }

        match self.trend {
            Some(Trend::Up) => {
                // Track new highs
                if candle.high > self.swing_high {
                    self.swing_high = candle.high;
                    self.swing_high_idx = self.candle_idx;
                }

                // Check for reversal down
                if self.swing_high - candle.low >= rev {
                    let peak = SwingPoint {
                        price: self.swing_high,
                        is_peak: true,
                    };
                    self.trend = Some(Trend::Down);
                    self.swing_low = candle.low;
                    self.swing_low_idx = self.candle_idx;
                    return Some(peak);
                }
            }
            Some(Trend::Down) => {
                // Track new lows
                if candle.low < self.swing_low {
                    self.swing_low = candle.low;
                    self.swing_low_idx = self.candle_idx;
                }

                // Check for reversal up
                if candle.high - self.swing_low >= rev {
                    let trough = SwingPoint {
                        price: self.swing_low,
                        is_peak: false,
                    };
                    self.trend = Some(Trend::Up);
                    self.swing_high = candle.high;
                    self.swing_high_idx = self.candle_idx;
                    return Some(trough);
                }
            }
            None => unreachable!(),
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(high: f64, low: f64, close: f64) -> Candle {
        Candle {
            open_time: 0,
            close_time: 0,
            open: close,
            high,
            low,
            close,
            volume: 0.0,
            num_trades: 0,
            interval: None,
            symbol: None,
        }
    }

    #[test]
    fn test_atr_calculation() {
        let mut atr = AtrCalculator::new(3);

        // First 3 candles to warm up
        assert!(atr.update(&make_candle(102.0, 98.0, 100.0)).is_none()); // TR = 4
        assert!(atr.update(&make_candle(104.0, 99.0, 102.0)).is_none()); // TR = 5
        let result = atr.update(&make_candle(103.0, 100.0, 101.0)); // TR = 3

        // Initial ATR = (4 + 5 + 3) / 3 = 4
        assert!(result.is_some());
        assert!((result.unwrap() - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_swing_detector_peak() {
        let mut detector = SwingDetector::new(1.0);
        let atr = 2.0;

        // Rising price
        detector.update(&make_candle(100.0, 98.0, 99.0), atr);
        detector.update(&make_candle(102.0, 99.0, 101.0), atr);
        detector.update(&make_candle(105.0, 101.0, 104.0), atr);

        // Drop that triggers reversal (drop >= 2.0 ATR)
        let swing = detector.update(&make_candle(104.0, 102.0, 102.5), atr);

        // Should detect peak at 105
        assert!(swing.is_some());
        let point = swing.unwrap();
        assert!(point.is_peak);
        assert!((point.price - 105.0).abs() < 0.01);
    }
}
