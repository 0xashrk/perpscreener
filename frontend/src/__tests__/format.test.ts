import { formatDistancePct } from "../utils/format";
import { formatAge } from "../utils/time";

describe("formatDistancePct", () => {
  it("formats percent with sign", () => {
    expect(formatDistancePct(0.1234)).toBe("+0.12%");
    expect(formatDistancePct(-1.2)).toBe("-1.20%");
  });
});

describe("formatAge", () => {
  it("formats seconds and minutes", () => {
    expect(formatAge(1000, 55000)).toBe("54s");
    expect(formatAge(1000, 121000)).toBe("2m");
  });

  it("handles empty timestamps", () => {
    expect(formatAge(0, 1000)).toBe("--");
  });
});
