# Signal Stream SSE Endpoint

Real-time signal evaluation stream implementing HL_ALPHA recipe (excluding execution).

## Endpoint

```
GET /signals/stream?coin={coin}&account_value={av}&pnl_day={pnl}&position_size={sz}&position_side={side}&liq_price={liq}&last_fill_ts={ts}&profile={profile}
```

### Query Parameters

| Param | Required | Default | Description |
|-------|----------|---------|-------------|
| `coin` | yes | - | Asset symbol (BTC, ETH, etc.) |
| `account_value` | yes | - | Account value in USD |
| `pnl_day` | no | 0 | Daily PnL in USD |
| `position_size` | no | 0 | Current position size (absolute) |
| `position_side` | no | null | "long" or "short" if in position |
| `liq_price` | no | null | Liquidation price if in position |
| `last_fill_ts` | no | null | Timestamp (ms) of last fill for cooldown |
| `profile` | no | "auto" | Profile override: auto, aggressive, balanced, conservative |

## SSE Event Format

Events pushed every 60 seconds:

```json
{
  "coin": "BTC",
  "timestamp": "2026-01-09T15:30:00Z",
  "profile": "balanced",
  "metrics": {
    "mid": 91500.0,
    "spread": 0.00011,
    "ob_imbalance": 1.42,
    "sma20": 91767.7,
    "sma50": 91094.86,
    "don_hi": 91627.0,
    "don_lo": 89645.0,
    "atr": 475.43,
    "atr_pct": 0.0052,
    "trend_strength": 0.0074,
    "pnl_day_pct": -0.005,
    "liq_dist_pct": 0.15
  },
  "signals": {
    "bull": true,
    "strong_long": true,
    "strong_short": false
  },
  "gates": {
    "daily_lock": false,
    "cooldown_ok": true,
    "entry_ok": true
  },
  "recommendation": {
    "action": "ENTER_LONG",
    "size": 0.0043,
    "leverage": 3,
    "stop_loss": 90812.36,
    "stop_dist": 687.64,
    "risk_usd": 40.0,
    "max_notional": 30000.0
  }
}
```

### Recommendation Actions

| Action | Description |
|--------|-------------|
| `ENTER_LONG` | Open long position |
| `ENTER_SHORT` | Open short position |
| `CLOSE` | Close current position (opposite signal) |
| `HOLD` | Keep current position |
| `NOOP` | No action (flat, no signal or gate blocked) |

## Recipe Implementation

### Profile Parameters

```
Common: donLen=20, atrLen=14, minStopDistPct=0.004, maxLev=5, minPnlEntryGate=-0.01

AGG: riskPct=0.006, maxNotMult=4.0, stopATR=1.2, cdMin=45, dayCut=0.035, obL=1.05, obS=0.95, sprMax=0.0012
BAL: riskPct=0.004, maxNotMult=3.0, stopATR=1.5, cdMin=60, dayCut=0.030, obL=1.10, obS=0.90, sprMax=0.0008
CON: riskPct=0.002, maxNotMult=1.5, stopATR=2.0, cdMin=120, dayCut=0.020, obL=1.15, obS=0.87, sprMax=0.0006
```

### Candle Rules

- Fetch 60x 4h candles, 45x 1h candles
- Drop most recent candle (use closed only)
- Require: 4h >= 55 closed, 1h >= 40 closed
- SMA20/50 on 4h close prices
- Donchian (20) on 1h high/low
- ATR (14) on 1h candles

### Derived Metrics

```
mid = (best_bid + best_ask) / 2
spread = (best_ask - best_bid) / mid
ob_imbalance = sum(bid_sz[0..9]) / sum(ask_sz[0..9])
atr_pct = atr / mid
trend_strength = abs(sma20 - sma50) / mid
pnl_day_pct = pnl_day / account_value
liq_dist_pct = (mid - liq_price) / mid  # for long; inverse for short
```

### Profile Selection (auto)

```
if pnl_day_pct <= -0.02
   OR (liq_dist_pct != null AND liq_dist_pct <= 0.07)
   OR atr_pct >= 0.06
   OR spread >= 0.0014:
    profile = CON
else if trend_strength >= 0.003
   AND 0.015 <= atr_pct <= 0.05
   AND spread <= 0.0011
   AND pnl_day_pct >= -0.01
   AND (liq_dist_pct == null OR liq_dist_pct >= 0.10):
    profile = AGG
else:
    profile = BAL
```

### Signal Evaluation

```
bull = sma20 > sma50
strong_long = bull AND mid > don_hi AND ob_imbalance >= obL AND spread <= sprMax
strong_short = !bull AND mid < don_lo AND ob_imbalance <= obS AND spread <= sprMax
```

### Gates

```
daily_lock = pnl_day <= -(dayCut * account_value)
cooldown_ok = last_fill_ts == null OR (now - last_fill_ts) >= cdMin * 60 * 1000
entry_ok = !daily_lock AND pnl_day_pct >= minPnlEntryGate AND cooldown_ok AND (strong_long OR strong_short)
```

### Position Sizing (when entry_ok)

```
stop_dist = max(stopATR * atr, minStopDistPct * mid)
max_notional = maxNotMult * account_value
risk_usd = riskPct * account_value
raw_size = risk_usd / stop_dist
cap_size = max_notional / mid
size = min(raw_size, cap_size)
leverage = clamp(1, maxLev, atr_pct > 0.04 ? 2 : (atr_pct < 0.02 ? 4 : 3))
stop_loss = mid - stop_dist  # for long; mid + stop_dist for short
```

### Action Logic

```
if position_size > 0:  # has position
    dir = position_side
    opposite_signal = (dir == "long" AND strong_short) OR (dir == "short" AND strong_long)
    if opposite_signal:
        action = CLOSE
    else:
        action = HOLD
else:  # flat
    if !entry_ok:
        action = NOOP
    else if strong_long:
        action = ENTER_LONG
    else if strong_short:
        action = ENTER_SHORT
    else:
        action = NOOP
```

## Implementation Notes

1. Reuse `HyperliquidClient` from `AppState` for candle and orderbook fetches
2. Evaluation runs every 60 seconds per connection
3. Only push event if data successfully fetched (skip on error, retry next tick)
4. Single handler file: `src/handlers/signal_stream.rs`
5. Register route: `GET /signals/stream`

## Error Handling

On fetch failure, push error event:
```json
{
  "error": "failed to fetch orderbook",
  "timestamp": "2026-01-09T15:30:00Z"
}
```

Client should handle reconnection on stream close.
