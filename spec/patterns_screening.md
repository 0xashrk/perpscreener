# Pattern Screening Spec (Multi-Pattern, Multi-Timeframe)

## Phase Index (Status)

| Phase | Scope | Status |
| --- | --- | --- |
| 0 | Data ingestion, storage, and feature precompute | Completed |
| 0a | Candle cache primitives + tests | Completed |
| 0b | Feature precompute primitives/store + tests | Completed |
| 0c | Ingestion wiring + state integration + tests | Completed |
| 0d | Frontend pattern screening scaffolding + tests | Completed |
| 1 | Core pattern detection (candlesticks + gaps) | Completed |
| 1a | Core detection data model + REST skeleton | Completed |
| 1b | Candlestick pattern detectors | Completed |
| 1c | Gap pattern detectors | Completed |
| 1d | Frontend core patterns list + filters | Completed |
| 2 | Chart patterns (continuation, reversal, channels) | Planned |
| 3 | Advanced patterns (Fibonacci, Elliott, fractals) | Planned |
| 4 | Aggregation, scoring, and client delivery (REST + SSE) | Planned |

---

## Goals

- Screen all tokens for all patterns listed in `spec/patterns/Trading Pattern Summary Table.md`.
- Classify detected patterns as bullish, bearish, or neutral with clear signal type.
- Support multi-timeframe screening using Hyperliquid candle snapshots.
- Separate "advanced" pattern results into a dedicated endpoint to control cost/complexity.

## Non-Goals

- Real-time tick-level detection (source is candle snapshots).
- Strategy optimization or backtesting.
- Final trading recommendations.

---

## Data Source

Primary candle source: `spec/hl_candle_snapshot.md`.

Constraints:
- Only the most recent 5000 candles are available per request.
- Supported intervals: "1m", "3m", "5m", "15m", "30m", "1h", "2h", "4h", "8h", "12h", "1d", "3d", "1w", "1M".
- All screening is based on these timeframes; no synthetic intervals unless explicitly defined later.

---

## Pattern Inventory and Classification

Source of truth:
- `spec/patterns/Trading Pattern Summary Table.md`
- `spec/patterns/trading_patterns_documentation.md` (formulas and definitions)

Categories:
- Candlestick: bullish and bearish reversal patterns.
- Chart patterns: continuation and reversal.
- Channels and gaps.
- Advanced: Fibonacci retracements, Elliott waves, Williams fractals.

Classification rules:
- Each detector must emit `classification` (bullish | bearish | neutral) and `signal_type` (reversal | continuation | trend | range | key_level | impulse | correction).
- Advanced category emits classification but is delivered via a separate endpoint.

---

## Phased Delivery Plan

### Phase 0: Data Ingestion and Feature Precompute

Scope:
- Candle retrieval for configured timeframes.
- Caching and incremental updates per token/timeframe.
- Shared feature precompute: pivots, trendlines, ranges, gap detection, body/range ratios, ATR/volatility.

Deliverables:
- Stable candle ingestion layer.
- Shared feature store for reuse by detectors.
- Frontend scaffolding for pattern screening views (routes, empty states, wiring stubs).

### Phase 1: Core Pattern Detection (Candlesticks + Gaps)

Scope:
- Candlestick patterns from the summary table.
- Gap patterns (breakaway, runaway, exhaustion, common).

Deliverables:
- Core detectors with confidence scoring.
- Basic REST response for latest detections.
- Frontend list/table view for core patterns with filters (coin, interval) and confidence display.

### Phase 2: Chart Patterns (Continuation, Reversal, Channels)

Scope:
- Triangles, flags, pennants, head/shoulders, double/triple tops/bottoms.
- Channels (ascending/descending/horizontal).

Deliverables:
- Pattern detectors that leverage pivots/trendlines.
- SSE stream for continuous updates.
- Frontend live updates (SSE) and initial chart overlays for continuation/reversal patterns.

### Phase 3: Advanced Patterns

Scope:
- Fibonacci retracements.
- Elliott wave 1-5 and A-B-C.
- Williams fractals (5-bar).

Deliverables:
- Advanced-only endpoint and stream.
- Heuristic transparency fields (see Data Model).
- Frontend advanced tab with visual annotations (Fibonacci levels, wave labels, fractals).

### Phase 4: Aggregation and Delivery

Scope:
- Aggregate bullish/bearish scoring across patterns and timeframes.
- Provide unified summaries and per-pattern details.
- SLA expectations and rate limiting.

Deliverables:
- Aggregated summary in core endpoint.
- Configurable weighting by timeframe and signal type.
- Frontend aggregation dashboard (bullish/bearish scores, top signals, timeframe weighting controls).

---

## API Design (High Level)

### Core Patterns

**Endpoint:** `GET /patterns`  
**Purpose:** latest pattern detections (excluding advanced category).

Optional query params:
- `coins` (comma-separated list)
- `intervals` (comma-separated list)
- `limit` (max patterns per coin/timeframe)
- `since_ms` (only return detections after timestamp)

### Core Patterns SSE

**Endpoint:** `GET /patterns/stream`  
**Response:** `text/event-stream`

Event types:
- `snapshot`: full current state (per coin/timeframe)
- `update`: incremental change
- `heartbeat`: optional keepalive

### Advanced Patterns

**Endpoint:** `GET /patterns/advanced`  
**Purpose:** Fibonacci, Elliott, and fractals only.

Optional query params mirror `/patterns`.

### Advanced Patterns SSE

**Endpoint:** `GET /patterns/advanced/stream`  
**Response:** `text/event-stream`

Event types mirror `/patterns/stream`.

---

## Data Model (Conceptual)

### Pattern Detection

```
{
  "coin": "BTC",
  "interval": "15m",
  "pattern": "Double Top",
  "category": "chart_reversal",
  "classification": "bearish",
  "signal_type": "reversal",
  "confidence": 0.78,
  "detected_at_ms": 1735689600000,
  "window_start_ms": 1735686000000,
  "window_end_ms": 1735689600000,
  "notes": "Optional short explanation"
}
```

### Advanced Detection Extensions

Advanced patterns include additional context for interpretive heuristics:

```
{
  "method": "elliott_wave",
  "basis": "pivots+trendlines",
  "assumptions": ["swing_threshold=1.5x ATR", "min_wave_length=12 bars"]
}
```

### Aggregated Summary (Core Endpoint)

```
{
  "coin": "BTC",
  "interval": "1h",
  "bullish_score": 0.62,
  "bearish_score": 0.24,
  "neutral_score": 0.14,
  "top_signals": [
    { "pattern": "Bull Flag", "classification": "bullish", "confidence": 0.81 }
  ]
}
```

---

## Scoring and Conflict Handling

- Weight by timeframe (higher timeframes > lower timeframes).
- Weight by signal type (trend/continuation signals may outweigh single-candle reversals).
- Penalize conflicts (bullish and bearish in the same window reduce confidence).
- Advanced endpoint results are not included in core scoring unless explicitly requested later.

---

## Performance and Limits

- Use incremental updates to avoid full re-scan on each poll.
- Configurable maximum coins per request and per cycle.
- Cap the number of patterns returned per coin/timeframe to reduce payload size.

---

## Open Questions

- Final list of default intervals to monitor.
- Threshold settings for pivots and volatility normalization.
- Whether advanced patterns should ever be merged into core scoring.
