import { useCallback, useEffect, useRef, useState, type DragEvent } from "react";
import {
  coverCredit,
  deleteImage,
  deletePlace,
  getPlace,
  importImageBytes,
  listImages,
  setCountryCover,
  setPlaceCover,
  updateImageCaption,
  updatePlace,
} from "../api";
import { assetUrl } from "../lib/assets";
import { normalizeImageFile } from "../lib/normalizeImage";
import type { ImageRecord, Place } from "../types";
import InlineText from "../ui/InlineText";
import Mark from "../ui/Mark";
import Skeleton from "../ui/Skeleton";
import TwoClickDelete from "../ui/TwoClickDelete";
import CoverPicker, { CoverCredit } from "./CoverPicker";
import Links from "./Links";
import MustDo from "./MustDo";
import PhotoAdd from "./PhotoAdd";
import Viewer from "./Viewer";

interface Props {
  placeId: string;
  countryId: string;
  libraryRoot: string;
  onDeleted: () => void;
  onLibraryChanged?: () => void;
  onError: (message: string | null) => void;
}

export default function PlaceView({
  placeId,
  countryId,
  libraryRoot,
  onDeleted,
  onLibraryChanged,
  onError,
}: Props) {
  const [place, setPlace] = useState<Place | null>(null);
  const [images, setImages] = useState<ImageRecord[]>([]);
  const [notes, setNotes] = useState("");
  const [whenText, setWhenText] = useState("");
  const [viewerIndex, setViewerIndex] = useState<number | null>(null);
  const [byDate, setByDate] = useState(false);
  const [drag, setDrag] = useState(false);
  const [picker, setPicker] = useState(false);
  const [adjusting, setAdjusting] = useState(false);
  const [adding, setAdding] = useState(false);
  const [credit, setCredit] = useState<string | null>(null);
  const notesTimer = useRef<number | null>(null);
  const whenTimer = useRef<number | null>(null);
  const notesRef = useRef(notes);
  const whenRef = useRef(whenText);
  const notesHydratedFor = useRef<string | null>(null);
  notesRef.current = notes;
  whenRef.current = whenText;

  const refresh = useCallback(async () => {
    const [nextPlace, nextImages] = await Promise.all([
      getPlace(placeId),
      listImages(placeId),
    ]);
    setPlace(nextPlace);
    setImages(nextImages);
    if (notesHydratedFor.current !== placeId) {
      setNotes(nextPlace.notes ?? "");
      setWhenText(nextPlace.when_text ?? "");
      notesHydratedFor.current = placeId;
    }
  }, [placeId]);

  useEffect(() => {
    refresh().catch((err) => onError(String(err)));
  }, [refresh, onError]);

  useEffect(() => {
    if (!place?.cover_relpath) {
      setCredit(null);
      return;
    }
    coverCredit(place.cover_relpath)
      .then((next) => setCredit(next?.credit ?? null))
      .catch(() => setCredit(null));
  }, [place?.cover_relpath]);

  const saveWhen = useCallback(
    async (value: string) => {
      try {
        const next = await updatePlace(placeId, { whenText: value });
        setPlace(next);
      } catch (err) {
        onError(String(err));
      }
    },
    [placeId, onError],
  );

  const saveNotes = useCallback(
    async (value: string) => {
      try {
        const next = await updatePlace(placeId, { notes: value });
        setPlace(next);
      } catch (err) {
        onError(String(err));
      }
    },
    [placeId, onError],
  );

  useEffect(() => {
    return () => {
      if (notesTimer.current != null) {
        window.clearTimeout(notesTimer.current);
        void saveNotes(notesRef.current);
      }
      if (whenTimer.current != null) {
        window.clearTimeout(whenTimer.current);
        void saveWhen(whenRef.current);
      }
    };
  }, [saveNotes, saveWhen]);

  const importFiles = useCallback(
    async (files: FileList | File[]) => {
      const list = Array.from(files).filter(
        (f) =>
          f.type.startsWith("image/") ||
          /\.(avif|jpe?g|png|webp)$/i.test(f.name),
      );
      if (list.length === 0) {
        onError("Drop or paste a JPEG, PNG, WebP, or AVIF image.");
        return;
      }
      try {
        for (const file of list) {
          const { filename, bytes } = await normalizeImageFile(file);
          await importImageBytes(placeId, filename, bytes);
        }
        await refresh();
        onLibraryChanged?.();
      } catch (err) {
        onError(String(err));
      }
    },
    [placeId, refresh, onError, onLibraryChanged],
  );

  useEffect(() => {
    function onPaste(e: ClipboardEvent) {
      const items = e.clipboardData?.items;
      if (!items) return;
      const files: File[] = [];
      for (const item of items) {
        if (item.kind === "file" && item.type.startsWith("image/")) {
          const file = item.getAsFile();
          if (file) files.push(file);
        }
      }
      if (files.length === 0) return;
      e.preventDefault();
      void importFiles(files);
    }
    window.addEventListener("paste", onPaste);
    return () => window.removeEventListener("paste", onPaste);
  }, [importFiles]);

  if (!place) return <Skeleton count={4} />;

  const coverId =
    images.find(
      (img) =>
        img.thumb_relpath === place.cover_relpath ||
        img.display_relpath === place.cover_relpath,
    )?.id ?? images[0]?.id;

  const ordered = [...images].sort((a, b) => {
    if (byDate) {
      return (b.taken_at ?? "").localeCompare(a.taken_at ?? "") || a.sort - b.sort;
    }
    if (a.id === coverId) return -1;
    if (b.id === coverId) return 1;
    return a.sort - b.sort;
  });

  return (
    <div
      className={drag ? "place-page drop-active" : "place-page"}
      onDragOver={(e: DragEvent) => {
        e.preventDefault();
        setDrag(true);
      }}
      onDragLeave={() => setDrag(false)}
      onDrop={(e: DragEvent) => {
        e.preventDefault();
        setDrag(false);
        void importFiles(e.dataTransfer.files);
      }}
    >
      <div className="page-header">
        <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
          <InlineText
            as="h1"
            className="page-title"
            value={place.name}
            onSave={async (name) => {
              const next = await updatePlace(placeId, { name });
              setPlace(next);
            }}
          />
          <Mark
            been={place.status === "been"}
            onToggle={() => {
              updatePlace(placeId, {
                status: place.status === "been" ? "dream" : "been",
              })
                .then(setPlace)
                .catch((err) => onError(String(err)));
            }}
          />
          <span className="meta">
            {images.length} photos
            {credit ? (
              <>
                {" · "}
                <CoverCredit credit={credit} />
              </>
            ) : null}
          </span>
        </div>
        <div className="page-actions">
          {images.length > 0 && (
            <button
              type="button"
              className={`text${byDate ? " current" : ""}`}
              onClick={() => setByDate((v) => !v)}
            >
              {byDate ? "By date" : "By import"}
            </button>
          )}
          <button type="button" className="text" onClick={() => setPicker(true)}>
            Cover
          </button>
          {place.cover_relpath && (
            <button
              type="button"
              className="text"
              onClick={() => {
                setAdjusting(true);
                setPicker(true);
              }}
            >
              Move
            </button>
          )}
          <button type="button" className="text" onClick={() => setAdding(true)}>
            Add photos
          </button>
          <TwoClickDelete
            label={place.name}
            confirmLabel={`Delete ${place.name}?`}
            onConfirm={async () => {
              await deletePlace(placeId);
              onDeleted();
            }}
          />
        </div>
      </div>

      <div className="place-spread">
      <div className="writing">
      <input
        className="when-line"
        value={whenText}
        placeholder="When, if you know — June 2027"
        onChange={(e) => {
          const value = e.target.value;
          setWhenText(value);
          if (whenTimer.current != null) window.clearTimeout(whenTimer.current);
          whenTimer.current = window.setTimeout(() => void saveWhen(value), 500);
        }}
        onBlur={() => {
          if (whenTimer.current != null) {
            window.clearTimeout(whenTimer.current);
            whenTimer.current = null;
          }
          void saveWhen(whenText);
        }}
      />

      <textarea
        className="notes"
        value={notes}
        placeholder="Notes"
        rows={3}
        onChange={(e) => {
          const value = e.target.value;
          setNotes(value);
          if (notesTimer.current != null) window.clearTimeout(notesTimer.current);
          notesTimer.current = window.setTimeout(() => {
            void saveNotes(value);
          }, 600);
        }}
        onBlur={() => {
          if (notesTimer.current != null) {
            window.clearTimeout(notesTimer.current);
            notesTimer.current = null;
          }
          void saveNotes(notes);
        }}
      />

      <MustDo placeId={placeId} onError={onError} />

      <Links
        libraryRoot={libraryRoot}
        placeId={placeId}
        countryId={countryId}
        onError={onError}
      />
      </div>

      <div className="place-gallery">
      {ordered.length === 0 ? (
        <div className="photo-grid empty">Drop photos here</div>
      ) : (
        <div className="photo-grid">
          {ordered.map((image, index) => (
            <button
              key={image.id}
              type="button"
              className={`photo-cell${image.id === coverId ? " cover" : ""}`}
              onClick={() => setViewerIndex(index)}
            >
              <img
                src={assetUrl(libraryRoot, image.thumb_relpath)}
                alt=""
                loading="lazy"
              />
            </button>
          ))}
        </div>
      )}
      </div>
      </div>

      {picker && place && (
        <CoverPicker
          libraryRoot={libraryRoot}
          initialQuery={place.name}
          target="place"
          ownerId={placeId}
          shape="wide"
          current={
            place.cover_relpath
              ? {
                  src: assetUrl(libraryRoot, place.cover_relpath),
                  x: place.cover_x,
                  y: place.cover_y,
                }
              : null
          }
          startOnCurrent={adjusting}
          onClose={() => {
            setPicker(false);
            setAdjusting(false);
          }}
          onApplied={async () => {
            await refresh();
            onLibraryChanged?.();
          }}
          onError={onError}
        />
      )}

      {adding && place && (
        <PhotoAdd
          placeId={placeId}
          placeName={place.name}
          libraryRoot={libraryRoot}
          onClose={() => setAdding(false)}
          onImported={async () => {
            await refresh();
            onLibraryChanged?.();
          }}
          onFiles={(files) => {
            setAdding(false);
            void importFiles(files);
          }}
          onError={onError}
        />
      )}

      {viewerIndex != null && ordered[viewerIndex] && (
        <Viewer
          images={ordered}
          index={viewerIndex}
          libraryRoot={libraryRoot}
          onClose={() => setViewerIndex(null)}
          onIndex={setViewerIndex}
          onDelete={async (imageId) => {
            await deleteImage(imageId);
            const next = ordered.filter((img) => img.id !== imageId);
            setImages(next);
            if (next.length === 0) setViewerIndex(null);
            else setViewerIndex(Math.min(viewerIndex, next.length - 1));
            await refresh();
          }}
          onSetPlaceCover={async (imageId) => {
            const next = await setPlaceCover(placeId, imageId);
            setPlace(next);
            onLibraryChanged?.();
          }}
          onSetCountryCover={async (imageId) => {
            await setCountryCover(countryId, imageId);
            onLibraryChanged?.();
          }}
          onCaption={async (imageId, caption) => {
            const next = await updateImageCaption(imageId, caption);
            setImages((current) =>
              current.map((img) => (img.id === next.id ? next : img)),
            );
          }}
        />
      )}
    </div>
  );
}
