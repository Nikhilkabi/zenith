export interface Goal {
  id: string;
  name: string;
  notes: string | null;
  status: "dream" | "done";
  cover_relpath: string | null;
  cover_credit: string | null;
  cover_credit_url: string | null;
  cover_x: number;
  cover_y: number;
  sort: number;
  created_at: string;
  done_year: number | null;
  month: number | null;
  year: number | null;
}

export interface CoverHit {
  id: string;
  thumb_relpath: string;
  credit: string;
}

export interface CoverPage {
  hits: CoverHit[];
  page: number;
  page_count: number;
  warning?: string | null;
}

export interface PhotoSources {
  pexels: string;
  pixabay: string;
}

export interface CoverCredit {
  credit: string;
  credit_url: string | null;
}

export interface Country {
  id: string;
  name: string;
  iso: string | null;
  cover_relpath: string | null;
  cover_x: number;
  cover_y: number;
  own_cover: boolean;
  sort: number;
  created_at: string;
  place_count: number;
  been_count: number;
}

export interface Place {
  id: string;
  country_id: string;
  name: string;
  cover_relpath: string | null;
  cover_x: number;
  cover_y: number;
  notes: string | null;
  status: "dream" | "been";
  sort: number;
  created_at: string;
  image_count: number;
  when_text: string | null;
  year: number | null;
}

export interface Trip {
  id: string;
  name: string;
  when_text: string | null;
  year: number | null;
  currency: string | null;
  notes: string | null;
  status: "dream" | "done";
  cover_relpath: string | null;
  cover_credit: string | null;
  cover_credit_url: string | null;
  cover_x: number;
  cover_y: number;
  sort: number;
  created_at: string;
  stop_count: number;
  total: number;
  cost_count: number;
  done_year: number | null;
  month: number | null;
}

export interface TripStop {
  place_id: string;
  country_id: string;
  place_name: string;
  country_name: string;
  status: "dream" | "been";
  sort: number;
}

export interface TripCost {
  category: string;
  amount: number | null;
}

export interface TripDetail {
  trip: Trip;
  stops: TripStop[];
  costs: TripCost[];
}

export interface PlaceTask {
  id: string;
  place_id: string;
  body: string;
  done: boolean;
  sort: number;
}

export const TRIP_COSTS: { category: string; label: string }[] = [
  { category: "visa", label: "Visas" },
  { category: "sim", label: "SIM" },
  { category: "flights", label: "Flights" },
  { category: "hotels", label: "Hotels" },
  { category: "food", label: "Food" },
  { category: "transport", label: "Transport" },
  { category: "tickets", label: "Tickets and entrance" },
  { category: "activities", label: "Activities" },
  { category: "guide", label: "Tour guide" },
  { category: "insurance", label: "Insurance" },
  { category: "other", label: "Other" },
];

export interface ImageRecord {
  id: string;
  place_id: string;
  original_relpath: string;
  display_relpath: string;
  thumb_relpath: string;
  sort: number;
  sha256: string | null;
  caption: string | null;
  taken_at: string | null;
}

export interface TrashItem {
  kind: "country" | "place" | "image" | "goal";
  id: string;
  title: string;
  deleted_at: string;
}

export interface LinkRecord {
  id: string;
  place_id: string | null;
  country_id: string | null;
  url: string;
  title: string | null;
  thumb_relpath: string | null;
  source: string;
}

export interface SearchHit {
  kind: "country" | "place" | "link" | "goal" | "trip";
  id: string;
  title: string;
  subtitle: string | null;
  country_id: string | null;
}

export type View =
  | { kind: "home" }
  | { kind: "country"; countryId: string }
  | { kind: "place"; placeId: string; countryId: string }
  | { kind: "goal"; goalId: string }
  | { kind: "trip"; tripId: string }
  | { kind: "trash" };

export const PRIMARIES = ["#1F4BA0", "#F5C400", "#D62B1F"] as const;

export function primaryFor(id: string): string {
  let hash = 0;
  for (let i = 0; i < id.length; i++) hash = (hash + id.charCodeAt(i) * (i + 1)) % 997;
  return PRIMARIES[hash % PRIMARIES.length];
}
