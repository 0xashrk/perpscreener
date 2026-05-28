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
use engine::{run, ObRecord, TradeRecord};
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

    /// Export PnL breakdown CSVs to this directory
    #[arg(long)]
    csv_dir: Option<String>,
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

    // Try loading micro candles from local CSV first (0xArchive export).
    let csv_path = format!("data/candles/{}_15m.csv", args.coin.to_lowercase());
    let candles_1m = if candle_minutes == 15 && std::path::Path::new(&csv_path).exists() {
        eprintln!("  Loading 15m candles from {}...", csv_path);
        load_candles_csv(&csv_path)?
    } else {
        eprintln!("  {} candles (fetching from HL)...", args.micro_interval);
        fetch_all_candles(&client, &args.coin, &args.micro_interval, start_ms, now_ms)
            .await
            .context("micro candle fetch failed")?
    };

    eprintln!(
        "Data: {} 4h, {} 1h, {} 1m candles\n",
        candles_4h.len(),
        candles_1h.len(),
        candles_1m.len()
    );

    // Load OB data if available.
    let ob_path = format!("data/ob/{}_ob.csv", args.coin.to_lowercase());
    let ob_data = load_ob_csv(&ob_path);
    if ob_data.is_empty() {
        eprintln!("No OB data at {} — running without OB gates", ob_path);
    } else {
        eprintln!("Loaded {} OB snapshots from {}", ob_data.len(), ob_path);
    }

    eprintln!("Running backtest...");
    let result = run(
        &candles_1m,
        &candles_1h,
        &candles_4h,
        args.av,
        args.check_interval,
        args.cooldown,
        candle_minutes,
        &ob_data,
    );

    let start_dt = DateTime::<Utc>::from_timestamp_millis(start_ms as i64).unwrap();
    print!(
        "{}",
        format_report(&args.coin, start_dt, now, args.av, &result.trades, result.final_equity, result.max_drawdown_pct)
    );

    if let Some(csv_dir) = &args.csv_dir {
        export_pnl_csvs(&args.coin, &result.trades, csv_dir)?;
    }

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

fn load_candles_csv(path: &str) -> Result<Vec<data::Candle>> {
    let mut rdr = csv::Reader::from_path(path).context("open candle CSV")?;
    let mut candles = Vec::new();
    for result in rdr.records() {
        let row = result.context("read candle row")?;
        let ts: u64 = row.get(0).and_then(|v| v.parse().ok()).unwrap_or(0);
        let o: f64 = row.get(1).and_then(|v| v.parse().ok()).unwrap_or(0.0);
        let h: f64 = row.get(2).and_then(|v| v.parse().ok()).unwrap_or(0.0);
        let l: f64 = row.get(3).and_then(|v| v.parse().ok()).unwrap_or(0.0);
        let c: f64 = row.get(4).and_then(|v| v.parse().ok()).unwrap_or(0.0);
        let v: f64 = row.get(5).and_then(|v| v.parse().ok()).unwrap_or(0.0);
        if ts > 0 {
            candles.push(data::Candle {
                t: ts,
                t_close: ts + 15 * 60_000,
                o, h, l, c, v, n: 0,
            });
        }
    }
    candles.sort_by_key(|c| c.t);
    eprintln!("  Loaded {} candles from CSV", candles.len());
    Ok(candles)
}

