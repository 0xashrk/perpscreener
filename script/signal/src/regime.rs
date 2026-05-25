use crate::indicators::BollingerBands;
use crate::vwap::VwapContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Regime {
    Trending,
    Ranging,
    Choppy,
}

impl Regime {
    pub fn as_str(self) -> &'static str {
        match self {
            Regime::Trending => "TRENDING",
            Regime::Ranging => "RANGING",
            Regime::Choppy => "CHOPPY",
        }
    }
}

/// Classify market regime from VWAP slope, BB width, and micro trend.
pub fn classify(
    vwap: &VwapContext,
    bb: Option<&BollingerBands>,
    micro_regime: &str,
    vwap_slope_threshold: f64,
    bb_tight_threshold: f64,
) -> Regime {
    if vwap.vwap_slope.abs() > vwap_slope_threshold && micro_regime == "TRENDING" {
        return Regime::Trending;
    }

    if let Some(bb) = bb {
        if bb.width < bb_tight_threshold && micro_regime != "TRENDING" {
            return Regime::Ranging;
        }
    }

    Regime::Choppy
}
