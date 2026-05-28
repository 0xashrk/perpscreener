use crate::daily_ctx::{DailyContext, VolumeContext};
use crate::indicators::BollingerBands;
use crate::macro_ctx::MacroContext;
use crate::micro_ctx::MicroContext;
use crate::regime::Regime;
use crate::vwap::VwapContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    Long,
    Short,
    Flat,
}

impl Signal {
    pub fn as_str(self) -> &'static str {
        match self {
            Signal::Long => "LONG",
            Signal::Short => "SHORT",
            Signal::Flat => "FLAT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Conviction {
    Strong,
    Normal,
    Weak,
    MeanRevert,
}

impl Conviction {
    pub fn as_str(self) -> &'static str {
        match self {
            Conviction::Strong => "STRONG",
            Conviction::Normal => "NORMAL",
            Conviction::Weak => "WEAK",
            Conviction::MeanRevert => "MR",
        }
    }

    pub fn risk_pct(self) -> f64 {
        match self {
            Conviction::Strong => 0.005,
            Conviction::Normal => 0.003,
            Conviction::Weak => 0.0015,
            Conviction::MeanRevert => 0.002,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    TrendFollow,
    MeanRevert,
    None,
}

impl Strategy {
    pub fn as_str(self) -> &'static str {
        match self {
            Strategy::TrendFollow => "trend-follow",
            Strategy::MeanRevert => "mean-revert",
            Strategy::None => "-",
        }
    }
}

pub struct TradeDecision {
    pub signal: Signal,
    pub conviction: Conviction,
    pub strategy: Strategy,
    pub limit_price: f64,
    pub size_asset: f64,
    pub size_usd: f64,
    pub risk_usd: f64,
    pub sl: f64,
    pub leverage: u32,
    pub max_leverage: u32,
    pub reason: String,
}

const MIN_STOP_PCT: f64 = 0.003;

/// Recommended leverage per conviction tier.
fn recommended_leverage(conviction: Conviction) -> u32 {
    match conviction {
        Conviction::Strong => 5,
        Conviction::Normal => 3,
        Conviction::Weak => 2,
        Conviction::MeanRevert => 2,
    }
}

pub fn decide(
    mac: &MacroContext,
    mic: &MicroContext,
    vwap: &VwapContext,
    regime: Regime,
    bb: Option<&BollingerBands>,
    rsi: Option<f64>,
    av: f64,
    asset_max_leverage: u32,
    daily: Option<&DailyContext>,
    vol_ctx: Option<&VolumeContext>,
) -> TradeDecision {
    let flat = || TradeDecision {
        signal: Signal::Flat,
        conviction: Conviction::Weak,
        strategy: Strategy::None,
        limit_price: mic.current_price,
        size_asset: 0.0,
        size_usd: 0.0,
        risk_usd: 0.0,
        sl: 0.0,
        leverage: 0,
        max_leverage: asset_max_leverage,
        reason: String::new(),
    };

    // Daily structure gate: block longs near resistance on declining volume.
    if let Some(d) = daily {
        if let Some(v) = vol_ctx {
            // Near 20-day high + declining volume → skip longs.
            if d.near_resistance && v.vol_declining {
                let mut r = flat();
                r.reason = format!(
                    "near 20d high ({:.2}%) + declining volume (ratio {:.2}) — no longs",
                    d.pct_from_high * 100.0, v.vol_ratio
                );
                return r;
            }
            // Near 20-day low + declining volume → skip shorts.
            if d.near_support && v.vol_declining {
                let mut r = flat();
                r.reason = format!(
                    "near 20d low ({:.2}%) + declining volume (ratio {:.2}) — no shorts",
                    d.pct_from_low * 100.0, v.vol_ratio
                );
                return r;
            }
        }
        // Near resistance without volume confirmation → downgrade longs (only allow STRONG).
        if d.near_resistance && regime == Regime::Trending {
            let vol_confirms = vol_ctx.map(|v| v.vol_confirms).unwrap_or(false);
            if !vol_confirms {
                // Still allow the trade but will be handled in trend_follow
                // by passing the daily context through.
            }
        }
    }

    match regime {
        Regime::Trending => {
            let d = trend_follow(mac, mic, vwap, av, asset_max_leverage, &flat);
            // Downgrade near resistance without volume.
            if let (Some(dc), Signal::Long) = (daily, d.signal) {
                if dc.near_resistance {
                    let vol_confirms = vol_ctx.map(|v| v.vol_confirms).unwrap_or(false);
                    if !vol_confirms && d.conviction != Conviction::Strong {
                        let mut r = flat();
                        r.reason = format!(
                            "near 20d high ({:+.1}%) — need STRONG + volume to go long",
                            dc.pct_from_high * 100.0
                        );
                        return r;
                    }
                }
            }
            // Downgrade near support without volume for shorts.
            if let (Some(dc), Signal::Short) = (daily, d.signal) {
                if dc.near_support {
                    let vol_confirms = vol_ctx.map(|v| v.vol_confirms).unwrap_or(false);
                    if !vol_confirms && d.conviction != Conviction::Strong {
                        let mut r = flat();
                        r.reason = format!(
                            "near 20d low ({:+.1}%) — need STRONG + volume to go short",
                            dc.pct_from_low * 100.0
                        );
                        return r;
                    }
                }
            }
            d
        }
        Regime::Ranging => mean_revert(mic, vwap, bb, rsi, mac, av, asset_max_leverage, flat),
        Regime::Choppy => {
            let mut d = flat();
            d.reason = "choppy regime — no trade".to_string();
            d
        }
    }
}

fn trend_follow(
    mac: &MacroContext,
    mic: &MicroContext,
    vwap: &VwapContext,
    av: f64,
    asset_max_lev: u32,
    flat: &impl Fn() -> TradeDecision,
) -> TradeDecision {
    // Gate: spread.
    if mac.spread_pct > 0.001 {
        let mut d = flat();
        d.reason = format!("spread too wide ({:.4}%)", mac.spread_pct * 100.0);
        return d;
    }

    // Gate: macro trend strength.
    if mac.trend_strength < 0.001 {
        let mut d = flat();
        d.reason = "no macro trend".to_string();
        return d;
    }

    // Gate: micro disagreement.
    match mic.agreement {
        "PULLBACK RISK" | "RECLAIM RISK" | "RANGE/FAKEOUTS" => {
            let mut d = flat();
            d.reason = format!("micro: {}", mic.agreement.to_lowercase());
            return d;
        }
        _ => {}
    }

    // Long signals.
    if mac.bull && mic.agreement == "CONTINUATION UP" {
        let conviction = if mac.at_breakout_long && mac.ob_imbalance >= 1.05 {
            Conviction::Strong
        } else {
            Conviction::Normal
        };
        let reason = if conviction == Conviction::Strong {
            "bull + breakout + VWAP above + micro cont + OB"
        } else {
            "bull + VWAP above + micro continuation UP"
        };
        return build_trend(
            Signal::Long,
            conviction,
            mic.current_price,
            vwap.vwap,
            mac.atr,
            mic.strength,
            av,
            asset_max_lev,
            reason,
        );
    }
    if mac.bull && mic.agreement == "NEUTRAL" && mic.strength > 20 {
        return build_trend(
            Signal::Long,
            Conviction::Weak,
            mic.current_price,
            vwap.vwap,
            mac.atr,
            mic.strength,
            av,
            asset_max_lev,
            "bull + micro neutral — weak alignment",
        );
    }

    // Short signals.
    if !mac.bull && mic.agreement == "CONTINUATION DOWN" {
        let conviction = if mac.at_breakout_short && mac.ob_imbalance <= 0.95 {
            Conviction::Strong
        } else {
            Conviction::Normal
        };
        let reason = if conviction == Conviction::Strong {
            "bear + breakdown + VWAP below + micro cont + OB"
        } else {
            "bear + VWAP below + micro continuation DOWN"
        };
        return build_trend(
            Signal::Short,
            conviction,
            mic.current_price,
            vwap.vwap,
            mac.atr,
            mic.strength,
            av,
            asset_max_lev,
            reason,
        );
    }
    if !mac.bull && mic.agreement == "NEUTRAL" && mic.strength > 20 {
        return build_trend(
            Signal::Short,
            Conviction::Weak,
            mic.current_price,
            vwap.vwap,
            mac.atr,
            mic.strength,
            av,
            asset_max_lev,
            "bear + micro neutral — weak alignment",
        );
    }

    let mut d = flat();
    d.reason = "macro and micro not aligned".to_string();
    d
}

fn build_trend(
    signal: Signal,
    conviction: Conviction,
    price: f64,
    vwap: f64,
    atr: f64,
    strength: u64,
    av: f64,
    asset_max_lev: u32,
    reason: &str,
) -> TradeDecision {
    let lev = recommended_leverage(conviction).min(asset_max_lev);
    let stop_dist = (1.5 * atr).max(MIN_STOP_PCT * price);

    let limit_price = match signal {
        Signal::Long => price.min(vwap),
        Signal::Short => price.max(vwap),
        Signal::Flat => price,
    };

    let sl = match signal {
        Signal::Long => limit_price - stop_dist,
        Signal::Short => limit_price + stop_dist,
        Signal::Flat => 0.0,
    };

    let risk_usd = conviction.risk_pct() * av;
    let momentum_mult = (strength as f64 / 70.0).clamp(0.3, 1.0);
    let raw_size = (risk_usd / stop_dist) * momentum_mult;
    let max_size = (lev as f64 * av) / price;
    let size_asset = raw_size.min(max_size);
    let size_usd = size_asset * price;

    TradeDecision {
        signal,
        conviction,
        strategy: Strategy::TrendFollow,
        limit_price,
        size_asset,
        size_usd,
        risk_usd,
        sl,
        leverage: lev,
        max_leverage: asset_max_lev,
        reason: reason.to_string(),
    }
}

fn mean_revert(
    mic: &MicroContext,
    _vwap: &VwapContext,
    bb: Option<&BollingerBands>,
    rsi: Option<f64>,
    mac: &MacroContext,
    av: f64,
    asset_max_lev: u32,
    flat: impl Fn() -> TradeDecision,
) -> TradeDecision {
    let bb = match bb {
        Some(b) => b,
        None => {
            let mut d = flat();
            d.reason = "ranging but BB unavailable".to_string();
            return d;
        }
    };
    let rsi_val = rsi.unwrap_or(50.0);
    let price = mic.current_price;
    let lev = recommended_leverage(Conviction::MeanRevert).min(asset_max_lev);

    if mac.spread_pct > 0.0008 {
        let mut d = flat();
        d.reason = format!("MR: spread too wide ({:.4}%)", mac.spread_pct * 100.0);
        return d;
    }

    if price <= bb.lower * 1.002 && rsi_val < 35.0 {
        let limit_price = bb.lower;
        let sl = limit_price * 0.99;
        let risk_usd = Conviction::MeanRevert.risk_pct() * av;
        let stop_dist = (limit_price - sl).abs();
        let size_asset = (risk_usd / stop_dist).min((lev as f64 * av) / price);
        return TradeDecision {
            signal: Signal::Long,
            conviction: Conviction::MeanRevert,
            strategy: Strategy::MeanRevert,
            limit_price,
            size_asset,
            size_usd: size_asset * price,
            risk_usd,
            sl,
            leverage: lev,
            max_leverage: asset_max_lev,
            reason: format!("MR long: BB lower ({:.4}), RSI={:.0}", bb.lower, rsi_val),
        };
    }

    if price >= bb.upper * 0.998 && rsi_val > 65.0 {
        let limit_price = bb.upper;
        let sl = limit_price * 1.01;
        let risk_usd = Conviction::MeanRevert.risk_pct() * av;
        let stop_dist = (sl - limit_price).abs();
        let size_asset = (risk_usd / stop_dist).min((lev as f64 * av) / price);
        return TradeDecision {
            signal: Signal::Short,
            conviction: Conviction::MeanRevert,
            strategy: Strategy::MeanRevert,
            limit_price,
            size_asset,
            size_usd: size_asset * price,
            risk_usd,
            sl,
            leverage: lev,
            max_leverage: asset_max_lev,
            reason: format!("MR short: BB upper ({:.4}), RSI={:.0}", bb.upper, rsi_val),
        };
    }

    let mut d = flat();
    d.reason = format!(
        "ranging, no BB touch (BB=[{:.4}, {:.4}], RSI={:.0})",
        bb.lower, bb.upper, rsi_val
    );
    d
}