fn export_pnl_csvs(coin: &str, trades: &[TradeRecord], dir: &str) -> Result<()> {
    use std::collections::BTreeMap;
    std::fs::create_dir_all(dir).context("create csv dir")?;
    let coin_lower = coin.to_lowercase();

    // Aggregate trades into daily, weekly, monthly buckets.
    let mut daily: BTreeMap<String, (usize, usize, f64)> = BTreeMap::new();
    let mut weekly: BTreeMap<String, (usize, usize, f64)> = BTreeMap::new();
    let mut monthly: BTreeMap<String, (usize, usize, f64)> = BTreeMap::new();

    for t in trades {
        let dt = DateTime::<Utc>::from_timestamp_millis(t.entry_time as i64).unwrap();
        let win = if t.pnl_usd > 0.0 { 1usize } else { 0 };

        // Daily: YYYY-MM-DD
        let day_key = dt.format("%Y-%m-%d").to_string();
        let d = daily.entry(day_key).or_insert((0, 0, 0.0));
        d.0 += 1;
        d.1 += win;
        d.2 += t.pnl_usd;

        // Weekly: ISO week YYYY-Www
        let week_key = dt.format("%G-W%V").to_string();
        let w = weekly.entry(week_key).or_insert((0, 0, 0.0));
        w.0 += 1;
        w.1 += win;
        w.2 += t.pnl_usd;

        // Monthly: YYYY-MM
        let month_key = dt.format("%Y-%m").to_string();
        let m = monthly.entry(month_key).or_insert((0, 0, 0.0));
        m.0 += 1;
        m.1 += win;
        m.2 += t.pnl_usd;
    }

    // Write daily CSV.
    let daily_path = format!("{}/{}_daily_pnl.csv", dir, coin_lower);
    let mut wtr = csv::Writer::from_path(&daily_path).context("create daily csv")?;
    wtr.write_record(["date", "trades", "wins", "win_pct", "pnl_usd", "cum_pnl_usd"])?;
    let mut cum = 0.0f64;
    for (date, (count, wins, pnl)) in &daily {
        cum += pnl;
        let wr = if *count > 0 { *wins as f64 / *count as f64 * 100.0 } else { 0.0 };
        wtr.write_record(&[
            date.clone(),
            count.to_string(),
            wins.to_string(),
            format!("{:.1}", wr),
            format!("{:.2}", pnl),
            format!("{:.2}", cum),
        ])?;
    }
    wtr.flush()?;

    // Write weekly CSV.
    let weekly_path = format!("{}/{}_weekly_pnl.csv", dir, coin_lower);
    let mut wtr = csv::Writer::from_path(&weekly_path).context("create weekly csv")?;
    wtr.write_record(["week", "trades", "wins", "win_pct", "pnl_usd", "cum_pnl_usd"])?;
    cum = 0.0;
    for (week, (count, wins, pnl)) in &weekly {
        cum += pnl;
        let wr = if *count > 0 { *wins as f64 / *count as f64 * 100.0 } else { 0.0 };
        wtr.write_record(&[
            week.clone(),
            count.to_string(),
            wins.to_string(),
            format!("{:.1}", wr),
            format!("{:.2}", pnl),
            format!("{:.2}", cum),
        ])?;
    }
    wtr.flush()?;

    // Write monthly CSV.
    let monthly_path = format!("{}/{}_monthly_pnl.csv", dir, coin_lower);
    let mut wtr = csv::Writer::from_path(&monthly_path).context("create monthly csv")?;
    wtr.write_record(["month", "trades", "wins", "win_pct", "pnl_usd", "cum_pnl_usd"])?;
    cum = 0.0;
    for (month, (count, wins, pnl)) in &monthly {
        cum += pnl;
        let wr = if *count > 0 { *wins as f64 / *count as f64 * 100.0 } else { 0.0 };
        wtr.write_record(&[
            month.clone(),
            count.to_string(),
            wins.to_string(),
            format!("{:.1}", wr),
            format!("{:.2}", pnl),
            format!("{:.2}", cum),
        ])?;
    }
    wtr.flush()?;

    eprintln!("Exported: {}, {}, {}", daily_path, weekly_path, monthly_path);
    Ok(())
}

fn load_ob_csv(path: &str) -> Vec<ObRecord> {
    let Ok(mut rdr) = csv::Reader::from_path(path) else {
        return Vec::new();
    };
    let mut records = Vec::new();
    for result in rdr.records() {
        let Ok(row) = result else { continue };
        let ts: u64 = row.get(0).and_then(|v| v.parse().ok()).unwrap_or(0);
        let imb: f64 = row.get(1).and_then(|v| v.parse().ok()).unwrap_or(1.0);
        let spread: f64 = row.get(2).and_then(|v| v.parse().ok()).unwrap_or(0.0);
        if ts > 0 {
            records.push(ObRecord { timestamp_ms: ts, ob_imbalance: imb, spread_pct: spread });
        }
    }
    records.sort_by_key(|r| r.timestamp_ms);
    records
}
