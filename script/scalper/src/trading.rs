use crate::db::Db;
use crate::models::{
    current_ts_millis, Direction, OrderbookStats, Position, PositionDraft, TraderState,
    ENTRY_FEE_RATE, EXIT_FEE_RATE, MAX_SPREAD, MIN_OB_LONG, RISK_PCT, STOP_PCT, TP_PCT,
};
use anyhow::Result;
use tracing::info;

pub fn evaluate_entry(
    state: &TraderState,
    ob_stats: &OrderbookStats,
    last_close: f64,
    prev_close: f64,
    now_ms: i64,
) -> Option<PositionDraft> {
    let momentum_up = last_close > prev_close;
    let momentum_dn = last_close < prev_close;

    let long_signal =
        momentum_up && ob_stats.ob_imbalance >= MIN_OB_LONG && ob_stats.spread <= MAX_SPREAD;
    let short_signal = momentum_dn
        && ob_stats.ob_imbalance <= (1.0 / MIN_OB_LONG)
        && ob_stats.spread <= MAX_SPREAD;

    let direction = if long_signal {
        Direction::Long
    } else if short_signal {
        Direction::Short
    } else {
        return None;
    };

    let risk_usd = state.capital * RISK_PCT;
    let stop_dist = ob_stats.mid * STOP_PCT;
    if stop_dist <= 0.0 || risk_usd <= 0.0 {
        return None;
    }

    let size_coins = risk_usd / stop_dist;
    if size_coins <= 0.0 {
        return None;
    }

    let notional = size_coins * ob_stats.mid;
    let entry_fee = notional * ENTRY_FEE_RATE;

    Some(PositionDraft {
        direction,
        entry_ts: now_ms,
        entry_px: ob_stats.mid,
        size_coins,
        notional,
        entry_fee,
    })
}

pub fn evaluate_exit(position: &Position, current_price: f64, now_ms: i64) -> Option<String> {
    let price_change_pct = (current_price - position.entry_px) / position.entry_px;
    let pnl_pct = match position.direction {
        Direction::Long => price_change_pct,
        Direction::Short => -price_change_pct,
    };

    let hold_time = now_ms - position.entry_ts;

    if pnl_pct >= TP_PCT {
        Some("TP".to_string())
    } else if pnl_pct <= -STOP_PCT {
        Some("SL".to_string())
    } else if hold_time >= crate::models::MAX_HOLD_MS {
        Some("TIMEOUT".to_string())
    } else {
        None
    }
}

pub fn close_position(
    db: &mut Db,
    state: &mut TraderState,
    position: &Position,
    exit_px: f64,
    exit_reason: &str,
) -> Result<()> {
    let price_change_pct = (exit_px - position.entry_px) / position.entry_px;
    let pnl_pct = match position.direction {
        Direction::Long => price_change_pct,
        Direction::Short => -price_change_pct,
    };

    let exit_fee = position.notional * EXIT_FEE_RATE;
    let gross_pnl = position.notional * pnl_pct;
    let net_pnl = gross_pnl - position.entry_fee - exit_fee;

    db.update_trade_exit(
        position.db_id,
        current_ts_millis(),
        exit_px,
        gross_pnl,
        position.entry_fee,
        exit_fee,
        net_pnl,
        exit_reason,
    )?;

    state.capital += net_pnl;
    state.realized_pnl += net_pnl;
    state.total_trades += 1;
    if net_pnl >= 0.0 {
        state.win_count += 1;
    } else {
        state.loss_count += 1;
    }
    state.position = None;
    state.last_trade_exit_ts = Some(current_ts_millis());

    info!(
        "closed {} @ {:.2} size {:.6} reason={} net={:.4} capital={:.2}",
        position.direction.as_str(),
        exit_px,
        position.size_coins,
        exit_reason,
        net_pnl,
        state.capital
    );

    Ok(())
}

pub fn log_health(state: &TraderState) {
    info!(
        "health | run_key={} capital={:.2} realized={:.4} total_trades={} open={} fetch_failures={} db_errors={} paused={}",
        state.run_key,
        state.capital,
        state.realized_pnl,
        state.total_trades,
        state.position.is_some(),
        state.fetch_failures,
        state.db_error_streak,
        state.trading_paused
    );
}

