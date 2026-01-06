# Perp Screener

Rust backend + React frontend for streaming perp market data and pattern signals over HTTP + SSE, powered by Hyperliquid candle snapshots.

## What it does
- Backend (Rust/axum): HTTP + SSE endpoints for chart candles, VWAP, double tops, core patterns, and advanced patterns; background monitors feed shared state; OpenAPI served at `/api-docs/openapi.json` with Swagger UI.
- Frontend (React/Vite): Screener table and Pattern Screening views consuming SSE streams for double tops, VWAP, and pattern detections; hash-based routing; dev proxy to the backend.
- Data source: Hyperliquid `candleSnapshot` at `https://api.hyperliquid.xyz/info` (see `spec/hl.md`; docs: hyperliquid.gitbook.io/.../info-endpoint#candle-snapshot).
- SSE behavior: All streams emit `snapshot` events with 15s keep-alives. VWAP stream snapshots every 60s, emits `heartbeat` if idle >90s, and supports `Last-Event-ID` resume.

## Status

| Area | Status | Notes |
| ---- | ------ | ----- |
| Chart candles | Implemented | GET + SSE; 14 intervals; 5000-candle cap |
| Double top detection | Implemented | Background monitor every 60s; snapshot + SSE |
| Core patterns | Implemented | Snapshot + SSE with summaries and per-coin/interval filters |
| Advanced patterns | Implemented | Monitor every 300s; snapshot + SSE |
| VWAP streaming | Implemented | Snapshot every 60s; heartbeat after 90s idle; Last-Event-ID supported |
| VWAP snapshot (GET) | Implemented | Same payload as SSE snapshot |

## Backend endpoints
- `GET /health` → `{ "status": "healthy" }`.
- `GET /chart` / `GET /chart/stream`
  - Params: `coin` (required), `interval` (required: 1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 8h, 12h, 1d, 3d, 1w, 1M), `limit` (default 200, max 5000).
  - SSE cadence: one snapshot per interval length.
- `GET /double-top` / `GET /double-top/stream`
  - Monitors `BTC`, `ETH`, `SOL` every 60s.
- `GET /patterns` / `GET /patterns/stream`
  - Filters: `coins`, `intervals`, `limit` (1–200, default 25), `since_ms`.
  - Initial snapshot then broadcast snapshots on updates.
- `GET /patterns/advanced` / `GET /patterns/advanced/stream`
  - Advanced detections emitted from the 300s monitor.
- `GET /vwap` / `GET /vwap/stream`
  - Params: `coin` (required), `timeframes` (default `session,4h`; supports session, 1h, 4h, weekly, monthly), `bands` (bool, default true), `interval` (defaults to 1m for intraday-only, else 1h).
  - SSE: snapshot every 60s; `heartbeat` event if no snapshot in 90s; respects `Last-Event-ID` header for resume.

## Background jobs
- Candle ingestion: pulls Hyperliquid candles every 60s for BTC/ETH/SOL across 14 intervals (1m–1M); enforces 5000-candle limit per request.
- Double top monitor: every 60s.
- Core pattern monitor: every 60s over ingestion intervals.
- Advanced pattern monitor: every 300s over ingestion intervals.

## Frontend (frontend/)
- Vite + React + TypeScript.
- Screens: Screener table (double tops + VWAP) and Pattern Screening (core + advanced lists, filters, weights, summaries).
- Data: SSE hooks for double tops, patterns, advanced patterns, and VWAP; hash-based navigation between views.
- Dev proxy: routes `/double-top`, `/patterns`, `/vwap`, `/chart`, `/health` to `http://localhost:30001` (see `frontend/vite.config.ts`).
- Scripts (bun): `bunx vite` (dev), `bunx vite build` (build), `bunx vite preview`, `bunx vitest run` (tests), `eslint "src/**/*.{ts,tsx}"` (lint).

## Quickstart (backend)
Requirements: Rust stable toolchain.

Build:
```bash
cargo build
```

Run:
```bash
cargo run
```

Dev (auto rebuild/restart):
```bash
cargo install bacon
bacon run
```

Server: http://localhost:30001
Swagger UI: http://localhost:30001/swagger-ui
OpenAPI JSON: http://localhost:30001/api-docs/openapi.json

## Frontend dev
```bash
cd frontend
bun install
bunx @react-grab/codex@latest && bunx vite         # dev
bunx vite build                                     # build
bunx vitest run                                     # tests
```

## Tests & lint
- Backend: `cargo test`
- Frontend: `cd frontend && bunx vitest run` (watch: `bunx vitest`)
- Lint frontend: `cd frontend && bun run lint`

## Specs
Specs live in `spec/`:
- `spec/vwap_sse_get.md`
- `spec/chart_sse.md`
- `spec/double_top_sse.md`
- `spec/double_top_detection.md`
- `spec/patterns_screening.md`

## Project structure
```
src/                        # Rust backend
frontend/                   # React/Vite frontend
spec/                       # API and feature specs
```
