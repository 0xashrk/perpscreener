import { parseDoubleTopSnapshot, parseVwapSnapshot } from "../services/parsers";

const doubleTopPayload = {
  as_of_ms: 1710000000000,
  patterns: [
    { coin: "BTC", state: "FORMING" },
    { coin: "ETH", state: "CONFIRMED" }
  ]
};

const vwapPayload = {
  as_of_ms: 1710000000000,
  coin: "BTC",
  current_price: 62000,
  vwaps: [
    { timeframe: "session", position: "above", distance_pct: 0.42, vwap: 61740 },
    { timeframe: "1h", position: "below", distance_pct: -0.18, vwap: 62110 },
    { timeframe: "4h", position: "above", distance_pct: 0.11, vwap: 61930 }
  ]
};

describe("parseDoubleTopSnapshot", () => {
  it("parses valid snapshot", () => {
    const result = parseDoubleTopSnapshot(JSON.stringify(doubleTopPayload));
    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value.patterns).toHaveLength(2);
      expect(result.value.patterns[0].coin).toBe("BTC");
      expect(result.value.patterns[1].state).toBe("CONFIRMED");
    }
  });

  it("rejects invalid payload", () => {
    const result = parseDoubleTopSnapshot("{}");
    expect(result.ok).toBe(false);
  });
});

describe("parseVwapSnapshot", () => {
  it("parses valid snapshot", () => {
    const result = parseVwapSnapshot(JSON.stringify(vwapPayload));
    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value.vwaps).toHaveLength(3);
      expect(result.value.vwaps[0].timeframe).toBe("session");
      expect(result.value.vwaps[1].position).toBe("below");
    }
  });

  it("rejects invalid payload", () => {
    const result = parseVwapSnapshot("{}");
    expect(result.ok).toBe(false);
  });
});
