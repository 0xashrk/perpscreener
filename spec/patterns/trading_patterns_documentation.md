# Comprehensive Guide to Algorithmic Trading Pattern Recognition

This document provides a detailed overview of various technical analysis price patterns, their classification as bullish or bearish, and the algorithms used to detect them from OHLCV (Open, High, Low, Close, Volume) data.


## 1. Candlestick Patterns

Candlestick patterns are short-term patterns that typically involve one to five candlesticks. They are used to predict short-term price movements.

### 1.1. Bullish Candlestick Patterns

Bullish candlestick patterns suggest that the price is likely to rise. Below are the formulas for detecting these patterns, based on the notation from TC2000.

**Notation Key:**
- C = Close, O = Open, H = High, L = Low
- C1, O1, H1, L1 = Previous candle values
- C2, O2, H2, L2 = 2 candles ago values
- AVGH10, AVGL10 = 10-period average High/Low
- STOC1 = Stochastic value
- MINL10 = Minimum Low over 10 periods
- MAXH10 = Maximum High over 10 periods

| Pattern | Formula |
|---|---|
| Abandoned Baby | `2 * ABS(C2 - O2) > H2 - L2 AND C2 > O2 AND 20 * ABS(C1 - O1) <= H1 - L1 AND 5 * ((C1 + O1) / 2 - L1) >= 2 * (H1 - L1) AND 5 * ((C1 + O1) / 2 - L1) <= 3 * (H1 - L1) AND L1 > H2 AND C < O AND H < L1 AND O > C2 AND (L > O2 OR C < L2)` |
| Belt Hold | `O = MINO10 AND O < L1 AND 10 * (C - O) >= 7 * (H - L) AND 5 * (H - L) >= 6 * (AVGH10 - AVGL10) AND 100 * (O - L) <= H - L AND 2 * C <= H1 - L1 AND H1 > L1 AND H > L AND C1 < C2 AND C2 < C3` |
| Breakaway | `C4 < O4 AND 2 * ABS(C4 - O4) > H4 - L4 AND C3 < O3 AND H3 < L4 AND C2 < C3 AND C1 < C2 AND 5 * ABS(C - O) > 3 * (H - L) AND C > O AND C > H3` |
| Concealing Baby Swallow | `O3 = H3 AND C3 = L3 AND O2 = H2 AND C2 = L2 AND C1 < O1 AND O1 < C2 AND H1 > C2 AND O = H AND C = L AND H > H1 AND L < L1` |
| Doji (Dragonfly) | `50 * ABS(O - C) <= H - L AND STOC1 >= 70 AND H - L >= AVGH10 - AVGL10 AND L = MINL10` |
| Doji (Gravestone) | `100 * ABS(O - C) <= H - L AND STOC1 <= 5 AND H > L AND 10 * L <= 3 * H1 + 7 * L1 AND H - L >= AVGH10-AVGL10` |
| Doji Star | `10 * (O1 - C1) >= 7 * (H1 - L1) AND H1 - L1 >= AVGH10.1 - AVGL10.1 AND C < C1 AND O < C1 AND 20 * ABS(C - O) <= H - L AND L = MINL10 AND H1 > L1 AND H > L` |
| Engulfing | `O1 > C1 AND 10 * (C - O) >= 7 * (H - L) AND C > O1 AND C1 > O AND 10 * (H - L) >= 12 * (AVGH10 - AVGL10)` |
| Hammer/Dragonfly Doji | `5 * ABS(C - O) <= H - L AND 10 * ABS(O - C) >= H - L AND 2 * O >= H + L AND STOC1 >= 50 AND (20 * O >= 19 * H + L OR STOC1 >= 95) AND 10 * (H - L) >= 8 * (AVGH10 - AVGL10) AND L = MINL5 AND H > L` |
| Harami | `10 * (O1 - C1) >= 7 * (H1 - L1) AND H1 - L1 >= AVGH10.1 - AVGL10.1 AND C > O AND O > C1 AND O1 > C AND 6 * (O1 - C1) >= 10 * (C - O)` |
| Harami Cross | `2 * ABS(C1 - O1) > H1 - L1 AND O1 > C1 AND O1 > H AND L > C1 AND 5 * ((C + O) / 2 - L) > 2 * (H - L) AND 5 * ((C + O) / 2 - L) < 3 * (H - L) AND 5 * ABS(C - O) < H - L` |
| Homing Pigeon | `C1 < O1 AND 5 * ABS(C - O) >= 3 * (H1 - L1) AND 2 * ABS(C1 - O1) > H1 - L1 AND H < O1 AND L > C1 AND C < O` |
| Inverted Hammer | `5 * ABS(O - C) <= H - L AND 10 * ABS(O - C) >= H - L AND 2 * (H - O) >= H - L AND 2 * (H - C) >= H - L AND (2 * (O - L) <= H - L OR 20 * (C - L) <= H - L) AND 5 * (H - L) >= 4 * (AVGH10 - AVGL10) AND 2 * O <= H1 + L1 AND STOC1 <= 50 AND L = MINL5 AND H > L` |
| Kicking | `5 * (O3 - C3) > 3 * (H3 - L3) AND 5 * (O2 - C2) > 3 * (H2 - L2) AND 5 * (O1 - C1) > 3 * (H1 - L1) AND C3 < O3 AND C2 < O2 AND C1 < O1 AND C > O AND O2 < C3 AND O1 < C2 AND O > O1 AND 5 * (C - O) > 3 * (H - L)` |
| Ladder Bottom | `O4 > C4 AND O3 < O4 AND C3 < C4 AND O2 < O3 AND C2 < C3 AND C1 < O1 AND H1 > O1 AND C > O AND O > O1` |
| Mat Hold | `C4 > O4 AND 2 * ABS(C4 - O4) > H4 - L4 AND C3 < H4 AND C2 < H4 AND C1 < H4 AND C3 > L4 AND C2 > L4 AND C1 > L4 AND C > C4 AND C > O AND H - L > AVGH21 - AVGL21 AND C2 < C3 AND C1 < C2 AND 4 * ABS(C3 - O3) <= 3 * ABS(C4 - O4) AND 4 * ABS(C2 - O2) <= .3 * ABS(C4 - O4) AND 4 * ABS(C2 - O2) <= 3 * ABS(C4 - O4)` |
| Matching Low | `C1 < O1 AND 20 * ABS(C1 - O1) > H1 - L1 AND C < O AND 100 * ABS(C / C1 -1) < 1` |
| Meeting Lines | `C1 < O1 AND H1 - L1 > AVGH21.1 - AVGL21.1 AND O1 < MINL3.3 AND C > O AND 100 * ABS(C / C1 - 1) < 1` |
| Morning Doji Star | `10 * (O2 - C2) >= 7 * (H2 - L2) AND H2 - L2 >= AVGH10.2 - AVGL10.2 AND 10 * (C - O) >= 7 * (H - L) AND O > C1 AND O > O1` |
| Morning Star | `O2 > C2 AND 5 * (O2 - C2) > 3 * (H2 - L2) AND C2 > O1 AND 2 * ABS(O1 - C1) < ABS(O2 - C2) AND H1 - L1 > 3 * (C1 - O1) AND C > O AND O > O1 AND O > C1` |
| Piercing Line | `O1 > C1 AND H1 - L1 >= AVGH10.1 - AVGL10.1 AND O < C1 AND 2 * C > C1 + O1 AND C < O1` |
| Rising Three Method | `10 * (C4 - O4) >= 7 * (H4 - L4) AND H4 - L4 >= AVGH20 - AVGL20 AND H4 = MAXH10.4 AND 2 * C3 = 2 * O4 + H4 - L4 AND O2 > O4 AND O > O4 AND 5 * O <= 3 * H4 + 2 * L4 AND C > C4` |
| Separating Lines | `C1 < O1 AND C > O AND 100 * ABS(O / O1 - 1) < 1` |
| Side by Side White Lines | `C2 > O2 AND C1 > O1 AND L1 > H2 AND 100 * ABS(C / C1 - 1) < 1 AND 100 * ABS(ABS(C - O) / ABS(C1 - O1) - 1) < 15` |
| Stick Sandwich | `C2 < O2 AND C1 > O1 AND L1 > C2 AND C < O AND 100 * ABS(C / C2 - 1) < 2` |
| Three Inside Up | `10 * (O2 - C2) >= 7 * (H2 - L2) AND (H2 - L2) >= AVGH10.2 - AVGL10.2 AND C1 > O1 AND O1 > C2 AND C1 < O2 AND 5 * (C1 - O1) <= 3 * (O2 - C2) AND O > O1 AND O < C1 AND C > C1 AND 10 * (C - O) >= 7 * (H - L)` |
| Three Line Strike | `C2 > C3 AND C1 > C2 AND H3 - L3 > AVGH21.3 - AVGL21.3 AND H2 - L2 > AVGH21.2 - AVGL21.2 AND H1 - L1 > AVGH21.1 - AVGL21.1 AND O > O3 AND C < O3` |
| Three Outside Up | `O2 > C2 AND 10 * (C1 - O1) >= 7 * (H1 - L1) AND C1 > O2 AND O1 < C2 AND 5 * (H1 - L1) >= 6 * (AVGH10.1 - AVGL10.1) AND O > O1 AND O < C1 AND C > C1 AND 10 * C - O >= 7 * (H - L)` |
| Three Stars in the South | `C2 < O2 AND 2 * ABS(C2 - O2) > H2 - L2 AND C2 - L2 > O2 - C2 AND C1 < O1 AND 2 * ABS(C1 - O1) > H1 - L1 AND C1 - L1 > O1 - C1 AND H1 - L1 < H2 - L2 AND L1 > L2 AND O = H AND C = L AND H < H1 AND L > L1` |
| Three White Soldiers | `C > C1 AND C1 > C2 AND C > O AND C1 > O1 AND C2 > O2 AND 2 * ABS(C2 - O2) > H2 - L2 AND 2 * ABS(C1 - O1) > H1 - L1 AND H - L > AVGH21 - AVGL21 AND O > O1 AND O < C1 AND O1 > O2 AND O1 < C2 AND O2 > O3 AND O2 < C3 AND 20 * C > 17 * H AND 20 * C1 > 17 * H1 AND 20 * C2 > 17 * H2` |
| Tri Star | `20 * ABS(C - O) <= H - L AND 5 * ((C + O) / 2 - L) >= 2 * (H - L) AND 5 * ((C + O) / 2 - L) <= 3 * (H - L) AND 20 * ABS(C1 - O1) <= H1 - L1 AND 5 * ((C1 + O1) / 2 - L) >= 2 * (H1 - L1) AND 5 * ((C1 + O1) / 2 - L1) <= 3 * (H1 - L1) AND 20 * ABS(C2 - O2) <= H2 - L2 AND 5 * ((C2 + O2) / 2 - L2) >= 2 * (H2 - L2) AND 5 * ((C2 + O2) / 2 - L2) <= 3 * (H2 - L2) AND H1 < L3 AND H1 < L1` |
| Tweezer Bottom | `L = L1 AND 5 * ABS(C - O) < ABS(C1 - O1) AND 10 * ABS(C1 - O1) >= 9 * (H1 - L1) AND 10 * (H1 - L1) >= 13 * (AVGH20 - AVGL20)` |
| Unique Three River Bottom | `10 * ABS(C2 - O2) >= 7 * (H2 - L2) AND 2 * ABS(C2 - O2) > H2 - L2 AND C1 < O1 AND O1 < O2 AND C1 > C2 AND L1 = MINL5.1 AND C > O AND C < C1` |
| Upside Gap Three Methods | `2 * ABS(C2 - O2) > H2 - L2 AND 2 * ABS(C1 - O1) > H1 - L1 AND L1 > H2 AND C < C2 AND O > O1` |
| Upside Tasuki Gap | `2 * ABS(C2 - O2) > H2 - L2 AND 2 * ABS(C1 - O1) > H1 - L1 AND L1 > H2 AND C < O AND C < O1 AND C > C2` |


