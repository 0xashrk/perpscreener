import { render, screen } from "@testing-library/react";
import { vi } from "vitest";
import { PatternStateMachineStack } from "../features/screener/PatternStateMachineStack";
import { PatternLifecycleEntry } from "../types/patterns";

const makeEntry = (overrides: Partial<PatternLifecycleEntry> = {}): PatternLifecycleEntry => ({
  coin: "BTC",
  interval: "1m",
  pattern: "Ascending Triangle",
  category: "chart_pattern",
  classification: "bullish",
  signalType: "continuation",
  state: "forming",
  confidence: 0.72,
  stateSinceMs: 1_000,
  lastUpdatedMs: 2_000,
  windowStartMs: 0,
  windowEndMs: 0,
  notes: "",
  ...overrides
});

vi.mock("../hooks/usePatternLifecycleStream", () => ({
  usePatternLifecycleStream: () => ({
    status: "open",
    snapshot: {
      asOfMs: 0,
      entries: [
        makeEntry(),
        makeEntry({
          coin: "ETH",
          pattern: "Double Top",
          classification: "bearish",
          signalType: "reversal",
          state: "confirmed",
          lastUpdatedMs: 3_000
        })
      ]
    },
    error: ""
  })
}));

vi.mock("../hooks/usePatternRegistry", () => ({
  usePatternRegistry: () => ({
    status: "ready",
    entries: [
      {
        pattern: "Ascending Triangle",
        category: "chart_pattern",
        classification: "bullish",
        signalType: "continuation",
        window: 8,
        maxAgeBars: 16
      },
      {
        pattern: "Double Top",
        category: "chart_pattern",
        classification: "bearish",
        signalType: "reversal",
        window: 8,
        maxAgeBars: 16
      }
    ],
    error: ""
  })
}));

describe("PatternStateMachineStack", () => {
  it("renders a legacy-style table for each detected pattern", () => {
    render(<PatternStateMachineStack tokens={["BTC", "ETH"]} nowMs={4_000} />);

    expect(screen.getByRole("columnheader", { name: /Ascending Triangle State/i })).toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: /Double Top State/i })).toBeInTheDocument();
    expect(screen.getAllByText(/No signal/i)).toHaveLength(2);
  });
});
