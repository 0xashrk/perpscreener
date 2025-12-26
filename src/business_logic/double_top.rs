use crate::business_logic::config::DoubleTopConfig;
use crate::business_logic::indicators::{AtrCalculator, SwingDetector, SwingPoint};
use crate::services::hyperliquid::Candle;
use std::collections::VecDeque;

/// Pattern detection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternState {
    /// Looking for first peak
    Watching,
    /// First peak identified, watching for pullback
    PeakFound,
    /// Pullback complete, watching for second approach
    TroughFound,
    /// Price approaching first peak level (early warning sent)
    Forming,
    /// Breakdown below neckline (confirmed)
    Confirmed,
    /// Pattern invalidated
    Invalidated,
}

/// Alert type
#[derive(Debug, Clone)]
pub enum Alert {
    EarlyWarning {
        coin: String,
        peak_price: f64,
        current_price: f64,
    },
    Confirmation {
        coin: String,
        neckline_price: f64,
        break_price: f64,
    },
}

/// Information about a detected peak
#[derive(Debug, Clone)]
struct PeakInfo {
    price: f64,
    candle_idx: usize,
}

/// Double top detector for a single coin
#[derive(Debug)]
pub struct DoubleTopDetector {
    coin: String,
    config: DoubleTopConfig,
    state: PatternState,
    atr: AtrCalculator,
    swing: SwingDetector,
    candles: VecDeque<Candle>,
    candle_count: usize,

    // Pattern tracking
    peak1: Option<PeakInfo>,
    trough_low: Option<f64>,
    peak2: Option<PeakInfo>,
    early_warning_sent: bool,
}

impl DoubleTopDetector {
    pub fn new(coin: String, config: DoubleTopConfig) -> Self {
        let atr = AtrCalculator::new(config.atr_period);
        let swing = SwingDetector::new(config.rev_atr);

        Self {
            coin,
            config,
            state: PatternState::Watching,
            atr,
            swing,
            candles: VecDeque::new(),
            candle_count: 0,
            peak1: None,
            trough_low: None,
            peak2: None,
            early_warning_sent: false,
        }
    }

    /// Process a new closed candle
    /// Returns an alert if triggered
    pub fn process_candle(&mut self, candle: &Candle) -> Option<Alert> {
        self.candle_count += 1;

        // Maintain rolling window
        self.candles.push_back(candle.clone());
        if self.candles.len() > self.config.history_window {
            self.candles.pop_front();
        }

        // Update ATR
        let atr = self.atr.update(candle);

        // Don't process until warmup complete
        if self.candle_count < self.config.warmup_candles {
            return None;
        }

        let atr = match atr {
            Some(a) => a,
            None => return None,
        };

        // Check for swing points
        if let Some(swing_point) = self.swing.update(candle, atr) {
            self.handle_swing_point(&swing_point);
        }

        // Check for state transitions and alerts
        self.check_state_transitions(candle, atr)
    }

    fn handle_swing_point(&mut self, swing_point: &SwingPoint) {
        match self.state {
            PatternState::Watching => {
                if swing_point.is_peak {
                    self.peak1 = Some(PeakInfo {
                        price: swing_point.price,
                        candle_idx: self.candle_count, // Use global counter, not swing detector's
                    });
                    self.state = PatternState::PeakFound;
                    self.trough_low = None;
                    self.peak2 = None;
                    self.early_warning_sent = false;
                    tracing::debug!(
                        "[{}] Peak 1 found at {} (candle {})",
                        self.coin,
                        swing_point.price,
                        self.candle_count
                    );
                }
            }
            PatternState::PeakFound | PatternState::TroughFound | PatternState::Forming => {
                if !swing_point.is_peak {
                    // Found a trough
                    if let Some(ref peak1) = self.peak1 {
                        let pullback_pct =
                            (peak1.price - swing_point.price) / peak1.price * 100.0;

                        if pullback_pct >= self.config.min_pullback_pct {
                            // Update trough if it's lower (neckline updates)
                            let should_update = self
                                .trough_low
                                .map(|t| swing_point.price < t)
                                .unwrap_or(true);

                            if should_update {
                                self.trough_low = Some(swing_point.price);
                                if self.state == PatternState::PeakFound {
                                    self.state = PatternState::TroughFound;
                                }
                                tracing::debug!(
                                    "[{}] Trough updated to {} (pullback {:.2}%)",
                                    self.coin,
                                    swing_point.price,
                                    pullback_pct
                                );
                            }
                        }
                    }
                } else {
                    // Found another peak - could be Peak 2
                    if self.state == PatternState::TroughFound
                        || self.state == PatternState::Forming
                    {
                        if let Some(ref peak1) = self.peak1 {
                            if self.peaks_match(peak1.price, swing_point.price) {
                                self.peak2 = Some(PeakInfo {
                                    price: swing_point.price,
                                    candle_idx: self.candle_count,
                                });
                                tracing::debug!(
                                    "[{}] Peak 2 found at {} (candle {})",
                                    self.coin,
                                    swing_point.price,
                                    self.candle_count
                                );
                            }
                        }
                    }
                }
            }
            PatternState::Confirmed | PatternState::Invalidated => {
                // Reset and start looking for new pattern
                if swing_point.is_peak {
                    self.reset_with_peak(swing_point);
                }
            }
        }
    }

