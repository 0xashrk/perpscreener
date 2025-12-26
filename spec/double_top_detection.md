# Double Top Detection Spec

## Data Source

**Endpoint:** `POST https://api.hyperliquid.xyz/info`

**Request:**
```json
{
  "type": "candleSnapshot",
  "req": {
    "coin": "<coin>",
    "interval": "1m",
    "startTime": <epoch_millis>,
    "endTime": <epoch_millis>
  }
}
```

**Response fields:**
- `t` - candle open time (epoch ms)
- `T` - candle close time (epoch ms)
- `o` - open price
- `h` - high price
- `l` - low price
- `c` - close price
- `v` - volume
- `n` - number of trades

---

## What is a Double Top?

A bearish reversal pattern consisting of:
1. **First Peak** - Price rises to a high point, then pulls back
2. **Trough (Neckline)** - Price drops to a support level
3. **Second Peak** - Price rises again to approximately the same level as first peak
4. **Breakdown** - Price drops below the neckline, confirming the pattern

```
    Peak 1          Peak 2
      /\              /\
     /  \            /  \
    /    \          /    \
   /      \        /      \
  /        \______/        \
           Trough           \  <-- Breakdown
          (Neckline)         \
```

---

## Alert Stages

### Stage 1: EARLY WARNING - "Potential Double Top Forming"

Trigger when:
1. First peak identified (confirmed local maximum)
2. Pullback to trough occurred (meaningful drop from peak)
3. Price is now rising back toward first peak level and within `approach_threshold`

```
    Peak 1
      /\              ?  <-- Price approaching peak level
     /  \            /
    /    \          /
   /      \        /
  /        \______/
           Trough
        (we are here)
```

**Alert message:** "Potential double top forming on {coin} - price approaching previous high of {peak_price}"

### Stage 2: CONFIRMATION - "Double Top Confirmed"

Trigger when:
1. Second peak formed at similar level to first peak (within `peak_tolerance`)
2. Price breaks below the neckline by `breakdown_buffer`
3. Confirmation mode:
   - `low` = aggressive (immediate trigger on wick break)
   - `close` = conservative (trigger on close below neckline)

```
    Peak 1          Peak 2
      /\              /\
     /  \            /  \
    /    \          /    \
   /      \        /      \
  /        \______/        \
           Trough           \  <-- Breakdown confirmed
          (Neckline)         \
```

**Alert message:** "Double top CONFIRMED on {coin} - broke neckline at {neckline_price}"

---

## Detection Algorithm

### Continuous Monitoring Loop

1. **Fetch latest 1m candles** (poll every minute)
2. **Identify confirmed peaks** - use look-ahead local maxima for backtests,
   or swing highs (ATR-based) for real-time
3. **Track pattern state per coin:**
   - `WATCHING` - looking for first peak
   - `PEAK_FOUND` - first peak identified, watching for pullback
   - `TROUGH_FOUND` - pullback complete, watching for second approach
   - `FORMING` - price approaching first peak level → **EARLY WARNING**
   - `CONFIRMED` - breakdown below neckline → **CONFIRMATION**
   - `INVALIDATED` - price exceeds Peak 1 by `peak_fail_pct` or time exceeds `max_peak_distance`

### Math

Given candles array where each candle has: `high`, `low`, `close`

Note: The `is_peak` / `is_trough` definitions below are look-ahead and
should only be used for backtests. For live trading, use swing detection.

#### Real-Time Swing Detection (No Look-Ahead)

```
atr = ATR(atr_period)
rev = rev_atr * atr

if trend == "up":
    swing_high = max(swing_high, current_high)
    if swing_high - current_low >= rev:
        confirm swing_high as a peak
        trend = "down"
        swing_low = current_low

if trend == "down":
    swing_low = min(swing_low, current_low)
    if current_high - swing_low >= rev:
        confirm swing_low as a trough
        trend = "up"
        swing_high = current_high
```

#### Peak Detection

```
is_peak(i, lookback) =
    candles[i].high > max(candles[i-lookback : i].high)
    AND
    candles[i].high > max(candles[i+1 : i+lookback+1].high)
```

Example with `lookback = 5`:
- Candle at index 10 is a peak if its high is greater than highs of candles 5-9 AND 11-15

#### Trough Detection

```
is_trough(i, lookback) =
    candles[i].low < min(candles[i-lookback : i].low)
    AND
    candles[i].low < min(candles[i+1 : i+lookback+1].low)
```

Neckline is the lowest `low` between Peak 1 and Peak 2.

#### Pullback Percentage

```
pullback_pct = (peak1_high - trough_low) / peak1_high * 100
```

Example: Peak at $100, trough at $95 → pullback = 5%

#### Peak Similarity (are two peaks at same level?)

```
peak_avg = (peak1_high + peak2_high) / 2
peak_diff_pct = abs(peak1_high - peak2_high) / peak_avg * 100

peaks_match = peak_diff_pct <= peak_tolerance
```