### 1.2. Bearish Candlestick Patterns

Bearish candlestick patterns suggest that the price is likely to fall. Below are the formulas for detecting these patterns, based on the notation from TC2000.

| Pattern | Formula |
|---|---|
| Abandoned Baby | `ABS(C2 - O2) > .5 * (H2 - L2) AND C2 > O2 AND ABS(C1 - O1) <= .05 * (H1 - L1) AND (C1 + O1) / 2 - L1 >= .4 * (H1 - L1) AND (C1 + O1) / 2 - L1 <= .6 * (H1 - L1) AND L1 > H2 AND C < O AND H < L1 AND O > C2 AND (L > O2 OR C < L2)` |
| Advance Block | `H - L > AVGH21 - AVGL21 AND ABS(C1 - O1) > .5 * (H1 - L1) AND ABS(C2 - O2) > .5 * (H2 - L2) AND C > C1 AND C1 > C2 AND O1 > O2 AND O1 < C2 AND O > O1 AND O < C1 AND H - C > O - L AND H1 - C1 > O1 - L1` |
| Belt Hold | `O = MAXO10 AND O > H1 AND O - C >= .7 * (H - L) AND H - L >= 1.2 * (AVGH10 - AVGL10) AND H - O <= .01 * (H - L) AND C >= H1 - .5 * (H1 - L1) AND H1 > L1 AND H > L AND C1 > C2 AND C2 < C3` |
| Breakaway | `ABS(C4 - O4) > .5 * (H4 - L4) AND C4 > O4 AND C3 > O3 AND L3 > H4 AND C2 > C3 AND C1 > C2 AND C < O AND L < H4 AND H > L3` |
| Dark Cloud Cover | `C1 - O1 >= .7 * (H1 - L1) AND H1 - L1 >= AVGH10.1 - AVGL10.1 AND O > C1 AND C < C1 - .5 * (C1 - O1) AND C > O1` |
| Deliberation | `ABS(C2-O2) > .5 * (H2 - L2) AND ABS(C1 - O1) > .5 * (H1 - L1) AND C1 > C2 AND C2 > O2 AND C1 > O1 AND O > H1 AND (C + O) / 2 - L > .4 * (H - L) AND (C + O) / 2 - L < .6 * (H - L) AND ABS(C - O) < .6 * (H - L)` |
| Downside Gap Three Methods | `ABS(C2-O2) > .5 * (H2 - L2) AND ABS(C1 - O1) > .5 * (H1 - L1) AND C2 < O2 AND C1 < O1 AND H1 < L2 AND L < H1 AND H > L2 AND C > O` |
| Downside Tasuki Gap | `C2 < O2 AND C1 < O1 AND H1 < L2 AND O > C1 AND O < O1 AND C > H1 AND C < L2` |
| Doji Star | `ABS(C1 - O1) > .5 * (H1 - L1) AND O > C1 AND ABS(C - O) < .05 * (H - L) AND H - L < .2 * (AVGH21 - AVGL21)` |
| Doji (Gravestone) | `ABS(O-C)<=.01*(H-L) AND (H-C)>=.95*(H-L) AND (H>L) AND (H=MAXH10) AND (H-L)>=(AVGH10-AVGL10)` |
| Dragonfly Doji/Hanging Man | `ABS(O-C)<=.02*(H-L) AND (H-C)<=.3*(H-L) AND (H-L)>=(AVGH10-AVGL10) AND (H>L) AND (H=MAXH10)` |
| Engulfing | `C1 > O1 AND O - C >= .7 * (H - L) AND C < O1 AND O > C1 AND H - L >= 1.2 * (AVGH10 - AVGL10)` |
| Evening Doji Star | `ABS(C2 - O2) > .5 * (H - L) AND C2 > O2 AND ABS(C1 - O1) < .05 * (H1 - L1) AND H1 - L1 < .2 * (AVGH21.1 - AVGL21.1) AND O1 > C2 AND C < O` |
| Evening Star | `C2 - O2 >= .7 * (H2 - L2) AND H2 - L2 >= AVGH10.2 - AVGL10.2 AND C1 > C2 AND O1 > C2 AND H - L >= AVGH10 - AVGL10 AND O - C >= .7 * (H - L) AND O < O1 AND O < C1` |
| Falling Three Methods | `ABS(C4 - O4) > .5 * (H4 - L4) AND C4 < O4 AND ABS(C3 - O3) < ABS(C4 - O4) AND ABS(C2 - O2) < ABS(C4 - O4) AND ABS(C1 - O1) < ABS(C4 - O4) AND L3 >= L4 AND H3 <= H4 AND L2 >= L4 AND H2 <= H4 AND L1 >= L4 AND H1 <= H4 AND H2 > H3 AND H1 > H2 AND C < O AND C < C4` |
| Grave Stone Doji/Shooting Star | `ABS(C - O) < (H - L) / 3 AND O > C1 AND (C + O) / 2 - L < .4 * (H - L) AND H = MAXH10` |
| Hanging Man | `ABS(C >= O) * O + ABS(C < O) * C - L >= 2 * ABS(C - O) AND (C + O) / 2 - L > 2 * (H - (C + O) / 2) AND ABS(C - O) > .01` |
| Harami (Bearish) | `C1 - O1 >= .7 * (H1 - L1) AND H1 - L1 >= AVGH10.1 - AVGL10.1 AND C < O AND O < C1 AND C > O1 AND O - C <= .6 * (C1 - O1)` |
| Harami Cross | `ABS(C1 - O1) > .5 * (H - L) AND C1 > O1 AND H < C1 AND L > O1 AND ABS(C - O) < .2 * (H - L)` |
| Identical Three Crows | `C2 < O2 AND C1 < O1 AND C < O AND C < L1 AND C1 < L2 AND O = C1 AND O1 = C2` |
| In Neck | `ABS(C1 - O1) >.5 * (H1 - L1) AND C1 < O1 AND O < L1 AND C >= C1 AND C < 1.05 * C1` |
| Kicking | `C = L AND O = H AND H > L AND H < L1 AND C1 = H1 AND O1 = L1 AND H1 > L1` |
| Meeting Lines | `ABS(C1 - O1) > .5 * (H1 - L1) AND C1 > O1 AND (C1 + O1) / 2 > H2 AND ABS(C - O) > .5 * (H - L) AND C < O AND (C + O) / 2 > H1 AND C = C1` |
| On Neck | `ABS(C1 - O1) > .5 * (H1 - L1) AND C1 < O1 AND O < L1 AND C = L1` |
| Separating Lines | `C1 > O1 AND C < O AND O = O1` |
| Shooting Star | `ABS(O-C)<=.2*(H-L) AND ABS(O-C)>=.1*(H-L) AND (H-O)>=.5*(H-L) AND (H-C)>=.5*(H-L) AND (O-L)<=.05*(H-L) OR (C-L)<=.05*(H-L) AND (H-L)>=.8*(AVGH10-AVGL10) AND (O>=(L1+.5*(H1-L1))) AND (C>=(L1+.5*(H1-L1))) AND (H=MAXH5) AND (H>L)` |
| Side-by-side White Lines | `C2 < O2 AND H1 < L2 AND C1 > O1 AND ABS(C1 - O1) > .95 * ABS(C - O) AND ABS(C1 - O1) < 1.95 * ABS(C - O) AND C > O AND C = C1` |
| Three Black Crows | `O1 < O2 AND O1 > C2 AND O < O1 AND O > C1 AND C1 < L2 AND C < L1 AND C2 < 1.05 * L2 AND C1 < 1.05 * L1 AND C < 1.05 * L` |
| Three Inside Down | `ABS(C2 - O2) > .5 * (H1 - L1) AND C2 > O2 AND C1 < O1 AND H1 < C2 AND L1 > O2 AND C < O AND C < C1` |
| Three Line Strike | `C3 < O3 AND C2 < O2 AND C2 < C3 AND C1 < O1 AND C1 < C2 AND O < C1 AND C > O3` |
| Three Outside Down | `C1 - O1 >= .7 * (H1 - L1) AND H1 - L1 >= AVGH10.1 - AVGL10.1 AND C < O AND O < C1 AND C > O1 AND O - C <= .6 * (C1 - O1)` |
| Thrusting | `ABS(C1 - O1) > .5 * (H1 - L1) AND C1 < O1 AND O < L1 AND C > C1 AND C < (C1 + O1) / 2` |
| Tri Star | `ABS(C - O) < .05 * (H - L) AND H - L < .2 * (AVGH21 - AVGL21) AND ABS(C1 - O1) < .05 * (H1 - L1) AND H1 - L < .2 * (AVGH21.1-AVGL21.1) AND ABS(C2 - O2) < .05 * (H2 - L2) AND H2 - L2 < .2 * (AVGH21.2 - AVGL21.2) AND L2 > H1 AND L2 > H` |
| Tweezer Top | `H = H1 AND ABS(C - O) < .2 * ABS(C1 - O1) AND ABS(C1 - O1) >= .9 * (H1 - L1) AND H1 - L1 >= 1.3 * (AVGH20 - AVGL20)` |
| Two Crows | `ABS(C2 - O2) > .5 * (H2 - L2) AND C2 > O2 AND L1 > H2 AND C1 < O1 AND O > C1 AND O < O1 AND C < C2 AND C > O2` |
| Upside Gap Two Crows | `ABS(C2 - O2) > .5 * (H2 - L2) AND C2 > O2 AND L1 > H2 AND C1 < O1 AND O > O1 AND C < C1 AND C > H2` |


