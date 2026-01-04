# Pattern State Machines Spec

## Goal

Replicate the double-top live screener behavior for every pattern defined in
`spec/patterns/trading_patterns_documentation.md` by introducing per-pattern
state machines, live transitions, and UI panels.

This spec defines a single lifecycle model and a registry-driven approach so
all patterns use real detection logic (no placeholders or hardcoded values).

## Scope

- All candlestick, gap, chart, and advanced patterns in the documentation.
- Per coin + interval lifecycle tracking.
- REST + SSE delivery for live updates.
- Frontend live panels that mirror the existing double-top monitor experience.

## Lifecycle Model

### State Enum (shared)

- `warming`: insufficient candles to evaluate the pattern window.
- `watching`: no partial match yet; scanning for the earliest required leg.
- `forming`: early legs match the documented formula; waiting on final legs.
- `confirmed`: full formula satisfied on the current candle.
- `invalidated`: pattern was forming but broke a required constraint.
- `expired`: pattern did not complete within its max window or time limit.

### Common Rules

- Each pattern defines a fixed `window` (N candles) derived from the formula.
- `forming` requires the formula clauses referencing prior candles (C1..Cn)
  to be satisfied while deferring the clauses that use the current candle (C).
- `confirmed` requires the full formula to be satisfied on the close of the
  current candle.
- `invalidated` triggers when a required constraint becomes impossible to
  satisfy before completion (gap filled, swing invalidated, breakout failed).
- `expired` triggers when the pattern window moves past the earliest forming
  candle without confirmation.

### Event Model

- `candle_update`: a new candle closes for a coin/interval.
- `forming_started`: earliest leg(s) satisfied, state enters `forming`.
- `confirmed`: full formula met; state enters `confirmed` with a confidence.
- `invalidated`: a forming pattern breaks a constraint.
- `expired`: forming state times out based on `max_age_bars`.

## Registry Model

Each pattern is defined in a registry entry (used by the engine):

- `name`
- `category` (candlestick | gap | chart | advanced)
- `classification` (bullish | bearish | neutral)
- `signal_type` (reversal | continuation | trend | range | key_level | impulse | correction)
- `window` (number of candles required)
- `max_age_bars` (timeout for forming state)
- `forming_predicates` (subset of doc formula without current-candle clauses)
- `confirm_predicates` (full doc formula)
- `invalidate_predicates` (hard breaks; gap fills, trendline breaks, etc.)

The registry is derived directly from the documentation formulas and mirrors
existing detector logic. No hardcoded or dummy transitions are allowed.

## State Machine Templates

### Candlestick Patterns (2-5 candles)

- `watching` → `forming` when the earliest candle leg(s) in the formula are
  satisfied (e.g., C2 and C1 clauses are true).
- `forming` → `confirmed` when the current candle completes the formula.
- `forming` → `invalidated` when a required gap, body ratio, or trend check is
  violated before the final candle closes.
- `forming` → `expired` when more than `max_age_bars` candles pass.

### Gap Patterns

- `watching` → `forming` when a gap is detected (current low > prior high or
  current high < prior low).
- `forming` → `confirmed` when the next candle closes without filling the gap
  (gap holds).
- `forming` → `invalidated` if the gap fills before confirmation.
- `confirmed` → `invalidated` if the gap fully fills within `max_age_bars`.

### Chart Patterns (trendlines, pivots)

- `watching` → `forming` when pivot/trendline geometry meets the documented
  pattern shape (triangle, channel, wedge, H&S, cup).
- `forming` → `confirmed` when price breaks out in the expected direction or
  completes the documented terminal leg.
- `forming` → `invalidated` when breakout occurs in the opposite direction or
  trendline integrity fails.

### Advanced Patterns

- Fibonacci retracements:
  - `forming` when a valid swing high/low pair is identified.
  - `confirmed` when price tags a target retracement band and reacts.
  - `invalidated` when price exceeds the swing extremes.
- Elliott waves:
  - `forming` when waves 1-4 are identified with valid ratios.
  - `confirmed` when wave 5 completes within ratio bounds.
  - `invalidated` when wave rules are violated.
- Williams fractals:
  - `forming` after bars 1-4 meet the fractal structure.
  - `confirmed` when bar 5 closes and validates the pivot.
  - `invalidated` when a higher high/lower low appears before confirmation.

## Pattern Registry (All Patterns)

### Candlestick Bullish

Template: Candlestick 2-5 candle lifecycle (forming on prior legs, confirm on
final candle).

