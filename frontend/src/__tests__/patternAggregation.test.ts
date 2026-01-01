import { describe, expect, it } from "vitest";
import { PatternDetection } from "../types/patterns";
import { createDefaultWeightConfig, summarizeDetections } from "../utils/patternAggregation";

const detection = (
  pattern: string,
  classification: PatternDetection["classification"],
  signalType: PatternDetection["signalType"],
  confidence: number
): PatternDetection => ({
  coin: "BTC",
  interval: "1h",
  pattern,
  category: "candlestick",
  classification,
  signalType,
  confidence,
  detectedAtMs: 0,
  windowStartMs: 0,
  windowEndMs: 0,
  notes: ""
});

describe("summarizeDetections", () => {
  it("normalizes scores per summary", () => {
    const weights = createDefaultWeightConfig();
    weights.timeframe["1h"] = 1;
    weights.signalType.reversal = 1;
    weights.signalType.trend = 1;
    weights.signalType.range = 1;

    const summaries = summarizeDetections(
      [
        detection("A", "bullish", "reversal", 0.8),
        detection("B", "bullish", "trend", 0.6),
        detection("C", "bearish", "reversal", 0.4),
        detection("D", "neutral", "range", 0.2)
      ],
      weights
    );

    const summary = summaries[0];
    expect(Math.abs(summary.bullishScore - 0.7)).toBeLessThan(1e-6);
    expect(Math.abs(summary.bearishScore - 0.2)).toBeLessThan(1e-6);
    expect(Math.abs(summary.neutralScore - 0.1)).toBeLessThan(1e-6);
  });

  it("orders top signals by weighted score", () => {
    const weights = createDefaultWeightConfig();
    weights.signalType.trend = 2;

    const summaries = summarizeDetections(
      [
        detection("Reversal", "bullish", "reversal", 0.9),
        detection("Trend", "bullish", "trend", 0.6)
      ],
      weights
    );

    expect(summaries[0].topSignals[0].pattern).toBe("Trend");
  });
});
