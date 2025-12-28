# VWAP SSE Streaming Spec

## Index

| Phase | Endpoint | Scope | Implementation Status |
| ----- | -------- | ----- | --------------------- |
| 1 | `GET /vwap` | Snapshot payload (same shape as SSE `snapshot` data) | TBD |
| 2 | `GET /vwap/stream` | SSE stream of snapshots and heartbeats | TBD |

## Overview

Provide an SSE endpoint that streams Volume Weighted Average Price (VWAP) data for multiple timeframes. Primary focus is day trading with session-anchored VWAP, with additional timeframes for swing trading context.

VWAP is calculated as: `Σ(Typical Price × Volume) / Σ(Volume)`
Where Typical Price = `(High + Low + Close) / 3`

---

## Data Source

**Endpoint:** `POST https://api.hyperliquid.xyz/info`

**Request:**
```json
{
  "type": "candleSnapshot",
  "req": {
    "coin": "<coin>",
    "interval": "<interval>",
    "startTime": <epoch_millis>,
    "endTime": <epoch_millis>
  }
}
```

**Notes:**
- Only the most recent 5000 candles are available.
- Supported intervals: `"1m"`, `"3m"`, `"5m"`, `"15m"`, `"30m"`, `"1h"`, `"2h"`, `"4h"`, `"8h"`, `"12h"`, `"1d"`, `"3d"`, `"1w"`, `"1M"`

---

## Phased Delivery

- Phase 1: `GET /vwap` returns a single snapshot payload (same shape as the SSE `snapshot` event data) to unblock integration and testing.
- Phase 2: `GET /vwap/stream` provides the SSE stream as defined in this spec.

---

## Snapshot Endpoint (Phase 1)

**Method:** `GET /vwap`
**Response:** `application/json`

**Headers:**
- `Content-Type: application/json`
- `Cache-Control: no-cache`

**Response Body:** Identical to the SSE `snapshot` event data payload.

---

## SSE Endpoint (Phase 2)

**Method:** `GET /vwap/stream`
**Response:** `text/event-stream`

**Headers:**
- `Content-Type: text/event-stream`
- `Cache-Control: no-cache`
- `Connection: keep-alive`

---

## Query Parameters (Shared)

These parameters apply to both `GET /vwap` and `GET /vwap/stream`.

| Name | Type | Required | Description |
| ---- | ---- | -------- | ----------- |
| `coin` | String | yes | Coin symbol (e.g., `BTC`) |
| `timeframes` | String | no | Comma-separated list of VWAP timeframes (default: `session,4h`) |
| `bands` | Boolean | no | Include standard deviation bands (default: `true`) |
| `interval` | String | no | Candle interval used for VWAP (default: `1m`) |

### Supported Timeframes

| Timeframe | Anchor Point | Use Case |
| --------- | ------------ | -------- |
| `session` | Daily 00:00 UTC | Day trading (primary) |
| `4h` | Rolling 4-hour window | Intraday momentum |
| `1h` | Rolling 1-hour window | Scalping reference |
| `weekly` | Monday 00:00 UTC | Swing trading |
| `monthly` | 1st of month 00:00 UTC | Swing trading |

**Day Trading Recommended:** `session,1h,4h`
**Swing Trading Recommended:** `session,weekly,monthly`

### Recommended Interval by Trading Style

Because Hyperliquid only returns the most recent 5000 candles, pick an interval that can cover the requested timeframe:

| Trading Style | Timeframes | Recommended Interval | Why |
| ------------ | ---------- | -------------------- | --- |
| Day trading | `session,1h,4h` | `1m` or `3m` | High resolution, enough depth for intraday |
| Swing trading | `session,weekly,monthly` | `1h` (or `4h`) | Enough depth to cover weekly/monthly anchors |

If `interval` is omitted, default to:
- `1m` when all requested timeframes are intraday (`session`, `1h`, `4h`)
- `1h` when any of `weekly` or `monthly` is requested

