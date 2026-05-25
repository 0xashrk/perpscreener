mod data;
mod engine;
mod strategy;

use std::collections::HashMap;
use std::fmt::Write as FmtWrite;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::Parser;
use reqwest::Client;

use data::fetch_all_candles;
use engine::{run, TradeRecord};
use strategy::{Signal, Strategy};

#[derive(Parser, Debug)]
#[command(name = "strattest", about = "Backtest the signal strategy on historical data")]
struct Args {
    /// Asset symbol
    #[arg(long)]
    coin: String,

    /// Lookback in months
    #[arg(long, default_value_t = 6)]
    months: u32,

    /// Starting account value in USD
    #[arg(long)]
    av: f64,

    /// Signal check interval in minutes (default: 15)
    #[arg(long, default_value_t = 15)]
    check_interval: u64,

    /// Cooldown after exit in minutes (default: 60)
    #[arg(long, default_value_t = 60)]
    cooldown: u64,

    /// Micro candle interval: 1m, 5m, or 15m (default: 5m for long backtests)
    #[arg(long, default_value = "5m")]
    micro_interval: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = Client::new();
    let now = Utc::now();
    let now_ms = now.timestamp_millis() as u64;
    let start_ms = now_ms - (args.months as u64) * 30 * 24 * 3_600_000;

    // Need extra history for SMA50 on 4h (~200h before start).
    let start_4h = start_ms - 201 * 3_600_000;
    // Extra for Donchian/ATR on 1h (~5 days before start).
    let start_1h = start_ms - 5 * 24 * 3_600_000;

    eprintln!("Fetching {} data ({} months)...", args.coin, args.months);

    eprintln!("  4h candles...");
    let candles_4h = fetch_all_candles(&client, &args.coin, "4h", start_4h, now_ms)
        .await
        .context("4h fetch failed")?;
    eprintln!("  1h candles...");
    let candles_1h = fetch_all_candles(&client, &args.coin, "1h", start_1h, now_ms)
        .await
        .context("1h fetch failed")?;
    let candle_minutes: u64 = match args.micro_interval.as_str() {
        "1m" => 1,
        "5m" => 5,
        "15m" => 15,
        other => anyhow::bail!("unsupported micro interval: {}", other),
    };

    eprintln!("  {} candles (this may take a moment)...", args.micro_interval);
    let candles_1m = fetch_all_candles(&client, &args.coin, &args.micro_interval, start_ms, now_ms)
        .await
        .context("micro candle fetch failed")?;

    eprintln!(
        "Data: {} 4h, {} 1h, {} 1m candles\n",
        candles_4h.len(),
        candles_1h.len(),
        candles_1m.len()
    );

    eprintln!("Running backtest...");
    let result = run(
        &candles_1m,
        &candles_1h,
        &candles_4h,
        args.av,
        args.check_interval,
        args.cooldown,
        candle_minutes,
    );

    let start_dt = DateTime::from_timestamp_millis(start_ms as i64).unwrap();
    print!(
        "{}",
        format_report(&args.coin, start_dt, now, args.av, &result.trades, result.final_equity, result.max_drawdown_pct)
    );

    Ok(())
}

// -- Formatting --------------------------------------------------------------