- Abandoned Baby
- Belt Hold
- Breakaway
- Concealing Baby Swallow
- Doji (Dragonfly)
- Doji (Gravestone)
- Doji Star
- Engulfing
- Hammer / Dragonfly Doji
- Harami
- Harami Cross
- Homing Pigeon
- Inverted Hammer
- Kicking
- Ladder Bottom
- Mat Hold
- Matching Low
- Meeting Lines
- Morning Doji Star
- Morning Star
- Piercing Line
- Rising Three Methods
- Separating Lines
- Side by Side White Lines
- Stick Sandwich
- Three Inside Up
- Three Line Strike
- Three Outside Up
- Three Stars in the South
- Three White Soldiers
- Tri Star
- Tweezer Bottom
- Unique Three River Bottom
- Upside Gap Three Methods
- Upside Tasuki Gap

### Candlestick Bearish

Template: Candlestick 2-5 candle lifecycle (forming on prior legs, confirm on
final candle).

- Abandoned Baby
- Advance Block
- Belt Hold
- Breakaway
- Dark Cloud Cover
- Deliberation
- Downside Gap Three Methods
- Downside Tasuki Gap
- Doji Star
- Doji (Gravestone)
- Dragonfly Doji / Hanging Man
- Engulfing
- Evening Doji Star
- Evening Star
- Falling Three Methods
- Grave Stone Doji / Shooting Star
- Hanging Man
- Harami (Bearish)
- Harami Cross
- Identical Three Crows
- In Neck
- Kicking
- Meeting Lines
- On Neck
- Separating Lines
- Shooting Star
- Side-by-side White Lines
- Three Black Crows
- Three Inside Down
- Three Line Strike
- Three Outside Down
- Thrusting
- Tri Star
- Tweezer Top
- Two Crows
- Upside Gap Two Crows

### Gaps

Template: Gap lifecycle with fill tracking.

- Breakaway Gap (up/down)
- Runaway Gap (up/down)
- Exhaustion Gap (up/down)
- Common Gap

### Chart Patterns

Template: Chart lifecycle with forming geometry and breakout confirmation.

- Ascending Triangle
- Descending Triangle
- Symmetrical Triangle
- Bull Flag
- Bear Flag
- Bull Pennant
- Bear Pennant
- Rising Wedge
- Falling Wedge
- Ascending Channel
- Descending Channel
- Horizontal Channel
- Head and Shoulders
- Inverse Head and Shoulders
- Double Top
- Double Bottom
- Triple Top
- Triple Bottom
- Cup and Handle

### Advanced Patterns

Template: Advanced lifecycle (swing identified → confirmed → invalidated).

- Fibonacci 38.2% Retracement
- Fibonacci 50% Retracement
- Fibonacci 61.8% Retracement
- Elliott Wave 1-5 (bullish)
- Elliott Wave 1-5 (bearish)
- Elliott Wave A-B-C
- Williams Fractal (up)
- Williams Fractal (down)

## Implementation Plan (Phase 6)

### 6a. Registry + Lifecycle Schema

- Add `PatternLifecycleState`, `PatternLifecycleEvent`, `PatternLifecycleEntry`.
- Build a registry for all patterns above, including per-candle predicates.
- Unit tests for registry coverage (no pattern missing from the doc).

### 6b. Candlestick State Machines

- Generate forming/confirm predicates from the registry.
- Track per coin/interval/pattern state transitions.
- Tests with synthetic candles for forming, confirm, invalidate, expire.

### 6c. Gap State Machines

- Track gap opening, fill, and expiration.
- Tests for partial fill, full fill, and confirmation paths.

### 6d. Chart Pattern State Machines

- Use existing feature store (pivots, trendlines) to emit `forming` states.
- Add breakout confirmation and invalidation rules.
- Tests for breakout vs failed breakouts.

### 6e. Advanced State Machines

- Fibonacci: swing + retracement zone tracking.
- Elliott: wave progress tracking and invalidation rules.
- Fractals: 5-bar window forming/confirm.
- Tests for each advanced family.

### 6f. REST + SSE Snapshot Delivery

- New snapshot payload with pattern lifecycle entries.
- Initial + incremental SSE events for lifecycle updates.
- Include confidence, current state, last transition, and timestamps.

### 6g. Frontend Live Pattern Board

- New panel that mirrors double-top UI (status + summary per pattern).
- Filters by coin, interval, and pattern group.
- Displays forming/confirmed/invalidated + confidence + last update.

### 6h. Alerts + UX Polish

- Optional alerts on `confirmed` transitions (per pattern + coin).
- UI tooltips for state definitions and confidence sources.

