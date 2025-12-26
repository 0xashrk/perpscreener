# Double Top SSE Streaming Spec

## Overview

Hyperliquid does not provide a live 1m candle stream for this use case, so the backend will continue polling candles once per minute. An SSE endpoint is provided to push updates to clients without them polling `/double-top`.

SSE only changes how clients receive updates; it does not change the polling cadence against Hyperliquid.

---

## Endpoint

**Method:** `GET /double-top/stream`  
**Response:** `text/event-stream`

**Headers:**
- `Content-Type: text/event-stream`
- `Cache-Control: no-cache`
- `Connection: keep-alive`

---

## Event Types

### `snapshot`

Sent on connect and after each monitoring cycle (once per minute) with the latest state for all coins.

```
event: snapshot
id: <epoch_ms>
data: {"as_of_ms": 1735689600000, "patterns": [ ... ]}
```

Payload:
```json
{
  "as_of_ms": 1735689600000,
  "patterns": [
    {
      "coin": "BTC",
      "state": "TROUGH_FOUND",
      "peak1_price": 105.0,
      "neckline_price": 98.5,
      "peak2_price": null,
      "is_warmed_up": true
    }
  ]
}
```

### `heartbeat`

Optional keepalive if no new snapshot was emitted within the last interval.

```
event: heartbeat
id: <epoch_ms>
data: {"as_of_ms": 1735689600000}
```

---

## Data Model

`patterns` uses the same shape as the `/double-top` response:

```json
{
  "coin": "BTC",
  "state": "WATCHING|PEAK_FOUND|TROUGH_FOUND|FORMING|CONFIRMED|INVALIDATED",
  "peak1_price": 0.0,
  "neckline_price": 0.0,
  "peak2_price": 0.0,
  "is_warmed_up": true
}
```

---

## Polling and Update Cadence

- Hyperliquid is polled every 60 seconds for closed 1m candles.
- Only candles with `close_time <= now - 60s` are processed.
- A `snapshot` is emitted after each poll cycle.
- No sub-minute updates are available because the source is polled.

---

## Reconnect Behavior

- Clients should reconnect automatically if the stream drops.
- The server includes an `id` (epoch ms). Clients may set `Last-Event-ID` on reconnect.
- On reconnect, the server should emit an immediate `snapshot` regardless of `Last-Event-ID`.

---

## Error Handling

- On internal errors, close the stream; clients reconnect.
- Log errors server-side; do not emit partial payloads.

