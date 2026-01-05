import { render, screen } from "@testing-library/react";
import { vi } from "vitest";
import { PatternScreeningPage } from "../features/patterns/PatternScreeningPage";

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

vi.mock("../hooks/usePatternStream", () => ({
  usePatternStream: () => ({
    status: "open",
    snapshot: { asOfMs: 0, detections: [], summaries: [] },
    error: ""
  })
}));

vi.mock("../hooks/useAdvancedPatternStream", () => ({
  useAdvancedPatternStream: () => ({
    status: "open",
    snapshot: { asOfMs: 0, detections: [] },
    error: ""
  })
}));

vi.mock("../hooks/usePatternLifecycleStream", () => ({
  usePatternLifecycleStream: () => ({
    status: "open",
    snapshot: { asOfMs: 0, entries: [] },
    error: ""
  })
}));

vi.mock("../hooks/useChartStream", () => ({
  useChartStream: () => ({
    status: "open",
    snapshot: { asOfMs: 0, coin: "BTC", interval: "1m", candles: [] },
    error: ""
  })
}));

describe("PatternScreeningPage", () => {
  it("renders the pattern screening header copy", () => {
    render(<PatternScreeningPage />);

    expect(screen.getByText(/Pattern Screening/i)).toBeInTheDocument();
    expect(screen.getByText(/Multi-timeframe pattern visualization/i)).toBeInTheDocument();
  });
});
