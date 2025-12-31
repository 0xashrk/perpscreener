import { render, screen } from "@testing-library/react";
import { PatternScreeningPage } from "../features/patterns/PatternScreeningPage";

describe("PatternScreeningPage", () => {
  it("renders the pattern screening header copy", () => {
    render(<PatternScreeningPage />);

    expect(screen.getByText(/Pattern Screening/i)).toBeInTheDocument();
    expect(screen.getByText(/Multi-timeframe pattern visualization/i)).toBeInTheDocument();
  });
});
