mod client;
mod micro;

use anyhow::{Context, Result};
use chrono::Utc;
use clap::Parser;
use reqwest::Client;

use client::fetch_candles;
use micro::{floor_to_hour, snapshot, MicroSnapshot};

#[derive(Parser, Debug)]
#[command(name = "monitor", about = "Watch an open position for signal changes")]
struct Args {
    /// Asset symbol
    #[arg(long)]
    coin: String,

    /// Entry price
    #[arg(long)]
    entry: f64,

    /// Position direction: long or short
    #[arg(long)]
    dir: String,

    /// TP price
    #[arg(long)]
    tp: f64,

    /// SL price
    #[arg(long)]
    sl: f64,

    /// Poll interval in minutes (default: 3)
    #[arg(long, default_value_t = 3)]
    interval: u64,

    /// Maximum runtime in minutes (default: 60)
    #[arg(long, default_value_t = 60)]
    max_minutes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PosDir {
    Long,
    Short,
}

impl PosDir {
    fn parse(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "long" | "l" => Ok(PosDir::Long),
            "short" | "s" => Ok(PosDir::Short),
            _ => anyhow::bail!("--dir must be 'long' or 'short'"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ExitReason {
    TpHit,
    SlHit,
    RegimeFlip,
    AgreementFlip,
    HourReset,
    Timeout,
}

impl ExitReason {
    fn label(&self) -> &'static str {
        match self {
            ExitReason::TpHit => "TP HIT",
            ExitReason::SlHit => "SL HIT",
            ExitReason::RegimeFlip => "REGIME FLIPPED",
            ExitReason::AgreementFlip => "AGREEMENT FLIPPED",
            ExitReason::HourReset => "HOUR RESET",
            ExitReason::Timeout => "MAX TIME REACHED",
        }
    }
}

fn check_exit(
    snap: &MicroSnapshot,
    pos_dir: PosDir,
    entry: f64,
    tp: f64,
    sl: f64,
    initial_agreement: &str,
    initial_regime: &str,
) -> Option<ExitReason> {
    // TP/SL check.
    match pos_dir {
        PosDir::Long => {
            if snap.price >= tp {
                return Some(ExitReason::TpHit);
            }
            if snap.price <= sl {
                return Some(ExitReason::SlHit);
            }
        }
        PosDir::Short => {
            if snap.price <= tp {
                return Some(ExitReason::TpHit);
            }
            if snap.price >= sl {
                return Some(ExitReason::SlHit);
            }
        }
    }

    // Agreement flip: entered on continuation, now seeing pullback/range.
    if snap.agreement != initial_agreement {
        let bad_flip = match pos_dir {
            PosDir::Long => matches!(
                snap.agreement,
                "PULLBACK RISK" | "RANGE/FAKEOUTS" | "CONTINUATION DOWN"
            ),
            PosDir::Short => matches!(
                snap.agreement,
                "RECLAIM RISK" | "RANGE/FAKEOUTS" | "CONTINUATION UP"
            ),
        };
        if bad_flip {
            return Some(ExitReason::AgreementFlip);
        }
    }

    // Regime flip from trending to choppy/flat.
    if initial_regime == "TRENDING"
        && (snap.trend_regime == "CHOPPY" || snap.trend_regime == "DRIFT/FLAT")
    {
        return Some(ExitReason::RegimeFlip);
    }

    _ = entry; // used for context, not exit logic
    None
}

fn fmt_pct(v: f64) -> String {
    format!("{:+.2}%", v * 100.0)
}

fn print_tick(tick: u32, snap: &MicroSnapshot, entry: f64, pnl_pct: f64) {
    println!(
        "[{:>3}] {} | {} | {} str={} | 5m={} 15m={} | P&L {}",
        tick,
        snap.price,
        snap.agreement,
        snap.trend_regime,
        snap.strength,
        snap.trend_5m.as_str(),
        snap.trend_15m.as_str(),
        fmt_pct(pnl_pct),
    );
    _ = entry;
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let pos_dir = PosDir::parse(&args.dir)?;
    let client = Client::new();

    println!(
        "Monitoring {} {} @ {} | TP={} SL={} | every {}m (max {}m)\n",
        args.coin,
        args.dir.to_uppercase(),
        args.entry,
        args.tp,
        args.sl,
        args.interval,
        args.max_minutes,
    );

    // Take initial snapshot to record starting agreement/regime.
    let now = Utc::now();
    let start_time = floor_to_hour(now);
    let start_ms = start_time.timestamp_millis() as u64;
    let now_ms = now.timestamp_millis() as u64;

    let candles = fetch_candles(&client, &args.coin, start_ms, now_ms)
        .await
        .context("initial fetch failed")?;
    let initial = snapshot(&candles, now).context("insufficient data for initial snapshot")?;

    let initial_agreement = initial.agreement.to_string();
    let initial_regime = initial.trend_regime.to_string();

    println!(
        "Initial: {} | {} | str={}",
        initial.agreement, initial.trend_regime, initial.strength
    );
    println!();

    let mut tick = 0u32;
    let max_ticks = args.max_minutes / args.interval;
    let sleep_dur = tokio::time::Duration::from_secs(args.interval * 60);

    loop {
        tick += 1;
        if tick > max_ticks as u32 {
            println!("\n>>> EXIT: {} — closing position", ExitReason::Timeout.label());
            break;
        }

        tokio::time::sleep(sleep_dur).await;

        let now = Utc::now();
        let current_hour = floor_to_hour(now);
        let current_start_ms = current_hour.timestamp_millis() as u64;
        let current_now_ms = now.timestamp_millis() as u64;

        // Detect hour boundary crossing.
        if current_start_ms != start_ms {
            println!("\n>>> EXIT: {} — hour boundary crossed", ExitReason::HourReset.label());
            break;
        }

        let candles = match fetch_candles(&client, &args.coin, current_start_ms, current_now_ms).await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[{:>3}] fetch error: {}", tick, e);
                continue;
            }
        };

        let snap = match snapshot(&candles, now) {
            Some(s) => s,
            None => {
                eprintln!("[{:>3}] no data", tick);
                continue;
            }
        };

        let pnl_pct = match pos_dir {
            PosDir::Long => (snap.price - args.entry) / args.entry,
            PosDir::Short => (args.entry - snap.price) / args.entry,
        };

        print_tick(tick, &snap, args.entry, pnl_pct);

        if let Some(reason) = check_exit(
            &snap,
            pos_dir,
            args.entry,
            args.tp,
            args.sl,
            &initial_agreement,
            &initial_regime,
        ) {
            let action = match reason {
                ExitReason::TpHit => "take profit",
                ExitReason::SlHit => "stop loss",
                _ => "close position",
            };
            println!(
                "\n>>> EXIT: {} — {} at {} (P&L {})",
                reason.label(),
                action,
                snap.price,
                fmt_pct(pnl_pct)
            );
            break;
        }
    }

    Ok(())
}
