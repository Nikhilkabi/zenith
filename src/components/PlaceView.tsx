import { open } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useState } from "react";
import { getPlace, importImageBytes, importImagePath, listImages } from "../api";
import { assetUrl } from "../lib/assets";
import type { ImageRecord, Place } from "../types";

interface Props {
  placeId: string;
  libraryRoot: string;
  onBack: () => void;
}

async function fileToBytes(file: File): Promise<number[]> {
  const buffer = await file.arrayBuffer();
  return Array.from(new Uint8Array(buffer));
}

export default function PlaceView({ placeId, libraryRoot, onBack }: Props) {
  const [place, setPlace] = useState<Place | null>(null);
  const [images, setImages] = useState<ImageRecord[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [dragActive, setDragActive] = useState(false);

  const refresh = useCallback(async () => {
    const [nextPlace, nextImages] = await Promise.all([
      getPlace(placeId),
      listImages(placeId),
    ]);
    setPlace(nextPlace);
    setImages(nextImages);
  }, [placeId]);

  useEffect(() => {
    refresh().catch((err) => setError(String(err)));
  }, [refresh]);

  async function importFiles(files: FileList | File[]) {
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
  }

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
  });

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
        <button type="button" onClick={() => void pickFiles()} disabled={busy}>
          Choose files
        </button>
      </div>

      {error && <div className="error-banner">{error}</div>}

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
        <p><strong>Drop photos here</strong></p>
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
