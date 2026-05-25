mod analysis;
mod client;

use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use chrono::Utc;
use clap::Parser;
use reqwest::Client;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use analysis::{compute_momentum, floor_to_hour, format_pct, MomentumResult};
use client::{fetch_hl_candles, fetch_top_assets, BackendClient};

#[derive(Parser, Debug)]
#[command(name = "momentum", about = "Intrahour momentum scanner")]
struct Args {
    /// Asset symbol (e.g., BTC, ETH). Ignored when --top is used.
    #[arg(long, required_unless_present = "top")]
    coin: Option<String>,

    /// Backend base URL
    #[arg(long, default_value = "http://localhost:30001")]
    backend: String,

    /// Fetch directly from Hyperliquid instead of backend
    #[arg(long, default_value_t = false)]
    use_hl: bool,

    /// Number of 1m candles to pull (must cover current hour)
    #[arg(long, default_value_t = 180)]
    limit: usize,

    /// Scan the top N assets by 24h volume from Hyperliquid
    #[arg(long, conflicts_with = "coin")]
    top: Option<usize>,
}

fn format_price(p: f64) -> String {
    if p >= 1000.0 {
        format!("{:.1}", p)
    } else if p >= 1.0 {
        format!("{:.3}", p)
    } else if p >= 0.01 {
        format!("{:.5}", p)
    } else {
        format!("{:.7}", p)
    }
}

fn format_single(r: &MomentumResult) -> String {
    let mut out = String::new();

    let agreement_detail = match r.agreement {
        "PULLBACK RISK" => "PULLBACK RISK (up hour, down micro)",
        "RECLAIM RISK" => "RECLAIM RISK (down hour, up micro)",
        "RANGE/FAKEOUTS" => "RANGE/FAKEOUTS LIKELY",
        other => other,
    };

    writeln!(
        &mut out,
        "Vs hour-open: {} by {:.4} ({})",
        r.direction_vs_open.as_str(),
        r.delta_price,
        format_pct(r.delta_pct)
    )
    .ok();
    writeln!(
        &mut out,
        "Trend: 5m={} ({}), 15m={} ({}) -> {} strength={}/100",
        r.trend_5m.as_str(),
        r.ret_5m
            .map(format_pct)
            .unwrap_or_else(|| "n/a".to_string()),
        r.trend_15m.as_str(),
        r.ret_15m
            .map(format_pct)
            .unwrap_or_else(|| "n/a".to_string()),
        r.trend_regime,
        r.strength
    )
    .ok();
    match r.target_band {
        Some((lo, hi)) => writeln!(&mut out, "Target band (5-15m): {:.4} to {:.4}", lo, hi),
        None => writeln!(&mut out, "Target band (5-15m): n/a"),
    }
    .ok();
    writeln!(&mut out, "Agreement signal: {}", agreement_detail).ok();
    writeln!(&mut out).ok();

    writeln!(&mut out, "| Field | Value |").ok();
    writeln!(&mut out, "|---|---|").ok();
    writeln!(&mut out, "| start_time_utc | {} |", r.start_time.to_rfc3339()).ok();
    writeln!(&mut out, "| now_utc | {} |", r.now.to_rfc3339()).ok();
    writeln!(
        &mut out,
        "| price_to_beat (open @ start) | {:.4} |",
        r.price_to_beat
    )
    .ok();
    writeln!(&mut out, "| current_price | {:.4} |", r.current_price).ok();
    writeln!(
        &mut out,
        "| direction_vs_open | {} |",
        r.direction_vs_open.as_str()
    )
    .ok();
    writeln!(&mut out, "| delta_price | {:.4} |", r.delta_price).ok();
    writeln!(&mut out, "| delta_pct | {} |", format_pct(r.delta_pct)).ok();
    writeln!(
        &mut out,
        "| ret_5m | {} |",
        r.ret_5m
            .map(format_pct)
            .unwrap_or_else(|| "n/a".to_string())
    )
    .ok();
    writeln!(&mut out, "| trend_5m | {} |", r.trend_5m.as_str()).ok();
    writeln!(
        &mut out,
        "| ret_15m | {} |",
        r.ret_15m
            .map(format_pct)
            .unwrap_or_else(|| "n/a".to_string())
    )
    .ok();
    writeln!(&mut out, "| trend_15m | {} |", r.trend_15m.as_str()).ok();
    writeln!(&mut out, "| trend_regime | {} |", r.trend_regime).ok();
    writeln!(&mut out, "| trend_strength (0..100) | {} |", r.strength).ok();
    match r.target_band {
        Some((lo, hi)) => {
            writeln!(&mut out, "| target_band (5-15m) | [{:.4}, {:.4}] |", lo, hi)
        }
        None => writeln!(&mut out, "| target_band (5-15m) | n/a |"),
    }
    .ok();
    writeln!(
        &mut out,
        "| current_streak | {}x{} |",
        r.streaks.current.0.as_str(),
        r.streaks.current.1
    )
    .ok();
    writeln!(&mut out, "| longest_up_streak | {} |", r.streaks.longest_up).ok();
    writeln!(
        &mut out,
        "| longest_down_streak | {} |",
        r.streaks.longest_down
    )
    .ok();
    writeln!(
        &mut out,
        "| vol_1m | {} |",
        r.vol_1m
            .map(|v| format!("{:.6}", v))
            .unwrap_or_else(|| "n/a".to_string())
    )
    .ok();
    writeln!(&mut out, "| window_high | {:.4} |", r.window_high).ok();
    writeln!(&mut out, "| window_low | {:.4} |", r.window_low).ok();
    writeln!(&mut out, "| resistance | {:.4} |", r.window_high).ok();
    writeln!(&mut out, "| support | {:.4} |", r.window_low).ok();
    writeln!(&mut out, "| range_pct | {} |", format_pct(r.range_pct)).ok();
    writeln!(&mut out, "| data_quality | {} |", r.data_quality).ok();

    out
}

