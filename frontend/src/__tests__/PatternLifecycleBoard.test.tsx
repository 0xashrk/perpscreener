import { render, screen } from "@testing-library/react";
import { PatternLifecycleBoard } from "../features/patterns/PatternLifecycleBoard";
import { PatternLifecycleEntry } from "../types/patterns";

const entry: PatternLifecycleEntry = {
  coin: "BTC",
  interval: "1m",
  pattern: "Ascending Triangle",
  category: "chart_continuation",
  classification: "bullish",
  signalType: "continuation",
  state: "confirmed",
  confidence: 0.72,
  stateSinceMs: 1,
  lastUpdatedMs: 2,
  windowStartMs: 0,
  windowEndMs: 0,
  notes: ""
};

describe("PatternLifecycleBoard", () => {
  it("renders active lifecycle entries", () => {
    render(
      <PatternLifecycleBoard entries={[entry]} status="open" error="" nowMs={10} />
    );

    expect(screen.getByText(/Live pattern board/i)).toBeInTheDocument();
    expect(screen.getByText(/Ascending Triangle/i)).toBeInTheDocument();
  });
});