    fn check_state_transitions(&mut self, candle: &Candle, atr: f64) -> Option<Alert> {
        // Check for invalidation first
        if let Some(ref peak1) = self.peak1 {
            // Price exceeded peak1 by too much
            let fail_level = peak1.price * (1.0 + self.config.peak_fail_pct / 100.0);
            if candle.high > fail_level {
                tracing::info!(
                    "[{}] Pattern INVALIDATED - price {} exceeded fail level {}",
                    self.coin,
                    candle.high,
                    fail_level
                );
                self.state = PatternState::Invalidated;
                return None;
            }

            // Too many candles since peak1
            let candles_since = self.candle_count - peak1.candle_idx;
            if candles_since > self.config.max_peak_distance {
                tracing::debug!(
                    "[{}] Pattern INVALIDATED - {} candles since peak1 (max: {})",
                    self.coin,
                    candles_since,
                    self.config.max_peak_distance
                );
                self.state = PatternState::Invalidated;
                return None;
            }
        }

        // Update trough_low if we're tracking and price makes new low
        if matches!(
            self.state,
            PatternState::TroughFound | PatternState::Forming
        ) {
            if let Some(trough) = self.trough_low {
                if candle.low < trough && self.peak2.is_none() {
                    self.trough_low = Some(candle.low);
                    tracing::debug!(
                        "[{}] Neckline updated to {} (new lower low)",
                        self.coin,
                        candle.low
                    );
                }
            }
        }

        match self.state {
            PatternState::TroughFound => {
                // Check for early warning
                if !self.early_warning_sent {
                    if let Some(alert) = self.check_early_warning(candle) {
                        self.state = PatternState::Forming;
                        self.early_warning_sent = true;
                        return Some(alert);
                    }
                }
            }
            PatternState::Forming => {
                // Check for confirmation
                if let Some(alert) = self.check_confirmation(candle, atr) {
                    self.state = PatternState::Confirmed;
                    return Some(alert);
                }
            }
            _ => {}
        }

        None
    }

    fn check_early_warning(&self, candle: &Candle) -> Option<Alert> {
        let peak1 = self.peak1.as_ref()?;
        let trough = self.trough_low?;

        // Check pattern height
        let pattern_height_pct = (peak1.price - trough) / peak1.price * 100.0;
        if pattern_height_pct < self.config.min_pattern_height {
            return None;
        }

        // Check distance to peak
        let distance_pct = (peak1.price - candle.close).abs() / peak1.price * 100.0;
        if distance_pct > self.config.approach_threshold {
            return None;
        }

        // Check uptrend
        if self.candles.len() > self.config.trend_lookback {
            let lookback_idx = self.candles.len() - self.config.trend_lookback - 1;
            let prev_close = self.candles[lookback_idx].close;
            if candle.close <= prev_close {
                return None;
            }
        }

        // Check not exceeding peak1
        let fail_level = peak1.price * (1.0 + self.config.peak_fail_pct / 100.0);
        if candle.high > fail_level {
            return None;
        }

        tracing::info!(
            "[{}] EARLY WARNING - price {} approaching peak {}",
            self.coin,
            candle.close,
            peak1.price
        );

        Some(Alert::EarlyWarning {
            coin: self.coin.clone(),
            peak_price: peak1.price,
            current_price: candle.close,
        })
    }

