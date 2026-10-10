import { invoke } from "@tauri-apps/api/core";
import type {
  Country,
  CoverCredit,
  CoverPage,
  Goal,
  PhotoSources,
  ImageRecord,
  LinkRecord,
  Place,
  PlaceTask,
  SearchHit,
  TrashItem,
  Trip,
  TripDetail,
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

export function listAllPlaces(): Promise<Place[]> {
  return invoke("list_all_places");
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
  patch: { name?: string; notes?: string; status?: "dream" | "been"; whenText?: string },
): Promise<Place> {
  return invoke("update_place", {
    placeId,
    name: patch.name ?? null,
    notes: patch.notes ?? null,
    status: patch.status ?? null,
    whenText: patch.whenText ?? null,
  });
}

export function deletePlace(placeId: string): Promise<void> {
  return invoke("delete_place", { placeId });
}

export function listImages(placeId: string): Promise<ImageRecord[]> {
  return invoke("list_images", { placeId });
}

export function deleteImage(imageId: string): Promise<void> {
  return invoke("delete_image", { imageId });
}

export function setPlaceCover(placeId: string, imageId: string): Promise<Place> {
  return invoke("set_place_cover", { placeId, imageId });
}

export function setCountryCover(
  countryId: string,
  imageId: string,
): Promise<Country> {
  return invoke("set_country_cover", { countryId, imageId });
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

export function search(query: string): Promise<SearchHit[]> {
  return invoke("search", { query });
}

export function exportLibraryZip(): Promise<void> {
  return invoke("export_library_zip");
}

export function openUrl(url: string): Promise<void> {
  return invoke("open_url", { url });
}

export function sweepOrphans(): Promise<number> {
  return invoke("sweep_orphans");
}

export function listCountryPhotos(countryId: string): Promise<ImageRecord[]> {
  return invoke("list_country_photos", { countryId });
}

export function reorderCountries(ids: string[]): Promise<void> {
  return invoke("reorder_countries", { ids });
}

export function reorderPlaces(ids: string[]): Promise<void> {
  return invoke("reorder_places", { ids });
}

export function updateImageCaption(
  imageId: string,
  caption?: string,
): Promise<ImageRecord> {
  return invoke("update_image_caption", { imageId, caption: caption ?? null });
}

export function listTrash(): Promise<TrashItem[]> {
  return invoke("list_trash");
}

export function restoreTrash(kind: string, id: string): Promise<void> {
  return invoke("restore_trash", { kind, id });
}

export function purgeTrash(): Promise<number> {
  return invoke("purge_trash");
}

export function dropOnCountry(
  countryId: string,
  filename: string,
  bytes: number[],
): Promise<ImageRecord> {
  return invoke("drop_on_country", { countryId, filename, bytes });
}

export function importLibraryZip(): Promise<number> {
  return invoke("import_library_zip");
}

export type CoverTarget = "goal" | "country" | "place" | "trip";

export function listTrips(): Promise<Trip[]> {
  return invoke("list_trips");
}

export function getTrip(tripId: string): Promise<TripDetail> {
  return invoke("get_trip", { tripId });
}

export function createTrip(name: string): Promise<Trip> {
  return invoke("create_trip", { name });
}

export function updateTrip(
  tripId: string,
  patch: {
    name?: string;
    whenText?: string;
    currency?: string;
    notes?: string;
    status?: "dream" | "done";
    markPlaces?: boolean;
    doneYear?: number;
    month?: number;
    planYear?: number;
  },
): Promise<Trip> {
  return invoke("update_trip", {
    tripId,
    name: patch.name ?? null,
    whenText: patch.whenText ?? null,
    currency: patch.currency ?? null,
    notes: patch.notes ?? null,
    status: patch.status ?? null,
    markPlaces: patch.markPlaces ?? false,
    doneYear: patch.doneYear ?? null,
    month: patch.month ?? null,
    planYear: patch.planYear ?? null,
  });
}

export function deleteTrip(tripId: string): Promise<void> {
  return invoke("delete_trip", { tripId });
}

export function addTripStop(tripId: string, placeId: string): Promise<TripDetail> {
  return invoke("add_trip_stop", { tripId, placeId });
}

export function removeTripStop(tripId: string, placeId: string): Promise<TripDetail> {
  return invoke("remove_trip_stop", { tripId, placeId });
}

export function reorderTripStops(tripId: string, placeIds: string[]): Promise<TripDetail> {
  return invoke("reorder_trip_stops", { tripId, placeIds });
}

export function setTripCost(
  tripId: string,
  category: string,
  amount: number | null,
): Promise<TripDetail> {
  return invoke("set_trip_cost", { tripId, category, amount });
}

export function listPlaceTasks(placeId: string): Promise<PlaceTask[]> {
  return invoke("list_place_tasks", { placeId });
}

export function addPlaceTask(placeId: string, body: string): Promise<PlaceTask> {
  return invoke("add_place_task", { placeId, body });
}

export function setPlaceTask(
  taskId: string,
  patch: { body?: string; done?: boolean },
): Promise<PlaceTask> {
  return invoke("set_place_task", {
    taskId,
    body: patch.body ?? null,
    done: patch.done ?? null,
  });
}

export function deletePlaceTask(taskId: string): Promise<void> {
  return invoke("delete_place_task", { taskId });
}

export function listGoals(): Promise<Goal[]> {
  return invoke("list_goals");
}

export function getGoal(goalId: string): Promise<Goal> {
  return invoke("get_goal", { goalId });
}

export function createGoal(name: string): Promise<Goal> {
  return invoke("create_goal", { name });
}

export function updateGoal(
  goalId: string,
  patch: { name?: string; notes?: string; status?: "dream" | "done"; doneYear?: number; month?: number; planYear?: number },
): Promise<Goal> {
  return invoke("update_goal", {
    goalId,
    name: patch.name ?? null,
    notes: patch.notes ?? null,
    status: patch.status ?? null,
    doneYear: patch.doneYear ?? null,
    month: patch.month ?? null,
    planYear: patch.planYear ?? null,
  });
}

export function deleteGoal(goalId: string): Promise<void> {
  return invoke("delete_goal", { goalId });
}

export function reorderGoals(ids: string[]): Promise<void> {
  return invoke("reorder_goals", { ids });
}

export function coverCredit(relpath: string): Promise<CoverCredit | null> {
  return invoke("cover_credit", { relpath });
}

export function getPhotoSources(): Promise<PhotoSources> {
  return invoke("get_photo_sources");
}

export function setPhotoSources(pexels: string, pixabay: string): Promise<PhotoSources> {
  return invoke("set_photo_sources", { pexels, pixabay });
}

export function importStockPhoto(placeId: string, imageId: string): Promise<ImageRecord> {
  return invoke("import_stock_photo", { placeId, imageId });
}

export function searchCovers(query: string, page: number): Promise<CoverPage> {
  return invoke("search_covers", { query, page });
}

export function applyStockCover(
  target: CoverTarget,
  ownerId: string,
  imageId: string,
  focusX: number,
  focusY: number,
): Promise<void> {
  return invoke("apply_stock_cover", { target, ownerId, imageId, focusX, focusY });
}

export function applyFileCover(
  target: CoverTarget,
  ownerId: string,
  filename: string,
  bytes: number[],
  focusX: number,
  focusY: number,
): Promise<void> {
  return invoke("apply_file_cover", { target, ownerId, filename, bytes, focusX, focusY });
}

export function setCoverFocus(
  target: CoverTarget,
  ownerId: string,
  focusX: number,
  focusY: number,
): Promise<void> {
  return invoke("set_cover_focus", { target, ownerId, focusX, focusY });
}

export async function fileToBytes(file: File): Promise<number[]> {
  const buffer = await file.arrayBuffer();
  return Array.from(new Uint8Array(buffer));
}

export { looksLikeUrl } from "./lib/url";
