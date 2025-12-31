import { render, screen } from "@testing-library/react";
import { PatternList } from "../features/patterns/PatternList";

describe("PatternList", () => {
  it("renders empty state when no detections", () => {
    render(<PatternList detections={[]} status="ready" error="" />);

    expect(screen.getByText(/No patterns detected yet/i)).toBeInTheDocument();
  });
});
