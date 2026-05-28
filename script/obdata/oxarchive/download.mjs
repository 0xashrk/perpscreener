#!/usr/bin/env node
/**
 * 0xArchive Historical Data Downloader
 *
 * Downloads 15m candles and L2 orderbook snapshots from 0xArchive's free trial.
 * Auth: burner EVM wallet → SIWE challenge → Bearer token (24h).
 *
 * Usage:
 *   node script/obdata/oxarchive/download.mjs --coins BTC,ETH,HYPE --months 6
 *
 * Output:
 *   data/candles/{coin}_15m.csv   — 15m candles (timestamp_ms,o,h,l,c,v)
 *   data/ob/{coin}_ob.csv         — OB snapshots (timestamp_ms,ob_imbalance,spread_pct,...)
 *
 * The burner wallet key is generated fresh each run. No funds needed.
 */

import { Wallet } from "ethers";
import { writeFileSync, mkdirSync, existsSync } from "fs";
import { parseArgs } from "util";

const API = "https://api.0xarchive.io";

// -- CLI args ----------------------------------------------------------------

const { values: args } = parseArgs({
  options: {
    coins: { type: "string", default: "BTC,ETH,HYPE" },
    months: { type: "string", default: "6" },
    "out-dir": { type: "string", default: "data" },
  },
});

const COINS = args.coins.split(",").map((c) => c.trim().toUpperCase());
const MONTHS = parseInt(args.months);
const OUT_DIR = args["out-dir"];

// -- Auth --------------------------------------------------------------------

async function authenticate() {
  // Generate burner wallet.
  const wallet = Wallet.createRandom();
  console.log(`Auth: burner wallet ${wallet.address}`);

  // Get SIWE challenge.
  const cr = await fetch(`${API}/v1/auth/web3/challenge`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ address: wallet.address }),
  });
  const { message } = await cr.json();

  // Sign and verify.
  const signature = await wallet.signMessage(message);
  const vr = await fetch(`${API}/v1/auth/web3/verify`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ message, signature }),
  });

  // Extract Bearer token from Set-Cookie.
  const cookies = vr.headers.getSetCookie?.() || [];
  const tokenCookie = cookies.find((c) => c.startsWith("ox_access_token="));
  if (!tokenCookie) throw new Error("No access token in response");
  const token = tokenCookie.split("=")[1].split(";")[0];

  const result = await vr.json();
  console.log(`Auth: ${result.user.tier} (trial until ${result.user?.current_period_end || "unknown"})`);
  return token;
}

// -- Data fetching -----------------------------------------------------------

async function fetchJson(url, token) {
  const resp = await fetch(url, {
    headers: { Authorization: `Bearer ${token}` },
  });
  if (resp.status === 429) {
    console.log("  Rate limited, waiting 5s...");
    await new Promise((r) => setTimeout(r, 5000));
    return fetchJson(url, token);
  }
  if (!resp.ok) {
    const text = await resp.text();
    throw new Error(`HTTP ${resp.status}: ${text.substring(0, 200)}`);
  }
  return resp.json();
}

async function downloadCandles(token, coin, startMs, endMs) {
  const allCandles = [];
  const chunkMs = 500 * 15 * 60_000; // 500 candles × 15m = ~5 days per chunk
  let cursor = startMs;
  let chunks = 0;

  while (cursor < endMs) {
    const chunkEnd = Math.min(cursor + chunkMs, endMs);
    const url = `${API}/v1/hyperliquid/candles/${coin}?interval=15m&start=${cursor}&end=${chunkEnd}&limit=500`;
    const result = await fetchJson(url, token);
    const data = result.data || [];
    allCandles.push(...data);
    cursor = chunkEnd;
    chunks++;
    if (chunks % 10 === 0) {
      process.stderr.write(`\r  ${coin} candles: ${allCandles.length} rows (${chunks} chunks)...`);
    }
    // Pace to avoid rate limits.
    await new Promise((r) => setTimeout(r, 100));
  }
  console.log(`\r  ${coin} candles: ${allCandles.length} rows (${chunks} chunks)`);
  return allCandles;
}

