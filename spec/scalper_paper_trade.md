# Scalper Paper Trading Script

Long-running Rust CLI that paper trades the SCALPER recipe, logging all signals and trades to SQLite for later evaluation.

## Location

```
script/scalper/
├── Cargo.toml
└── src/
    └── main.rs
```

## CLI

```
cargo run -p scalper -- --coin ETH --capital 100 --duration 24h
```

### Arguments

| Arg | Required | Default | Description |
|-----|----------|---------|-------------|
| `--coin` | yes | - | Asset symbol (BTC, ETH, etc.) |
| `--capital` | no | 100 | Starting capital in USD |
| `--duration` | no | unlimited | Run duration (e.g., 1h, 24h, 7d) |
| `--backend` | no | http://localhost:30001 | Backend base URL |
| `--db` | no | data/scalper.db | SQLite database path |
| `--interval` | no | 60 | Poll interval in seconds |

## Recipe Config

From `recipe/SCALPER.md`:

```
riskPct     = 0.005      # 0.5% of capital per trade (tuned from 0.15%)
stopPct     = 0.0015     # 0.15% stop loss
tpPct       = 0.004      # 0.4% take profit (widened to absorb fees)
maxHoldSec  = 90         # max position hold time
cooldownSec = 120        # cooldown between trades
minOB       = 1.2        # min OB imbalance for long
maxSpread   = 0.0005     # max spread to enter
```

## Fee Structure

Hyperliquid perp fees:
- Maker (limit order): 0.015%
- Taker (market order): 0.045%

Assumed execution:
- Entry: maker (0.015%) - using limit order
- Exit: taker (0.045%) - market order for TP/SL/timeout
- Round-trip: 0.06% of notional

## Database Schema

File: `data/scalper.db`

```sql
-- Every poll cycle (1 row per interval)
CREATE TABLE IF NOT EXISTS signals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_key INTEGER NOT NULL,           -- start_ts of the run
    ts INTEGER NOT NULL,
    coin TEXT NOT NULL,
    price REAL NOT NULL,
    bid REAL NOT NULL,
    ask REAL NOT NULL,
    spread REAL NOT NULL,
    ob_imbalance REAL NOT NULL,
    last_close REAL NOT NULL,
    prev_close REAL NOT NULL,
    momentum TEXT NOT NULL,        -- 'up', 'down', 'flat'
    signal TEXT NOT NULL,          -- 'LONG', 'SHORT', 'NONE'
    reason TEXT,                   -- why signal was/wasn't generated
    position_open INTEGER NOT NULL -- 1 if position was open, 0 if flat
);
CREATE INDEX IF NOT EXISTS signals_run_key_idx ON signals (run_key);

-- Paper trades
CREATE TABLE IF NOT EXISTS trades (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_key INTEGER NOT NULL,
    coin TEXT NOT NULL,
    direction TEXT NOT NULL,       -- 'LONG', 'SHORT'
    entry_ts INTEGER NOT NULL,
    entry_px REAL NOT NULL,
    size REAL NOT NULL,            -- position size in coin
    notional REAL NOT NULL,        -- position size in USD
    exit_ts INTEGER,
    exit_px REAL,
    gross_pnl REAL,
    entry_fee REAL,
    exit_fee REAL,
    net_pnl REAL,
    exit_reason TEXT               -- 'TP', 'SL', 'TIMEOUT', NULL if open
);
CREATE INDEX IF NOT EXISTS trades_run_key_idx ON trades (run_key);

-- Equity snapshots (every poll)
CREATE TABLE IF NOT EXISTS equity (
    run_key INTEGER NOT NULL,
    ts INTEGER NOT NULL,
    capital REAL NOT NULL,
    unrealized_pnl REAL NOT NULL,
    realized_pnl REAL NOT NULL,
    total_trades INTEGER NOT NULL,
    win_count INTEGER NOT NULL,
    loss_count INTEGER NOT NULL,
    PRIMARY KEY (run_key, ts)
);
CREATE INDEX IF NOT EXISTS equity_run_key_idx ON equity (run_key);

-- Run metadata
CREATE TABLE IF NOT EXISTS runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    start_ts INTEGER NOT NULL,
    end_ts INTEGER,
    coin TEXT NOT NULL,
    initial_capital REAL NOT NULL,
    final_capital REAL,
    config TEXT NOT NULL           -- JSON of recipe config
);
```

- `run_key` is the run's `start_ts` (seconds) and is included on every row so long runs and restarts stay isolated.
- SQLite settings: enable WAL mode and set a `busy_timeout`; checkpoint periodically (e.g., every 10k writes) to avoid runaway WAL size during week-long runs.

## Core Logic

### Main Loop