## 2. Chart Patterns

Chart patterns are formations that appear on price charts and can indicate potential future price movements. They are broadly categorized into continuation and reversal patterns.

### 2.1. Continuation Patterns

Continuation patterns suggest that a prevailing trend will continue after a brief pause or consolidation.

#### 2.1.1. Triangle Patterns

Triangles are continuation patterns that form as the price range narrows over time, indicating a period of consolidation before a breakout.

**Types of Triangles:**
- **Ascending Triangle:** Bullish continuation pattern with a flat resistance line and a rising support line.
- **Descending Triangle:** Bearish continuation pattern with a falling resistance line and a flat support line.
- **Symmetrical Triangle:** Neutral pattern where the direction of the breakout determines the next move, typically in the direction of the prior trend.

**Detection Algorithm:**

1.  **Find Pivot Points:** Identify local highs and lows in the price data.
2.  **Linear Regression:** Fit trendlines to the pivot highs and pivot lows.
3.  **Classify Pattern:** Based on the slopes of the trendlines.

| Pattern | Upper Trendline (Highs) | Lower Trendline (Lows) | Classification |
| :--- | :--- | :--- | :--- |
| Ascending | Flat (slope ≈ 0) | Rising (slope > 0) | Bullish |
| Descending | Falling (slope < 0) | Flat (slope ≈ 0) | Bearish |
| Symmetrical | Falling (slope < 0) | Rising (slope > 0) | Neutral/Continuation |

