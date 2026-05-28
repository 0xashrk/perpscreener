use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::Datelike;
use clap::Parser;
use flate2::read::GzDecoder;

#[derive(Parser, Debug)]
#[command(name = "obdata", about = "Download and aggregate historical L2 orderbook data from Tardis.dev")]
struct Args {
    /// Asset symbol (e.g., BTC, ETH, HYPE)
    #[arg(long)]
    coin: String,

    /// Start date YYYY-MM-DD (Tardis free data: 1st of each month only)
    #[arg(long)]
    start: String,

    /// End date YYYY-MM-DD
    #[arg(long)]
    end: String,

    /// Snapshot interval in minutes (default: 15)
    #[arg(long, default_value_t = 15)]
    interval: u64,

    /// Output directory (default: data/ob/)
    #[arg(long, default_value = "data/ob")]
    out_dir: String,
}

struct ObSnapshot {
    timestamp_ms: u64,
    ob_imbalance: f64,
    spread_pct: f64,
    best_bid: f64,
    best_ask: f64,
    bid_depth: f64,
    ask_depth: f64,
}

/// Ordered f64 for BTreeMap keys.
#[derive(Clone, Copy, PartialEq)]
struct Of64(f64);
impl Eq for Of64 {}
impl PartialOrd for Of64 {
    fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(o)) }
}
impl Ord for Of64 {
    fn cmp(&self, o: &Self) -> std::cmp::Ordering {
        self.0.partial_cmp(&o.0).unwrap_or(std::cmp::Ordering::Equal)
    }
}

fn compute_ob_metrics(
    bids: &BTreeMap<Of64, f64>,
    asks: &BTreeMap<Of64, f64>,
) -> Option<ObSnapshot> {
    let best_bid = bids.keys().next_back()?.0;
    let best_ask = asks.keys().next()?.0;
    if best_bid <= 0.0 || best_ask <= 0.0 || best_ask <= best_bid {
        return None;
    }
    let mid = (best_bid + best_ask) / 2.0;
    let spread_pct = (best_ask - best_bid) / mid;
    let bid_depth: f64 = bids.values().rev().take(10).sum();
    let ask_depth: f64 = asks.values().take(10).sum();
    let ob_imb = if ask_depth > 0.0 { bid_depth / ask_depth } else { 1.0 };
    Some(ObSnapshot {
        timestamp_ms: 0, // filled by caller
        ob_imbalance: ob_imb,
        spread_pct,
        best_bid,
        best_ask,
        bid_depth,
        ask_depth,
    })
}

fn generate_dates(start: &str, end: &str) -> Result<Vec<String>> {
    let s = chrono::NaiveDate::parse_from_str(start, "%Y-%m-%d").context("invalid start date")?;
    let e = chrono::NaiveDate::parse_from_str(end, "%Y-%m-%d").context("invalid end date")?;
    let mut dates = Vec::new();
    let mut d = s;
    while d <= e {
        if d.day() == 1 {
            dates.push(d.format("%Y/%m/%d").to_string());
        }
        d += chrono::Duration::days(1);
    }
    Ok(dates)
}

fn download_day(coin: &str, date_path: &str) -> Result<Vec<u8>> {
    let url = format!(
        "https://datasets.tardis.dev/v1/hyperliquid/incremental_book_L2/{}/{}.csv.gz",
        date_path, coin
    );
    eprintln!("  Downloading {}...", url);
    let resp = reqwest::blocking::get(&url).context("download failed")?;
    if !resp.status().is_success() {
        anyhow::bail!("HTTP {}: {}", resp.status(), url);
    }
    Ok(resp.bytes().context("read body failed")?.to_vec())
}

