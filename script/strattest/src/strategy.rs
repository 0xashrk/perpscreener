use crate::data::Candle;

// -- Enums -------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction { Up, Down, Flat }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal { Long, Short, Flat }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Conviction { Strong, Normal, Weak, MeanRevert }

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
pub enum Strategy { TrendFollow, MeanRevert, None }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Regime { Trending, Ranging, Choppy }

// -- Macro context -----------------------------------------------------------

pub struct MacroCtx {
    pub bull: bool,
    pub trend_strength: f64,
    pub atr: f64,
    pub at_breakout_long: bool,
    pub at_breakout_short: bool,
}

pub fn compute_macro(candles_4h: &[Candle], candles_1h: &[Candle], mid: f64) -> MacroCtx {
    let closes_4h: Vec<f64> = candles_4h.iter().map(|c| c.c).collect();
    let sma20 = sma(&closes_4h, 20).unwrap_or(0.0);
    let sma50 = sma(&closes_4h, 50).unwrap_or(0.0);
    let (don_hi, don_lo) = donchian(candles_1h, 20).unwrap_or((0.0, 0.0));
    let atr = compute_atr(candles_1h, 14).unwrap_or(0.0);
    let bull = sma20 > sma50;
    let trend_strength = if mid > 0.0 { (sma20 - sma50).abs() / mid } else { 0.0 };
    MacroCtx {
        bull, trend_strength, atr,
        at_breakout_long: mid > don_hi && don_hi > 0.0,
        at_breakout_short: mid < don_lo && don_lo > 0.0,
    }
}

fn sma(values: &[f64], period: usize) -> Option<f64> {
    if values.len() < period { return None; }
    let sum: f64 = values[values.len() - period..].iter().sum();
    Some(sum / period as f64)
}

fn donchian(candles: &[Candle], length: usize) -> Option<(f64, f64)> {
    if candles.len() < length { return None; }
    let r = &candles[candles.len() - length..];
    Some((r.iter().map(|c| c.h).fold(f64::MIN, f64::max),
          r.iter().map(|c| c.l).fold(f64::MAX, f64::min)))
}

fn compute_atr(candles: &[Candle], period: usize) -> Option<f64> {
    if candles.len() < period + 1 { return None; }
    let trs: Vec<f64> = (1..candles.len()).map(|i| {
        (candles[i].h - candles[i].l)
            .max((candles[i].h - candles[i - 1].c).abs())
            .max((candles[i].l - candles[i - 1].c).abs())
    }).collect();
    if trs.len() < period { return None; }
    Some(trs[trs.len() - period..].iter().sum::<f64>() / period as f64)
}

// -- VWAP --------------------------------------------------------------------

pub struct VwapCtx {
    pub vwap: f64,
    pub price_vs_vwap: f64,
    pub slope: f64,
}

/// Compute VWAP from day candles (filtered to today before calling).
pub fn compute_vwap(day_candles: &[Candle], price: f64) -> Option<VwapCtx> {
    if day_candles.is_empty() { return None; }
    let mut cum_tp_vol = 0.0f64;
    let mut cum_vol = 0.0f64;
    let mut vwaps = Vec::with_capacity(day_candles.len());
    for c in day_candles {
        let tp = (c.h + c.l + c.c) / 3.0;
        cum_tp_vol += tp * c.v;
        cum_vol += c.v;
        if cum_vol > 0.0 { vwaps.push(cum_tp_vol / cum_vol); }
    }
    let vwap = *vwaps.last()?;
    let pvw = if vwap > 0.0 { (price - vwap) / vwap } else { 0.0 };
    let slope = if vwaps.len() >= 4 {
        let t = &vwaps[vwaps.len() - 4..];
        if t[0] > 0.0 { (t[3] - t[0]) / t[0] } else { 0.0 }
    } else { 0.0 };
    Some(VwapCtx { vwap, price_vs_vwap: pvw, slope })
}

// -- BB / RSI ----------------------------------------------------------------

pub struct BollingerBands {
    pub upper: f64,
    pub lower: f64,
    pub width: f64,
}

pub fn compute_bb(candles: &[Candle], period: usize, mult: f64) -> Option<BollingerBands> {
    if candles.len() < period { return None; }
    let closes: Vec<f64> = candles[candles.len() - period..].iter().map(|c| c.c).collect();
    let mean = closes.iter().sum::<f64>() / period as f64;
    let var = closes.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / (period as f64 - 1.0);
    let std = var.sqrt();
    let upper = mean + mult * std;
    let lower = mean - mult * std;
    Some(BollingerBands { upper, lower, width: if mean > 0.0 { (upper - lower) / mean } else { 0.0 } })
}

