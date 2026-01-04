import { render, screen } from "@testing-library/react";
import { ScreenerTable } from "../components/ScreenerTable";
import { PatternState, VwapTokenState } from "../types/ui";
import { VwapTimeframe } from "../types/vwap";

const timeframes: VwapTimeframe[] = ["session", "1h", "4h"];

const buildVwapState = (): VwapTokenState => ({
  lastUpdatedMs: 1710000000000,
  byTimeframe: {
    session: { position: "above", distancePct: 0.2, hasData: true },
    "1h": { position: "below", distancePct: -0.1, hasData: true },
    "4h": { position: "above", distancePct: 0.4, hasData: true }
  }
});

const buildPatternState = (label: string): PatternState => ({
  stateKey: label === "Forming" ? "FORMING" : "CONFIRMED",
  stateLabel: label,
  lastUpdatedMs: 1710000000000,
  hasData: true
});

describe("ScreenerTable", () => {
  it("renders token rows and VWAP cells", () => {
    render(
      <ScreenerTable
        tokens={["BTC", "ETH"]}
        patternsByToken={{
          BTC: buildPatternState("Forming"),
          ETH: buildPatternState("Confirmed")
        }}
        vwapByToken={{
          BTC: buildVwapState(),
          ETH: buildVwapState()
        }}
        timeframes={timeframes}
        nowMs={1710000100000}
      />
    );

    expect(screen.getByText("BTC")).toBeInTheDocument();
    expect(screen.getByText("ETH")).toBeInTheDocument();
    expect(screen.getAllByText("Forming").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Confirmed").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Above").length).toBeGreaterThan(0);
  });
});
