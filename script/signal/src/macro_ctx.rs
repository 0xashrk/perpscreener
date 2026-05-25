use crate::client::{Candle, L2BookResponse};

#[allow(dead_code)]
pub struct MacroContext {
    pub mid: f64,
    pub sma20: f64,
    pub sma50: f64,
    pub bull: bool,
    pub trend_strength: f64,
    pub don_hi: f64,
    pub don_lo: f64,
    pub atr: f64,
    pub atr_pct: f64,
    pub spread_pct: f64,
    pub ob_imbalance: f64,
    pub at_breakout_long: bool,
    pub at_breakout_short: bool,
}

fn compute_sma(closes: &[f64], period: usize) -> Option<f64> {
    if closes.len() < period {
        return None;
    }
    let sum: f64 = closes[closes.len() - period..].iter().sum();
    Some(sum / period as f64)
}

fn compute_donchian(candles: &[Candle], length: usize) -> Option<(f64, f64)> {
    if candles.len() < length {
        return None;
    }
    let recent = &candles[candles.len() - length..];
    let hi = recent.iter().map(|c| c.h).fold(f64::MIN, f64::max);
    let lo = recent.iter().map(|c| c.l).fold(f64::MAX, f64::min);
    Some((hi, lo))
}

fn compute_atr(candles: &[Candle], period: usize) -> Option<f64> {
    if candles.len() < period + 1 {
        return None;
    }
    let mut trs = Vec::with_capacity(candles.len() - 1);
    for i in 1..candles.len() {
        let tr = (candles[i].h - candles[i].l)
            .max((candles[i].h - candles[i - 1].c).abs())
            .max((candles[i].l - candles[i - 1].c).abs());
        trs.push(tr);
    }
    if trs.len() < period {
        return None;
    }
    let sum: f64 = trs[trs.len() - period..].iter().sum();
    Some(sum / period as f64)
}

fn process_orderbook(ob: &L2BookResponse) -> (f64, f64, f64) {
    let bids = ob.levels.first().map(|v| v.as_slice()).unwrap_or(&[]);
    let asks = ob.levels.get(1).map(|v| v.as_slice()).unwrap_or(&[]);
    let bid = bids.first().map(|l| l.px).unwrap_or(0.0);
    let ask = asks.first().map(|l| l.px).unwrap_or(0.0);
    let mid = (bid + ask) / 2.0;
    let spread_pct = if mid > 0.0 {
        (ask - bid) / mid
    } else {
        0.0
    };
    let bid_sz: f64 = bids.iter().take(10).map(|l| l.sz).sum();
    let ask_sz: f64 = asks.iter().take(10).map(|l| l.sz).sum();
    let ob_imb = if ask_sz > 0.0 { bid_sz / ask_sz } else { 1.0 };
    (mid, spread_pct, ob_imb)
}

/// Compute macro context from 4h candles (trend), 1h candles (levels/vol), and L2 book.
pub fn compute_macro(
    candles_4h: &[Candle],
    candles_1h: &[Candle],
    ob: &L2BookResponse,
) -> MacroContext {
    // Drop most recent candle (not yet closed) for indicator accuracy.
    let closed_4h = if candles_4h.len() > 1 {
        &candles_4h[..candles_4h.len() - 1]
    } else {
        candles_4h
    };
    let closed_1h = if candles_1h.len() > 1 {
        &candles_1h[..candles_1h.len() - 1]
    } else {
        candles_1h
    };

    let closes_4h: Vec<f64> = closed_4h.iter().map(|c| c.c).collect();
    let sma20 = compute_sma(&closes_4h, 20).unwrap_or(0.0);
    let sma50 = compute_sma(&closes_4h, 50).unwrap_or(0.0);
    let (don_hi, don_lo) = compute_donchian(closed_1h, 20).unwrap_or((0.0, 0.0));
    let atr = compute_atr(closed_1h, 14).unwrap_or(0.0);

    let (mid, spread_pct, ob_imbalance) = process_orderbook(ob);
    let bull = sma20 > sma50;
    let trend_strength = if mid > 0.0 {
        (sma20 - sma50).abs() / mid
    } else {
        0.0
    };
    let atr_pct = if mid > 0.0 { atr / mid } else { 0.0 };
    let at_breakout_long = mid > don_hi && don_hi > 0.0;
    let at_breakout_short = mid < don_lo && don_lo > 0.0;

    MacroContext {
        mid,
        sma20,
        sma50,
        bull,
        trend_strength,
        don_hi,
        don_lo,
        atr,
        atr_pct,
        spread_pct,
        ob_imbalance,
        at_breakout_long,
        at_breakout_short,
    }
}
