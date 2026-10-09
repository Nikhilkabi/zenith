import { describe, expect, it } from "vitest";
import { ISO_COUNTRIES, matchCountries } from "./countries";

describe("matchCountries", () => {
  it("includes the full ISO set, not a short A–Z sample", () => {
    expect(ISO_COUNTRIES.length).toBeGreaterThanOrEqual(200);
    const codes = new Set(ISO_COUNTRIES.map((c) => c.iso));
    for (const iso of ["AD", "AO", "JP", "UY", "YE", "ZW", "XK"]) {
      expect(codes.has(iso)).toBe(true);
    }
    expect(new Set(codes).size).toBe(ISO_COUNTRIES.length);
  });

  it("lists every country when the field is empty", () => {
    expect(matchCountries("")).toHaveLength(ISO_COUNTRIES.length);
    expect(matchCountries("").some((c) => c.name === "Zimbabwe")).toBe(true);
  });

  it("matches name, alias, or ISO code", () => {
    expect(matchCountries("indo").some((c) => c.iso === "ID")).toBe(true);
    expect(matchCountries("jp")[0]).toMatchObject({ name: "Japan", iso: "JP" });
    expect(matchCountries("uk")[0]).toMatchObject({ iso: "GB" });
    const letterA = matchCountries("a");
    expect(letterA.length).toBeGreaterThan(8);
    expect(letterA.some((c) => c.iso === "AD")).toBe(true);
  });
});
