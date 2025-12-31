import { render, screen } from "@testing-library/react";
import { AdvancedPatternList } from "../features/patterns/AdvancedPatternList";

describe("AdvancedPatternList", () => {
  it("renders empty state when no detections", () => {
    render(<AdvancedPatternList detections={[]} status="open" error="" />);

    expect(screen.getByText(/No advanced patterns detected yet/i)).toBeInTheDocument();
  });
});
