# Perp Screener Frontend UI Spec

## Index

| Phase | Scope | Implementation Status |
| ----- | ----- | --------------------- |
| 1 | Multi-token screener view (Double Top + VWAP summary) | Implemented |
| 2 | Optional token detail drawer (expanded VWAP + pattern detail) | Not started |

---

## Overview

Build a trader-focused screener that highlights two live signals across multiple tokens: Double Top pattern detection and VWAP levels by timeframe. The primary view is a multi-token table so users can quickly scan for setups.

The UI starts with 3 tokens and should be easy to extend to more.

---

## Goals

- Provide a clean, fast multi-token screener.
- Surface Double Top state clearly per token (forming/confirmed/invalidated).
- Surface VWAP summary per token (per interval with distance and position).
- Support streaming updates via SSE.

---

## Non-Goals

- No additional pattern types beyond Double Top (for now).
- No single-token dashboard in Phase 1.
- No alerts/notifications.

---

## UX Layout (Phase 1)

### Top Bar
- Token filter (initially 3 tokens, expandable later).
- Timeframe filter (applies to pattern display and VWAP columns).
- Stream status indicator (connected/reconnecting/error).

### Screener Table (Primary View)

Columns (Phase 1):
- Token
- Double Top State (forming/confirmed/invalidated)
- Double Top Timeframe
- Double Top Age (time since last update)
- VWAP Session Position (above/below + distance %)
- VWAP 1h Position (above/below + distance %)
- VWAP 4h Position (above/below + distance %)

Notes:
- Keep the table dense and sortable.
- Use minimal color cues for state (green/red/gray).
- Include an on-screen legend describing the Double Top state meanings:
  - Watching: waiting for a first peak
  - Peak Found: first peak confirmed
  - Trough Found: pullback/neckline formed
  - Forming: price approaching the first peak
  - Confirmed: breakdown below neckline
  - Invalidated: pattern failed (broke above peak or timed out)

---

## Data Sources

### Double Top
- `GET /double-top/stream`
- Stream includes multiple coins; filter client-side for the selected token.
- Use `snapshot` events; ignore `heartbeat` events in UI except for status.

### VWAP
- `GET /vwap/stream?coin=<COIN>&timeframes=session,1h,4h&interval=1m`
- Use `snapshot` events; show last valid snapshot.
- For multiple tokens, maintain one stream per token (initially 3).
- If a stream fails, allow fallback to `GET /vwap` for that token.

---

## Interaction Flow

- Default to the configured token list on load.
- Double Top stream remains shared; UI maps per-coin rows.
- VWAP streams are per token; update table cells on each snapshot.
- Keep latest valid snapshot for each token; show stale state if a stream drops.

---

## Error, Empty, and Loading States

- Loading: show skeleton rows in widgets and a chart loader.
- Empty pattern: show "No Double Top detected" with timestamp.
- Stream error: banner in top bar; keep last snapshot visible.

---

## Visual and Code Standards

- Use Tailwind CSS for all styling (no custom CSS unless explicitly approved).
- Follow frontend guidelines: no `any`, `unknown`, `undefined`, or `null` in app types.
- Clean up legacy patterns in touched areas as features evolve.

---

## Phase 2: Optional Detail Drawer

- Row click opens a detail drawer for the selected token.
- Drawer shows full Double Top details and expanded VWAP table.
- Phase 1 screener remains the primary view.

---

## Test Plan (Required)

- **Component tests:** Render Double Top widget states and VWAP table rows.
- **Stream handling:** Mock SSE for chart/VWAP/patterns; verify reconnect behavior and stale-state display.
- **Interaction tests:** Token switch updates widgets and chart data.
- **Error states:** Stream failure retains last snapshot and shows status banner.
