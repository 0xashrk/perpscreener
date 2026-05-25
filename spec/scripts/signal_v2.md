# Signal V2 Spec

Day-trading signal engine with regime routing, VWAP, and limit-order execution.

## Architecture

```
Every 5 min:
  MACRO  (4h)  → SMA20/50 bull/bear, trend strength
  LEVELS (1h)  → Donchian hi/lo, ATR
  VWAP   (15m) → price vs VWAP, slope, bands → regime
  MICRO  (15m) → momentum, trend strength, streaks

  REGIME ROUTER → trending / ranging / choppy
    trending + macro aligned  → trend-follow
    ranging + flat VWAP       → mean-revert (BB + RSI)
    choppy                    → no trade

  EXECUTION → limit orders, trailing stops, hold across hours
```

## Timeframe Hierarchy

| Timeframe | Purpose | Indicators |
|-----------|---------|------------|
| 4h | Macro trend direction | SMA20, SMA50, bull/bear |
| 1h | Support/resistance, volatility | Donchian(20), ATR(14) |
| 15m | Primary signal + regime detection | VWAP, VWAP slope, VWAP bands, momentum, RSI(14), BB(20,2) |
| 5m | Entry timing + trailing stop mgmt | Price action vs limit levels |

## Data Sources

All from Hyperliquid `/info` endpoint:

| Request | Interval | Lookback | Purpose |
|---------|----------|----------|---------|
| candleSnapshot | 4h | 50 periods (~8 days) | SMA20/50 |
| candleSnapshot | 1h | 5 days | Donchian, ATR |
| candleSnapshot | 15m | current day (from 00:00 UTC) | VWAP, momentum, BB, RSI |
| candleSnapshot | 5m | current hour | entry timing, trailing stop |
| l2Book | - | current snapshot | OB imbalance, spread |

## Indicators

### Macro (4h)

- `sma20`, `sma50`: simple moving averages on closed 4h candle closes.
- `bull = sma20 > sma50`.
- `trend_strength = abs(sma20 - sma50) / mid`.

### Levels (1h)

