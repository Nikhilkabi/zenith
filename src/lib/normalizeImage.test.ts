import { describe, expect, it } from "vitest";
import { isAvifBytes } from "./normalizeImage";

describe("isAvifBytes", () => {
  it("detects an AVIF ftyp brand", () => {
    const bytes = new Uint8Array(16);
    bytes.set([0x00, 0x00, 0x00, 0x14, 0x66, 0x74, 0x79, 0x70, 0x61, 0x76, 0x69, 0x66]);
    expect(isAvifBytes(bytes)).toBe(true);
  });

  it("rejects a JPEG header", () => {
    expect(isAvifBytes([0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10])).toBe(false);
  });
});