---

## Event Types

### `snapshot`

Sent on connect and every 60 seconds with updated VWAP values.

```
event: snapshot
id: <epoch_ms>
data: {"as_of_ms": 1735689600000, "coin": "BTC", "vwaps": [ ... ]}
```

Payload:
```json
{
  "as_of_ms": 1735689600000,
  "coin": "BTC",
  "current_price": 98500.0,
  "vwaps": [
    {
      "timeframe": "session",
      "anchor_time_ms": 1735689600000,
      "vwap": 97850.25,
      "cumulative_volume": 1234.56,
      "distance_pct": 0.66,
      "position": "above",
      "upper_band_1": 98200.50,
      "lower_band_1": 97500.00,
      "upper_band_2": 98550.75,
      "lower_band_2": 97149.75
    },
    {
      "timeframe": "4h",
      "anchor_time_ms": 1735675200000,
      "vwap": 98100.00,
      "cumulative_volume": 456.78,
      "distance_pct": 0.41,
      "position": "above",
      "upper_band_1": 98350.00,
      "lower_band_1": 97850.00,
      "upper_band_2": 98600.00,
      "lower_band_2": 97600.00
    }
  ]
}
```

### `heartbeat`

Sent if no snapshot was emitted within 90 seconds (keepalive).

```
event: heartbeat
id: <epoch_ms>
data: {"as_of_ms": 1735689600000}
```

---

## Data Model

### VWAP Entry

| Field | Type | Description |
| ----- | ---- | ----------- |
| `timeframe` | String | Timeframe identifier (`session`, `4h`, etc.) |
| `anchor_time_ms` | Integer | Epoch ms when this VWAP period started |
| `vwap` | Float | Current VWAP value |
| `cumulative_volume` | Float | Total volume since anchor |
| `distance_pct` | Float | Percentage distance from current price to VWAP |
| `position` | String | `"above"` or `"below"` VWAP |
| `upper_band_1` | Float | +1 standard deviation (if `bands=true`) |
| `lower_band_1` | Float | -1 standard deviation (if `bands=true`) |
| `upper_band_2` | Float | +2 standard deviation (if `bands=true`) |
| `lower_band_2` | Float | -2 standard deviation (if `bands=true`) |

Band fields are omitted if `bands=false`.

---

## Polling and Update Cadence

- VWAP is recalculated every 60 seconds using closed candles from Hyperliquid at the selected `interval`.
- Session VWAP resets at 00:00 UTC daily.
- Rolling timeframes (1h, 4h) use a sliding window of closed candles.
- Weekly resets Monday 00:00 UTC; Monthly resets 1st of month 00:00 UTC.
-
- If the requested timeframe cannot be covered within 5000 candles for the selected interval, return `400` with a message indicating to use a larger interval.

---

## Trading Signals (Informational)

The server may optionally include a `signals` array for actionable context:

```json
{
  "signals": [
    {
      "type": "vwap_touch",
      "timeframe": "session",
      "message": "Price touched session VWAP from above"
    },
    {
      "type": "band_touch",
      "timeframe": "session",
      "band": "upper_2",
      "message": "Price at +2σ session VWAP band (overextended)"
    }
  ]
}
```

Signal types:
- `vwap_cross_up` — Price crossed above VWAP
- `vwap_cross_down` — Price crossed below VWAP
- `vwap_touch` — Price touched VWAP (within 0.1%)
- `band_touch` — Price touched a deviation band

---

## Reconnect Behavior

- Clients should reconnect automatically if the stream drops.
- Server includes `id` (epoch ms); clients may set `Last-Event-ID` on reconnect.
- On reconnect, server emits an immediate `snapshot`.

---

## Error Handling

- Invalid `coin` or `timeframes` returns `400` with JSON error body.
- On internal/upstream errors, close the stream; clients should reconnect.
- Log errors server-side; do not emit partial payloads.

