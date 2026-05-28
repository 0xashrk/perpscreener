#![allow(dead_code)]

use crate::data::Candle;
use crate::strategy::*;

// -- Types -------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitReason {
    TpHit,
    SlHit,
    TrailingStop,
    AgreementFlip,
    RegimeFlip,
    VwapCross,
    EndOfData,
}

impl ExitReason {
    pub fn as_str(self) -> &'static str {
        match self {
            ExitReason::TpHit => "TP",
            ExitReason::SlHit => "SL",
            ExitReason::TrailingStop => "TRAIL",
            ExitReason::AgreementFlip => "AGREE_FLIP",
            ExitReason::RegimeFlip => "REGIME_FLIP",
            ExitReason::VwapCross => "VWAP_CROSS",
            ExitReason::EndOfData => "END",
        }
    }
}

struct Position {
    signal: Signal,
    conviction: Conviction,
    strategy: Strategy,
    entry: f64,
    sl: f64,
    tp: f64,
    trail_stop: f64,
    size_asset: f64,
    entry_time: u64,
    high_water: f64,
    low_water: f64,
    vwap_cross_count: u32,
    initial_regime: String,
    funding_paid: f64,
    last_funding_hour: u64,
}

pub struct TradeRecord {
    pub signal: Signal,
    pub conviction: Conviction,
    pub strategy: Strategy,
    pub entry: f64,
    pub exit: f64,
    pub entry_time: u64,
    pub exit_time: u64,
    pub pnl_usd: f64,
    pub pnl_pct: f64,
    pub exit_reason: ExitReason,
}

pub struct BacktestResult {
    pub trades: Vec<TradeRecord>,
    pub final_equity: f64,
    pub max_drawdown_pct: f64,
}

// -- Engine ------------------------------------------------------------------

/// OB snapshot loaded from CSV.
pub struct ObRecord {
    pub timestamp_ms: u64,
    pub ob_imbalance: f64,
    pub spread_pct: f64,
}

/// Look up the nearest OB record at or before `ts`.
fn lookup_ob(ob_data: &[ObRecord], ts: u64) -> Option<(f64, f64)> {
    let idx = ob_data.partition_point(|r| r.timestamp_ms <= ts);
    if idx == 0 { return None; }
    let r = &ob_data[idx - 1];
    // Only use if within 30 minutes.
    if ts - r.timestamp_ms < 30 * 60_000 {
        Some((r.ob_imbalance, r.spread_pct))
    } else {
        None
    }
}