async function downloadOrderbook(token, coin, startMs, endMs) {
  const allSnaps = [];
  const chunkMs = 3_600_000; // 1 hour per request
  let cursor = startMs;
  let chunks = 0;

  while (cursor < endMs) {
    const chunkEnd = Math.min(cursor + chunkMs, endMs);
    const url = `${API}/v1/hyperliquid/orderbook/${coin}/history?start=${cursor}&end=${chunkEnd}&limit=100`;
    try {
      const result = await fetchJson(url, token);
      const data = result.data || [];
      allSnaps.push(...data);
    } catch (e) {
      // OB history may not be available for all dates — skip silently.
    }
    cursor = chunkEnd;
    chunks++;
    if (chunks % 100 === 0) {
      process.stderr.write(`\r  ${coin} OB: ${allSnaps.length} snaps (${chunks} chunks)...`);
    }
    await new Promise((r) => setTimeout(r, 50));
  }
  console.log(`\r  ${coin} OB: ${allSnaps.length} snapshots (${chunks} chunks)`);
  return allSnaps;
}

// -- CSV output --------------------------------------------------------------

function writeCandlesCsv(candles, path) {
  const header = "timestamp_ms,o,h,l,c,v\n";
  const rows = candles.map((c) => {
    const ts = new Date(c.timestamp).getTime();
    return `${ts},${c.open},${c.high},${c.low},${c.close},${c.volume}`;
  });
  writeFileSync(path, header + rows.join("\n") + "\n");
}

function writeObCsv(snapshots, path) {
  const header = "timestamp_ms,ob_imbalance,spread_pct,best_bid,best_ask,bid_depth,ask_depth\n";
  const rows = snapshots
    .map((s) => {
      const ts = new Date(s.timestamp).getTime();
      const bids = s.bids || [];
      const asks = s.asks || [];
      if (bids.length === 0 || asks.length === 0) return null;

      const bestBid = parseFloat(bids[0].px);
      const bestAsk = parseFloat(asks[0].px);
      if (bestBid <= 0 || bestAsk <= 0) return null;

      const mid = (bestBid + bestAsk) / 2;
      const spreadPct = (bestAsk - bestBid) / mid;
      const bidDepth = bids.slice(0, 10).reduce((s, l) => s + parseFloat(l.sz), 0);
      const askDepth = asks.slice(0, 10).reduce((s, l) => s + parseFloat(l.sz), 0);
      const imbalance = askDepth > 0 ? bidDepth / askDepth : 1;

      return `${ts},${imbalance.toFixed(4)},${spreadPct.toFixed(6)},${bestBid.toFixed(2)},${bestAsk.toFixed(2)},${bidDepth.toFixed(4)},${askDepth.toFixed(4)}`;
    })
    .filter(Boolean);
  writeFileSync(path, header + rows.join("\n") + "\n");
}

// -- Main --------------------------------------------------------------------

async function main() {
  console.log(`0xArchive Downloader: ${COINS.join(", ")} | ${MONTHS} months\n`);

  const token = await authenticate();

  const now = Date.now();
  const startMs = now - MONTHS * 30 * 24 * 3_600_000;

  mkdirSync(`${OUT_DIR}/candles`, { recursive: true });
  mkdirSync(`${OUT_DIR}/ob`, { recursive: true });

  for (const coin of COINS) {
    console.log(`\n--- ${coin} ---`);

    // Download 15m candles.
    const candles = await downloadCandles(token, coin, startMs, now);
    const candlePath = `${OUT_DIR}/candles/${coin.toLowerCase()}_15m.csv`;
    writeCandlesCsv(candles, candlePath);
    console.log(`  Wrote ${candles.length} candles → ${candlePath}`);

    // Download OB snapshots.
    const ob = await downloadOrderbook(token, coin, startMs, now);
    const obPath = `${OUT_DIR}/ob/${coin.toLowerCase()}_ob.csv`;
    writeObCsv(ob, obPath);
    console.log(`  Wrote ${ob.length} OB snapshots → ${obPath}`);
  }

  console.log("\nDone.");
}

main().catch((e) => {
  console.error("Error:", e.message);
  process.exit(1);
});