pub fn unrealized_pnl(position: Option<&Position>, mid: f64) -> f64 {
    if let Some(pos) = position {
        let price_change_pct = (mid - pos.entry_px) / pos.entry_px;
        let pnl_pct = match pos.direction {
            Direction::Long => price_change_pct,
            Direction::Short => -price_change_pct,
        };
        pos.notional * pnl_pct
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;
    use crate::models::{OrderbookStats, TraderState};
    use std::time::Instant;
    use tempfile::tempdir;

    fn base_state() -> TraderState {
        TraderState {
            capital: 100.0,
            realized_pnl: 0.0,
            total_trades: 0,
            win_count: 0,
            loss_count: 0,
            position: None,
            last_trade_exit_ts: None,
            last_mid: None,
            fetch_failures: 0,
            db_error_streak: 0,
            trading_paused: false,
            last_health_log: Instant::now(),
            run_key: 123,
            initial_capital: 100.0,
        }
    }

    #[test]
    fn evaluate_entry_signals_long_and_short() {
        let now = 0;
        let ob_good = OrderbookStats {
            bid: 99.9,
            ask: 100.1,
            mid: 100.0,
            spread: 0.0004,
            ob_imbalance: 1.4,
        };
        let ob_short = OrderbookStats {
            bid: 99.9,
            ask: 100.1,
            mid: 100.0,
            spread: 0.0004,
            ob_imbalance: 0.7,
        };

        let mut state = base_state();
        let long_draft = evaluate_entry(&state, &ob_good, 101.0, 100.0, now).expect("long draft");
        assert!(matches!(long_draft.direction, Direction::Long));
        assert!((long_draft.size_coins - 3.3333333).abs() < 1e-4);

        let short_draft = evaluate_entry(&state, &ob_short, 99.0, 100.0, now).expect("short draft");
        assert!(matches!(short_draft.direction, Direction::Short));
        assert!(short_draft.notional > 0.0);

        let ob_wide_spread = OrderbookStats { spread: 0.002, ..ob_good };
        state.capital = 50.0;
        assert!(evaluate_entry(&state, &ob_wide_spread, 101.0, 100.0, now).is_none());
    }

    #[test]
    fn evaluate_exit_respects_tp_sl_timeout() {
        let position = Position {
            db_id: 1,
            direction: Direction::Long,
            entry_ts: 0,
            entry_px: 100.0,
            size_coins: 1.0,
            notional: 100.0,
            entry_fee: 0.0,
        };

        let tp = evaluate_exit(&position, 104.0, 1_000).expect("tp");
        assert_eq!(tp, "TP");

        let sl = evaluate_exit(&position, 98.0, 1_000).expect("sl");
        assert_eq!(sl, "SL");

        let timeout = evaluate_exit(&position, 100.0, position.entry_ts + crate::models::MAX_HOLD_MS + 1)
            .expect("timeout");
        assert_eq!(timeout, "TIMEOUT");

        let none = evaluate_exit(&position, 100.2, 10_000);
        assert!(none.is_none());
    }

    #[test]
    fn close_position_updates_state_and_clears_open_position() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let db_path = dir.path().join("scalper.db");
        let mut db = Db::new(db_path.to_str().expect("path"))?;

        let mut state = base_state();
        let draft = PositionDraft {
            direction: Direction::Long,
            entry_ts: 0,
            entry_px: 100.0,
            size_coins: 1.0,
            notional: 100.0,
            entry_fee: 100.0 * ENTRY_FEE_RATE,
        };
        let trade_id = db.insert_trade_entry(state.run_key, "BTC", &draft)?;
        let position = Position {
            db_id: trade_id,
            direction: draft.direction,
            entry_ts: draft.entry_ts,
            entry_px: draft.entry_px,
            size_coins: draft.size_coins,
            notional: draft.notional,
            entry_fee: draft.entry_fee,
        };
        state.position = Some(position.clone());

        close_position(&mut db, &mut state, &position, 104.0, "TP")?;

        assert!(state.position.is_none());
        assert!(state.capital > 100.0);
        assert_eq!(state.total_trades, 1);
        assert_eq!(state.win_count, 1);
        assert!(state.last_trade_exit_ts.is_some());
        Ok(())
    }

    #[test]
    fn unrealized_pnl_computes_for_open_position() {
        let position = Position {
            db_id: 1,
            direction: Direction::Short,
            entry_ts: 0,
            entry_px: 100.0,
            size_coins: 1.0,
            notional: 100.0,
            entry_fee: 0.0,
        };
        let pnl = unrealized_pnl(Some(&position), 95.0);
        assert!((pnl - 5.0).abs() < 1e-6);
        assert_eq!(unrealized_pnl(None, 95.0), 0.0);
    }
}