Example with `peak_tolerance = 1.5%`:
- Peak 1: $100, Peak 2: $101 → diff = 1% → ✓ match
- Peak 1: $100, Peak 2: $103 → diff = 3% → ✗ no match

#### Early Warning Trigger

```
distance_to_peak_pct = abs(peak1_high - current_close) / peak1_high * 100

early_warning =
    peak1 exists
    AND trough exists
    AND pullback_pct >= min_pullback_pct (e.g., 2%)
    AND distance_to_peak_pct <= approach_threshold (e.g., 1%)
    AND price_trending_up (current_close > candles[i - trend_lookback].close)
    AND current_high <= peak1_high * (1 + peak_fail_pct)
```

#### Confirmation Trigger

```
break_level = trough_low - breakdown_buffer

confirmed =
    peak2 exists
    AND peaks_match(peak1, peak2)
    AND (
        (confirmation_mode == "low" AND current_low < break_level)
        OR
        (confirmation_mode == "close" AND current_close < break_level)
    )
```

#### Invalidation Trigger

```
invalidated =
    current_high > peak1_high * (1 + peak_fail_pct)
    OR candles_since_peak1 > max_peak_distance
```

#### Neckline Break Strength (optional)

```
break_pct = (trough_low - current_close) / trough_low * 100
```

Stronger break = more confidence in pattern

---

## Parameters to Tune

| Parameter | Description | Suggested Range |
|-----------|-------------|-----------------|
| `peak_lookback` | Candles on each side to confirm peak | 5-15 |
| `max_peak_distance` | Max candles between two peaks | 20-100 |
| `peak_tolerance` | Max % difference between peak prices | 0.5% - 3% |
| `min_pullback_pct` | Min % drop to trough from first peak | 1% - 5% |
| `min_pattern_height` | Min % from peaks to neckline | 2% - 5% |
| `approach_threshold` | % distance to Peak 1 to flag early warning | 0.5% - 2% |
| `atr_period` | ATR window for volatility scaling | 10-20 |
| `rev_atr` | Swing reversal size (ATR multiplier) | 0.8 - 1.2 |
| `breakdown_buffer` | Buffer below neckline to confirm break | 0.2 - 0.5 * ATR |
| `confirmation_mode` | `low` (aggressive) or `close` (conservative) | low / close |
| `peak_fail_pct` | % above Peak 1 that invalidates pattern | 1% - 2% |
| `trend_lookback` | Candles to check for uptrend in early warning | 3-5 |

---

## Edge Cases to Handle

1. **Multiple peaks at same level** - Which two to pair?
2. **Nested patterns** - Smaller double top within larger one
3. **Failed patterns** - Second peak significantly exceeds first (becomes uptrend)
4. **Noise in 1m data** - May need smoothing or higher lookback values

---

## Implementation Phases

### Phase 1: Data Layer
- [ ] Hyperliquid candle fetching service
- [ ] Candle data structures
- [ ] Basic polling mechanism

### Phase 2: Core Detection
- [ ] ATR calculation
- [ ] Swing high/low detection (real-time, no look-ahead)
- [ ] Peak/trough identification

### Phase 3: Pattern State Machine
- [ ] State enum (WATCHING, PEAK_FOUND, TROUGH_FOUND, FORMING, CONFIRMED, INVALIDATED)
- [ ] State transitions
- [ ] Per-coin state tracking

### Phase 4: Alerts
- [ ] Early warning trigger
- [ ] Confirmation trigger
- [ ] Console logging (MVP)

### Phase 5: Polish
- [ ] Alert cooldown/dedup
- [ ] Multiple coin support
- [ ] Parameter configuration

---

## Test Cases

### Test 1: Classic Double Top (Should Confirm)

```
Price
102 |         Peak1         Peak2
101 |          /\            /\
100 |         /  \          /  \
 99 |        /    \        /    \
 98 |       /      \      /      \
 97 |      /        \    /        \
 96 |     /          \__/          \
 95 |    /          Trough          \
 94 |   /                            \  <-- Breakdown
 93 |  /                              \
    +--0--5--10--15--20--25--30--35--40-- Candle Index
```

**Mock Data:**
```
candles = [
  { i: 0,  h: 93, l: 92, c: 93 },   // start
  { i: 5,  h: 96, l: 95, c: 96 },   // rising
  { i: 10, h: 102, l: 100, c: 101 }, // Peak 1
  { i: 15, h: 98, l: 97, c: 97 },   // pullback
  { i: 20, h: 96, l: 95, c: 96 },   // Trough
  { i: 25, h: 99, l: 98, c: 99 },   // rising again
  { i: 30, h: 101, l: 100, c: 100 }, // Peak 2 (within tolerance)
  { i: 35, h: 97, l: 96, c: 96 },   // dropping
  { i: 40, h: 95, l: 93, c: 94 },   // Breakdown below neckline (95)
]
```

**Expected:**
- `i=20`: State → `TROUGH_FOUND`
- `i=28`: State → `FORMING`, Early Warning triggered
- `i=30`: Peak 2 detected
- `i=40`: State → `CONFIRMED`, Confirmation triggered

