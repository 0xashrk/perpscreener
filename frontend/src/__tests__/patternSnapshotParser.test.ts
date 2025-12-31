import { parsePatternSnapshot } from "../services/patterns";

const payloadWithSummaries = {
  as_of_ms: 1710000000000,
  detections: [
    {
      coin: "BTC",
      interval: "1h",
      pattern: "Hammer",
      category: "candlestick_reversal",
      classification: "bullish",
      signal_type: "reversal",
      confidence: 0.7,
      detected_at_ms: 1710000000000,
      window_start_ms: 1709990000000,
      window_end_ms: 1710000000000,
      notes: "Test"
    }
  ],
  summaries: [
    {
      coin: "BTC",
      interval: "1h",
      bullish_score: 1.2,
      bearish_score: 0.1,
      neutral_score: 0.2,
      top_signals: [
        {
          pattern: "Hammer",
          classification: "bullish",
          confidence: 0.7
        }
      ]
    }
  ]
};

const payloadWithoutSummaries = {
  as_of_ms: 1710000000000,
  detections: [
    {
      coin: "ETH",
      interval: "4h",
      pattern: "Doji",
      category: "candlestick_reversal",
      classification: "neutral",
      signal_type: "reversal",
      confidence: 0.55,
      detected_at_ms: 1710000000000,
      window_start_ms: 1709990000000,
      window_end_ms: 1710000000000,
      notes: ""
    }
  ]
};

describe("parsePatternSnapshot", () => {
  it("parses summaries when present", () => {
    const result = parsePatternSnapshot(JSON.stringify(payloadWithSummaries));
    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value.summaries).toHaveLength(1);
      expect(result.value.summaries[0].coin).toBe("BTC");
      expect(result.value.summaries[0].topSignals[0].pattern).toBe("Hammer");
    }
  });

  it("defaults summaries to empty when missing", () => {
    const result = parsePatternSnapshot(JSON.stringify(payloadWithoutSummaries));
    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value.summaries).toEqual([]);
    }
  });
});
