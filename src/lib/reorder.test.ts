import { describe, expect, it } from "vitest";
import { moveItem } from "./reorder";

describe("moveItem", () => {
  it("moves an id in front of another", () => {
    const next = moveItem(
      [{ id: "a" }, { id: "b" }, { id: "c" }],
      "c",
      "a",
    );
    expect(next.map((i) => i.id)).toEqual(["c", "a", "b"]);
  });

  it("leaves the list alone when ids are missing", () => {
    const items = [{ id: "a" }, { id: "b" }];
    expect(moveItem(items, "z", "a")).toBe(items);
  });
});
