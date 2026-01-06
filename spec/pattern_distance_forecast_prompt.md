# Pattern Distance Forecast Prompt

Use curl to fetch pattern data from the backend and predict which tokens are bullish or bearish for the next 12-24 hours.

## Step 1: Fetch Data

```bash
curl -s http://localhost:30001/patterns
curl -s http://localhost:30001/patterns/advanced
```

Reference schemas: `openapi/backend_openapi.outline.json`

## Step 2: Extract All Tokens

Get the list of all tokens from the responses:
- From `/patterns`: all keys in `summaries` object
- From `/patterns/advanced`: all unique `coin` values in `detections` array

Union these to get the complete token list. Analyze ALL tokens.

## Step 3: Filter to Relevant Timeframes

For 12-24 hour predictions, only use intervals: 4h, 8h, 12h, 1d.

Interval weights for averaging: 4h=1.0, 8h=1.2, 12h=1.5, 1d=0.8

## Step 4: Score Each Token

For each token, for each relevant interval:

**Core patterns** (`/patterns`):
- `core_bull` = `summaries[coin].bullish_score` (already 0-1)
- `core_bear` = `summaries[coin].bearish_score` (already 0-1)

**Advanced patterns** (`/patterns/advanced`):
- Count detections by classification:
  - Bullish: Williams Fractal (Down), Elliott Wave Up
  - Bearish: Williams Fractal (Up), Elliott Wave Down
  - Ignore: Fibonacci levels, Elliott Wave A-B-C
- `adv_bull` = bullish_count / (bullish_count + bearish_count), or 0.5 if no detections
- `adv_bear` = bearish_count / (bullish_count + bearish_count), or 0.5 if no detections

**Per-interval combined score**:
- `bull_i` = 0.6 * core_bull + 0.4 * adv_bull
- `bear_i` = 0.6 * core_bear + 0.4 * adv_bear

**Weighted average across intervals**:
- `final_bull` = Σ(weight_i * bull_i) / Σ(weight_i)
- `final_bear` = Σ(weight_i * bear_i) / Σ(weight_i)

Skip intervals with no data; if all intervals missing, skip the token.

## Step 5: Compute Euclidean Distance

```
d_bull = sqrt((final_bull - 1)² + final_bear²)
d_bear = sqrt(final_bull² + (final_bear - 1)²)

max_dist = sqrt(2)  # normalize to 0-1
confidence = abs(d_bear - d_bull) / max_dist

if d_bull < d_bear → BULLISH
if d_bear < d_bull → BEARISH
if d_bull == d_bear → NEUTRAL
```
