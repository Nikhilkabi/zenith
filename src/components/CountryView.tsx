import { useEffect, useState, type FormEvent } from "react";
import { createPlace, listPlaces } from "../api";
import { assetUrl } from "../lib/assets";
import type { Country, Place } from "../types";

interface Props {
  country: Country;
  libraryRoot: string;
  onOpenPlace: (placeId: string) => void;
  onBack: () => void;
}

export default function CountryView({
  country,
  libraryRoot,
  onOpenPlace,
  onBack,
}: Props) {
  const [places, setPlaces] = useState<Place[]>([]);
  const [name, setName] = useState("");
  const [status, setStatus] = useState<"dream" | "been">("dream");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    listPlaces(country.id)
      .then(setPlaces)
      .catch((err) => setError(String(err)));
  }, [country.id]);

  async function handleCreate(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setError(null);
    try {
      const place = await createPlace(country.id, name, status);
      setPlaces((current) => [...current, place]);
      setName("");
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }

  const heroRelpath =
    country.cover_relpath ?? places.find((p) => p.cover_relpath)?.cover_relpath ?? null;

  return (
    <>
      <div className="section-header">
        <div>
          <button type="button" onClick={onBack}>
            ← All countries
          </button>
          <h1 style={{ marginTop: "0.75rem" }}>{country.name}</h1>
        </div>
      </div>

      <div className="hero">
        {heroRelpath ? (
          <img src={assetUrl(libraryRoot, heroRelpath)} alt={country.name} />
        ) : (
          <div className="hero-text">
            <h1>{country.name}</h1>
            <p>{country.iso?.toUpperCase() ?? "No cover yet — add a place with photos."}</p>
          </div>
        )}
      </div>

      {error && <div className="error-banner">{error}</div>}

      <form onSubmit={handleCreate} className="form-inline" style={{ marginBottom: "1.5rem" }}>
        <div className="form-row">
          <label htmlFor="place-name">Add place</label>
          <input
            id="place-name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Kyoto"
            required
          />
        </div>
        <div className="form-row" style={{ maxWidth: 160 }}>
          <label htmlFor="place-status">Status</label>
          <select
            id="place-status"
            value={status}
            onChange={(e) => setStatus(e.target.value as "dream" | "been")}
          >
            <option value="dream">Dream</option>
            <option value="been">Been</option>
          </select>
        </div>
        <button className="primary" type="submit" disabled={busy || !name.trim()}>
          Add place
        </button>
      </form>

      {places.length === 0 ? (
        <div className="empty-state">
          <h2>No places yet</h2>
          <p>Add a place to start dropping photos into your scrapbook.</p>
        </div>
      ) : (
        <div className="grid">
          {places.map((place) => (
            <article
              key={place.id}
              className="card"
              onClick={() => onOpenPlace(place.id)}
            >
              <div className={`card-cover${place.cover_relpath ? "" : " typographic"}`}>
                {place.cover_relpath ? (
                  <img
                    src={assetUrl(libraryRoot, place.cover_relpath)}
                    alt={place.name}
                    loading="lazy"
                  />
                ) : (
                  <span>{place.name.charAt(0).toUpperCase()}</span>
                )}
              </div>
              <div className="card-body">
                <h3>{place.name}</h3>
                <p>
                  <span className={`status-pill ${place.status}`}>{place.status}</span>
                </p>
              </div>
            </article>
          ))}
        </div>
      )}
    </>
  );
}
