import { convertFileSrc } from "@tauri-apps/api/core";

export function assetUrl(libraryRoot: string, relpath: string): string {
  const normalized = relpath.replace(/\\/g, "/");
  const abs = `${libraryRoot.replace(/\\/g, "/")}/${normalized}`;
  return convertFileSrc(abs);
}
