import { render, screen } from "@testing-library/react";
import { vi } from "vitest";
import { PatternLiveCanvas } from "../features/patterns/PatternLiveCanvas";

vi.mock("../hooks/useChartStream", () => ({
  useChartStream: () => ({
    status: "open",
    snapshot: {
      asOfMs: 1000,
      coin: "BTC",
      interval: "1m",
      candles: [
        {
          openTime: 1,
          closeTime: 2,
          open: 100,
          high: 110,
          low: 90,
          close: 105,
          volume: 12,
          numTrades: 3
        }
      ]
    },
    error: ""
  })
}));

describe("PatternLiveCanvas", () => {
  it("renders live canvas with overlay markers", () => {
    render(
      <PatternLiveCanvas
        signals={[
          {
            pattern: "Double Top",
            coin: "BTC",
            interval: "1m",
            detectedAtMs: 1000,
            windowEndMs: 2
          }
        ]}
        status="open"
        lastUpdatedMs={1000}
      />
    );

    expect(screen.getByText(/Live Canvas/i)).toBeInTheDocument();
    expect(screen.getByText(/Overlay markers highlight/i)).toBeInTheDocument();
    expect(screen.getByText(/Double Top/i)).toBeInTheDocument();
  });
});