pub fn compute_rsi(candles: &[Candle], period: usize) -> Option<f64> {
    if candles.len() < period + 1 { return None; }
    let r = &candles[candles.len() - period - 1..];
    let (mut g, mut l) = (0.0f64, 0.0f64);
    for i in 1..r.len() {
        let ch = r[i].c - r[i - 1].c;
        if ch > 0.0 { g += ch; } else { l += ch.abs(); }
    }
    let ag = g / period as f64;
    let al = l / period as f64;
    if al == 0.0 { return Some(100.0); }
    Some(100.0 - 100.0 / (1.0 + ag / al))
}

// -- Regime ------------------------------------------------------------------

pub fn classify_regime(vwap: &VwapCtx, bb: Option<&BollingerBands>, micro_regime: &str) -> Regime {
    if vwap.slope.abs() > 0.0003 && micro_regime == "TRENDING" {
        return Regime::Trending;
    }
    if let Some(bb) = bb {
        if bb.width < 0.015 && micro_regime != "TRENDING" {
            return Regime::Ranging;
        }
    }
    Regime::Choppy
}

// -- Micro context -----------------------------------------------------------

pub struct MicroCtx {
    pub price: f64,
    pub trend_regime: &'static str,
    pub strength: u64,
    pub agreement: &'static str,
}

/// Compute micro context using VWAP-based agreement.
pub fn compute_micro(candles: &[Candle], vwap: &VwapCtx, candle_minutes: u64) -> Option<MicroCtx> {
    if candles.is_empty() { return None; }
    let price = candles.last()?.c;
    let pvw = if vwap.price_vs_vwap > 0.0001 { Direction::Up }
              else if vwap.price_vs_vwap < -0.0001 { Direction::Down }
              else { Direction::Flat };
    let lb1 = (15 / candle_minutes).max(1) as usize;  // ~15 min
    let lb4 = (60 / candle_minutes).max(1) as usize;  // ~1 hour
    let ret1 = ret_over(candles, lb1);
    let ret4 = ret_over(candles, lb4);
    let t1 = trend_label(ret1);
    let t4 = trend_label(ret4);
    let regime: &'static str = match (t1, t4) {
        (Direction::Flat, Direction::Flat) => "DRIFT/FLAT",
        (a, b) if a == b && a != Direction::Flat => "TRENDING",
        _ => "CHOPPY",
    };
    let vol = log_ret_stddev(candles);
    let strength = trend_strength(ret1, ret4, vol, regime);
    let agreement: &'static str = match (pvw, regime, t1) {
        (Direction::Up, "TRENDING", Direction::Up) => "CONTINUATION UP",
        (Direction::Down, "TRENDING", Direction::Down) => "CONTINUATION DOWN",
        (Direction::Up, _, Direction::Down) => "PULLBACK RISK",
        (Direction::Down, _, Direction::Up) => "RECLAIM RISK",
        (_, "CHOPPY", _) => "RANGE/FAKEOUTS",
        _ => "NEUTRAL",
    };
    Some(MicroCtx { price, trend_regime: regime, strength, agreement })
}

fn ret_over(candles: &[Candle], n: usize) -> Option<f64> {
    if candles.len() <= n { return None; }
    Some(candles.last()?.c / candles.get(candles.len() - n - 1)?.c - 1.0)
}

fn trend_label(ret: Option<f64>) -> Direction {
    match ret {
        Some(r) if r.abs() >= 0.0002 => if r > 0.0 { Direction::Up } else { Direction::Down },
        _ => Direction::Flat,
    }
}

fn log_ret_stddev(candles: &[Candle]) -> Option<f64> {
    let rets: Vec<f64> = candles.windows(2)
        .filter_map(|w| if w[0].c > 0.0 && w[1].c > 0.0 { Some((w[1].c / w[0].c).ln()) } else { None })
        .collect();
    if rets.len() < 2 { return None; }
    let mean = rets.iter().sum::<f64>() / rets.len() as f64;
    let var = rets.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (rets.len() as f64 - 1.0);
    Some(var.sqrt())
}

fn trend_strength(r1: Option<f64>, r4: Option<f64>, vol: Option<f64>, regime: &str) -> u64 {
    let mag = match (r1, r4) {
        (Some(a), Some(b)) => (a.abs() + b.abs()) / 2.0,
        (Some(a), None) | (None, Some(a)) => a.abs(),
        _ => 0.0,
    };
    let mut s = (mag * 10_000.0).clamp(0.0, 100.0);
    if regime == "TRENDING" { s = (s + 10.0).min(100.0); }
    else if regime == "CHOPPY" { s = (s - 15.0).max(0.0); }
    if let Some(v) = vol { s = (s - (v * 5_000.0).min(30.0)).max(0.0); }
    s.round() as u64
}

// -- Decision ----------------------------------------------------------------

const MAX_LEV: f64 = 5.0;
const MIN_STOP_PCT: f64 = 0.003;

