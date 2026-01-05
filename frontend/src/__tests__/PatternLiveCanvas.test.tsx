import { render, screen } from "@testing-library/react";
import { vi } from "vitest";
import { PatternLiveCanvas } from "../features/patterns/PatternLiveCanvas";

const useChartStreamMock = vi.hoisted(() =>
  vi.fn(() => ({
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
  }))
);

vi.mock("../hooks/useChartStream", () => ({
  useChartStream: (...args: [string, string, number]) => useChartStreamMock(...args)
}));

vi.mock("lightweight-charts", () => {
  const series = {
    setData: vi.fn(),
    setMarkers: vi.fn()
  };
  const chart = {
    addCandlestickSeries: vi.fn(() => series),
    timeScale: () => ({ fitContent: vi.fn() }),
    applyOptions: vi.fn(),
    remove: vi.fn()
  };
  return {
    createChart: vi.fn(() => chart),
    ColorType: { Solid: "solid" },
    CrosshairMode: { Normal: 0 }
  };
});

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
    expect(useChartStreamMock).toHaveBeenCalledWith("BTC", "1m", 180);
  });
});