pub fn run(
    candles_micro: &[Candle],
    candles_1h: &[Candle],
    candles_4h: &[Candle],
    initial_av: f64,
    check_interval_min: u64,
    cooldown_min: u64,
    candle_minutes: u64,
    ob_data: &[ObRecord],
) -> BacktestResult {
    let mut equity = initial_av;
    let mut peak = initial_av;
    let mut max_dd = 0.0f64;
    let mut trades: Vec<TradeRecord> = Vec::new();
    let mut position: Option<Position> = None;
    let mut last_exit_ts: u64 = 0;
    let cooldown_ms = cooldown_min * 60_000;

    for (i, candle) in candles_micro.iter().enumerate() {
        let ts = candle.t;
        let day_ms = ts - (ts % 86_400_000); // midnight UTC
        let minute_of_day = (ts - day_ms) / 60_000;

        // --- TP/SL + trailing stop + funding check every candle ---
        if let Some(ref mut pos) = position {
            // Update high/low water marks.
            pos.high_water = pos.high_water.max(candle.h);
            pos.low_water = pos.low_water.min(candle.l);

            // Track funding if we crossed an hourly boundary.
            // Cost is accumulated in pos.funding_paid and deducted in compute_pnl.
            charge_funding(pos, ts, candle.c);

            // Update trailing stop for trend-follow.
            if pos.strategy == Strategy::TrendFollow {
                update_trailing(pos);
            }

            if let Some((exit_price, reason)) = check_exits(pos, candle) {
                let pnl = compute_pnl(pos, exit_price, reason);
                equity += pnl;
                trades.push(make_record(pos, exit_price, ts, pnl, reason));
                last_exit_ts = ts;
                position = None;
            }
        }

        // --- Decision checkpoint ---
        if minute_of_day % check_interval_min == 0 && minute_of_day >= 15 {
            // Compute VWAP from today's candles up to this point.
            let day_start = day_start_idx(candles_micro, day_ms, i);
            let day_candles = &candles_micro[day_start..=i];
            let price = candle.c;

            let vwap = match compute_vwap(day_candles, price) {
                Some(v) => v,
                None => continue,
            };

            // Recent candles for BB/RSI (use all available up to this point, last 30+ candles).
            let lookback_start = if i >= 30 { i - 30 } else { 0 };
            let recent = &candles_micro[lookback_start..=i];

            let bb = compute_bb(recent, 20, 2.0);
            let rsi = compute_rsi(recent, 14);

            let mic = match compute_micro(day_candles, &vwap, candle_minutes) {
                Some(m) => m,
                None => continue,
            };

            let regime = classify_regime(&vwap, bb.as_ref(), mic.trend_regime);

            // If in position, check signal-based exits.
            if let Some(ref mut pos) = position {
                // VWAP cross check.
                let wrong_side = match pos.signal {
                    Signal::Long => price < vwap.vwap,
                    Signal::Short => price > vwap.vwap,
                    Signal::Flat => false,
                };
                if wrong_side {
                    pos.vwap_cross_count += 1;
                } else {
                    pos.vwap_cross_count = 0;
                }
                if pos.vwap_cross_count >= 2 {
                    let pnl = compute_pnl(pos, price, ExitReason::VwapCross);
                    equity += pnl;
                    trades.push(make_record(pos, price, ts, pnl, ExitReason::VwapCross));
                    last_exit_ts = ts;
                    position = None;
                }

                // Regime/agreement flip.
                if position.is_some() {
                    let pos = position.as_ref().unwrap();
                    if should_exit_signal(pos, &mic) {
                        let reason = if mic.trend_regime != pos.initial_regime {
                            ExitReason::RegimeFlip
                        } else {
                            ExitReason::AgreementFlip
                        };
                        let pnl = compute_pnl(pos, price, reason);
                        equity += pnl;
                        trades.push(make_record(pos, price, ts, pnl, reason));
                        last_exit_ts = ts;
                        position = None;
                    }
                }
            }

            // Try to enter.
            if position.is_none() && ts.saturating_sub(last_exit_ts) >= cooldown_ms {
                let closed_4h = closed_before(candles_4h, ts);
                let closed_1h = closed_before(candles_1h, ts);

                if closed_4h.len() >= 50 && closed_1h.len() >= 20 {
                    let mac = compute_macro(closed_4h, closed_1h, price);
                    // Look up OB data for this timestamp.
                    let (ob_imb, ob_spread) = match lookup_ob(ob_data, ts) {
                        Some((imb, sp)) => (Some(imb), Some(sp)),
                        None => (None, None),
                    };
                    // Use current equity for sizing (compounding).
                    let setup = decide(&mac, &mic, &vwap, regime, bb.as_ref(), rsi, equity, ob_imb, ob_spread);
                    if setup.signal != Signal::Flat {
                        position = Some(open_position(setup, ts, mic.trend_regime));
                    }
                }
            }
        }

        peak = peak.max(equity);
        let dd = (peak - equity) / peak;
        max_dd = max_dd.max(dd);
    }

    // Force-close remaining position.
    if let Some(ref pos) = position {
        if let Some(last) = candles_micro.last() {
            let pnl = compute_pnl(pos, last.c, ExitReason::EndOfData);
            equity += pnl;
            trades.push(make_record(pos, last.c, last.t, pnl, ExitReason::EndOfData));
        }
    }

    BacktestResult { trades, final_equity: equity, max_drawdown_pct: max_dd * 100.0 }
}

// -- Helpers -----------------------------------------------------------------

fn update_trailing(pos: &mut Position) {
    let pnl_pct = match pos.signal {
        Signal::Long => (pos.high_water - pos.entry) / pos.entry,
        Signal::Short => (pos.entry - pos.low_water) / pos.entry,
        Signal::Flat => 0.0,
    };

    let new_trail = match pos.signal {
        Signal::Long => {
            if pnl_pct >= 0.01 {
                pos.high_water - 0.004 * pos.entry  // tight trail
            } else if pnl_pct >= 0.006 {
                pos.entry + 0.003 * pos.entry        // lock profit
            } else if pnl_pct >= 0.003 {
                pos.entry                             // breakeven
            } else {
                return; // no trail yet
            }
        }
        Signal::Short => {
            if pnl_pct >= 0.01 {
                pos.low_water + 0.004 * pos.entry
            } else if pnl_pct >= 0.006 {
                pos.entry - 0.003 * pos.entry
            } else if pnl_pct >= 0.003 {
                pos.entry
            } else {
                return;
            }
        }
        Signal::Flat => return,
    };

    // Only tighten, never loosen.
    match pos.signal {
        Signal::Long => pos.trail_stop = pos.trail_stop.max(new_trail),
        Signal::Short => {
            if pos.trail_stop == 0.0 { pos.trail_stop = new_trail; }
            else { pos.trail_stop = pos.trail_stop.min(new_trail); }
        }
        Signal::Flat => {}
    }
}

fn check_exits(pos: &Position, candle: &Candle) -> Option<(f64, ExitReason)> {
    match pos.signal {
        Signal::Long => {
            if candle.l <= pos.sl { return Some((pos.sl, ExitReason::SlHit)); }
            if pos.trail_stop > 0.0 && candle.l <= pos.trail_stop {
                return Some((pos.trail_stop, ExitReason::TrailingStop));
            }
            if pos.tp > 0.0 && candle.h >= pos.tp { return Some((pos.tp, ExitReason::TpHit)); }
        }
        Signal::Short => {
            if candle.h >= pos.sl { return Some((pos.sl, ExitReason::SlHit)); }
            if pos.trail_stop > 0.0 && candle.h >= pos.trail_stop {
                return Some((pos.trail_stop, ExitReason::TrailingStop));
            }
            if pos.tp > 0.0 && candle.l <= pos.tp { return Some((pos.tp, ExitReason::TpHit)); }
        }
        Signal::Flat => {}
    }
    None
}