    fn check_confirmation(&self, candle: &Candle, atr: f64) -> Option<Alert> {
        let peak1 = self.peak1.as_ref()?;
        let trough = self.trough_low?;
        let peak2 = self.peak2.as_ref()?;

        // Verify peaks match
        if !self.peaks_match(peak1.price, peak2.price) {
            return None;
        }

        // Check pattern height
        let pattern_height_pct = (peak1.price - trough) / peak1.price * 100.0;
        if pattern_height_pct < self.config.min_pattern_height {
            return None;
        }

        // Calculate break level
        let breakdown_buffer_price = self.config.breakdown_buffer * atr;
        let break_level = trough - breakdown_buffer_price;

        // Check for breakdown (using close price for conservative confirmation)
        let broken = candle.close < break_level;

        if broken {
            let break_price = candle.close;

            tracing::info!(
                "[{}] CONFIRMED - broke neckline {} (break level: {}, actual: {})",
                self.coin,
                trough,
                break_level,
                break_price
            );

            return Some(Alert::Confirmation {
                coin: self.coin.clone(),
                neckline_price: trough,
                break_price,
            });
        }

        None
    }

    fn peaks_match(&self, peak1: f64, peak2: f64) -> bool {
        let peak_avg = (peak1 + peak2) / 2.0;
        let peak_diff_pct = (peak1 - peak2).abs() / peak_avg * 100.0;
        peak_diff_pct <= self.config.peak_tolerance
    }

    fn reset_with_peak(&mut self, swing_point: &SwingPoint) {
        self.peak1 = Some(PeakInfo {
            price: swing_point.price,
            candle_idx: self.candle_count,
        });
        self.state = PatternState::PeakFound;
        self.trough_low = None;
        self.peak2 = None;
        self.early_warning_sent = false;
        tracing::debug!(
            "[{}] Reset with new Peak 1 at {} (candle {})",
            self.coin,
            swing_point.price,
            self.candle_count
        );
    }

    /// Get current pattern state
    pub fn state(&self) -> PatternState {
        self.state
    }

    /// Check if detector is warmed up
    pub fn is_warmed_up(&self) -> bool {
        self.candle_count >= self.config.warmup_candles
    }

    /// Get peak 1 price if found
    pub fn peak1_price(&self) -> Option<f64> {
        self.peak1.as_ref().map(|p| p.price)
    }

    /// Get neckline (trough) price if found
    pub fn neckline_price(&self) -> Option<f64> {
        self.trough_low
    }

    /// Get peak 2 price if found
    pub fn peak2_price(&self) -> Option<f64> {
        self.peak2.as_ref().map(|p| p.price)
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
        }
    }

    fn make_config() -> DoubleTopConfig {
        DoubleTopConfig {
            warmup_candles: 20, // Small warmup for tests
            history_window: 100,
            max_peak_distance: 50,
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

    fn warmup_detector(detector: &mut DoubleTopDetector) {
        // Feed some candles to warm up
        for i in 0..20 {
            let price = 95.0 + (i as f64 * 0.1);
            detector.process_candle(&make_candle(price + 0.5, price - 0.5, price));
        }
    }

    #[test]
    fn test_initial_state() {
        let config = make_config();
        let detector = DoubleTopDetector::new("BTC".to_string(), config);
        assert_eq!(detector.state(), PatternState::Watching);
    }

    #[test]
    fn test_peak_detection() {
        let config = make_config();
        let mut detector = DoubleTopDetector::new("BTC".to_string(), config);

        warmup_detector(&mut detector);

        // Create a clear peak
        detector.process_candle(&make_candle(100.0, 98.0, 99.0));
        detector.process_candle(&make_candle(102.0, 99.0, 101.0));
        detector.process_candle(&make_candle(105.0, 100.0, 104.0)); // Peak

        // Sharp drop to trigger swing detection
        detector.process_candle(&make_candle(103.0, 98.0, 99.0));
        detector.process_candle(&make_candle(99.0, 96.0, 97.0));

        // Should have found peak
        assert!(
            detector.state() == PatternState::PeakFound
                || detector.state() == PatternState::TroughFound
        );
    }
}
