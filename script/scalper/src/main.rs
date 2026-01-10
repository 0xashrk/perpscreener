mod db;
mod models;
mod trading;

use crate::db::Db;
use crate::models::{
    build_signal_reason, cooldown_elapsed, current_ts_millis, current_ts_seconds,
    describe_momentum, signal_label, RecipeConfig, SignalRecord, TraderState, DB_ERROR_LIMIT,
    FETCH_FAILURE_LIMIT, HEALTH_LOG_SECS, MAX_SPREAD, MIN_OB_LONG,
};
use crate::trading::{close_position, evaluate_entry, evaluate_exit, log_health, unrealized_pnl};
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use humantime::parse_duration;
use reqwest::Client;
use serde::Deserialize;
use std::pin::Pin;
use std::time::{Duration, Instant};
use tokio::select;
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(name = "scalper", about = "Paper trade the SCALPER recipe")]
struct Args {
    /// Asset symbol (e.g., BTC, ETH)
    #[arg(long)]
    coin: String,

    /// Starting capital in USD
    #[arg(long, default_value = "100")]
    capital: f64,

    /// Run duration (e.g., 24h, 7d); omit for unlimited
    #[arg(long)]
    duration: Option<String>,

    /// Backend base URL
    #[arg(long, default_value = "http://localhost:30001")]
    backend: String,

    /// SQLite database path
    #[arg(long, default_value = "data/scalper.db")]
    db: String,

    /// Poll interval in seconds
    #[arg(long, default_value = "60")]
    interval: u64,
}

