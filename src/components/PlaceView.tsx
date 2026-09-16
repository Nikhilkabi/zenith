import { open } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useState } from "react";
import {
  deletePlace,
  getPlace,
  importImageBytes,
  importImagePath,
  listImages,
  updatePlace,
} from "../api";
import { assetUrl } from "../lib/assets";
import type { ImageRecord, Place } from "../types";

interface Props {
  placeId: string;
  libraryRoot: string;
  onBack: () => void;
  onDeleted: () => void;
}

async function fileToBytes(file: File): Promise<number[]> {
  const buffer = await file.arrayBuffer();
  return Array.from(new Uint8Array(buffer));
}

export default function PlaceView({ placeId, libraryRoot, onBack, onDeleted }: Props) {
  const [place, setPlace] = useState<Place | null>(null);
  const [images, setImages] = useState<ImageRecord[]>([]);
  const [notes, setNotes] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [dragActive, setDragActive] = useState(false);

  const refresh = useCallback(async () => {
    const [nextPlace, nextImages] = await Promise.all([
      getPlace(placeId),
      listImages(placeId),
    ]);
    setPlace(nextPlace);
    setNotes(nextPlace.notes ?? "");
    setImages(nextImages);
  }, [placeId]);

  useEffect(() => {
    refresh().catch((err) => setError(String(err)));
  }, [refresh]);

  const importFiles = useCallback(
    async (files: FileList | File[]) => {
      const list = Array.from(files).filter((file) => file.type.startsWith("image/"));
      if (list.length === 0) {
        setError("Drop or paste a JPEG, PNG, or WebP image.");
        return;
      }

      setBusy(true);
      setError(null);
      try {
        for (const file of list) {
          const bytes = await fileToBytes(file);
          await importImageBytes(placeId, file.name, bytes);
        }
        await refresh();
      } catch (err) {
        setError(String(err));
      } finally {
        setBusy(false);
      }
    },
    [placeId, refresh],
  );

  async function pickFiles() {
    const selected = await open({
      multiple: true,
      filters: [{ name: "Images", extensions: ["jpg", "jpeg", "png", "webp"] }],
    });
    if (!selected) return;

    setBusy(true);
    setError(null);
    try {
      const paths = Array.isArray(selected) ? selected : [selected];
      for (const path of paths) {
        if (typeof path === "string") {
          await importImagePath(placeId, path);
        }
      }
      await refresh();
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => {
    function onPaste(event: ClipboardEvent) {
      const items = event.clipboardData?.items;
      if (!items) return;

      const files: File[] = [];
      for (const item of items) {
        if (item.kind === "file" && item.type.startsWith("image/")) {
          const file = item.getAsFile();
          if (file) files.push(file);
        }
      }
      if (files.length > 0) {
        event.preventDefault();
        void importFiles(files);
      }
    }

    window.addEventListener("paste", onPaste);
    return () => window.removeEventListener("paste", onPaste);
  }, [importFiles]);

  if (!place) {
    return <p>Loading place…</p>;
  }

  return (
    <>
      <div className="section-header">
        <div>
          <button type="button" onClick={onBack}>
            ← Back
          </button>
          <h1 style={{ marginTop: "0.75rem" }}>{place.name}</h1>
          <p>
            <span className={`status-pill ${place.status}`}>{place.status}</span>
          </p>
        </div>
        <div className="topbar-actions">
          <button
            type="button"
            disabled={busy}
            onClick={() =>
              void updatePlace(placeId, {
                status: place.status === "been" ? "dream" : "been",
              })
                .then(setPlace)
                .catch((err) => setError(String(err)))
            }
          >
            {place.status === "been" ? "Mark as dream" : "Been there"}
          </button>
          <button type="button" onClick={() => void pickFiles()} disabled={busy}>
            Choose files
          </button>
          <button
            type="button"
            className="danger"
            disabled={busy}
            onClick={() => {
              if (!window.confirm(`Delete ${place.name}? This cannot be undone.`)) {
                return;
              }
              setBusy(true);
              deletePlace(placeId)
                .then(onDeleted)
                .catch((err) => {
                  setError(String(err));
                  setBusy(false);
                });
            }}
          >
            Delete place
          </button>
        </div>
      </div>

      {error && <div className="error-banner">{error}</div>}

      <div className="form-row" style={{ marginBottom: "1.5rem" }}>
        <label htmlFor="place-notes">Notes</label>
        <textarea
          id="place-notes"
          rows={4}
          value={notes}
          onChange={(e) => setNotes(e.target.value)}
          placeholder="Why this place, what to do, who to go with…"
        />
        <button
          type="button"
          disabled={busy}
          onClick={() => {
            setBusy(true);
            updatePlace(placeId, { notes })
              .then((next) => {
                setPlace(next);
                setNotes(next.notes ?? "");
              })
              .catch((err) => setError(String(err)))
              .finally(() => setBusy(false));
          }}
        >
          Save notes
        </button>
      </div>

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
          <strong>Drop photos here</strong>
        </p>
        <p>Or press Ctrl+V to paste from clipboard</p>
      </div>

      {images.length === 0 ? (
        <div className="empty-state">
          <h2>No photos yet</h2>
          <p>This place is waiting for its first memory.</p>
        </div>
      ) : (
        <div className="image-grid">
          {images.map((image) => (
            <img
              key={image.id}
              src={assetUrl(libraryRoot, image.thumb_relpath)}
              alt=""
              loading="lazy"
            />
          ))}
        </div>
      )}
    </>
  );
}