- `don_hi`, `don_lo`: Donchian channel high/low over 20 closed 1h candles.
- `atr`: Average True Range over 14 closed 1h candles (simple average, not Wilder's).
- `atr_pct = atr / mid`.

### VWAP (15m)

Computed from 15m candles since 00:00 UTC (daily reset).

```
typical_price = (high + low + close) / 3
cum_tp_vol += typical_price * volume
cum_vol += volume
vwap = cum_tp_vol / cum_vol
```

Derived:

- `price_vs_vwap`: price above/below VWAP as percentage.
- `vwap_slope`: rate of change of VWAP over last 4 candles (1 hour). Positive = rising, negative = falling.
- `vwap_band_upper`, `vwap_band_lower`: VWAP +/- 1 standard deviation of (typical_price - vwap) over the day's candles. Acts like volume-anchored Bollinger Bands.

### Momentum (15m)

Same logic as current momentum script, adapted to 15m candles:

- `ret_1c`: 1-candle return (= 15m return).
- `ret_4c`: 4-candle return (= 1h return).
- `trend_1c`, `trend_4c`: UP/DOWN/FLAT with 0.02% threshold.
- `trend_regime`: TRENDING (both agree), CHOPPY (disagree), DRIFT/FLAT (both flat).
- `trend_strength`: 0-100 score based on return magnitude, regime, and volatility.
- `agreement`: combines direction vs VWAP (replaces direction vs hour open) with trend regime.

Agreement signals (updated for VWAP):

| price_vs_vwap | regime | trend_1c | Signal |
|---------------|--------|----------|--------|
| ABOVE | TRENDING | UP | CONTINUATION UP |
| BELOW | TRENDING | DOWN | CONTINUATION DOWN |
| ABOVE | any | DOWN | PULLBACK RISK |
| BELOW | any | UP | RECLAIM RISK |
| any | CHOPPY | any | RANGE/FAKEOUTS |
| any | DRIFT/FLAT | any | NEUTRAL |

### Mean-Revert Indicators (15m)

Only computed when regime = RANGING.

- `bb_upper`, `bb_lower`: Bollinger Bands — SMA(20) +/- 2 * stddev on 15m closes.
- `rsi`: RSI(14) on 15m closes.
- `bb_width`: (bb_upper - bb_lower) / sma20. Narrow = tight range.

### Orderbook (live)

- `ob_imbalance`: sum(bid_sz top 10) / sum(ask_sz top 10).
- `spread_pct`: (ask - bid) / mid.

## Regime Router

Evaluated every 5 minutes. Uses VWAP slope + trend_regime + bb_width to classify:

```
IF vwap_slope magnitude > threshold AND trend_regime == "TRENDING":
    regime = TRENDING

ELSE IF bb_width < tight_threshold AND trend_regime != "TRENDING":
    regime = RANGING

ELSE:
    regime = CHOPPY
```

Thresholds (tunable):

| Param | Default | Description |
|-------|---------|-------------|
| `vwap_slope_threshold` | 0.0003 | Min abs VWAP slope to qualify as trending |
| `bb_tight_threshold` | 0.015 | Max BB width to qualify as ranging |

## Signal Logic

### Trend-Follow (regime = TRENDING)

Entry conditions — all must be true:

```
macro:   bull (for long) OR bear (for short)
vwap:    price above VWAP (long) OR below (short)
micro:   agreement == CONTINUATION UP (long) or DOWN (short)
ob:      imbalance >= 1.05 (long) or <= 0.95 (short)
spread:  spread_pct <= 0.10%
```

Conviction tiers:

| Tier | Extra condition | Risk % |
|------|----------------|--------|
| STRONG | at Donchian breakout + OB confirms | 0.50% |
| NORMAL | all entry conditions met | 0.30% |
| WEAK | micro = NEUTRAL + macro strong | 0.15% |

### Mean-Revert (regime = RANGING)

Entry conditions — all must be true:

```
Long:
  price <= bb_lower (or touched in prior candle and bounced)
  rsi < 35 (oversold approaching)
  spread_pct <= 0.08%

Short:
  price >= bb_upper (or touched and rejected)
  rsi > 65 (overbought approaching)
  spread_pct <= 0.08%
```

Single conviction tier. Risk: 0.20%.

### No Trade (regime = CHOPPY)

Flat. No entries. If in a position from a prior regime, apply tighter trailing stop.

## Execution

### Entry: Limit Orders

Do NOT enter at market. Place limit orders:

- **Trend-follow long**: limit at `min(last_close, vwap)` — buy at VWAP or current price, whichever is lower.
- **Trend-follow short**: limit at `max(last_close, vwap)`.
- **Mean-revert long**: limit at `bb_lower`.
- **Mean-revert short**: limit at `bb_upper`.

Order lifetime: **3 candles (15 min)**. If not filled, cancel. This ensures we only enter at good prices and pay maker fees.

### Position Sizing

Same risk-based approach as V1:

```
stop_dist = max(1.5 * atr, 0.3% * entry)
risk_usd = risk_pct * account_value
momentum_mult = clamp(trend_strength / 70, 0.3, 1.0)
size = min(risk_usd / stop_dist * momentum_mult, max_leverage * av / entry)
```

Max leverage: 5x. Fee assumption: 0.035% maker round-trip (0.07% worst case taker).

### Stop Loss

Initial SL:

- **Trend-follow**: `entry - 1.5 * ATR` (long) or `entry + 1.5 * ATR` (short).
- **Mean-revert**: `entry - 1.0% ` (long) or `entry + 1.0%` (short). Tighter because thesis invalidates quickly.

### Trailing Stop (replaces fixed TP)

No fixed take-profit. Instead, a trailing stop managed on 5m candles:

```
IF unrealized_pnl >= 0.3%:
    move stop to breakeven (entry price)

IF unrealized_pnl >= 0.6%:
    trail stop at entry + 0.3% (lock in profit)

IF unrealized_pnl >= 1.0%:
    trail stop at highest_profit - 0.4% (tighten)

For mean-revert:
    fixed TP at 0.8% (these don't trend, take the profit)
```

The trailing stop is evaluated every 5m candle against the candle's low (long) or high (short).

### Hold Duration

No fixed hold limit. Position stays open as long as:

1. Trailing stop not hit.
2. Macro trend intact (bull for long, bear for short).
3. Regime hasn't flipped to CHOPPY (tighten trail) or opposing trend.
4. VWAP hasn't been decisively crossed against position (e.g., long position, price closes below VWAP on 2 consecutive 15m candles).

This allows holding across hours and even across multiple hours when the trend is strong.

### Exit Triggers (priority order)

1. **SL/trailing stop hit** → immediate market exit.
2. **VWAP cross against** → 2 consecutive 15m closes on wrong side of VWAP → exit.
3. **Regime flip to CHOPPY** → tighten trailing stop to current_price - 0.2%. If already trailing, keep tighter of the two.
4. **Macro flip** → SMA20 crosses SMA50 against position → exit.
5. **Mean-revert TP** → 0.8% profit target hit → exit.
6. **Daily loss cutoff** → cumulative daily loss >= 1.5% of AV → stop trading for the day.

## Cooldowns and Limits

| Param | Default | Description |
|-------|---------|-------------|
| `cooldown_min` | 30 | Minutes after exit before new entry |
| `max_daily_trades` | 8 | Max entries per day |
| `daily_loss_cutoff` | 1.5% | Stop trading after this daily loss |
| `max_concurrent` | 1 | Positions open at once (start with 1) |

## CLI Interface

```bash
cargo run -p signal -- \
  --coin HYPE \
  --av 10000 \
  [--top 10] \
  [--mode live|scan] \
  [--vwap-slope-threshold 0.0003] \
  [--bb-tight-threshold 0.015]
```

- `--mode scan` (default): one-shot signal check, print result, exit.
- `--mode live`: continuous loop every 5 min, manages orders and positions.
- `--top N`: scan mode only, ranks top N assets by signal quality.

## Output — Scan Mode

Single coin:
```
=== HYPE: LONG (trend-follow) ===

Regime:  TRENDING (VWAP slope +0.12%, macro BULL)
Conv:    NORMAL
Reason:  bull + above VWAP + micro continuation

Limit:   55.80 (at VWAP)
Size:    18.2 HYPE ($1016)
Risk:    $30 (0.30% of $10,000)

SL:      54.42 (-2.47%)
Trail:   breakeven at +0.3%, lock at +0.6%

Macro:   BULL (SMA20=55.9 > SMA50=49.4) + BREAKOUT
VWAP:    56.12 (price +0.57% above) slope=+0.0012
Micro:   CONT UP | str=65 | 15m=UP 1h=UP
OB:      imb=1.18 spread=0.004%
```

Multi-coin table:
```
| # | Coin | Regime | Signal | Conv | Limit | SL | Macro | VWAP | Micro |
```

## Output — Live Mode

Logs to stdout with timestamps:

```
[14:05] SCAN: HYPE TRENDING — LONG NORMAL, placing limit 55.80
[14:05] ORDER: HYPE limit buy 18.2 @ 55.80
[14:10] FILL: HYPE filled 18.2 @ 55.79
[14:25] TRAIL: HYPE +0.35% — moving stop to breakeven (55.79)
[14:40] TRAIL: HYPE +0.72% — locking profit, stop → 55.96
[15:10] EXIT: HYPE trailing stop hit @ 56.31 (+0.93%)
[15:10] COOLDOWN: 30 min until next entry
```

## Fee Budget

Target: net positive after fees on every conviction tier.

| Entry | Exit | Round-trip | Min avg win needed |
|-------|------|------------|-------------------|
| Maker (0.035%) | Maker (0.035%) | 0.02% | > 0.10% |
| Maker (0.035%) | Taker (0.1%) | 0.045% | > 0.15% |
| Taker (0.1%) | Taker (0.1%) | 0.07% | > 0.20% |

Strategy always enters maker (limit). Exits are taker (market) for SL/trail, maker for mean-revert TP.

With avg win +0.52% and maker entry, fee drag is ~0.045% per trade = ~8.6% of the avg win. Acceptable.

## Backtest Considerations

- VWAP requires volume data — HL candles include `v` field.
- OB data unavailable historically — backtest should run with and without OB gates and compare.
- Use 15m candles as primary (2 months history available).
- Trailing stop simulation: check 5m candle highs/lows for stop triggers. If 5m data unavailable, use 15m candle highs/lows (less precise).
- Record and store OB snapshots going forward for future backtesting.
