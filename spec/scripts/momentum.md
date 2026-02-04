# Momentum Script Spec

CLI tool to compute the BTC intrahour momentum context defined in `recipe/MOMENTUM.md` using backend data.

## Purpose

Read-only status report for the current hour: where price sits vs the hour open, micro/meso trends, streaks, volatility, and an agreement signal.

## CLI Interface

```bash
cargo run -p momentum -- --coin BTC [--backend http://localhost:30001] [--limit 180]
```

### Arguments

| Arg | Default | Description |
|-----|---------|-------------|
| `--coin` | required | Asset symbol (BTC, ETH, etc.) |
| `--backend` | `http://localhost:30001` | Backend base URL |
| `--limit` | `180` | Number of 1m candles to pull (must cover current hour)

## Data Sources

All data fetched from backend endpoints:

| Endpoint | Data | Used For |
|----------|------|----------|
| `/chart?coin={}&interval=1m&limit={limit}` | 1m candles | All calculations (hour anchor, trends, streaks, volatility)

## Calculations (from recipe)

- `start_time_utc = floor_to_hour(now_utc)`
- Filter candles to `[start_time_utc, now_utc]`; validate alignment and continuity (gaps flagged)
- `price_to_beat = OPEN at start_time_utc`
- `current_price = last_close`
- `direction_vs_open`, `delta_price`, `delta_pct`
- Micro trend (5m): `ret_5m`, `trend_5m` with flat threshold `abs(ret_5m) < 0.0002`
- Meso trend (15m): `ret_15m`, `trend_15m` with same flat threshold
- `trend_regime` and `trend_strength` per recipe guidance (strength up when trends agree/magnitude high; down with high realized vol or choppiness)
- Target band: `proj_5m`, `proj_15m`, `target_band = [min, max]`
- Streaks over 1m candles: current streak, longest up, longest down, compact breakdown
- Volatility/range: log-return stdev (`vol_1m`), `window_high`, `window_low`, `range_pct`

## Output

1–4 line quick summary followed by a table:

| Field | Value |
|---|---|
| start_time_utc |  |
| now_utc |  |
| price_to_beat (open @ start) |  |
| current_price |  |
| direction_vs_open |  |
| delta_price |  |
| delta_pct |  |
| ret_5m |  |
| trend_5m |  |
| ret_15m |  |
| trend_15m |  |
| trend_regime |  |
| trend_strength (0..100) |  |
| target_band (5–15m) |  |
| current_streak |  |
| longest_up_streak |  |
| longest_down_streak |  |
| vol_1m |  |
| window_high |  |
| window_low |  |
| range_pct |  |
| data_quality | OK / gaps / alignment warning / missing candles |

## Error Handling

- Backend unreachable or parse error → exit with error
- Insufficient candles to cover the hour → exit with error
- Non-contiguous candles flagged in `data_quality`
- Misaligned first candle flagged in `data_quality`