---

### Test 2: Failed Double Top - Breakout (Should Invalidate)

```
Price
105 |                       /  <-- Breakout, not double top
104 |                      /
103 |                     /
102 |         Peak1      /
101 |          /\       /
100 |         /  \     /
 99 |        /    \   /
 98 |       /      \ /
 97 |      /        X
 96 |     /        Trough
    +--0--5--10--15--20--25--30-- Candle Index
```

**Mock Data:**
```
candles = [
  { i: 0,  h: 96, l: 95, c: 96 },
  { i: 10, h: 102, l: 100, c: 101 }, // Peak 1
  { i: 20, h: 97, l: 96, c: 96 },    // Trough
  { i: 25, h: 100, l: 99, c: 100 },  // approaching
  { i: 30, h: 105, l: 103, c: 105 }, // Breakout! Exceeds peak1 by > peak_fail_pct
]
```

**Expected:**
- `i=25`: State → `FORMING`, Early Warning triggered
- `i=30`: State → `INVALIDATED` (price exceeded Peak 1 + peak_fail_pct)

---

### Test 3: No Pullback (Should Not Trigger)

```
Price
102 |         Peak1-----Peak2  <-- No meaningful trough
101 |          /          \
100 |         /            \
 99 |        /              \
 98 |       /                \
    +--0--5--10--15--20--25--30-- Candle Index
```

**Mock Data:**
```
candles = [
  { i: 0,  h: 98, l: 97, c: 98 },
  { i: 10, h: 102, l: 100, c: 101 }, // Peak 1
  { i: 15, h: 101, l: 100, c: 100 }, // tiny pullback (1%)
  { i: 20, h: 102, l: 101, c: 101 }, // Peak 2
  { i: 25, h: 99, l: 98, c: 98 },
]
```

**Expected:**
- Never reaches `TROUGH_FOUND` (pullback < min_pullback_pct)
- No early warning, no confirmation

---

### Test 4: Peaks Too Far Apart (Should Invalidate)

```
Price
102 |   Peak1                                    Peak2
101 |    /\                                       /\
100 |   /  \                                     /  \
 99 |  /    \___________________________________/    \
 98 | /                   (too long)                  \
    +--0----10----20----30----40----50----60----70----80-- Candle Index
```

**Mock Data:**
```
candles = [
  { i: 10, h: 102, l: 100, c: 101 }, // Peak 1
  { i: 20, h: 99, l: 98, c: 99 },    // Trough
  // ... 50+ candles of sideways action ...
  { i: 75, h: 101, l: 100, c: 100 }, // Peak 2 (too late)
]
```

**Expected:**
- `i > 10 + max_peak_distance`: State → `INVALIDATED`

---

### Test 5: Asymmetric Peaks (Should Not Match)

```
Price
105 |                     Peak2  <-- Too high vs Peak 1
104 |                      /\
103 |                     /  \
102 |         Peak1      /    \
101 |          /\       /      \
100 |         /  \     /        \
 99 |        /    \   /          \
 98 |       /      \_/            \
 97 |      /       Trough          \
    +--0--5--10--15--20--25--30--35-- Candle Index
```

**Mock Data:**
```
candles = [
  { i: 10, h: 102, l: 100, c: 101 }, // Peak 1 = 102
  { i: 20, h: 98, l: 97, c: 98 },    // Trough
  { i: 30, h: 105, l: 104, c: 104 }, // Peak 2 = 105 (diff = 2.9%, > tolerance)
  { i: 35, h: 97, l: 96, c: 96 },    // drops below neckline
]
```

**Expected (with peak_tolerance = 1.5%):**
- Peaks don't match (diff 2.9% > 1.5%)
- State → `INVALIDATED` (or reset to look for new pattern)
- No confirmation triggered

---

### Test 6: Early Warning Only (Pattern Still Forming)

```
Price
102 |         Peak1
101 |          /\           ?  <-- Currently here
100 |         /  \         /
 99 |        /    \       /
 98 |       /      \     /
 97 |      /        \   /
 96 |     /          \_/
 95 |    /          Trough
    +--0--5--10--15--20--25--30-- Candle Index
```

**Mock Data:**
```
candles = [
  { i: 10, h: 102, l: 100, c: 101 }, // Peak 1
  { i: 20, h: 96, l: 95, c: 96 },    // Trough (6% pullback)
  { i: 25, h: 99, l: 98, c: 99 },    // rising
  { i: 30, h: 101, l: 100, c: 101 }, // within approach_threshold of Peak 1
]
```

**Expected (with approach_threshold = 1%):**
- `i=30`: State → `FORMING`
- Early Warning triggered: "approaching previous high of 102"
- No confirmation yet (pattern still forming)

---

## Open Questions

1. What coins to scan? Single coin or multiple?
2. Notification method? (console log, webhook, telegram, etc.)
3. How to avoid spam? (cooldown between alerts for same coin?)