```python
from scipy.stats import linregress

def detect_triangle(ohlc, lookback=25, rlimit=0.9, sl_limit=0.00001):
    # Simplified logic for demonstration
    # 1. Find pivot points (highs and lows)
    # ... (code to find pivots) ...

    # 2. Fit linear regression to pivots
    slmin, _, rmin, _, _ = linregress(low_indices, low_prices)
    slmax, _, rmax, _, _ = linregress(high_indices, high_prices)

    # 3. Classify based on slopes
    if abs(rmax) >= rlimit and abs(rmin) >= rlimit:
        if slmin > sl_limit and slmax < -sl_limit:
            return "Symmetrical Triangle"
        elif slmin > sl_limit and -sl_limit <= slmax <= sl_limit:
            return "Ascending Triangle"
        elif slmax < -sl_limit and -sl_limit <= slmin <= sl_limit:
            return "Descending Triangle"
    return None
```

#### 2.1.2. Flag and Pennant Patterns

Flags and pennants are short-term continuation patterns that form after a strong price movement (the flagpole).

-   **Flag:** A rectangular consolidation pattern that slopes against the preceding trend.
-   **Pennant:** A small, symmetrical triangle-shaped consolidation.

| Pattern | Preceding Trend | Consolidation Shape | Classification |
| :--- | :--- | :--- | :--- |
| Bull Flag/Pennant | Strong Upward Move | Sloping down rectangle/triangle | Bullish Continuation |
| Bear Flag/Pennant | Strong Downward Move | Sloping up rectangle/triangle | Bearish Continuation |

