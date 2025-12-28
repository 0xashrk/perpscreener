# Typed Query Parameters Spec

## Index

| Phase | Scope | Status |
| ----- | ----- | ------ |
| 1 | Introduce enums for candle intervals and VWAP timeframes while keeping query shapes stable | Implemented |
| 2 | Improve OpenAPI documentation for comma-separated timeframes | Implemented |

## Overview

Move interval and timeframe inputs from stringly-typed parsing to enums for safer validation, cleaner code, and clearer API docs. Preserve existing query parameter shapes (no breaking changes to clients).

## Goals

- Use enums for candle intervals and VWAP timeframes in models, services, and handlers.
- Keep query parameters backward compatible:
  - `interval=1m` remains a string in the URL.
  - `timeframes=session,4h` remains a comma-separated string in the URL.
- Ensure invalid values return `400` with a helpful message.
- Improve OpenAPI schema clarity for intervals/timeframes.

## Non-Goals

- Changing the wire format to arrays or repeated query params.
- Adding new timeframes or intervals.
- Implementing new endpoints.

## API Behavior

### Chart Endpoints

- `interval` query param is deserialized into `CandleInterval`.
- Allowed values: `1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 8h, 12h, 1d, 3d, 1w, 1M`.
- Invalid values return `400`.

### VWAP Endpoints

- `timeframes` query param remains a comma-separated string and is parsed into `Vec<VwapTimeframe>`.
- Allowed values: `session, 1h, 4h, weekly, monthly`.
- Duplicates are ignored; whitespace is trimmed.
- Empty or invalid input returns `400`.
- `interval` query param is deserialized into `CandleInterval` when provided.
- Default interval behavior is unchanged:
  - Intraday-only timeframes default to `1m`.
  - Weekly/monthly timeframes default to `1h`.

## Data Model

### CandleInterval

Enum values:
`1m`, `3m`, `5m`, `15m`, `30m`, `1h`, `2h`, `4h`, `8h`, `12h`, `1d`, `3d`, `1w`, `1M`.

### VwapTimeframe

Enum values:
`session`, `1h`, `4h`, `weekly`, `monthly`.

### TimeframeList

Wrapper type to parse `timeframes` from a comma-separated query string into `Vec<VwapTimeframe>`.

## OpenAPI Documentation

- `CandleInterval` should render as an enum for `interval` query params.
- `timeframes` remains a string param with an example and an explicit allowed-values description.

## Error Handling

- Invalid `interval` returns `400` with a message listing allowed values.
- Invalid `timeframes` returns `400` with a message listing allowed values.

## Test Plan

- Interval enum parsing and `ms()` mapping.
- Timeframe list parsing (valid, invalid, duplicates, whitespace).
- Default interval selection for VWAP (intraday vs swing timeframes).

## Phase 1 Details

- Introduce `CandleInterval` and `VwapTimeframe` enums in models.
- Use typed enums in chart and VWAP query models.
- Update services/handlers to accept enums.
- Add/adjust unit tests.

## Phase 2 Details

- Improve OpenAPI for `timeframes`:
  - Explicitly document allowed values in the schema description.
  - Keep query shape unchanged.
