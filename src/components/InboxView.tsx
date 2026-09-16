import { useEffect, useState, type FormEvent } from "react";
import {
  captureInboxImage,
  captureInboxUrl,
  deleteInbox,
  fileInbox,
  listInbox,
  listPlaces,
} from "../api";
import { assetUrl } from "../lib/assets";
import type { Country, InboxItem, Place } from "../types";

interface Props {
  countries: Country[];
  libraryRoot: string;
  onFiled: () => void;
  onBack: () => void;
}

async function fileToBytes(file: File): Promise<number[]> {
  const buffer = await file.arrayBuffer();
  return Array.from(new Uint8Array(buffer));
}

export default function InboxView({
  countries,
  libraryRoot,
  onFiled,
  onBack,
}: Props) {
  const [items, setItems] = useState<InboxItem[]>([]);
  const [url, setUrl] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [dragActive, setDragActive] = useState(false);

  async function refresh() {
    setItems(await listInbox());
  }

  useEffect(() => {
    refresh().catch((err) => setError(String(err)));
  }, []);

  async function handlePaste(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setError(null);
    try {
      await captureInboxUrl(url);
      setUrl("");
      await refresh();
      onFiled();
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }

  async function importFiles(files: FileList | File[]) {
    const list = Array.from(files).filter((file) => file.type.startsWith("image/"));
    if (list.length === 0) {
      setError("Drop a JPEG, PNG, or WebP — or paste a URL above.");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      for (const file of list) {
        await captureInboxImage(file.name, await fileToBytes(file));
      }
      await refresh();
      onFiled();
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <div className="section-header">
        <div>
          <button type="button" onClick={onBack}>
            ← Countries
          </button>
          <h1 className="display" style={{ marginTop: "0.75rem" }}>
            Inbox
          </h1>
          <p className="lede">Dump a link or a still. File it when you know the country.</p>
        </div>
      </div>

      {error && <div className="error-banner">{error}</div>}

      <form onSubmit={handlePaste} className="form-inline" style={{ marginBottom: "1rem" }}>
        <div className="form-row">
          <label htmlFor="inbox-url">Paste URL</label>
          <input
            id="inbox-url"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            placeholder="YouTube, Instagram, or any page"
          />
        </div>
        <button className="primary" type="submit" disabled={busy || !url.trim()}>
          Save to inbox
        </button>
      </form>

      <div
        className={`dropzone${dragActive ? " active" : ""}`}
        onDragOver={(event) => {
          event.preventDefault();
          setDragActive(true);
        }}
        onDragLeave={() => setDragActive(false)}
        onDrop={(event) => {
          event.preventDefault();
          setDragActive(false);
          void importFiles(event.dataTransfer.files);
        }}
      >
        <p>
          <strong>Drop screenshots here</strong>
        </p>
        <p>Instagram usually needs a screenshot. YouTube can fetch its own title.</p>
      </div>

      {items.length === 0 ? (
        <div className="empty-state">
          <h2>Inbox is clear</h2>
          <p>Paste a URL or drop a frame the next time something catches you.</p>
        </div>
      ) : (
        <div className="inbox-list">
          {items.map((item) => (
            <InboxCard
              key={item.id}
              item={item}
              countries={countries}
              libraryRoot={libraryRoot}
              busy={busy}
              onChange={async () => {
                await refresh();
                onFiled();
              }}
              onError={setError}
              setBusy={setBusy}
            />
          ))}
        </div>
      )}
    </>
  );
}

function InboxCard({
  item,
  countries,
  libraryRoot,
  busy,
  onChange,
  onError,
  setBusy,
}: {
  item: InboxItem;
  countries: Country[];
  libraryRoot: string;
  busy: boolean;
  onChange: () => Promise<void>;
  onError: (message: string | null) => void;
  setBusy: (value: boolean) => void;
}) {
  const [countryId, setCountryId] = useState(countries[0]?.id ?? "");
  const [placeId, setPlaceId] = useState("");
  const [newPlace, setNewPlace] = useState("");
  const [places, setPlaces] = useState<Place[]>([]);

  useEffect(() => {
    if (!countryId) {
      setPlaces([]);
      return;
    }
    listPlaces(countryId)
      .then(setPlaces)
      .catch((err) => onError(String(err)));
  }, [countryId]);

  const thumb = item.thumb_relpath || item.image_relpath;

  return (
    <article className="inbox-card">
      <div className={`card-cover${thumb ? "" : " typographic"}`}>
        {thumb ? (
          <img src={assetUrl(libraryRoot, thumb)} alt="" />
        ) : (
          <span>{item.source.slice(0, 2).toUpperCase()}</span>
        )}
      </div>
      <div className="inbox-body">
        <h3>{item.title || item.url || "Untitled save"}</h3>
        {item.url ? (
          <p className="mono clip">{item.url}</p>
        ) : (
          <p>Screenshot</p>
        )}
        <div className="form-inline" style={{ marginTop: "0.75rem" }}>
          <div className="form-row">
            <label>Country</label>
            <select value={countryId} onChange={(e) => setCountryId(e.target.value)}>
              {countries.length === 0 && <option value="">Add a country first</option>}
              {countries.map((country) => (
                <option key={country.id} value={country.id}>
                  {country.name}
                </option>
              ))}
            </select>
          </div>
          <div className="form-row">
            <label>Place (optional)</label>
            <select value={placeId} onChange={(e) => setPlaceId(e.target.value)}>
              <option value="">Country only</option>
              {places.map((place) => (
                <option key={place.id} value={place.id}>
                  {place.name}
                </option>
              ))}
            </select>
          </div>
          <div className="form-row">
            <label>Or new place</label>
            <input
              value={newPlace}
              onChange={(e) => setNewPlace(e.target.value)}
              placeholder="Mount Bromo"
            />
          </div>
        </div>
        <div className="topbar-actions" style={{ marginTop: "0.75rem" }}>
          <button
            type="button"
            className="primary"
            disabled={busy || !countryId}
            onClick={() => {
              setBusy(true);
              onError(null);
              fileInbox({
                inboxId: item.id,
                countryId,
                placeId: placeId || undefined,
                newPlaceName: newPlace || undefined,
              })
                .then(onChange)
                .catch((err) => onError(String(err)))
                .finally(() => setBusy(false));
            }}
          >
            File
          </button>
          <button
            type="button"
            className="danger"
            disabled={busy}
            onClick={() => {
              if (!window.confirm("Remove this inbox item?")) return;
              setBusy(true);
              deleteInbox(item.id)
                .then(onChange)
                .catch((err) => onError(String(err)))
                .finally(() => setBusy(false));
            }}
          >
            Discard
          </button>
        </div>
      </div>
    </article>
  );
}
