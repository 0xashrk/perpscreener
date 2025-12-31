import { render, screen } from "@testing-library/react";
import { vi } from "vitest";
import { PatternScreeningPage } from "../features/patterns/PatternScreeningPage";

vi.mock("../hooks/usePatternStream", () => ({
  usePatternStream: () => ({
    status: "open",
    snapshot: { asOfMs: 0, detections: [] },
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

describe("PatternScreeningPage", () => {
  it("renders the pattern screening header copy", () => {
    render(<PatternScreeningPage />);

    expect(screen.getByText(/Pattern Screening/i)).toBeInTheDocument();
    expect(screen.getByText(/Multi-timeframe pattern visualization/i)).toBeInTheDocument();
  });
});
