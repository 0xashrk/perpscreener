mod client;
mod daily_ctx;
mod decision;
mod indicators;
mod macro_ctx;
mod micro_ctx;
mod regime;
mod vwap;

use std::fmt::Write as FmtWrite;
use std::sync::Arc;

use anyhow::{Context, Result};
use chrono::Utc;
use clap::Parser;
use reqwest::Client;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use client::{fetch_asset_max_leverage, fetch_candles, fetch_l2_book, fetch_top_assets, AssetMeta};
use daily_ctx::{compute_daily, compute_volume, DailyContext, VolumeContext};
use decision::{decide, Signal, Strategy, TradeDecision};
use indicators::{compute_bb, compute_rsi};
use macro_ctx::{compute_macro, MacroContext};
use micro_ctx::{compute_micro, MicroContext};
use regime::{classify, Regime};
use vwap::{compute_vwap, VwapContext};

#[derive(Parser, Debug)]
#[command(name = "signal", about = "Signal V2 — VWAP, regime routing, limit orders")]
struct Args {
    /// Asset symbol
    #[arg(long, required_unless_present = "top")]
    coin: Option<String>,

    /// Scan top N assets by 24h volume
    #[arg(long, conflicts_with = "coin")]
    top: Option<usize>,

    /// Account value in USD
    #[arg(long, default_value_t = 100.0)]
    av: f64,

    /// VWAP slope threshold for TRENDING classification
    #[arg(long, default_value_t = 0.0003)]
    vwap_slope_threshold: f64,

    /// BB width threshold for RANGING classification
    #[arg(long, default_value_t = 0.015)]
    bb_tight_threshold: f64,
}

struct CoinResult {
    coin: String,
    decision: TradeDecision,
    regime: Regime,
    mac: MacroContext,
    mic: MicroContext,
    vwap: VwapContext,
    rsi: Option<f64>,
    daily: Option<DailyContext>,
    volume: Option<VolumeContext>,
}

async fn scan_coin(
    client: &Client,
    coin: &str,
    av: f64,
    asset_max_lev: u32,
    vwap_slope_threshold: f64,
    bb_tight_threshold: f64,
) -> Result<CoinResult> {
    let now = Utc::now();
    let now_ms = now.timestamp_millis() as u64;

    // Midnight UTC for VWAP reset.
    let midnight = now
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Utc)
        .unwrap();
    let midnight_ms = midnight.timestamp_millis() as u64;

    // Extra 6h before midnight for BB/RSI warmup.
    let candle_15m_start = midnight_ms - 6 * 3_600_000;
    let start_4h = now_ms - 201 * 3_600_000;
    let start_1h = now_ms - 5 * 24 * 3_600_000;

    // 30 days of daily candles for structure levels.
    let start_1d = now_ms - 30 * 24 * 3_600_000;

    let (c4h, c1h, c15m, c1d, ob) = tokio::try_join!(
        fetch_candles(client, coin, "4h", start_4h, now_ms),
        fetch_candles(client, coin, "1h", start_1h, now_ms),
        fetch_candles(client, coin, "15m", candle_15m_start, now_ms),
        fetch_candles(client, coin, "1d", start_1d, now_ms),
        fetch_l2_book(client, coin),
    )?;

    let mac = compute_macro(&c4h, &c1h, &ob);

    // VWAP: only today's candles.
    let day_candles: Vec<_> = c15m.iter().filter(|c| c.t >= midnight_ms).cloned().collect();
    let price = c15m.last().map(|c| c.c).unwrap_or(0.0);
    let vwap_ctx = compute_vwap(&day_candles, price).context("VWAP computation failed")?;

    // Micro: use recent 15m candles.
    let mic = compute_micro(&c15m, &vwap_ctx).context("micro computation failed")?;

    // BB and RSI on 15m candles.
    let bb = compute_bb(&c15m, 20, 2.0);
    let rsi = compute_rsi(&c15m, 14);

    // Regime classification.
    let regime = classify(
        &vwap_ctx,
        bb.as_ref(),
        mic.trend_regime,
        vwap_slope_threshold,
        bb_tight_threshold,
    );

    // Daily structure: 20-day high/low resistance/support zones.
    // Drop most recent daily candle (not closed).
    let closed_1d = if c1d.len() > 1 { &c1d[..c1d.len() - 1] } else { &c1d };
    let daily = compute_daily(closed_1d, price);

    // Volume trend from 15m candles.
    let volume = compute_volume(&c15m);

    let d = decide(
        &mac, &mic, &vwap_ctx, regime, bb.as_ref(), rsi, av, asset_max_lev,
        daily.as_ref(), volume.as_ref(),
    );

    Ok(CoinResult {
        coin: coin.to_string(),
        decision: d,
        regime,
        mac,
        mic,
        vwap: vwap_ctx,
        rsi,
        daily,
        volume,
    })
}

// -- Formatting --------------------------------------------------------------