**Detection Algorithm:**

1.  **Identify Flagpole:** Detect a sharp, significant price move.
2.  **Identify Consolidation:** Look for a period of consolidation with specific trendline characteristics (parallel for flags, converging for pennants).
3.  **Confirm Breakout:** The price breaks out of the consolidation in the direction of the original trend.

#### 2.1.3. Wedge Patterns

Wedges are reversal patterns characterized by two converging trendlines. Unlike triangles, both trendlines in a wedge pattern slope in the same direction (either up or down).

-   **Rising Wedge:** A bearish reversal pattern where both support and resistance lines are sloping upwards, but the support line is steeper.
-   **Falling Wedge:** A bullish reversal pattern where both support and resistance lines are sloping downwards, but the resistance line is steeper.

| Pattern | Trendlines | Slope Condition | Classification |
| :--- | :--- | :--- | :--- |
| Rising Wedge | Both rising | `slope_lows > slope_highs > 0` | Bearish Reversal |
| Falling Wedge | Both falling | `slope_highs < slope_lows < 0` | Bullish Reversal |
_

### 2.2. Reversal Patterns

Reversal patterns indicate that a prevailing trend is likely to change direction.

#### 2.2.1. Head and Shoulders

The Head and Shoulders pattern is a classic bearish reversal pattern that signals a potential trend change from bullish to bearish.

-   **Structure:** Consists of three peaks: a central peak (the head) that is higher than the two surrounding peaks (the shoulders).
-   **Neckline:** A support level that connects the lows of the two troughs between the three peaks.
-   **Classification:** Bearish Reversal

**Inverse Head and Shoulders:** The bullish counterpart, signaling a potential trend change from bearish to bullish.

**Detection Algorithm:**

The detection of this pattern often involves more advanced techniques like Dynamic Time Warping (DTW) to measure the similarity between the price series and a reference Head and Shoulders pattern.

