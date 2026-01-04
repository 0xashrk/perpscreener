# Pattern Live Canvas

## Goal
Replace the current Pattern Screening "Live Canvas" stub with a real candlestick view that overlays detected patterns for the selected token/interval. The canvas should live in the Pattern Screening page and keep the existing visual language (glass panels, rounded cards).

## Scope
- Frontend-only visualization using existing backend chart and pattern streams.
- No new backend logic beyond consuming existing chart stream.

## Data Sources
- Chart stream: `GET /chart/stream` (SSE snapshot events).
- Core patterns: `GET /patterns/stream`.
- Advanced patterns: `GET /patterns/advanced/stream`.

## UX Requirements
- Render a candlestick chart for a selected token + interval.
- Overlay recent pattern detections (for the selected token/interval) on the chart.
- Provide token and interval selectors inside the canvas card (does not affect list filters).
- Show stream status for chart + patterns.
- Preserve the existing panel hierarchy and layout on `PatternScreeningPage`.

## Rendering Rules
- Use a lightweight SVG candlestick renderer (no external chart library).
- Color candles: bullish (close >= open) = green, bearish = red.
- Compute bounds from candle highs/lows; pad vertically to avoid flat charts.
- Overlay markers based on `window_end_ms` (fallback to `detected_at_ms`) mapped to closest candle time.
- Display up to 5 most recent pattern markers for the selected token/interval.

## Empty/Error States
- If chart stream is loading, show a neutral placeholder in the chart area.
- If chart stream errors, show the error message in the canvas card.
- If no pattern detections match the selection, render the chart without markers and show “No overlays yet.”

## Tests
- Add a parser test for chart snapshot parsing.
- Add a component test confirming the canvas renders with a mocked chart stream.

## File Targets
- `frontend/src/features/patterns/PatternLiveCanvas.tsx`
- `frontend/src/hooks/useChartStream.tsx`
- `frontend/src/services/chart.ts`
- `frontend/src/types/chart.ts`
- `frontend/src/__tests__/chartParser.test.ts`
- `frontend/src/__tests__/PatternLiveCanvas.test.tsx`
- Update `frontend/src/features/patterns/PatternScreeningPage.tsx` to use the new canvas.
