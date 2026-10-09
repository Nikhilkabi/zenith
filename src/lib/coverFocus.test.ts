import { describe, expect, it } from "vitest";
import { focusAfterDrag, objectPosition } from "./coverFocus";

describe("cover focus", () => {
  it("keeps a centered photo centered", () => {
    expect(objectPosition(1000, 800, 400, 200, 50, 50)).toBe("50% 50%");
  });

  it("pins a high point to the middle of a wide frame", () => {
    expect(objectPosition(1000, 2000, 400, 200, 50, 20)).toBe("50% 10%");
  });

  it("moves the stored point up when the photo is dragged down", () => {
    const next = focusAfterDrag(50, 50, 0, 80, 400, 800, 400, 200);
    expect(next.y).toBe(40);
    expect(next.x).toBe(50);
  });

  it("does not shift an axis the frame already shows in full", () => {
    const next = focusAfterDrag(50, 50, 40, 0, 400, 800, 400, 200);
    expect(next.x).toBe(50);
  });
});
