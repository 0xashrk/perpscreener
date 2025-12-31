import { render, screen, waitFor } from "@testing-library/react";
import { useHashRoute } from "../hooks/useHashRoute";

const RouteProbe = () => {
  const route = useHashRoute();
  return <div>{route}</div>;
};

describe("useHashRoute", () => {
  afterEach(() => {
    window.location.hash = "";
  });

  it("updates when the hash changes", async () => {
    window.location.hash = "#/";
    render(<RouteProbe />);

    expect(screen.getByText("/")).toBeInTheDocument();

    window.location.hash = "#/patterns";
    window.dispatchEvent(new Event("hashchange"));

    await waitFor(() => {
      expect(screen.getByText("/patterns")).toBeInTheDocument();
    });
  });
});
