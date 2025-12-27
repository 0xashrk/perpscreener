# Trading Pattern Summary Table

This table provides a quick reference for all trading patterns covered in the comprehensive documentation.

## Pattern Classification Summary

| Category | Pattern | Classification | Signal Type |
|----------|---------|----------------|-------------|
| **Candlestick - Bullish** | Abandoned Baby | Bullish | Reversal |
| | Belt Hold | Bullish | Reversal |
| | Breakaway | Bullish | Reversal |
| | Doji (Dragonfly) | Bullish | Reversal |
| | Doji Star | Bullish | Reversal |
| | Engulfing | Bullish | Reversal |
| | Hammer | Bullish | Reversal |
| | Harami | Bullish | Reversal |
| | Inverted Hammer | Bullish | Reversal |
| | Morning Star | Bullish | Reversal |
| | Morning Doji Star | Bullish | Reversal |
| | Piercing Line | Bullish | Reversal |
| | Three White Soldiers | Bullish | Reversal |
| | Tweezer Bottom | Bullish | Reversal |
| **Candlestick - Bearish** | Abandoned Baby | Bearish | Reversal |
| | Belt Hold | Bearish | Reversal |
| | Dark Cloud Cover | Bearish | Reversal |
| | Doji (Gravestone) | Bearish | Reversal |
| | Engulfing | Bearish | Reversal |
| | Evening Star | Bearish | Reversal |
| | Evening Doji Star | Bearish | Reversal |
| | Hanging Man | Bearish | Reversal |
| | Harami | Bearish | Reversal |
| | Shooting Star | Bearish | Reversal |
| | Three Black Crows | Bearish | Reversal |
| | Tweezer Top | Bearish | Reversal |
| **Chart Patterns - Continuation** | Ascending Triangle | Bullish | Continuation |
| | Descending Triangle | Bearish | Continuation |
| | Symmetrical Triangle | Neutral | Continuation |
| | Bull Flag | Bullish | Continuation |
| | Bear Flag | Bearish | Continuation |
| | Bull Pennant | Bullish | Continuation |
| | Bear Pennant | Bearish | Continuation |
| | Rising Three Methods | Bullish | Continuation |
| | Falling Three Methods | Bearish | Continuation |
| **Chart Patterns - Reversal** | Head and Shoulders | Bearish | Reversal |
| | Inverse Head and Shoulders | Bullish | Reversal |
| | Double Top | Bearish | Reversal |
| | Double Bottom | Bullish | Reversal |
| | Triple Top | Bearish | Reversal |
| | Triple Bottom | Bullish | Reversal |
| | Rising Wedge | Bearish | Reversal |
| | Falling Wedge | Bullish | Reversal |
| | Cup and Handle | Bullish | Continuation |
| **Channels** | Ascending Channel | Bullish | Trend |
| | Descending Channel | Bearish | Trend |
| | Horizontal Channel | Neutral | Range |
| **Gaps** | Breakaway Gap (Up) | Bullish | Trend Start |
| | Breakaway Gap (Down) | Bearish | Trend Start |
| | Runaway Gap (Up) | Bullish | Continuation |
| | Runaway Gap (Down) | Bearish | Continuation |
| | Exhaustion Gap (Up) | Bearish | Reversal |
| | Exhaustion Gap (Down) | Bullish | Reversal |
| | Common Gap | Neutral | None |
| **Advanced** | Fibonacci 38.2% Retracement | Support/Resistance | Key Level |
| | Fibonacci 50% Retracement | Support/Resistance | Key Level |
| | Fibonacci 61.8% Retracement | Support/Resistance | Key Level |
| | Elliott Wave 1-2-3-4-5 (Up) | Bullish | Impulse |
| | Elliott Wave 1-2-3-4-5 (Down) | Bearish | Impulse |
| | Elliott Wave A-B-C | Counter-trend | Correction |
| | Williams Fractal (Up) | Resistance | Reversal Point |
| | Williams Fractal (Down) | Support | Reversal Point |

## Key Detection Formulas

### Basic Candlestick Calculations

| Calculation | Formula |
|-------------|---------|
| Body Size | `ABS(Close - Open)` |
| Upper Shadow | `High - MAX(Open, Close)` |
| Lower Shadow | `MIN(Open, Close) - Low` |
| Total Range | `High - Low` |
| Body to Range Ratio | `ABS(Close - Open) / (High - Low)` |
| Bullish Candle | `Close > Open` |
| Bearish Candle | `Close < Open` |
| Doji | `ABS(Close - Open) < 0.1 * (High - Low)` |

### Trendline Calculations

| Calculation | Formula |
|-------------|---------|
| Slope | `(Price2 - Price1) / (Index2 - Index1)` |
| Intercept | `Price1 - Slope * Index1` |
| Price at Index | `Slope * Index + Intercept` |
| R-Squared | `Correlation^2` |

### Fibonacci Levels (Uptrend)

| Level | Formula |
|-------|---------|
| 0% | `Swing_High` |
| 23.6% | `Swing_High - 0.236 * (Swing_High - Swing_Low)` |
| 38.2% | `Swing_High - 0.382 * (Swing_High - Swing_Low)` |
| 50% | `Swing_High - 0.500 * (Swing_High - Swing_Low)` |
| 61.8% | `Swing_High - 0.618 * (Swing_High - Swing_Low)` |
| 78.6% | `Swing_High - 0.786 * (Swing_High - Swing_Low)` |
| 100% | `Swing_Low` |

### Gap Detection

| Gap Type | Formula |
|----------|---------|
| Up Gap | `Current_Low > Previous_High` |
| Down Gap | `Current_High < Previous_Low` |
| Gap Size | `ABS(Gap_Start - Gap_End)` |
| Gap Percentage | `Gap_Size / Previous_Close * 100` |

### Fractal Detection (5-Bar)

| Fractal Type | Formula |
|--------------|---------|
| Bullish (Up) | `High[i] > High[i-2] AND High[i] > High[i-1] AND High[i] > High[i+1] AND High[i] > High[i+2]` |
| Bearish (Down) | `Low[i] < Low[i-2] AND Low[i] < Low[i-1] AND Low[i] < Low[i+1] AND Low[i] < Low[i+2]` |