---

## Test Cases

### Phase 1: GET Snapshot Endpoint

### Test 1: Default Timeframes (GET)

**Request:**
```
GET /vwap?coin=BTC
```

**Expected:**
- Status `200`
- Response body contains `session` and `4h` VWAPs
- All band fields present

---

### Test 2: Custom Timeframes (GET)

**Request:**
```
GET /vwap?coin=ETH&timeframes=session,weekly,monthly
```

**Expected:**
- Status `200`
- Response body contains exactly 3 VWAP entries for requested timeframes
- Interval selection defaults to `1h` (since weekly/monthly requested)

---

### Test 3: Bands Disabled (GET)

**Request:**
```
GET /vwap?coin=SOL&bands=false
```

**Expected:**
- Status `200`
- VWAP entries do not include band fields

---

### Test 4: Invalid Timeframe (GET)

**Request:**
```
GET /vwap?coin=BTC&timeframes=session,invalid
```

**Expected:**
- Status `400`
- Error message lists supported timeframes

---

### Test 5: Candle Limit Enforcement (GET)

**Setup:** Request `monthly` with `interval=1m`

**Expected:**
- Status `400`
- Error message instructs using a larger `interval`

---

### Test 6: Payload Shape (GET)

**Request:**
```
GET /vwap?coin=BTC
```

**Expected:**
- `as_of_ms`, `coin`, `current_price`, `vwaps` present
- Each VWAP entry includes `timeframe`, `anchor_time_ms`, `vwap`, `cumulative_volume`, `distance_pct`, `position`

---

### Phase 2: SSE Streaming Endpoint

### Test 1: Default Timeframes (SSE)

**Request:**
```
GET /vwap/stream?coin=BTC
```

**Expected:**
- Status `200`
- Initial `snapshot` contains `session` and `4h` VWAPs
- All band fields present

---

### Test 2: Custom Timeframes (SSE)

**Request:**
```
GET /vwap/stream?coin=ETH&timeframes=session,weekly,monthly
```

**Expected:**
- Status `200`
- Snapshot contains exactly 3 VWAP entries for requested timeframes

---

### Test 3: Bands Disabled (SSE)

**Request:**
```
GET /vwap/stream?coin=SOL&bands=false
```

**Expected:**
- Status `200`
- VWAP entries do not include band fields

---

### Test 4: Invalid Timeframe (SSE)

**Request:**
```
GET /vwap/stream?coin=BTC&timeframes=session,invalid
```

**Expected:**
- Status `400`
- Error message lists supported timeframes

---

### Test 5: Session Reset at UTC Midnight (SSE)

**Setup:** Clock crosses 00:00 UTC

**Expected:**
- Session VWAP resets
- `anchor_time_ms` updates to new day start
- `cumulative_volume` resets

---

### Test 6: Distance Calculation (SSE)

**Setup:** Current price = 100, Session VWAP = 98

**Expected:**
- `distance_pct` = 2.04 (approx)
- `position` = "above"

---

### Test 7: Rolling Window (4h) (SSE)

**Setup:** Connect at 14:30 UTC

**Expected:**
- 4h VWAP uses candles from 12:00-14:30 UTC
- `anchor_time_ms` = 12:00 UTC epoch

---

### Test 8: Update Cadence (SSE)

**Request:**
```
GET /vwap/stream?coin=BTC
```

**Expected:**
- Initial snapshot on connect
- Next snapshot ~60 seconds later
- Values may change as new candles close

---

### Test 9: Heartbeat Emission (SSE)

**Setup:** No snapshot emitted within 90 seconds

**Expected:**
- `heartbeat` event is sent with `as_of_ms`

---

## Open Questions

1. Should we support custom anchor times for session VWAP (e.g., market open for specific exchanges)?
2. Should we add a `trend` field indicating VWAP slope direction?
3. Maximum number of timeframes per request?