fn fmt_price(p: f64) -> String {
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

fn fmt_pct(v: f64) -> String {
    format!("{:+.2}%", v * 100.0)
}

fn fmt_size(size: f64, coin: &str) -> String {
    if size >= 100.0 {
        format!("{:.1} {}", size, coin)
    } else if size >= 1.0 {
        format!("{:.2} {}", size, coin)
    } else if size >= 0.01 {
        format!("{:.4} {}", size, coin)
    } else {
        format!("{:.6} {}", size, coin)
    }
}

fn short_agreement(a: &str) -> &str {
    match a {
        "CONTINUATION UP" => "CONT UP",
        "CONTINUATION DOWN" => "CONT DN",
        "PULLBACK RISK" => "PULLBACK",
        "RECLAIM RISK" => "RECLAIM",
        "RANGE/FAKEOUTS" => "RANGE",
        other => other,
    }
}

fn format_single(r: &CoinResult, av: f64) -> String {
    let d = &r.decision;
    let mut out = String::new();

    writeln!(
        &mut out,
        "=== {}: {} ({}) ===\n",
        r.coin,
        d.signal.as_str(),
        d.strategy.as_str()
    )
    .ok();
    writeln!(&mut out, "Regime:  {}", r.regime.as_str()).ok();

    if d.signal == Signal::Flat {
        writeln!(&mut out, "Reason:  {}\n", d.reason).ok();
    } else {
        writeln!(&mut out, "Conv:    {}", d.conviction.as_str()).ok();
        writeln!(&mut out, "Reason:  {}\n", d.reason).ok();
        writeln!(&mut out, "Limit:   {}", fmt_price(d.limit_price)).ok();
        writeln!(
            &mut out,
            "Size:    {} (${:.2})",
            fmt_size(d.size_asset, &r.coin),
            d.size_usd
        )
        .ok();
        writeln!(&mut out, "Lev:     {}x (max: {}x)", d.leverage, d.max_leverage).ok();
        writeln!(
            &mut out,
            "Risk:    ${:.2} ({:.2}% of ${:.0})\n",
            d.risk_usd,
            d.risk_usd / av * 100.0,
            av
        )
        .ok();
        writeln!(
            &mut out,
            "SL:      {} ({})",
            fmt_price(d.sl),
            fmt_pct((d.sl - d.limit_price) / d.limit_price)
        )
        .ok();
        if d.strategy == Strategy::MeanRevert {
            let tp = match d.signal {
                Signal::Long => d.limit_price * 1.008,
                Signal::Short => d.limit_price * 0.992,
                _ => 0.0,
            };
            writeln!(&mut out, "TP:      {} (+0.80%)", fmt_price(tp)).ok();
        } else {
            writeln!(&mut out, "Trail:   BE at +0.3%, lock at +0.6%, tight at +1.0%").ok();
        }
        writeln!(&mut out).ok();
    }

    // Context block.
    let trend = if r.mac.bull { "BULL" } else { "BEAR" };
    let bo = if r.mac.at_breakout_long {
        " + BREAKOUT"
    } else if r.mac.at_breakout_short {
        " + BREAKDOWN"
    } else {
        ""
    };
    writeln!(
        &mut out,
        "Macro:   {} (SMA20={} SMA50={}){}",
        trend,
        fmt_price(r.mac.sma20),
        fmt_price(r.mac.sma50),
        bo
    )
    .ok();
    writeln!(
        &mut out,
        "         Donch=[{}, {}] ATR={} ({:.2}%)",
        fmt_price(r.mac.don_lo),
        fmt_price(r.mac.don_hi),
        fmt_price(r.mac.atr),
        r.mac.atr_pct * 100.0
    )
    .ok();
    writeln!(
        &mut out,
        "VWAP:    {} (price {}) slope={:+.4}",
        fmt_price(r.vwap.vwap),
        fmt_pct(r.vwap.price_vs_vwap),
        r.vwap.vwap_slope
    )
    .ok();
    writeln!(
        &mut out,
        "         bands=[{}, {}]",
        fmt_price(r.vwap.band_lower),
        fmt_price(r.vwap.band_upper)
    )
    .ok();
    writeln!(
        &mut out,
        "Micro:   {} | {} | str={}/100",
        r.mic.agreement, r.mic.trend_regime, r.mic.strength
    )
    .ok();
    writeln!(
        &mut out,
        "         15m={} 1h={} | streak={}x{}",
        r.mic.trend_1c.as_str(),
        r.mic.trend_4c.as_str(),
        r.mic.streak_dir.as_str(),
        r.mic.streak_len
    )
    .ok();
    writeln!(
        &mut out,
        "OB:      imb={:.2} spread={:.4}%{}",
        r.mac.ob_imbalance,
        r.mac.spread_pct * 100.0,
        r.rsi
            .map(|v| format!(" | RSI={:.0}", v))
            .unwrap_or_default()
    )
    .ok();
    if let Some(ref dc) = r.daily {
        writeln!(
            &mut out,
            "Daily:   20d hi={} lo={} | price {:+.1}% from hi{}",
            fmt_price(dc.daily_high),
            fmt_price(dc.daily_low),
            dc.pct_from_high * 100.0,
            if dc.near_resistance { " *** RESISTANCE ZONE ***" } else { "" }
        )
        .ok();
    }
    if let Some(ref vc) = r.volume {
        writeln!(
            &mut out,
            "Volume:  ratio={:.2}x avg{}{}",
            vc.vol_ratio,
            if vc.vol_declining { " | DECLINING" } else { "" },
            if vc.vol_confirms { " | CONFIRMS" } else { "" }
        )
        .ok();
    }

    out
}

fn format_multi(results: &[CoinResult], av: f64) -> String {
    let mut out = String::new();
    writeln!(
        &mut out,
        "Signal V2 — {} (AV: ${:.0})\n",
        Utc::now().format("%Y-%m-%d %H:%M UTC"),
        av
    )
    .ok();

    writeln!(
        &mut out,
        "| # | Coin | Regime | Signal | Conv | Strat | Limit | SL | Macro | VWAP | Micro |"
    )
    .ok();
    writeln!(
        &mut out,
        "|---|------|--------|--------|------|-------|-------|----|-------|------|-------|"
    )
    .ok();

    for (i, r) in results.iter().enumerate() {
        let d = &r.decision;
        let macro_lbl = format!(
            "{}{}",
            if r.mac.bull { "BULL" } else { "BEAR" },
            if r.mac.at_breakout_long {
                "+BO"
            } else if r.mac.at_breakout_short {
                "+BD"
            } else {
                ""
            }
        );
        let vwap_lbl = format!(
            "{} s{:+.1}",
            fmt_pct(r.vwap.price_vs_vwap),
            r.vwap.vwap_slope * 10000.0
        );
        let micro_lbl = format!("{} {}", short_agreement(r.mic.agreement), r.mic.strength);

        if d.signal == Signal::Flat {
            writeln!(
                &mut out,
                "| {} | {} | {} | FLAT | - | - | - | - | {} | {} | {} |",
                i + 1,
                r.coin,
                r.regime.as_str(),
                macro_lbl,
                vwap_lbl,
                micro_lbl,
            )
            .ok();
        } else {
            writeln!(
                &mut out,
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                i + 1,
                r.coin,
                r.regime.as_str(),
                d.signal.as_str(),
                d.conviction.as_str(),
                d.strategy.as_str(),
                fmt_price(d.limit_price),
                fmt_price(d.sl),
                macro_lbl,
                vwap_lbl,
                micro_lbl,
            )
            .ok();
        }
    }

    out
}

// -- Main --------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = Client::new();

    let assets: Vec<AssetMeta> = if let Some(n) = args.top {
        eprintln!("Fetching top {} assets by 24h volume...", n);
        fetch_top_assets(&client, n).await?
    } else {
        let coin = args.coin.unwrap();
        let max_lev = fetch_asset_max_leverage(&client, &coin).await?;
        vec![AssetMeta { name: coin, max_leverage: max_lev }]
    };

    let multi = assets.len() > 1;

    let results = if multi {
        scan_multi(
            &client,
            &assets,
            args.av,
            args.vwap_slope_threshold,
            args.bb_tight_threshold,
        )
        .await
    } else {
        let a = &assets[0];
        match scan_coin(
            &client,
            &a.name,
            args.av,
            a.max_leverage,
            args.vwap_slope_threshold,
            args.bb_tight_threshold,
        )
        .await
        {
            Ok(r) => vec![r],
            Err(e) => {
                eprintln!("Error: {:#}", e);
                vec![]
            }
        }
    };

    if multi {
        print!("{}", format_multi(&results, args.av));
    } else if let Some(r) = results.first() {
        print!("{}", format_single(r, args.av));
    } else {
        eprintln!("No results.");
    }

    Ok(())
}

async fn scan_multi(
    client: &Client,
    assets: &[AssetMeta],
    av: f64,
    vwap_thresh: f64,
    bb_thresh: f64,
) -> Vec<CoinResult> {
    let sem = Arc::new(Semaphore::new(5));
    let mut set = JoinSet::new();

    for a in assets.iter() {
        let client = client.clone();
        let sem = sem.clone();
        let coin = a.name.clone();
        let max_lev = a.max_leverage;
        set.spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            scan_coin(&client, &coin, av, max_lev, vwap_thresh, bb_thresh)
                .await
                .map(|r| (coin, r))
        });
    }

    let mut results = Vec::new();
    while let Some(res) = set.join_next().await {
        match res {
            Ok(Ok((_coin, r))) => results.push(r),
            Ok(Err(e)) => eprintln!("  skip: {}", e),
            Err(e) => eprintln!("  task panic: {}", e),
        }
    }

    let names: Vec<&str> = assets.iter().map(|a| a.name.as_str()).collect();
    results.sort_by_key(|r| {
        names
            .iter()
            .position(|c| *c == r.coin)
            .unwrap_or(usize::MAX)
    });
    results
}