fn process_day(gz_data: &[u8], interval_ms: u64) -> Result<Vec<ObSnapshot>> {
    let decoder = GzDecoder::new(gz_data);
    let reader = BufReader::new(decoder);

    // Read all rows, group by timestamp, compute OB metrics at interval boundaries.
    let mut bids: BTreeMap<Of64, f64> = BTreeMap::new();
    let mut asks: BTreeMap<Of64, f64> = BTreeMap::new();
    let mut snapshots: Vec<ObSnapshot> = Vec::new();
    let mut next_snap_ts: u64 = 0;
    let mut current_ts: u64 = 0;
    let mut line_count = 0u64;

    for line in reader.lines().skip(1) {
        let line = line.context("read line")?;
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < 8 { continue; }

        let timestamp_us: u64 = fields[2].parse().unwrap_or(0);
        let side = fields[5];
        let price: f64 = fields[6].parse().unwrap_or(0.0);
        let amount: f64 = fields[7].parse().unwrap_or(0.0);
        let ts_ms = timestamp_us / 1000;

        // New snapshot timestamp = complete book replacement.
        // Process the PREVIOUS book state before clearing.
        if timestamp_us != current_ts && current_ts != 0 {
            // Check if we crossed an interval boundary.
            if next_snap_ts == 0 {
                next_snap_ts = ts_ms - (ts_ms % interval_ms) + interval_ms;
            }
            if ts_ms >= next_snap_ts {
                if let Some(mut snap) = compute_ob_metrics(&bids, &asks) {
                    snap.timestamp_ms = next_snap_ts;
                    snapshots.push(snap);
                }
                next_snap_ts += interval_ms;
                while next_snap_ts <= ts_ms {
                    next_snap_ts += interval_ms;
                }
            }
            // Clear for new snapshot.
            bids.clear();
            asks.clear();
        }
        current_ts = timestamp_us;

        // Update book.
        let book = if side == "bid" { &mut bids } else { &mut asks };
        if amount == 0.0 {
            book.remove(&Of64(price));
        } else {
            book.insert(Of64(price), amount);
        }

        line_count += 1;
        if line_count % 1_000_000 == 0 {
            eprint!("\r    {} M rows...", line_count / 1_000_000);
        }
    }

    // Final snapshot.
    if !bids.is_empty() && !asks.is_empty() && next_snap_ts > 0 {
        if let Some(mut snap) = compute_ob_metrics(&bids, &asks) {
            snap.timestamp_ms = next_snap_ts;
            snapshots.push(snap);
        }
    }

    if line_count > 1_000_000 { eprintln!(); }
    eprintln!("    {} rows -> {} snapshots", line_count, snapshots.len());
    Ok(snapshots)
}

fn main() -> Result<()> {
    let args = Args::parse();
    let interval_ms = args.interval * 60 * 1000;
    let dates = generate_dates(&args.start, &args.end)?;

    if dates.is_empty() {
        anyhow::bail!("no valid dates (Tardis free data: 1st of each month only)");
    }

    eprintln!(
        "OB Data Pipeline: {} | {} dates | {}m snapshots",
        args.coin, dates.len(), args.interval
    );

    let out_dir = PathBuf::from(&args.out_dir);
    fs::create_dir_all(&out_dir).context("create output dir")?;

    let mut all: Vec<ObSnapshot> = Vec::new();
    for date_path in &dates {
        match download_day(&args.coin, date_path) {
            Ok(gz) => all.extend(process_day(&gz, interval_ms)?),
            Err(e) => eprintln!("  Skip {}: {}", date_path, e),
        }
    }

    if all.is_empty() {
        anyhow::bail!("no snapshots generated");
    }
    all.sort_by_key(|s| s.timestamp_ms);

    let path = out_dir.join(format!("{}_ob.csv", args.coin.to_lowercase()));
    let mut wtr = csv::Writer::from_path(&path).context("create CSV")?;
    wtr.write_record(["timestamp_ms", "ob_imbalance", "spread_pct", "best_bid", "best_ask", "bid_depth", "ask_depth"])?;
    for s in &all {
        wtr.write_record(&[
            s.timestamp_ms.to_string(),
            format!("{:.4}", s.ob_imbalance),
            format!("{:.6}", s.spread_pct),
            format!("{:.2}", s.best_bid),
            format!("{:.2}", s.best_ask),
            format!("{:.4}", s.bid_depth),
            format!("{:.4}", s.ask_depth),
        ])?;
    }
    wtr.flush()?;

    let avg_imb: f64 = all.iter().map(|s| s.ob_imbalance).sum::<f64>() / all.len() as f64;
    let avg_spread: f64 = all.iter().map(|s| s.spread_pct).sum::<f64>() / all.len() as f64;
    eprintln!(
        "\nWrote {} snapshots to {}\nAvg OB imbalance: {:.3} | Avg spread: {:.4}%",
        all.len(), path.display(), avg_imb, avg_spread * 100.0
    );
    Ok(())
}
