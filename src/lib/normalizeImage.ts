import { fileToBytes } from "../api";

/** ISO-BMFF AVIF/AVIS signature — Chrome often saves these with a .jpg name. */
export function isAvifBytes(bytes: number[] | Uint8Array): boolean {
  if (bytes.length < 12) return false;
  const b = bytes instanceof Uint8Array ? bytes : Uint8Array.from(bytes);
  if (b[4] !== 0x66 || b[5] !== 0x74 || b[6] !== 0x79 || b[7] !== 0x70) {
    return false;
  }
  const brand = String.fromCharCode(b[8], b[9], b[10], b[11]);
  if (brand === "avif" || brand === "avis" || brand === "mif1") return true;
  // Compatible brands further in the ftyp box
  const head = String.fromCharCode(...b.slice(8, Math.min(b.length, 32)));
  return head.includes("avif") || head.includes("avis");
}

function isBrowserNative(
  file: File,
  bytes: number[],
): boolean {
  const lower = file.name.toLowerCase();
  if (lower.endsWith(".avif") || file.type === "image/avif" || isAvifBytes(bytes)) {
    return false;
  }
  if (
    file.type === "image/jpeg" ||
    file.type === "image/png" ||
    file.type === "image/webp" ||
    lower.endsWith(".jpg") ||
    lower.endsWith(".jpeg") ||
    lower.endsWith(".png") ||
    lower.endsWith(".webp")
  ) {
    return true;
  }
  return false;
}

/**
 * Ensure bytes are JPEG/PNG/WebP for the Rust importer.
 * AVIF (and other browser-decodable formats) are re-encoded to JPEG via canvas.
 */
export async function normalizeImageFile(
  file: File,
): Promise<{ filename: string; bytes: number[] }> {
  const raw = await fileToBytes(file);

  if (isBrowserNative(file, raw) && !isAvifBytes(raw)) {
    return { filename: file.name, bytes: raw };
  }

  let bitmap: ImageBitmap;
  try {
    bitmap = await createImageBitmap(file);
  } catch {
    throw new Error(
      "Could not read this image. If it is AVIF, try Copy image in the browser, then Ctrl+V here — or save as JPEG/PNG.",
    );
  }

  try {
    const canvas = document.createElement("canvas");
    canvas.width = bitmap.width;
    canvas.height = bitmap.height;
    const ctx = canvas.getContext("2d");
    if (!ctx) throw new Error("Could not convert image.");
    ctx.drawImage(bitmap, 0, 0);

    const blob = await new Promise<Blob>((resolve, reject) => {
      canvas.toBlob(
        (b) => (b ? resolve(b) : reject(new Error("Could not convert image."))),
        "image/jpeg",
        0.92,
      );
    });

    const base =
      file.name.replace(/\.[^.]+$/, "").trim() || "photo";
    return {
      filename: `${base}.jpg`,
      bytes: await fileToBytes(new File([blob], `${base}.jpg`, { type: "image/jpeg" })),
    };
  } finally {
    bitmap.close();
  }
}