#[allow(dead_code)]
pub struct TradeSetup {
    pub signal: Signal,
    pub conviction: Conviction,
    pub strategy: Strategy,
    pub entry: f64,
    pub size_asset: f64,
    pub risk_usd: f64,
    pub sl: f64,
    pub tp: f64,       // 0 for trend-follow (uses trailing)
}

pub fn decide(
    mac: &MacroCtx, mic: &MicroCtx, vwap: &VwapCtx,
    regime: Regime, bb: Option<&BollingerBands>, rsi: Option<f64>, equity: f64,
    ob_imbalance: Option<f64>, spread_pct: Option<f64>,
) -> TradeSetup {
    let flat = TradeSetup {
        signal: Signal::Flat, conviction: Conviction::Weak, strategy: Strategy::None,
        entry: mic.price, size_asset: 0.0, risk_usd: 0.0, sl: 0.0, tp: 0.0,
    };
    // Spread gate.
    if let Some(sp) = spread_pct {
        if sp > 0.001 { return flat; }
    }

    match regime {
        Regime::Trending => decide_trend(mac, mic, vwap, equity, ob_imbalance).unwrap_or(flat),
        Regime::Ranging => decide_mr(mic, bb, rsi, equity).unwrap_or(flat),
        Regime::Choppy => flat,
    }
}

fn decide_trend(mac: &MacroCtx, mic: &MicroCtx, vwap: &VwapCtx, equity: f64, ob_imb: Option<f64>) -> Option<TradeSetup> {
    if mac.trend_strength < 0.001 { return None; }
    match mic.agreement {
        "PULLBACK RISK" | "RECLAIM RISK" | "RANGE/FAKEOUTS" => return None,
        _ => {}
    }
    // OB confirms: imbalance >= 1.05 for longs, <= 0.95 for shorts.
    let ob_confirms_long = ob_imb.map(|v| v >= 1.05).unwrap_or(true);
    let ob_confirms_short = ob_imb.map(|v| v <= 0.95).unwrap_or(true);

    let (signal, conviction) = if mac.bull && mic.agreement == "CONTINUATION UP" {
        let strong = mac.at_breakout_long && ob_confirms_long;
        (Signal::Long, if strong { Conviction::Strong } else { Conviction::Normal })
    } else if !mac.bull && mic.agreement == "CONTINUATION DOWN" {
        let strong = mac.at_breakout_short && ob_confirms_short;
        (Signal::Short, if strong { Conviction::Strong } else { Conviction::Normal })
    } else if mac.bull && mic.agreement == "NEUTRAL" && mic.strength > 20 {
        (Signal::Long, Conviction::Weak)
    } else if !mac.bull && mic.agreement == "NEUTRAL" && mic.strength > 20 {
        (Signal::Short, Conviction::Weak)
    } else {
        return None;
    };

    let entry = mic.price.min(vwap.vwap); // limit at VWAP for longs (approx)
    let stop_dist = (1.5 * mac.atr).max(MIN_STOP_PCT * entry);
    let sl = match signal {
        Signal::Long => entry - stop_dist,
        Signal::Short => entry + stop_dist,
        Signal::Flat => 0.0,
    };
    let risk_usd = conviction.risk_pct() * equity;
    let mm = (mic.strength as f64 / 70.0).clamp(0.3, 1.0);
    let size = (risk_usd / stop_dist * mm).min((MAX_LEV * equity) / entry);

    Some(TradeSetup {
        signal, conviction, strategy: Strategy::TrendFollow,
        entry, size_asset: size, risk_usd, sl, tp: 0.0,
    })
}

fn decide_mr(mic: &MicroCtx, bb: Option<&BollingerBands>, rsi: Option<f64>, equity: f64) -> Option<TradeSetup> {
    let bb = bb?;
    let rsi_val = rsi.unwrap_or(50.0);
    let price = mic.price;

    let (signal, limit, sl) = if price <= bb.lower * 1.002 && rsi_val < 35.0 {
        (Signal::Long, bb.lower, bb.lower * 0.99)
    } else if price >= bb.upper * 0.998 && rsi_val > 65.0 {
        (Signal::Short, bb.upper, bb.upper * 1.01)
    } else {
        return None;
    };

    let stop_dist = (limit - sl).abs();
    let risk_usd = Conviction::MeanRevert.risk_pct() * equity;
    let size = (risk_usd / stop_dist).min((MAX_LEV * equity) / price);
    let tp = match signal {
        Signal::Long => limit * 1.008,
        Signal::Short => limit * 0.992,
        Signal::Flat => 0.0,
    };

    Some(TradeSetup {
        signal, conviction: Conviction::MeanRevert, strategy: Strategy::MeanRevert,
        entry: limit, size_asset: size, risk_usd, sl, tp,
    })
}
