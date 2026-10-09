import { describe, expect, it } from "vitest";
import { looksLikeUrl } from "./url";

describe("looksLikeUrl", () => {
  it("accepts http and https", () => {
    expect(looksLikeUrl("https://example.com/a")).toBe(true);
    expect(looksLikeUrl("http://example.com")).toBe(true);
    expect(looksLikeUrl("www.example.com/x")).toBe(true);
  });

  it("rejects dangerous schemes", () => {
    expect(looksLikeUrl("javascript:alert(1)")).toBe(false);
    expect(looksLikeUrl("file:///C:/Windows/win.ini")).toBe(false);
    expect(looksLikeUrl("data:text/html,hi")).toBe(false);
  });
});