1.  **Generate Reference Patterns:** Create idealized Head and Shoulders patterns with some randomness.
2.  **Scan Price Data:** Slide a window across the historical price data.
3.  **Downsample and Normalize:** For each window, downsample the data to match the reference pattern's length and normalize it.
4.  **Compare and Score:** Use DTW and correlation coefficients to measure the similarity between the price subsequence and the reference patterns.
5.  **Identify Pattern:** A high correlation score and low DTW distance indicate a potential Head and Shoulders pattern.

```python
# Conceptual algorithm using DTW and correlation
from dtw import dtw
from scipy.stats import pearsonr

def detect_head_and_shoulders(price_series, reference_pattern):
    # Ensure series have the same length (e.g., by downsampling)
    # ...

    # Calculate DTW distance and path
    alignment = dtw(price_series, reference_pattern, keep_internals=True)
    dtw_distance = alignment.distance

    # Align the series using the DTW path
    path = alignment.path
    aligned_series = price_series[path[0]]
    aligned_ref = reference_pattern[path[1]]

    # Calculate correlation of aligned series
    correlation, _ = pearsonr(aligned_series, aligned_ref)

    # High correlation and low DTW distance suggest a match
    if correlation > 0.8 and dtw_distance < threshold:
        return "Head and Shoulders Detected"
    return None
```

#### 2.2.2. Double Top and Double Bottom

These are common reversal patterns that signal a potential trend change.

-   **Double Top:** A bearish reversal pattern that looks like the letter "M". It forms after an uptrend when the price reaches a high, retraces, and then fails to break above the previous high, indicating a potential move lower.
-   **Double Bottom:** A bullish reversal pattern that looks like the letter "W". It forms after a downtrend when the price reaches a low, rallies, and then fails to break below the previous low, suggesting a potential move higher.

**Detection Algorithm:**

1.  **Identify Pivot Points:** Use a ZigZag indicator or similar method to find significant peaks and troughs.
2.  **Look for Pattern:**
    *   For a Double Top, find two consecutive peaks at roughly the same price level, separated by a valley.
    *   For a Double Bottom, find two consecutive troughs at roughly the same price level, separated by a peak.
3.  **Confirm with Neckline Break:**
    *   For a Double Top, a sell signal is generated when the price breaks below the support level (neckline) of the intervening valley.
    *   For a Double Bottom, a buy signal is generated when the price breaks above the resistance level (neckline) of the intervening peak.

```python
# Conceptual algorithm for Double Top
def detect_double_top(ohlc, tolerance=0.03):
    # 1. Find pivot highs and lows
    # ... (code to find pivots) ...

    # 2. Check for two consecutive peaks at similar levels
    if len(pivot_highs) >= 2 and len(pivot_lows) >= 1:
        peak1 = pivot_highs[-2]
        peak2 = pivot_highs[-1]
        valley = pivot_lows[-1]

        # Check if peaks are at similar height
        price_diff = abs(peak1['price'] - peak2['price']) / peak1['price']
        if price_diff <= tolerance:
            # Check if valley is between peaks
            if peak1['index'] < valley['index'] < peak2['index']:
                # Check for neckline break
                if ohlc.iloc[-1]['close'] < valley['price']:
                    return "Double Top Detected (Bearish)"
    return None
```

#### 2.2.3. Cup and Handle

The Cup and Handle is a bullish continuation pattern developed by William O'Neil. It signals a consolidation period followed by a breakout.

-   **Structure:** The pattern resembles a teacup, with a "U"-shaped cup and a slightly downward-drifting handle.
-   **Classification:** Bullish Continuation

**Detection Algorithm:**

1.  **Identify Prior Trend:** The pattern must be preceded by a clear uptrend.
2.  **Find Cup:** Look for a "U"-shaped consolidation period. The cup should not be too deep (ideally retracing 1/3 or less of the prior advance).
3.  **Find Handle:** After the cup is formed, a shorter consolidation period (the handle) forms, which should retrace less than 1/3 of the cup's advance.
4.  **Confirm Breakout:** A buy signal is generated when the price breaks above the resistance line formed by the handle, ideally with a surge in volume.

| Criteria | Valid Range | Description |
| :--- | :--- | :--- |
| Cup Shape | U-shaped | A rounded bottom, not a V-shape. |
| Cup Depth | 15% - 67% | Retracement of the prior advance. |
| Handle Depth | < 33% | Retracement of the cup's height. |
| Volume | > 1.5x Average | On breakout above the handle's resistance. |

```python
# Conceptual algorithm for Cup and Handle
def detect_cup_and_handle(ohlc):
    # 1. Identify prior uptrend
    # ...

    # 2. Find cup formation (U-shaped)
    # Find left peak, cup bottom, and right peak
    # ...

    # 3. Validate cup shape and depth
    # ...

    # 4. Find handle formation
    # ...

    # 5. Validate handle depth
    # ...

    # 6. Check for breakout with volume
    if breakout and volume_surge:
        return "Cup and Handle Detected (Bullish)"
    return None
```

### 2.3. Gap Patterns

Gaps are areas on a chart where no trading takes place, resulting in a space between the previous day's close and the current day's open. They often signal a strong shift in market sentiment.

**Types of Gaps:**