```
1. Start run: set `run_key = now_seconds()`, enable WAL, set busy_timeout, insert into `runs`
2. Every {interval} seconds:
   a. Fetch /chart?coin={coin}&interval=1m&limit=10
   b. Fetch /orderbook?coin={coin}
   c. If position open:
      - Check TP/SL/timeout conditions
      - Close if triggered, record trade
   d. If flat and cooldown elapsed:
      - Evaluate entry conditions
      - Open position if signal fires
   e. Log signal to `signals` table
   f. Log equity snapshot to `equity` table
3. On shutdown (ctrl+c or duration reached):
   - Close any open position at current price
   - Update `runs.end_ts` and `runs.final_capital`
   - Print summary
```

### Long-Run & Restart Safety

- Store `run_key` on every insert; queries filter by `run_key` to avoid mixing runs.
- On startup, check the most recent `run_key` for open trades and resume them; otherwise create a new run.
- Track consecutive fetch failures; if N (e.g., 3) happen while a position is open, force-close at last known price and pause new entries until data recovers.
- Emit hourly health stats to console (uptime, missed polls, DB errors, max hold time observed).

### Entry Conditions

```rust
let spread = (ask - bid) / mid;
let ob_imb = bid_qty_sum / ask_qty_sum;  // top 5 levels
let momentum_up = last_close > prev_close;
let momentum_dn = last_close < prev_close;

let long_signal = momentum_up
    && ob_imb >= 1.2
    && spread <= 0.0005;

let short_signal = momentum_dn
    && ob_imb <= (1.0 / 1.2)  // 0.833
    && spread <= 0.0005;
```

### Position Sizing

```rust
let risk_usd = capital * 0.005;           // 0.5% risk
let stop_dist = mid * 0.0015;             // 0.15% stop
let size_coins = risk_usd / stop_dist;
let notional = size_coins * mid;

// Entry fee (maker)
let entry_fee = notional * 0.00015;
```

### Exit Conditions

```rust
let hold_time = now - entry_ts;
let price_change_pct = (current_price - entry_px) / entry_px;
let pnl_pct = if long { price_change_pct } else { -price_change_pct };

// Check conditions in order
if pnl_pct >= 0.004 {
    exit_reason = "TP";
} else if pnl_pct <= -0.0015 {
    exit_reason = "SL";
} else if hold_time >= 90_000 {  // 90 seconds in ms
    exit_reason = "TIMEOUT";
}

// Exit fee (taker)
let exit_fee = notional * 0.00045;
let gross_pnl = notional * pnl_pct;
let net_pnl = gross_pnl - entry_fee - exit_fee;
```

### Cooldown

```rust
let cooldown_elapsed = match last_trade_exit_ts {
    Some(ts) => (now - ts) >= 120_000,  // 120 seconds
    None => true,
};
```

## Output

### Live Console Output

```
[2026-01-10 10:00:00] ETH $3450.25 | OB: 1.45 | Spread: 0.02% | Momentum: UP
                      Signal: LONG | Capital: $100.00

[2026-01-10 10:00:01] OPENED LONG @ $3450.25 | Size: 0.0145 ETH | Notional: $50.03

[2026-01-10 10:01:00] ETH $3458.50 | Position: +0.24% | Unrealized: +$0.12

[2026-01-10 10:01:30] CLOSED LONG @ $3464.06 | Reason: TP
                      Gross: +$2.00 | Fees: -$0.30 | Net: +$1.70
                      Capital: $101.70 | Win #1
```

### Shutdown Summary

```
============================================================
SCALPER PAPER TRADE COMPLETE
============================================================
Duration:    24h 0m 0s
Coin:        ETH

PERFORMANCE
-----------
Starting Capital:  $100.00
Final Capital:     $142.35
Total P&L:         +$42.35 (+42.35%)

TRADES
------
Total Trades:      47
Wins:              28 (59.6%)
Losses:            19 (40.4%)
Avg Win:           +$2.15
Avg Loss:          -$0.72
Profit Factor:     2.21

FEES
----
Total Fees Paid:   $8.45
Fees as % of P&L:  16.6%

TIMING
------
Avg Hold Time:     34.2s
Signals Checked:   1,440
Signal Rate:       3.3% (47/1440)
============================================================
```

## Implementation Notes

1. Use `tokio` for async runtime with graceful shutdown
2. Use `rusqlite` for database (same as existing `TokenStore`)
3. Use `reqwest` for HTTP client (same as existing scripts)
4. Use `clap` for CLI args (same as existing scripts)
5. Use `tracing` for logging
6. Handle ctrl+c gracefully - close position, update DB, print summary
7. On startup, check for unclosed positions from previous run

## Error Handling

- On fetch error: log warning, skip cycle, retry next interval
- On fetch error while a position is open: increment consecutive failure count; after threshold, close at last known price and cool down until data returns
- On DB error: log error; after threshold (e.g., 3 in a row), pause trading until DB succeeds, then resume
- On position close failure: retry up to 3 times, then force close at last known price

## Future Extensions

- Multiple coins in parallel
- WebSocket instead of polling for lower latency
- Real execution mode (with HL API keys)
- Telegram/Discord alerts on trades