fn format_report(
    coin: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    initial_av: f64,
    trades: &[TradeRecord],
    final_equity: f64,
    max_dd: f64,
) -> String {
    let mut out = String::new();

    writeln!(&mut out, "=== {} Backtest ===\n", coin).ok();
    writeln!(
        &mut out,
        "Period:    {} to {}",
        start.format("%Y-%m-%d"),
        end.format("%Y-%m-%d")
    )
    .ok();
    writeln!(&mut out, "Account:   ${:.0}", initial_av).ok();
    writeln!(
        &mut out,
        "Strategy:  Signal (macro SMA/Donch/ATR + micro momentum)\n"
    )
    .ok();

    if trades.is_empty() {
        writeln!(&mut out, "No trades generated.").ok();
        return out;
    }

    let total = trades.len();
    let wins: Vec<&TradeRecord> = trades.iter().filter(|t| t.pnl_usd > 0.0).collect();
    let losses: Vec<&TradeRecord> = trades.iter().filter(|t| t.pnl_usd <= 0.0).collect();
    let win_count = wins.len();
    let loss_count = losses.len();
    let win_rate = win_count as f64 / total as f64 * 100.0;

    let total_pnl: f64 = trades.iter().map(|t| t.pnl_usd).sum();
    let total_pnl_pct = (final_equity - initial_av) / initial_av * 100.0;

    let avg_win = if wins.is_empty() {
        0.0
    } else {
        wins.iter().map(|t| t.pnl_pct).sum::<f64>() / wins.len() as f64 * 100.0
    };
    let avg_loss = if losses.is_empty() {
        0.0
    } else {
        losses.iter().map(|t| t.pnl_pct).sum::<f64>() / losses.len() as f64 * 100.0
    };
    let max_win = trades
        .iter()
        .map(|t| t.pnl_pct)
        .fold(f64::MIN, f64::max)
        * 100.0;
    let max_loss = trades
        .iter()
        .map(|t| t.pnl_pct)
        .fold(f64::MAX, f64::min)
        * 100.0;

    let gross_profit: f64 = wins.iter().map(|t| t.pnl_usd).sum();
    let gross_loss: f64 = losses.iter().map(|t| t.pnl_usd.abs()).sum();
    let profit_factor = if gross_loss > 0.0 {
        gross_profit / gross_loss
    } else {
        f64::INFINITY
    };

    let avg_hold: f64 = trades
        .iter()
        .map(|t| (t.exit_time - t.entry_time) as f64 / 60_000.0)
        .sum::<f64>()
        / total as f64;

    writeln!(&mut out, "--- Performance ---").ok();
    writeln!(
        &mut out,
        "Total trades:    {} ({}W / {}L)",
        total, win_count, loss_count
    )
    .ok();
    writeln!(&mut out, "Win rate:        {:.1}%", win_rate).ok();
    writeln!(
        &mut out,
        "Total P&L:       {:+.0} ({:+.1}%)",
        total_pnl, total_pnl_pct
    )
    .ok();
    writeln!(
        &mut out,
        "Avg win/loss:    {:+.2}% / {:.2}%",
        avg_win, avg_loss
    )
    .ok();
    writeln!(
        &mut out,
        "Best/worst:      {:+.2}% / {:.2}%",
        max_win, max_loss
    )
    .ok();
    writeln!(&mut out, "Profit factor:   {:.2}", profit_factor).ok();
    writeln!(&mut out, "Max drawdown:    {:.1}%", max_dd).ok();
    writeln!(&mut out, "Avg hold:        {:.0} min\n", avg_hold).ok();

    // Direction breakdown.
    let longs: Vec<&TradeRecord> = trades.iter().filter(|t| t.signal == Signal::Long).collect();
    let shorts: Vec<&TradeRecord> = trades
        .iter()
        .filter(|t| t.signal == Signal::Short)
        .collect();
    let long_wins = longs.iter().filter(|t| t.pnl_usd > 0.0).count();
    let short_wins = shorts.iter().filter(|t| t.pnl_usd > 0.0).count();

    writeln!(&mut out, "--- Breakdown ---").ok();
    if !longs.is_empty() {
        writeln!(
            &mut out,
            "Longs:   {} ({:.1}% win)",
            longs.len(),
            long_wins as f64 / longs.len() as f64 * 100.0
        )
        .ok();
    }
    if !shorts.is_empty() {
        writeln!(
            &mut out,
            "Shorts:  {} ({:.1}% win)",
            shorts.len(),
            short_wins as f64 / shorts.len() as f64 * 100.0
        )
        .ok();
    }

    // Strategy breakdown.
    let tf: Vec<&TradeRecord> = trades.iter().filter(|t| t.strategy == Strategy::TrendFollow).collect();
    let mr: Vec<&TradeRecord> = trades.iter().filter(|t| t.strategy == Strategy::MeanRevert).collect();
    if !tf.is_empty() {
        let w = tf.iter().filter(|t| t.pnl_usd > 0.0).count();
        let p: f64 = tf.iter().map(|t| t.pnl_usd).sum();
        writeln!(&mut out, "Trend-F: {} trades, {:.1}% win, {:+.0}", tf.len(), w as f64 / tf.len() as f64 * 100.0, p).ok();
    }
    if !mr.is_empty() {
        let w = mr.iter().filter(|t| t.pnl_usd > 0.0).count();
        let p: f64 = mr.iter().map(|t| t.pnl_usd).sum();
        writeln!(&mut out, "Mean-R:  {} trades, {:.1}% win, {:+.0}", mr.len(), w as f64 / mr.len() as f64 * 100.0, p).ok();
    }

    // Conviction breakdown.
    for conv in &["STRONG", "NORMAL", "WEAK", "MR"] {
        let subset: Vec<&TradeRecord> = trades
            .iter()
            .filter(|t| t.conviction.as_str() == *conv)
            .collect();
        if subset.is_empty() {
            continue;
        }
        let w = subset.iter().filter(|t| t.pnl_usd > 0.0).count();
        let pnl: f64 = subset.iter().map(|t| t.pnl_usd).sum();
        writeln!(
            &mut out,
            "{:<8} {} trades, {:.1}% win, {:+.0}",
            conv,
            subset.len(),
            w as f64 / subset.len() as f64 * 100.0,
            pnl
        )
        .ok();
    }
    writeln!(&mut out).ok();

    // Exit reason breakdown.
    writeln!(&mut out, "--- Exit Reasons ---").ok();
    let mut reason_counts: HashMap<&str, usize> = HashMap::new();
    for t in trades {
        *reason_counts.entry(t.exit_reason.as_str()).or_insert(0) += 1;
    }
    let mut reasons: Vec<(&&str, &usize)> = reason_counts.iter().collect();
    reasons.sort_by(|a, b| b.1.cmp(a.1));
    for (reason, count) in reasons {
        writeln!(&mut out, "  {:<14} {}", reason, count).ok();
    }
    writeln!(&mut out).ok();

    // Monthly breakdown.
    writeln!(&mut out, "--- Monthly ---").ok();
    writeln!(
        &mut out,
        "| Month   | Trades | Win% | P&L |"
    )
    .ok();
    writeln!(
        &mut out,
        "|---------|--------|------|-----|"
    )
    .ok();

    let mut monthly: HashMap<String, (usize, usize, f64)> = HashMap::new();
    for t in trades {
        let dt = DateTime::from_timestamp_millis(t.entry_time as i64).unwrap();
        let key = dt.format("%Y-%m").to_string();
        let entry = monthly.entry(key).or_insert((0, 0, 0.0));
        entry.0 += 1;
        if t.pnl_usd > 0.0 {
            entry.1 += 1;
        }
        entry.2 += t.pnl_usd;
    }
    let mut months: Vec<(String, (usize, usize, f64))> = monthly.into_iter().collect();
    months.sort_by(|a, b| a.0.cmp(&b.0));
    for (month, (count, wins, pnl)) in &months {
        let wr = if *count > 0 {
            *wins as f64 / *count as f64 * 100.0
        } else {
            0.0
        };
        writeln!(
            &mut out,
            "| {} | {:>6} | {:>3.0}% | {:>+7.0} |",
            month, count, wr, pnl
        )
        .ok();
    }

    out
}