| Gap Type | Volume | Trend Context | Signal |
| :--- | :--- | :--- | :--- |
| **Common Gap** | Low | Range-bound | Neutral |
| **Breakaway Gap** | High | Breaking out of a range | Trend Start |
| **Runaway Gap** | High | Mid-trend | Continuation |
| **Exhaustion Gap** | Very High | End of a trend | Reversal |

**Detection Algorithm:**

1.  **Identify Gap:**
    *   **Up Gap (Bullish):** `current_low > previous_high`
    *   **Down Gap (Bearish):** `current_high < previous_low`
2.  **Classify Gap:** The classification depends on the context, including the prevailing trend, volume, and whether the gap occurs within a trading range or as a breakout.

```python
# Conceptual algorithm for gap classification
def classify_gap(ohlc, gap_index):
    gap_type = get_gap_type(ohlc, gap_index) # Up or Down

    if is_exhaustion_gap(ohlc, gap_index, gap_type):
        return "Exhaustion Gap"
    elif is_breakaway_gap(ohlc, gap_index, gap_type):
        return "Breakaway Gap"
    elif is_runaway_gap(ohlc, gap_index, gap_type):
        return "Runaway Gap"
    else:
        return "Common Gap"
```

### 2.4. Trendlines and Price Channels

Trendlines and price channels are fundamental tools in technical analysis used to identify and confirm trends.

-   **Trendline:** A line drawn over pivot highs or under pivot lows to show the prevailing direction of price. A support trendline (uptrend) connects rising lows, while a resistance trendline (downtrend) connects falling highs.
-   **Price Channel:** Consists of two parallel trendlines that contain the price action. Channels can be ascending (bullish), descending (bearish), or horizontal (neutral).

**Detection Algorithm:**

1.  **Find Local Extrema:** Identify significant pivot highs and lows.
2.  **Find Trendlines:** Use linear regression to fit lines through the pivot points. The best-fit lines with a high R-squared value are considered valid trendlines.
3.  **Detect Channels:** Look for pairs of support and resistance trendlines with similar slopes (i.e., they are parallel).

| Pattern | Structure | Classification |
| :--- | :--- | :--- |
| Ascending Channel | Parallel rising lines | Bullish |
| Descending Channel | Parallel falling lines | Bearish |
| Horizontal Channel | Parallel horizontal lines | Neutral |

```python
# Conceptual algorithm for channel detection
from scipy.stats import linregress

def detect_price_channel(ohlc, tolerance=0.02):
    # 1. Find support and resistance trendlines
    support_lines, resistance_lines = find_support_resistance_trendlines(ohlc)

    for support in support_lines:
        for resistance in resistance_lines:
            # 2. Check if lines are parallel
            slope_diff = abs(support['slope'] - resistance['slope'])
            if slope_diff <= tolerance:
                # 3. Classify channel type based on slope
                if support['slope'] > 0:
                    return "Ascending Channel"
                elif support['slope'] < 0:
                    return "Descending Channel"
                else:
                    return "Horizontal Channel"
    return None
```

## 3. Advanced Technical Indicators

Advanced indicators and theories provide a more complex framework for analyzing market movements.

### 3.1. Fibonacci Retracement

Fibonacci retracement is a tool used to identify potential support and resistance levels. It is based on the key numbers in the Fibonacci sequence.

-   **Concept:** After a significant price move (a swing high to a swing low, or vice versa), the price will often retrace or pull back to certain predictable levels before continuing in the original direction.
-   **Key Ratios:** 23.6%, 38.2%, 50%, 61.8%, and 78.6%.

**Detection Algorithm:**

1.  **Identify Swing High and Swing Low:** Find the most recent significant peak and trough in the price data.
2.  **Determine Trend:** If the swing high occurred after the swing low, the trend is up. If the swing low occurred after the swing high, the trend is down.
3.  **Calculate Fibonacci Levels:**
    *   In an uptrend, subtract the Fibonacci percentages of the total price move from the swing high to find support levels.
    *   In a downtrend, add the Fibonacci percentages of the total price move to the swing low to find resistance levels.

| Trend | Calculation for 38.2% Level | Signal |
| :--- | :--- | :--- |
| Uptrend | `Swing High - (Swing High - Swing Low) * 0.382` | Potential Support |
| Downtrend | `Swing Low + (Swing High - Swing Low) * 0.382` | Potential Resistance |

```python
# Conceptual algorithm for Fibonacci levels in an uptrend
def calculate_fib_retracement_uptrend(swing_high, swing_low):
    price_move = swing_high - swing_low
    levels = {
        "23.6%": swing_high - price_move * 0.236,
        "38.2%": swing_high - price_move * 0.382,
        "61.8%": swing_high - price_move * 0.618,
    }
    return levels
```

### 3.2. Elliott Wave Theory

Elliott Wave Theory posits that market price movements are not random but follow predictable, repetitive patterns or "waves" driven by investor psychology.

-   **Impulse Waves (5-wave pattern):** The main trend, consisting of five waves (1-2-3-4-5). Waves 1, 3, and 5 are motive waves that move with the trend, while waves 2 and 4 are corrective waves.
-   **Corrective Waves (3-wave pattern):** A counter-trend move, consisting of three waves (A-B-C).

**The 3 Cardinal Rules:**

1.  **Rule 1:** Wave 3 can never be the shortest impulse wave.
2.  **Rule 2:** Wave 2 can never retrace more than 100% of Wave 1.
3.  **Rule 3:** Wave 4 can never overlap with the price territory of Wave 1.

