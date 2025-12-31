mod atr;
mod body_ratio;
mod gaps;
mod pivots;
mod ranges;
mod trendlines;
mod volatility;

pub mod types;

use crate::models::candle::Candle;

pub use types::{
    AtrPoint, AtrSnapshot, CandleBodyRatio, FeatureConfig, FeatureSnapshot, Gap, GapDirection,
    Pivot, PivotKind, PriceRange, Trendline, TrendlineKind, VolatilityPoint, VolatilitySnapshot,
};

pub fn compute_features(candles: &[Candle], config: &FeatureConfig) -> FeatureSnapshot {
    if candles.is_empty() {
        return FeatureSnapshot::empty();
    }

    let body_ratios = body_ratio::compute_body_ratios(candles);
    let gaps = gaps::detect_gaps(candles, config.gap_min_pct);
    let pivots = pivots::detect_pivots(candles, config.pivot_left, config.pivot_right);
    let trendlines = trendlines::derive_trendlines(&pivots, config.trendline_min_points);
    let ranges = ranges::compute_ranges(candles, config.range_window);
    let atr = atr::compute_atr(candles, config.atr_period);
    let volatility = volatility::compute_volatility(candles, config.volatility_window);

    FeatureSnapshot {
        as_of_ms: FeatureSnapshot::as_of(candles),
        body_ratios,
        gaps,
        pivots,
        trendlines,
        ranges,
        atr,
        volatility,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_features_returns_empty_for_no_candles() {
        let snapshot = compute_features(&[], &FeatureConfig::default());
        assert_eq!(snapshot.as_of_ms, 0);
        assert!(snapshot.body_ratios.is_empty());
        assert!(snapshot.gaps.is_empty());
    }
}
