import { invoke } from "@tauri-apps/api/core";
import type {
  Country,
  ImageRecord,
  InboxItem,
  LinkRecord,
  Place,
  SearchHit,
} from "./types";

export function listCountries(): Promise<Country[]> {
  return invoke("list_countries");
}

export function createCountry(name: string, iso?: string): Promise<Country> {
  return invoke("create_country", { name, iso: iso || null });
}

export function updateCountry(
  countryId: string,
  patch: { name?: string; iso?: string },
): Promise<Country> {
  return invoke("update_country", {
    countryId,
    name: patch.name ?? null,
    iso: patch.iso ?? null,
  });
}

export function deleteCountry(countryId: string): Promise<void> {
  return invoke("delete_country", { countryId });
}

export function getLibraryPath(): Promise<string> {
  return invoke("get_library_path");
}

export function revealLibrary(): Promise<void> {
  return invoke("reveal_library");
}

export function listPlaces(countryId: string): Promise<Place[]> {
  return invoke("list_places", { countryId });
}

export function getPlace(placeId: string): Promise<Place> {
  return invoke("get_place", { placeId });
}

export function createPlace(
  countryId: string,
  name: string,
  status?: "dream" | "been",
): Promise<Place> {
  return invoke("create_place", { countryId, name, status: status ?? null });
}

export function updatePlace(
  placeId: string,
  patch: { name?: string; notes?: string; status?: "dream" | "been" },
): Promise<Place> {
  return invoke("update_place", {
    placeId,
    name: patch.name ?? null,
    notes: patch.notes ?? null,
    status: patch.status ?? null,
  });
}

export function deletePlace(placeId: string): Promise<void> {
  return invoke("delete_place", { placeId });
}

export function listImages(placeId: string): Promise<ImageRecord[]> {
  return invoke("list_images", { placeId });
}

export function importImageBytes(
  placeId: string,
  filename: string,
  bytes: number[],
): Promise<ImageRecord> {
  return invoke("import_image_bytes", { placeId, filename, bytes });
}

export function importImagePath(
  placeId: string,
  sourcePath: string,
): Promise<ImageRecord> {
  return invoke("import_image_path", { placeId, sourcePath });
}

export function listLinks(args: {
  placeId?: string;
  countryId?: string;
}): Promise<LinkRecord[]> {
  return invoke("list_links", {
    placeId: args.placeId ?? null,
    countryId: args.countryId ?? null,
  });
}

export function addLink(args: {
  url: string;
  title?: string;
  placeId?: string;
  countryId?: string;
}): Promise<LinkRecord> {
  return invoke("add_link", {
    url: args.url,
    title: args.title ?? null,
    placeId: args.placeId ?? null,
    countryId: args.countryId ?? null,
  });
}

export function deleteLink(linkId: string): Promise<void> {
  return invoke("delete_link", { linkId });
}

export function listInbox(): Promise<InboxItem[]> {
  return invoke("list_inbox");
}

export function inboxCount(): Promise<number> {
  return invoke("inbox_count");
}

export function captureInboxUrl(url: string): Promise<InboxItem> {
  return invoke("capture_inbox_url", { url });
}

export function captureInboxImage(
  filename: string,
  bytes: number[],
): Promise<InboxItem> {
  return invoke("capture_inbox_image", { filename, bytes });
}

export function fileInbox(args: {
  inboxId: string;
  countryId: string;
  placeId?: string;
  newPlaceName?: string;
}): Promise<InboxItem> {
  return invoke("file_inbox", {
    inboxId: args.inboxId,
    countryId: args.countryId,
    placeId: args.placeId ?? null,
    newPlaceName: args.newPlaceName ?? null,
  });
}

export function deleteInbox(inboxId: string): Promise<void> {
  return invoke("delete_inbox", { inboxId });
}

export function search(query: string): Promise<SearchHit[]> {
  return invoke("search", { query });
}

export function exportLibraryZip(destPath: string): Promise<void> {
  return invoke("export_library_zip", { destPath });
}