impl Args {
    fn duration_duration(&self) -> Result<Option<Duration>> {
        match &self.duration {
            Some(raw) => Ok(Some(
                parse_duration(raw).context("failed to parse --duration")?,
            )),
            None => Ok(None),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ChartSnapshot {
    candles: Vec<Candle>,
}

#[derive(Debug, Deserialize, Clone)]
struct Candle {
    #[serde(rename = "c")]
    close: f64,
    #[serde(rename = "T")]
    #[allow(dead_code)]
    close_time: i64,
}

#[derive(Debug, Deserialize)]
struct L2BookSnapshot {
    levels: (Vec<L2BookLevel>, Vec<L2BookLevel>),
}

#[derive(Debug, Deserialize)]
struct L2BookLevel {
    px: String,
    sz: String,
}

struct BackendClient {
    base_url: String,
    client: Client,
}

impl BackendClient {
    fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: Client::new(),
        }
    }

    async fn fetch_candles(&self, coin: &str) -> Result<Vec<Candle>> {
        let url = format!("{}/chart?coin={}&interval=1m&limit=10", self.base_url, coin);

        let snapshot: ChartSnapshot = self
            .client
            .get(&url)
            .send()
            .await
            .context("failed to fetch candles")?
            .error_for_status()
            .context("chart request failed")?
            .json()
            .await
            .context("failed to parse candles response")?;

        if snapshot.candles.len() < 2 {
            return Err(anyhow!(
                "expected at least 2 candles, got {}",
                snapshot.candles.len()
            ));
        }

        Ok(snapshot.candles)
    }

    async fn fetch_orderbook(&self, coin: &str) -> Result<L2BookSnapshot> {
        let url = format!("{}/orderbook?coin={}", self.base_url, coin);

        let snapshot: L2BookSnapshot = self
            .client
            .get(&url)
            .send()
            .await
            .context("failed to fetch orderbook")?
            .error_for_status()
            .context("orderbook request failed")?
            .json()
            .await
            .context("failed to parse orderbook response")?;

        Ok(snapshot)
    }
}

fn calc_orderbook(book: &L2BookSnapshot) -> Option<models::OrderbookStats> {
    let (bids, asks) = &book.levels;
    let best_bid: f64 = bids.get(0)?.px.parse().ok()?;
    let best_ask: f64 = asks.get(0)?.px.parse().ok()?;
    let bid_qty_sum: f64 = bids
        .iter()
        .take(5)
        .filter_map(|l| l.sz.parse::<f64>().ok())
        .sum();
    let ask_qty_sum: f64 = asks
        .iter()
        .take(5)
        .filter_map(|l| l.sz.parse::<f64>().ok())
        .sum();
    let mid = (best_bid + best_ask) / 2.0;
    if mid <= 0.0 {
        return None;
    }
    let spread = (best_ask - best_bid) / mid;
    let ob_imb = if ask_qty_sum > 0.0 {
        bid_qty_sum / ask_qty_sum
    } else {
        1.0
    };
    Some(models::OrderbookStats {
        bid: best_bid,
        ask: best_ask,
        mid,
        spread,
        ob_imbalance: ob_imb,
    })
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    let duration = args.duration_duration()?;
    let recipe = RecipeConfig {
        risk_pct: models::RISK_PCT,
        stop_pct: models::STOP_PCT,
        tp_pct: models::TP_PCT,
        max_hold_ms: models::MAX_HOLD_MS,
        cooldown_ms: models::COOLDOWN_MS,
        min_ob: MIN_OB_LONG,
        max_spread: MAX_SPREAD,
    };

    let backend = BackendClient::new(&args.backend);
    let mut db = Db::new(&args.db)?;

    let resume = db.load_resume_state()?;
    let (run_key, mut state) = if let Some(resume_state) = resume {
        info!(
            "resuming run {} with open position and capital {:.2}",
            resume_state.run_key, resume_state.capital
        );
        (
            resume_state.run_key,
            TraderState {
                capital: resume_state.capital,
                realized_pnl: resume_state.realized_pnl,
                total_trades: resume_state.total_trades,
                win_count: resume_state.win_count,
                loss_count: resume_state.loss_count,
                position: Some(resume_state.position),
                last_trade_exit_ts: resume_state.last_trade_exit_ts,
                last_mid: None,
                fetch_failures: 0,
                db_error_streak: 0,
                trading_paused: false,
                last_health_log: Instant::now(),
                run_key: resume_state.run_key,
                initial_capital: resume_state.initial_capital,
            },
        )
    } else {
        let run_key = current_ts_seconds();
        db.insert_run(
            run_key,
            &args.coin,
            args.capital,
            &args.backend,
            args.interval,
            duration.map(|d| d.as_secs()),
            &recipe,
        )?;
        (
            run_key,
            TraderState {
                capital: args.capital,
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
                run_key,
                initial_capital: args.capital,
            },
        )
    };

    let mut ticker = tokio::time::interval(Duration::from_secs(args.interval));
    let mut shutdown = Box::pin(tokio::signal::ctrl_c());
    let mut duration_sleep: Option<Pin<Box<tokio::time::Sleep>>> =
        duration.map(|d| Box::pin(tokio::time::sleep(d)));

    loop {
        select! {
            _ = ticker.tick() => {
                if let Err(err) = poll_cycle(&backend, &mut db, &args, &mut state).await {
                    error!("cycle error: {err:?}");
                }
                if state.last_health_log.elapsed() >= Duration::from_secs(HEALTH_LOG_SECS) {
                    log_health(&state);
                    state.last_health_log = Instant::now();
                }
            }
            _ = shutdown.as_mut() => {
                info!("received shutdown signal");
                break;
            }
            _ = async {
                if let Some(sleep) = duration_sleep.as_mut() {
                    sleep.as_mut().await;
                }
            }, if duration_sleep.is_some() => {
                info!("duration reached, stopping");
                break;
            }
        }
    }

    if let Some(position) = state.position.take() {
        let exit_px = state.last_mid.unwrap_or(position.entry_px);
        if let Err(err) = close_position(&mut db, &mut state, &position, exit_px, "SHUTDOWN") {
            error!("failed to close position on shutdown: {err:?}");
        }
    }

    db.update_run_final(run_key, state.capital)?;
    info!(
        "run complete | start_capital={:.2} capital={:.2} realized={:.4} total_trades={} wins={} losses={}",
        state.initial_capital,
        state.capital,
        state.realized_pnl,
        state.total_trades,
        state.win_count,
        state.loss_count
    );

    Ok(())
}

async fn poll_cycle(
    backend: &BackendClient,
    db: &mut Db,
    args: &Args,
    state: &mut TraderState,
) -> Result<()> {
    let now_ms = current_ts_millis();

    let data_result = tokio::try_join!(
        backend.fetch_candles(&args.coin),
        backend.fetch_orderbook(&args.coin)
    );
    let (candles, orderbook) = match data_result {
        Ok(data) => {
            state.fetch_failures = 0;
            data
        }
        Err(err) => {
            state.fetch_failures += 1;
            warn!(
                "data fetch failed (streak {}): {err:?}",
                state.fetch_failures
            );
            if state.fetch_failures >= FETCH_FAILURE_LIMIT {
                if let Some(position) = state.position.clone() {
                    let exit_px = state.last_mid.unwrap_or(position.entry_px);
                    if let Err(close_err) =
                        close_position(db, state, &position, exit_px, "DATA_OUTAGE")
                    {
                        state.db_error_streak += 1;
                        error!("failed to close on data outage: {close_err:?}");
                        if state.db_error_streak >= DB_ERROR_LIMIT {
                            state.trading_paused = true;
                            warn!("trading paused due to DB errors");
                        }
                    } else {
                        state.db_error_streak = 0;
                        state.trading_paused = false;
                    }
                }
            }
            return Ok(());
        }
    };

    let (last_close, prev_close) = (
        candles.last().map(|c| c.close).unwrap_or(0.0),
        candles.iter().rev().nth(1).map(|c| c.close).unwrap_or(0.0),
    );

    let ob_stats = match calc_orderbook(&orderbook) {
        Some(v) => v,
        None => {
            warn!("orderbook missing usable levels");
            return Ok(());
        }
    };

    state.last_mid = Some(ob_stats.mid);

    if let Some(position) = state.position.clone() {
        if let Some(exit_reason) = evaluate_exit(&position, ob_stats.mid, now_ms) {
            if let Err(close_err) = close_position(db, state, &position, ob_stats.mid, &exit_reason)
            {
                state.db_error_streak += 1;
                error!("failed to close position: {close_err:?}");
                if state.db_error_streak >= DB_ERROR_LIMIT {
                    state.trading_paused = true;
                    warn!("trading paused due to DB errors");
                }
            } else {
                state.db_error_streak = 0;
                state.trading_paused = false;
            }
        }
    }

    if state.position.is_none()
        && !state.trading_paused
        && cooldown_elapsed(state.last_trade_exit_ts, now_ms)
    {
        if let Some(draft) = evaluate_entry(state, &ob_stats, last_close, prev_close, now_ms) {
            match db.insert_trade_entry(state.run_key, &args.coin, &draft) {
                Ok(trade_id) => {
                    state.db_error_streak = 0;
                    state.trading_paused = false;
                    state.position = Some(models::Position {
                        db_id: trade_id,
                        direction: draft.direction,
                        entry_ts: draft.entry_ts,
                        entry_px: draft.entry_px,
                        size_coins: draft.size_coins,
                        notional: draft.notional,
                        entry_fee: draft.entry_fee,
                    });
                    info!(
                        "opened {} @ {:.2} size {:.6} notional {:.2}",
                        draft.direction.as_str(),
                        draft.entry_px,
                        draft.size_coins,
                        draft.notional
                    );
                }
                Err(err) => {
                    state.db_error_streak += 1;
                    error!("failed to record entry: {err:?}");
                    if state.db_error_streak >= DB_ERROR_LIMIT {
                        state.trading_paused = true;
                        warn!("trading paused due to DB errors");
                    }
                }
            }
        }
    }

    let signal = SignalRecord {
        run_key: state.run_key,
        ts: now_ms,
        coin: args.coin.clone(),
        price: ob_stats.mid,
        bid: ob_stats.bid,
        ask: ob_stats.ask,
        spread: ob_stats.spread,
        ob_imbalance: ob_stats.ob_imbalance,
        last_close,
        prev_close,
        momentum: describe_momentum(last_close, prev_close).to_string(),
        signal: signal_label(
            ob_stats.spread,
            ob_stats.ob_imbalance,
            last_close,
            prev_close,
        )
        .to_string(),
        reason: build_signal_reason(
            ob_stats.mid,
            ob_stats.spread,
            ob_stats.ob_imbalance,
            last_close,
            prev_close,
        ),
        position_open: state.position.is_some(),
    };
    match db.log_signal(&signal) {
        Ok(_) => {
            state.db_error_streak = 0;
            state.trading_paused = false;
        }
        Err(err) => {
            state.db_error_streak += 1;
            error!("failed to log signal: {err:?}");
            if state.db_error_streak >= DB_ERROR_LIMIT {
                state.trading_paused = true;
                warn!("trading paused due to DB errors");
            }
        }
    }

    let equity = models::EquitySnapshot {
        run_key: state.run_key,
        ts: now_ms,
        capital: state.capital,
        unrealized_pnl: unrealized_pnl(state.position.as_ref(), ob_stats.mid),
        realized_pnl: state.realized_pnl,
        total_trades: state.total_trades,
        win_count: state.win_count,
        loss_count: state.loss_count,
    };

    if let Err(err) = db.insert_equity(&equity) {
        state.db_error_streak += 1;
        error!("failed to log equity: {err:?}");
        if state.db_error_streak >= DB_ERROR_LIMIT {
            state.trading_paused = true;
            warn!("trading paused due to DB errors");
        }
    } else {
        state.db_error_streak = 0;
        state.trading_paused = false;
    }

    Ok(())
}
