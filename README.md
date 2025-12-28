# Perp Screener

Rust service for streaming perp market data and pattern signals over HTTP + SSE, backed by Hyperliquid candle snapshots.

## Status

| Area | Status | Notes |
| ---- | ------ | ----- |
| Chart candles | Implemented | Snapshot + SSE streaming |
| Double top detection | Implemented | Background monitor + snapshot + SSE |
| VWAP streaming | Implemented (partial) | SSE snapshots only; no heartbeat event or Last-Event-ID support |
| VWAP snapshot (GET) | Planned | Spec tracked in `spec/vwap_sse_get.md` |

## Quickstart

Requirements: Rust stable toolchain.

Build:
```bash
cargo build
```

Run:
```bash
cargo run
```

Dev (auto rebuild/restart on changes):
```bash
cargo install bacon
bacon run
```

Server: http://localhost:3000  
Swagger UI: http://localhost:3000/swagger-ui  
OpenAPI JSON: http://localhost:3000/api-docs/openapi.json

## API

All SSE endpoints emit `snapshot` events. Axum keep-alive comments are sent every 15 seconds.

### Health

`GET /health` returns `{ "status": "healthy" }`.

### Chart Candles

- `GET /chart`
- `GET /chart/stream`

Query params:
- `coin` (string, required)
- `interval` (string, required): `1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 8h, 12h, 1d, 3d, 1w, 1M`
- `limit` (int, optional, default 200, max 5000)

SSE cadence: one snapshot per interval length.

### Double Top Patterns

- `GET /double-top`
- `GET /double-top/stream`

The background monitor runs every 60 seconds and currently tracks `BTC`, `ETH`, `SOL` (see `src/main.rs`).

### VWAP

- `GET /vwap/stream`

Query params:
- `coin` (string, required)
- `timeframes` (string, optional, default `session,4h`): `session, 1h, 4h, weekly, monthly`
- `bands` (bool, optional, default true)
- `interval` (string, optional): defaults to `1m` for intraday-only timeframes, otherwise `1h`

Notes:
- Enforces the 5000-candle limit based on the requested timeframe and interval.
- Current implementation emits snapshots every 60 seconds.

## Data Sources

Hyperliquid `POST https://api.hyperliquid.xyz/info` using `candleSnapshot` requests (see `spec/hl.md`).

## Specs

Specs live in `spec/`:
- `spec/vwap_sse_get.md`
- `spec/chart_sse.md`
- `spec/double_top_sse.md`
- `spec/double_top_detection.md`

## Tests

```bash
cargo test
```

## Project Structure

```
src/
├── main.rs              # Entry point, router setup
├── handlers/            # HTTP handlers
├── services/            # External API calls, orchestration
├── business_logic/      # Core algorithms, pattern detection
├── models/              # Request/response DTOs
├── errors/              # AppError and error responses
└── state.rs             # Shared application state
```