fn format_multi(results: &[MomentumResult]) -> String {
    let mut out = String::new();

    let header = results
        .first()
        .map(|r| r.start_time.format("%Y-%m-%d %H:00 UTC").to_string())
        .unwrap_or_default();
    writeln!(
        &mut out,
        "Intrahour Momentum Scanner — {} ({} assets)\n",
        header,
        results.len()
    )
    .ok();

    writeln!(
        &mut out,
        "| # | Coin | Price | vs Open | Delta% | 5m | 15m | Regime | Str | Streak | Vol | Signal |"
    )
    .ok();
    writeln!(
        &mut out,
        "|---|------|-------|---------|--------|----|-----|--------|-----|--------|-----|--------|"
    )
    .ok();

    for (i, r) in results.iter().enumerate() {
        writeln!(
            &mut out,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {}x{} | {} | {} |",
            i + 1,
            r.coin,
            format_price(r.current_price),
            r.direction_vs_open.as_str(),
            format_pct(r.delta_pct),
            r.trend_5m.as_str(),
            r.trend_15m.as_str(),
            r.trend_regime,
            r.strength,
            r.streaks.current.0.as_str(),
            r.streaks.current.1,
            r.vol_1m
                .map(|v| format!("{:.6}", v))
                .unwrap_or_else(|| "n/a".to_string()),
            r.agreement,
        )
        .ok();
    }

    out
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let now = Utc::now();
    let start_time = floor_to_hour(now);
    let start_ms =
        u64::try_from(start_time.timestamp_millis()).context("start time millis negative")?;
    let now_ms = u64::try_from(now.timestamp_millis()).context("now millis negative")?;

    let use_hl = args.use_hl || args.top.is_some();

    let coins: Vec<String> = if let Some(n) = args.top {
        let client = Client::new();
        eprintln!("Fetching top {} assets by 24h volume...", n);
        fetch_top_assets(&client, n).await?
    } else {
        vec![args.coin.unwrap_or_else(|| "BTC".to_string())]
    };

    let multi = coins.len() > 1;

    let (mut results, errors) = if use_hl {
        scan_hl(&coins, start_ms, now_ms, now, start_time).await
    } else {
        let backend = BackendClient::new(&args.backend);
        let coin = &coins[0];
        let candles = backend.fetch_candles(coin, args.limit).await?;
        match compute_momentum(coin, candles, now, start_time) {
            Ok(r) => (vec![r], vec![]),
            Err(e) => (vec![], vec![(coin.clone(), e.to_string())]),
        }
    };

    // Preserve original volume ranking order.
    results.sort_by_key(|r| coins.iter().position(|c| c == &r.coin).unwrap_or(usize::MAX));

    let out = if multi {
        format_multi(&results)
    } else if let Some(r) = results.first() {
        format_single(r)
    } else {
        "No results.\n".to_string()
    };

    print!("{}", out);

    if !errors.is_empty() && multi {
        eprintln!("\nSkipped {} assets (insufficient data):", errors.len());
        for (coin, err) in &errors {
            eprintln!("  {}: {}", coin, err);
        }
    }

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("momentum.txt");
    fs::write(&path, &out).with_context(|| format!("failed to write {}", path.display()))?;

    Ok(())
}

/// Fetch candles from Hyperliquid for all coins concurrently (max 10 in-flight).
async fn scan_hl(
    coins: &[String],
    start_ms: u64,
    now_ms: u64,
    now: chrono::DateTime<Utc>,
    start_time: chrono::DateTime<Utc>,
) -> (Vec<MomentumResult>, Vec<(String, String)>) {
    let hl_client = Client::new();
    let sem = Arc::new(Semaphore::new(10));
    let mut set = JoinSet::new();

    for coin in coins.iter().cloned() {
        let client = hl_client.clone();
        let sem = sem.clone();
        set.spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let candles = fetch_hl_candles(&client, &coin, start_ms, now_ms).await;
            (coin, candles)
        });
    }

    let mut results = Vec::new();
    let mut errors = Vec::new();

    while let Some(res) = set.join_next().await {
        match res {
            Ok((coin, Ok(candles))) => match compute_momentum(&coin, candles, now, start_time) {
                Ok(m) => results.push(m),
                Err(e) => errors.push((coin, e.to_string())),
            },
            Ok((coin, Err(e))) => errors.push((coin, e.to_string())),
            Err(e) => eprintln!("task panic: {}", e),
        }
    }

    (results, errors)
}
