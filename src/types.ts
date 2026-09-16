export interface Country {
  id: string;
  name: string;
  iso: string | null;
  cover_relpath: string | null;
  sort: number;
  created_at: string;
  place_count: number;
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

export interface LinkRecord {
  id: string;
  place_id: string | null;
  country_id: string | null;
  url: string;
  title: string | null;
  thumb_relpath: string | null;
  source: string;
}

export interface InboxItem {
  id: string;
  url: string;
  title: string | null;
  thumb_relpath: string | null;
  image_relpath: string | null;
  source: string;
  created_at: string;
  filed_at: string | null;
}

export interface SearchHit {
  kind: "country" | "place" | "inbox";
  id: string;
  title: string;
  subtitle: string | null;
  country_id: string | null;
}

export type View =
  | { kind: "home" }
  | { kind: "inbox" }
  | { kind: "country"; countryId: string }
  | { kind: "place"; placeId: string; countryId: string };