fn should_exit_signal(pos: &Position, mic: &MicroCtx) -> bool {
    let bad = match pos.signal {
        Signal::Long => matches!(mic.agreement, "PULLBACK RISK" | "RANGE/FAKEOUTS" | "CONTINUATION DOWN"),
        Signal::Short => matches!(mic.agreement, "RECLAIM RISK" | "RANGE/FAKEOUTS" | "CONTINUATION UP"),
        _ => false,
    };
    if bad { return true; }
    if pos.initial_regime == "TRENDING"
        && (mic.trend_regime == "CHOPPY" || mic.trend_regime == "DRIFT/FLAT") {
        return true;
    }
    false
}

// HL fee schedule.
const MAKER_FEE: f64 = 0.00035;  // 0.035% — limit orders (entry + TP exit)
const TAKER_FEE: f64 = 0.001;    // 0.10%  — market orders (SL, trail, signal exits)
const FUNDING_RATE: f64 = 0.0001; // 0.01%  — default hourly funding rate
                                   // Longs pay when positive, shorts pay when negative.
                                   // Conservative estimate; actual rate varies.

/// Charge funding when position crosses an hourly boundary.
/// Returns the funding cost (positive = cost to position holder).
fn charge_funding(pos: &mut Position, current_ts: u64, current_price: f64) -> f64 {
    let current_hour = current_ts - (current_ts % 3_600_000);
    if current_hour <= pos.last_funding_hour {
        return 0.0;
    }
    // Count how many hourly settlements we crossed.
    let hours_crossed = (current_hour - pos.last_funding_hour) / 3_600_000;
    pos.last_funding_hour = current_hour;

    let notional = current_price * pos.size_asset;
    // Longs pay positive funding, shorts receive it (and vice versa).
    // Using a fixed conservative rate as we don't have historical funding per candle.
    let cost_per_hour = match pos.signal {
        Signal::Long => notional * FUNDING_RATE,   // longs pay when rate > 0
        Signal::Short => -notional * FUNDING_RATE,  // shorts receive when rate > 0
        Signal::Flat => 0.0,
    };
    let total = cost_per_hour * hours_crossed as f64;
    pos.funding_paid += total;
    total
}

fn compute_pnl(pos: &Position, exit_price: f64, exit_reason: ExitReason) -> f64 {
    let raw = match pos.signal {
        Signal::Long => (exit_price - pos.entry) * pos.size_asset,
        Signal::Short => (pos.entry - exit_price) * pos.size_asset,
        Signal::Flat => 0.0,
    };
    // Entry is always maker (limit order). Exit depends on reason.
    let entry_fee = pos.entry * pos.size_asset * MAKER_FEE;
    let exit_fee_rate = match exit_reason {
        ExitReason::TpHit => MAKER_FEE, // TP can be limit
        _ => TAKER_FEE,                  // SL, trail, signal exits are market
    };
    let exit_fee = exit_price * pos.size_asset * exit_fee_rate;
    // Funding already deducted from equity during hold; include in P&L record.
    raw - entry_fee - exit_fee - pos.funding_paid
}

fn open_position(setup: TradeSetup, ts: u64, regime: &str) -> Position {
    let hour_ms = ts - (ts % 3_600_000);
    Position {
        signal: setup.signal,
        conviction: setup.conviction,
        strategy: setup.strategy,
        entry: setup.entry,
        sl: setup.sl,
        tp: setup.tp,
        trail_stop: 0.0,
        size_asset: setup.size_asset,
        entry_time: ts,
        funding_paid: 0.0,
        last_funding_hour: hour_ms,
        high_water: setup.entry,
        low_water: setup.entry,
        vwap_cross_count: 0,
        initial_regime: regime.to_string(),
    }
}

fn make_record(pos: &Position, exit_price: f64, exit_time: u64, pnl_usd: f64, reason: ExitReason) -> TradeRecord {
    let pnl_pct = match pos.signal {
        Signal::Long => (exit_price - pos.entry) / pos.entry,
        Signal::Short => (pos.entry - exit_price) / pos.entry,
        Signal::Flat => 0.0,
    };
    TradeRecord {
        signal: pos.signal, conviction: pos.conviction, strategy: pos.strategy,
        entry: pos.entry, exit: exit_price, entry_time: pos.entry_time,
        exit_time, pnl_usd, pnl_pct, exit_reason: reason,
    }
}

fn day_start_idx(candles: &[Candle], day_ms: u64, current: usize) -> usize {
    let mut start = current;
    while start > 0 && candles[start - 1].t >= day_ms { start -= 1; }
    start
}

fn closed_before(candles: &[Candle], ts: u64) -> &[Candle] {
    let idx = candles.partition_point(|c| c.t_close < ts);
    &candles[..idx]
}
