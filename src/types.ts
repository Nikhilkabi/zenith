export interface Country {
  id: string;
  name: string;
  iso: string | null;
  cover_relpath: string | null;
  sort: number;
  created_at: string;
}

export interface Place {
  id: string;
  country_id: string;
  name: string;
  cover_relpath: string | null;
  notes: string | null;
  status: "dream" | "been";
  sort: number;
  created_at: string;
}

export interface ImageRecord {
  id: string;
  place_id: string;
  original_relpath: string;
  display_relpath: string;
  thumb_relpath: string;
  sort: number;
}

export type View =
  | { kind: "home" }
  | { kind: "country"; countryId: string }
  | { kind: "place"; placeId: string; countryId: string };