**Detection Algorithm:**

Automated Elliott Wave detection is complex and often relies on a combination of ZigZag indicators to identify pivot points and a rule-based engine to validate the wave patterns against the cardinal rules and Fibonacci relationships.

1.  **Identify MonoWaves:** Break down the price action into a series of simple up and down moves (MonoWaves) connecting pivot points.
2.  **Generate Wave Combinations:** Combine consecutive MonoWaves to form potential 5-wave impulse patterns and 3-wave corrective patterns.
3.  **Validate Patterns:** Test each combination against the three cardinal rules and other guidelines (e.g., Fibonacci relationships between waves).

```python
# Conceptual algorithm for validating an impulse wave
def validate_impulse_wave(waves):
    if len(waves) != 5:
        return False

    wave1, wave2, wave3, wave4, wave5 = waves

    # Rule 1: Wave 3 not the shortest
    rule1 = not (wave3.length < wave1.length and wave3.length < wave5.length)

    # Rule 2: Wave 2 retracement
    rule2 = wave2.low > wave1.start_price # For an uptrend

    # Rule 3: Wave 4 overlap
    rule3 = wave4.low > wave1.high # For an uptrend

    return rule1 and rule2 and rule3
```

### 3.3. Williams Fractal

The Williams Fractal is an indicator that identifies potential reversal points by locating local highs and lows in the price data.

-   **Structure:** A fractal is formed by a series of five consecutive bars.
    *   A **bullish fractal (up fractal)** occurs when the middle bar has the highest high.
    *   A **bearish fractal (down fractal)** occurs when the middle bar has the lowest low.
-   **Signal:** Fractals themselves are not trading signals but rather indicate potential support or resistance levels. A trade is often triggered when the price breaks beyond a previous fractal.

**Detection Algorithm:**

1.  **Scan for Fractals:** Iterate through the price data with a five-bar window.
2.  **Identify Bullish Fractal:** If the high of the middle bar is greater than the highs of the two preceding and two succeeding bars, a bullish fractal is identified.
3.  **Identify Bearish Fractal:** If the low of the middle bar is lower than the lows of the two preceding and two succeeding bars, a bearish fractal is identified.

```python
# Conceptual algorithm for detecting a 5-bar bullish fractal
def is_bullish_fractal(ohlc, index):
    if index < 2 or index > len(ohlc) - 3:
        return False

    middle_high = ohlc.iloc[index]["high"]
    prev1_high = ohlc.iloc[index - 1]["high"]
    prev2_high = ohlc.iloc[index - 2]["high"]
    next1_high = ohlc.iloc[index + 1]["high"]
    next2_high = ohlc.iloc[index + 2]["high"]

    return (middle_high > prev1_high and middle_high > prev2_high and
            middle_high > next1_high and middle_high > next2_high)
```

## 4. References

[1] TC2000. "Bullish Candlestick Patterns Formulas Table." *TC2000 Help Site*. [Online]. Available: https://help.tc2000.com/m/69445/l/800589-bullish-candlestick-patterns-formulas-table

[2] TC2000. "Bearish Candlestick Patterns Formulas Table." *TC2000 Help Site*. [Online]. Available: https://help.tc2000.com/m/69445/l/800590-bearish-candlestick-patterns-formulas-table

[3] QuantConnect. "Head & Shoulders (TA) Pattern Detection." *QuantConnect Research*. [Online]. Available: https://www.quantconnect.com/research/15603/head-amp-shoulders-ta-pattern-detection

[4] wl8380. "Automating Double/Triple Top and Bottom Detection." *Medium*, 2021. [Online]. Available: https://medium.com/@wl8380/automating-double-triple-top-and-bottom-detection-05be618bc3cf

[5] ProRealCode. "Detecting Double Top and Bottom Patterns." *ProRealCode*. [Online]. Available: https://www.prorealcode.com/prorealtime-indicators/detecting-double-top-and-bottom-patterns/

[6] zeta-zetra. "chart_patterns." *GitHub*. [Online]. Available: https://github.com/zeta-zetra/chart_patterns

[7] StockCharts. "Cup with Handle." *StockCharts ChartSchool*. [Online]. Available: https://chartschool.stockcharts.com/table-of-contents/chart-analysis/chart-patterns/cup-with-handle

[8] StockCharts. "Gaps and Gap Analysis." *StockCharts ChartSchool*. [Online]. Available: https://chartschool.stockcharts.com/table-of-contents/chart-analysis/gaps-and-gap-analysis

[9] QuantInsti. "Fibonacci Retracement Trading Strategy using Python." *QuantInsti Blog*. [Online]. Available: https://blog.quantinsti.com/fibonacci-retracement-trading-strategy-python/

[10] drstevendev. "ElliottWaveAnalyzer." *GitHub*. [Online]. Available: https://github.com/drstevendev/ElliottWaveAnalyzer

[11] BabyPips. "The 3 Cardinal Rules and Some Guidelines." *BabyPips.com*. [Online]. Available: https://www.babypips.com/learn/forex/the-3-cardinal-rules-and-some-guidelines

[12] TradingView. "Williams Fractal." *TradingView Support*. [Online]. Available: https://www.tradingview.com/support/solutions/43000591663-williams-fractal/

[13] GregoryMorse. "trendln." *GitHub*. [Online]. Available: https://github.com/GregoryMorse/trendln
