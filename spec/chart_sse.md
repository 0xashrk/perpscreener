# Chart Candle SSE Spec

## Overview

Provide an SSE endpoint that streams candle snapshots from Hyperliquid using the `candleSnapshot`
API. Clients can select the candle interval (all Hyperliquid-supported intervals are allowed).

Only the most recent 5000 candles are available from Hyperliquid; the server enforces this limit.

---

## Endpoint

**Method:** `GET /chart/stream`  
**Response:** `text/event-stream`

**Headers:**
- `Content-Type: text/event-stream`
- `Cache-Control: no-cache`
- `Connection: keep-alive`

---

## Query Parameters

| Name | Type | Required | Description |
| ---- | ---- | -------- | ----------- |
| `coin` | String | yes | Coin symbol (e.g., `BTC`) |
| `interval` | String | yes | Candle interval; must be one of the supported intervals |
| `limit` | Integer | no | Number of candles to return (default: 200, max: 5000) |

### Supported Intervals

`"1m"`, `"3m"`, `"5m"`, `"15m"`, `"30m"`, `"1h"`, `"2h"`, `"4h"`, `"8h"`, `"12h"`, `"1d"`, `"3d"`, `"1w"`, `"1M"`

---

## Event Types

### `snapshot`

Sent on connect and then once per `interval` with the latest candle snapshot.

```
event: snapshot
id: <epoch_ms>
data: {"as_of_ms": 1735689600000, "coin": "BTC", "interval": "15m", "candles": [ ... ]}
```

Payload:
```json
{
  "as_of_ms": 1735689600000,
  "coin": "BTC",
  "interval": "15m",
  "candles": [
    {
      "t": 1681923600000,
      "T": 1681924499999,
      "o": 29295.0,
      "h": 29309.0,
      "l": 29250.0,
      "c": 29258.0,
      "v": 0.98639,
      "n": 189,
      "i": "15m",
      "s": "BTC"
    }
  ]
}
```

---

## Polling and Update Cadence

- The server polls Hyperliquid using `candleSnapshot` with the requested `interval`.
- Poll interval is aligned with the requested candle interval.
- Each poll emits a full snapshot of up to `limit` most recent candles.
- The server fills in `i` and `s` fields if the upstream response omits them.

---

## Error Handling

- If validation fails (invalid `interval`, `coin`, or `limit`), return `400` with a JSON error body.
- If the Hyperliquid request fails, close the stream; clients should reconnect.
- The server should log upstream errors.

---

## Test Cases

### Test 1: Valid Interval (Accepted)

**Request:**
```
GET /chart/stream?coin=BTC&interval=15m
```

**Expected:**
- Status `200`
- Initial `snapshot` event emitted
- `snapshot.interval == "15m"`

---

### Test 2: Invalid Interval (Rejected)

**Request:**
```
GET /chart/stream?coin=BTC&interval=10m
```

**Expected:**
- Status `400`
- Error message lists supported intervals

---

### Test 3: Limit Default

**Request:**
```
GET /chart/stream?coin=ETH&interval=1m
```

**Expected:**
- Status `200`
- Snapshot contains <= 200 candles

---

### Test 4: Limit Upper Bound (Accepted)

**Request:**
```
GET /chart/stream?coin=ETH&interval=1h&limit=5000
```

**Expected:**
- Status `200`
- Snapshot contains <= 5000 candles

---

### Test 5: Limit Above Bound (Rejected)

**Request:**
```
GET /chart/stream?coin=ETH&interval=1h&limit=5001
```

**Expected:**
- Status `400`
- Error message indicates `limit` must be between 1 and 5000

---

### Test 6: Snapshot Fields Filled

**Setup:** Mock upstream candles missing `i`/`s` fields

**Expected:**
- Server fills `i` with requested interval
- Server fills `s` with requested coin

---

### Test 7: Polling Cadence

**Request:**
```
GET /chart/stream?coin=SOL&interval=5m
```

**Expected:**
- After initial snapshot, server emits next snapshot ~5 minutes later
- No more frequent snapshots unless the client reconnects

---

### Test 8: Upstream Failure (Stream Ends)

**Setup:** Hyperliquid returns an error or times out

**Expected:**
- SSE stream closes
- Server logs the error

