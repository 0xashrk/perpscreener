import { parseChartSnapshot } from "../services/chart";

describe("parseChartSnapshot", () => {
  it("parses chart snapshots", () => {
    const payload = {
      as_of_ms: 1000,
      coin: "BTC",
      interval: "1m",
      candles: [
        {
          t: 1,
          T: 2,
          o: 100,
          h: 110,
          l: 90,
          c: 105,
          v: 12,
          n: 3
        }
      ]
    };

    const result = parseChartSnapshot(JSON.stringify(payload));
    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value.candles).toHaveLength(1);
      expect(result.value.candles[0].close).toBe(105);
    }
  });
});
