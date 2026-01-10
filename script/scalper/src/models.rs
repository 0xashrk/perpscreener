use chrono::Utc;
use serde::Serialize;
use std::time::Instant;

pub const RISK_PCT: f64 = 0.005;
pub const STOP_PCT: f64 = 0.0015;
pub const TP_PCT: f64 = 0.004;
pub const MAX_HOLD_MS: i64 = 90_000;
pub const COOLDOWN_MS: i64 = 120_000;
pub const MIN_OB_LONG: f64 = 1.2;
pub const MAX_OB_SHORT: f64 = 1.0 / 1.2;
pub const MAX_SPREAD: f64 = 0.0005;
pub const ENTRY_FEE_RATE: f64 = 0.00015;
pub const EXIT_FEE_RATE: f64 = 0.00045;
pub const FETCH_FAILURE_LIMIT: u32 = 3;
pub const DB_ERROR_LIMIT: u32 = 3;
pub const HEALTH_LOG_SECS: u64 = 3600;
pub const CHECKPOINT_EVERY: usize = 10_000;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct RecipeConfig {
    pub risk_pct: f64,
    pub stop_pct: f64,
    pub tp_pct: f64,
    pub max_hold_ms: i64,
    pub cooldown_ms: i64,
    pub min_ob: f64,
    pub max_spread: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Long,
    Short,
}

impl Direction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Direction::Long => "LONG",
            Direction::Short => "SHORT",
        }
    }

    pub fn from_str(raw: &str) -> Option<Self> {
        match raw.to_uppercase().as_str() {
            "LONG" => Some(Direction::Long),
            "SHORT" => Some(Direction::Short),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Position {
    pub db_id: i64,
    pub direction: Direction,
    pub entry_ts: i64,
    pub entry_px: f64,
    pub size_coins: f64,
    pub notional: f64,
    pub entry_fee: f64,
}

#[derive(Debug)]
pub struct PositionDraft {
    pub direction: Direction,
    pub entry_ts: i64,
    pub entry_px: f64,
    pub size_coins: f64,
    pub notional: f64,
    pub entry_fee: f64,
}

#[derive(Debug)]
pub struct OrderbookStats {
    pub bid: f64,
    pub ask: f64,
    pub mid: f64,
    pub spread: f64,
    pub ob_imbalance: f64,
}

#[derive(Debug)]
pub struct SignalRecord {
    pub run_key: i64,
    pub ts: i64,
    pub coin: String,
    pub price: f64,
    pub bid: f64,
    pub ask: f64,
    pub spread: f64,
    pub ob_imbalance: f64,
    pub last_close: f64,
    pub prev_close: f64,
    pub momentum: String,
    pub signal: String,
    pub reason: String,
    pub position_open: bool,
}

#[derive(Debug)]
pub struct EquitySnapshot {
    pub run_key: i64,
    pub ts: i64,
    pub capital: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub total_trades: i64,
    pub win_count: i64,
    pub loss_count: i64,
}

#[derive(Debug)]
pub struct ResumeState {
    pub run_key: i64,
    pub position: Position,
    pub capital: f64,
    pub realized_pnl: f64,
    pub total_trades: i64,
    pub win_count: i64,
    pub loss_count: i64,
    pub last_trade_exit_ts: Option<i64>,
    pub initial_capital: f64,
}

#[derive(Debug)]
pub struct TraderState {
    pub capital: f64,
    pub realized_pnl: f64,
    pub total_trades: i64,
    pub win_count: i64,
    pub loss_count: i64,
    pub position: Option<Position>,
    pub last_trade_exit_ts: Option<i64>,
    pub last_mid: Option<f64>,
    pub fetch_failures: u32,
    pub db_error_streak: u32,
    pub trading_paused: bool,
    pub last_health_log: Instant,
    pub run_key: i64,
    pub initial_capital: f64,
}

pub fn current_ts_seconds() -> i64 {
    Utc::now().timestamp()
}

pub fn current_ts_millis() -> i64 {
    Utc::now().timestamp_millis()
}

pub fn describe_momentum(last_close: f64, prev_close: f64) -> &'static str {
    if last_close > prev_close {
        "up"
    } else if last_close < prev_close {
        "down"
    } else {
        "flat"
    }
}

pub fn signal_label(spread: f64, ob_imb: f64, last_close: f64, prev_close: f64) -> &'static str {
    let momentum_up = last_close > prev_close;
    let momentum_dn = last_close < prev_close;

    let long_signal = momentum_up && ob_imb >= MIN_OB_LONG && spread <= MAX_SPREAD;
    let short_signal = momentum_dn && ob_imb <= MAX_OB_SHORT && spread <= MAX_SPREAD;

    if long_signal {
        "LONG"
    } else if short_signal {
        "SHORT"
    } else {
        "NONE"
    }
}

pub fn build_signal_reason(
    mid: f64,
    spread: f64,
    ob_imb: f64,
    last_close: f64,
    prev_close: f64,
) -> String {
    let mut reasons = Vec::with_capacity(4);
    reasons.push(format!(
        "momentum:{}",
        describe_momentum(last_close, prev_close)
    ));
    reasons.push(format!("spread:{:.5}", spread));
    reasons.push(format!("ob_imb:{:.2}", ob_imb));
    reasons.push(format!("mid:{:.2}", mid));
    reasons.join(" | ")
}

pub fn cooldown_elapsed(last_trade_exit_ts: Option<i64>, now_ms: i64) -> bool {
    match last_trade_exit_ts {
        Some(ts) => now_ms - ts >= COOLDOWN_MS,
        None => true,
    }
}
